use ksni::{Tray, Handle as KsniHandle};
use ksni::menu::{MenuItem, StandardItem, SubMenu};
use crate::state::{AppState, RecentCommand};
use crate::api_client::{BLNode, BLNodeKind};
use std::sync::{Arc, Mutex};
use tokio::runtime::Handle;

pub struct BroadlinkTray {
    state: Arc<AppState>,
    handle: Handle,
    tray_handle: Arc<Mutex<Option<KsniHandle<BroadlinkTray>>>>,
}

impl BroadlinkTray {
    pub fn new(state: Arc<AppState>, tray_handle: Arc<Mutex<Option<KsniHandle<BroadlinkTray>>>>) -> Self {
        Self {
            state,
            handle: Handle::current(),
            tray_handle,
        }
    }

    fn flatten_node(&self, node: &BLNode, controller: &str, device: &str, device_label: &str, prefix: Option<String>, items: &mut Vec<MenuItem<Self>>) {
        let current_name = node.friendly_name.as_deref().unwrap_or(&node.name).to_string();
        let label = match prefix {
            Some(p) => format!("{} > {}", p, current_name),
            None => current_name,
        };

        match node.kind {
            BLNodeKind::Group => {
                for child in &node.children {
                    self.flatten_node(child, controller, device, device_label, Some(label.clone()), items);
                }
            }
            BLNodeKind::Command => {
                let controller = controller.to_string();
                let device = device.to_string();
                let device_label = device_label.to_string();
                let cmd_path = node.command_path.clone().unwrap_or_default();
                let state = self.state.clone();
                let handle = self.handle.clone();
                let tray_handle = self.tray_handle.clone();
                let label_clone = label.clone();
                
                items.push(MenuItem::Standard(StandardItem {
                    label,
                    enabled: !node.disabled,
                    activate: Box::new(move |_| {
                        let state = state.clone();
                        let handle = handle.clone();
                        let controller = controller.clone();
                        let device = device.clone();
                        let device_label = device_label.clone();
                        let cmd_path = cmd_path.clone();
                        let tray_handle = tray_handle.clone();
                        let label = label_clone.clone();
                        
                        // Execute async task from sync callback
                        handle.spawn(async move {
                            match state.client.send_command(&controller, &device, &cmd_path).await {
                                Ok(true) => {
                                    log::info!("✅ Sent: {}/{}/{}", controller, device, cmd_path);
                                    state.add_recent_command(RecentCommand {
                                        controller: controller.clone(),
                                        device: device.clone(),
                                        device_label,
                                        command_path: cmd_path.clone(),
                                        label,
                                    }).await;
                                    // Refresh tray to show updated recents
                                    if let Ok(h) = tray_handle.lock() {
                                        if let Some(h) = h.as_ref() {
                                            h.update(|_| {});
                                        }
                                    }
                                }
                                Ok(false) => log::warn!("⚠️ Failed to send: {}/{}/{}", controller, device, cmd_path),
                                Err(e) => log::error!("❌ Error sending command: {}", e),
                            }
                        });
                    }),
                    ..Default::default()
                }));
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

        let state_refresh = self.state.clone();
        let handle_refresh = self.handle.clone();
        let tray_handle_refresh = self.tray_handle.clone();
        
        items.push(MenuItem::Standard(StandardItem {
            label: "Refresh devices".to_string(),
            activate: Box::new(move |_| {
                let state = state_refresh.clone();
                let handle = handle_refresh.clone();
                let tray_handle = tray_handle_refresh.clone();
                handle.spawn(async move {
                    state.refresh_devices().await;
                    if let Ok(h) = tray_handle.lock() {
                        if let Some(h) = h.as_ref() {
                            h.update(|_| {});
                        }
                    }
                });
            }),
            ..Default::default()
        }));

        items.push(MenuItem::Separator);

        // Recent Commands
        let recent = futures::executor::block_on(self.state.recent_commands.read());
        if !recent.is_empty() {
            for rc in recent.iter().take(5) {
                let rc = rc.clone();
                let state = self.state.clone();
                let handle = self.handle.clone();
                let tray_handle = self.tray_handle.clone();
                items.push(MenuItem::Standard(StandardItem {
                    label: format!("★ {} ({})", rc.label, rc.device_label),
                    activate: Box::new(move |_| {
                        let state = state.clone();
                        let handle = handle.clone();
                        let rc = rc.clone();
                        let tray_handle = tray_handle.clone();
                        handle.spawn(async move {
                            if let Ok(true) = state.client.send_command(&rc.controller, &rc.device, &rc.command_path).await {
                                log::info!("✅ Recent: {} sent", rc.label);
                                state.add_recent_command(rc).await;
                                if let Ok(h) = tray_handle.lock() {
                                    if let Some(h) = h.as_ref() {
                                        h.update(|_| {});
                                    }
                                }
                            }
                        });
                    }),
                    ..Default::default()
                }));
            }
            let state_clear = self.state.clone();
            let handle_clear = self.handle.clone();
            let tray_handle_clear = self.tray_handle.clone();
            items.push(MenuItem::Standard(StandardItem {
                label: "Clear recent".to_string(),
                activate: Box::new(move |_| {
                    let state = state_clear.clone();
                    let handle = handle_clear.clone();
                    let tray_handle = tray_handle_clear.clone();
                    handle.spawn(async move {
                        state.clear_recent_commands().await;
                        if let Ok(h) = tray_handle.lock() {
                            if let Some(h) = h.as_ref() {
                                h.update(|_| {});
                            }
                        }
                    });
                }),
                ..Default::default()
            }));
            items.push(MenuItem::Separator);
        }

        // Blocking read for menu generation
        let controllers = futures::executor::block_on(self.state.controllers.read());
        let scripts_cache = futures::executor::block_on(self.state.scripts_cache.read());
        let tree_cache = futures::executor::block_on(self.state.tree_cache.read());
        let selected_controllers = futures::executor::block_on(self.state.selected_controllers.read());

        // Controllers Selector
        let mut controller_items = Vec::new();
        for ctrl in controllers.iter() {
            let ctrl_name = ctrl.name.clone();
            let state = self.state.clone();
            let tray_handle = self.tray_handle.clone();
            let handle = self.handle.clone();
            let is_selected = selected_controllers.contains(&ctrl_name);
            let label = format!("{} {}", if is_selected { "☑" } else { "☐" }, ctrl.friendly_name.clone().unwrap_or_else(|| ctrl.name.clone()));
            
            controller_items.push(MenuItem::Standard(StandardItem {
                label,
                activate: Box::new(move |_| {
                    let state = state.clone();
                    let ctrl_name = ctrl_name.clone();
                    let tray_handle = tray_handle.clone();
                    handle.spawn(async move {
                        state.toggle_controller(ctrl_name).await;
                        if let Ok(h) = tray_handle.lock() {
                            if let Some(h) = h.as_ref() {
                                h.update(|_| {});
                            }
                        }
                    });
                }),
                ..Default::default()
            }));
        }
        items.push(MenuItem::SubMenu(SubMenu {
            label: "Controllers".to_string(),
            submenu: controller_items,
            ..Default::default()
        }));

        items.push(MenuItem::Separator);

        // Filtered Devices & Scripts
        for ctrl in controllers.iter() {
            if !selected_controllers.contains(&ctrl.name) {
                continue;
            }

            // Scripts for this controller
            if let Some(scripts) = scripts_cache.get(&ctrl.name) {
                let mut script_items = Vec::new();
                for script in scripts {
                    let controller = ctrl.name.clone();
                    let script_name = script.name.clone();
                    let state = self.state.clone();
                    let handle = self.handle.clone();
                    let script_friendly_name = script.friendly_name.clone().unwrap_or_else(|| script.name.clone());
                    script_items.push(MenuItem::Standard(StandardItem {
                        label: script_friendly_name.clone(),
                        activate: Box::new(move |_| {
                            let state = state.clone();
                            let handle = handle.clone();
                            let controller = controller.clone();
                            let script_name = script_name.clone();
                            handle.spawn(async move {
                                if let Ok(true) = state.client.run_script(&controller, &script_name).await {
                                    log::info!("✅ Script: {}/{} ran successfully", controller, script_name);
                                }
                            });
                        }),
                        ..Default::default()
                    }));
                }
                if !script_items.is_empty() {
                    items.push(MenuItem::SubMenu(SubMenu {
                        label: format!("{} - Scripts", ctrl.friendly_name.as_ref().unwrap_or(&ctrl.name)),
                        submenu: script_items,
                        ..Default::default()
                    }));
                }
            }

            // Devices for this controller
            if let Some(dev_map) = tree_cache.get(&ctrl.name) {
                for (dev_name, root_node) in dev_map {
                    let mut dev_items = Vec::new();
                    let dev_friendly_name = root_node.friendly_name.clone().unwrap_or_else(|| dev_name.clone());
                    self.flatten_node(root_node, &ctrl.name, dev_name, &dev_friendly_name, None, &mut dev_items);
                    items.push(MenuItem::SubMenu(SubMenu {
                        label: dev_friendly_name,
                        submenu: dev_items,
                        ..Default::default()
                    }));
                }
            }
        }

        items.push(MenuItem::Separator);
        items.push(MenuItem::Standard(StandardItem {
            label: "Quit".to_string(),
            activate: Box::new(|_| std::process::exit(0)),
            ..Default::default()
        }));

        items
    }
}
