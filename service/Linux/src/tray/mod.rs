use ksni::{Tray, MenuItem, menu};
use crate::state::AppState;
use crate::api_client::{BLNode, BLNodeKind};
use std::sync::Arc;
use tokio::runtime::Handle;

pub struct BroadlinkTray {
    state: Arc<AppState>,
}

impl BroadlinkTray {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    fn build_node_menu(&self, node: &BLNode, controller: &str, device: &str) -> MenuItem<Self> {
        let title = node.friendly_name.as_deref().unwrap_or(&node.name).to_string();
        
        match node.kind {
            BLNodeKind::Group => {
                let mut sub_items = Vec::new();
                for child in &node.children {
                    sub_items.push(self.build_node_menu(child, controller, device));
                }
                MenuItem::SubMenu(title, sub_items)
            }
            BLNodeKind::Command => {
                let controller = controller.to_string();
                let device = device.to_string();
                let cmd_path = node.command_path.clone().unwrap_or_default();
                let state = self.state.clone();
                
                MenuItem::Standard {
                    label: title,
                    enabled: !node.disabled,
                    activate: Box::new(move |_| {
                        let state = state.clone();
                        let controller = controller.clone();
                        let device = device.clone();
                        let cmd_path = cmd_path.clone();
                        
                        // Execute async task from sync callback
                        Handle::current().spawn(async move {
                            match state.client.send_command(&controller, &device, &cmd_path).await {
                                Ok(true) => log::info!("✅ Sent: {}/{}/{}", controller, device, cmd_path),
                                Ok(false) => log::warn!("⚠️ Failed to send: {}/{}/{}", controller, device, cmd_path),
                                Err(e) => log::error!("❌ Error sending command: {}", e),
                            }
                        });
                    }),
                    ..Default::default()
                }
            }
        }
    }
}

impl Tray for BroadlinkTray {
    fn icon_name(&self) -> String {
        "network-wireless".to_string() // Example icon name
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let mut items = Vec::new();

        let state = self.state.clone();
        items.push(MenuItem::Standard {
            label: "Refresh devices".to_string(),
            activate: Box::new(move |_| {
                let state = state.clone();
                Handle::current().spawn(async move {
                    state.refresh_devices().await;
                });
            }),
            ..Default::default()
        });

        items.push(MenuItem::Separator);

        // Blocking read for menu generation (ksni calls this from its own thread)
        let controllers = futures_util::executor::block_on(self.state.controllers.read());
        let scripts_cache = futures_util::executor::block_on(self.state.scripts_cache.read());
        let tree_cache = futures_util::executor::block_on(self.state.tree_cache.read());

        for ctrl in controllers.iter() {
            let mut ctrl_items = Vec::new();
            
            // Scripts
            if let Some(scripts) = scripts_cache.get(&ctrl.name) {
                let mut script_items = Vec::new();
                for script in scripts {
                    let controller = ctrl.name.clone();
                    let script_name = script.name.clone();
                    let state = self.state.clone();
                    script_items.push(MenuItem::Standard {
                        label: script.friendly_name.clone().unwrap_or_else(|| script.name.clone()),
                        activate: Box::new(move |_| {
                            let state = state.clone();
                            let controller = controller.clone();
                            let script_name = script_name.clone();
                            Handle::current().spawn(async move {
                                if let Ok(true) = state.client.run_script(&controller, &script_name).await {
                                    log::info!("✅ Script: {}/{} ran successfully", controller, script_name);
                                }
                            });
                        }),
                        ..Default::default()
                    });
                }
                if !script_items.is_empty() {
                    ctrl_items.push(MenuItem::SubMenu("Scripts".to_string(), script_items));
                    ctrl_items.push(MenuItem::Separator);
                }
            }

            // Devices
            if let Some(dev_map) = tree_cache.get(&ctrl.name) {
                for (dev_name, root_node) in dev_map {
                    ctrl_items.push(self.build_node_menu(root_node, &ctrl.name, dev_name));
                }
            }

            items.push(MenuItem::SubMenu(ctrl.friendly_name.clone().unwrap_or_else(|| ctrl.name.clone()), ctrl_items));
        }

        items.push(MenuItem::Separator);
        items.push(MenuItem::Standard {
            label: "Quit".to_string(),
            activate: Box::new(|_| std::process::exit(0)),
            ..Default::default()
        });

        items
    }
}
