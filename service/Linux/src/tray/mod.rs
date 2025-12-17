use ksni::{Tray, Handle as KsniHandle};
use ksni::menu::{MenuItem, StandardItem, SubMenu};
use crate::state::AppState;
use crate::api_client::{BLNode, BLNodeKind};
use std::sync::{Arc, Mutex};
use tokio::runtime::Handle;

pub struct BroadlinkTray {
    state: Arc<AppState>,
    handle: Handle,
    tray_handle: Arc<Mutex<Option<KsniHandle<BroadlinkTray>>>>,
    menu_cache: Vec<CachedMenu>,
}

#[derive(Clone)]
enum CachedMenu {
    Separator,
    Standard {
        label: String,
        enabled: bool,
        action: CachedAction,
    },
    SubMenu {
        label: String,
        items: Vec<CachedMenu>,
    },
}

#[derive(Clone)]
enum CachedAction {
    Refresh,
    ToggleController(String),
    RunScript {
        controller: String,
        script_name: String,
    },
    SendCommand {
        controller: String,
        device: String,
        cmd_path: String,
    },
    Quit,
}

impl BroadlinkTray {
    pub fn new(state: Arc<AppState>, tray_handle: Arc<Mutex<Option<KsniHandle<BroadlinkTray>>>>) -> Self {
        Self {
            state,
            handle: Handle::current(),
            tray_handle,
            menu_cache: Vec::new(),
        }
    }

    fn flatten_node_cached(node: &BLNode, controller: &str, device: &str, prefix: Option<String>, items: &mut Vec<CachedMenu>) {
        let current_name = node.friendly_name.as_deref().unwrap_or(&node.name).to_string();
        let label = match prefix {
            Some(p) => format!("{} > {}", p, current_name),
            None => current_name,
        };

        match node.kind {
            BLNodeKind::Group => {
                for child in &node.children {
                    Self::flatten_node_cached(child, controller, device, Some(label.clone()), items);
                }
            }
            BLNodeKind::Command => {
                items.push(CachedMenu::Standard {
                    label,
                    enabled: !node.disabled,
                    action: CachedAction::SendCommand {
                        controller: controller.to_string(),
                        device: device.to_string(),
                        cmd_path: node.command_path.clone().unwrap_or_default(),
                    },
                });
            }
        }
    }

    pub fn update_menu(&mut self) {
        log::info!("Updating tray menu cache");
        let mut new_cache = Vec::new();

        new_cache.push(CachedMenu::Standard {
            label: "Refresh devices".to_string(),
            enabled: true,
            action: CachedAction::Refresh,
        });

        new_cache.push(CachedMenu::Separator);

        // Blocking read for menu generation
        let controllers = futures::executor::block_on(self.state.controllers.read());
        let scripts_cache = futures::executor::block_on(self.state.scripts_cache.read());
        let tree_cache = futures::executor::block_on(self.state.tree_cache.read());
        let selected_controllers = futures::executor::block_on(self.state.selected_controllers.read());

        // Controllers Selector
        let mut controller_items = Vec::new();
        let mut sorted_controllers: Vec<_> = controllers.iter().collect();
        sorted_controllers.sort_by(|a, b| a.name.cmp(&b.name));

        for ctrl in &sorted_controllers {
            let is_selected = selected_controllers.contains(&ctrl.name);
            let label = format!("{} {}", if is_selected { "☑" } else { "☐" }, ctrl.friendly_name.clone().unwrap_or_else(|| ctrl.name.clone()));
            
            controller_items.push(CachedMenu::Standard {
                label,
                enabled: true,
                action: CachedAction::ToggleController(ctrl.name.clone()),
            });
        }
        new_cache.push(CachedMenu::SubMenu {
            label: "Controllers".to_string(),
            items: controller_items,
        });

        new_cache.push(CachedMenu::Separator);

        // Filtered Devices & Scripts
        for ctrl in &sorted_controllers {
            if !selected_controllers.contains(&ctrl.name) {
                continue;
            }

            // Scripts for this controller
            if let Some(scripts) = scripts_cache.get(&ctrl.name) {
                let mut script_items = Vec::new();
                let mut sorted_scripts = scripts.clone();
                sorted_scripts.sort_by(|a, b| a.name.cmp(&b.name));

                for script in sorted_scripts {
                    let label = script.friendly_name.clone().unwrap_or_else(|| script.name.clone());
                    script_items.push(CachedMenu::Standard {
                        label,
                        enabled: true,
                        action: CachedAction::RunScript {
                            controller: ctrl.name.clone(),
                            script_name: script.name.clone(),
                        },
                    });
                }
                if !script_items.is_empty() {
                    new_cache.push(CachedMenu::SubMenu {
                        label: format!("{} - Scripts", ctrl.friendly_name.as_ref().unwrap_or(&ctrl.name)),
                        items: script_items,
                    });
                }
            }

            // Devices for this controller
            if let Some(dev_map) = tree_cache.get(&ctrl.name) {
                let mut sorted_devs: Vec<_> = dev_map.iter().collect();
                sorted_devs.sort_by(|a, b| a.0.cmp(b.0));

                for (dev_name, root_node) in sorted_devs {
                    let mut dev_items = Vec::new();
                    let dev_friendly_name = root_node.friendly_name.clone().unwrap_or_else(|| dev_name.clone());
                    Self::flatten_node_cached(root_node, &ctrl.name, dev_name, None, &mut dev_items);
                    new_cache.push(CachedMenu::SubMenu {
                        label: dev_friendly_name,
                        items: dev_items,
                    });
                }
            }
        }

        new_cache.push(CachedMenu::Separator);
        new_cache.push(CachedMenu::Standard {
            label: "Quit".to_string(),
            enabled: true,
            action: CachedAction::Quit,
        });

        self.menu_cache = new_cache;
    }

    fn create_menu_item(&self, item: &CachedMenu) -> MenuItem<Self> {
        match item {
            CachedMenu::Separator => MenuItem::Separator,
            CachedMenu::Standard { label, enabled, action } => {
                let action = action.clone();
                let state = self.state.clone();
                let handle = self.handle.clone();
                let tray_handle = self.tray_handle.clone();
                
                MenuItem::Standard(StandardItem {
                    label: label.clone(),
                    enabled: *enabled,
                    activate: Box::new(move |_| {
                        let state = state.clone();
                        let handle = handle.clone();
                        let tray_handle = tray_handle.clone();
                        let action = action.clone();
                        
                        handle.spawn(async move {
                            match action {
                                CachedAction::Refresh => {
                                    state.refresh_devices().await;
                                    if let Ok(h) = tray_handle.lock() {
                                        if let Some(h) = h.as_ref() {
                                            h.update(|tray| tray.update_menu());
                                        }
                                    }
                                }
                                CachedAction::ToggleController(name) => {
                                    state.toggle_controller(name).await;
                                    if let Ok(h) = tray_handle.lock() {
                                        if let Some(h) = h.as_ref() {
                                            h.update(|tray| tray.update_menu());
                                        }
                                    }
                                }
                                CachedAction::RunScript { controller, script_name } => {
                                    if let Ok(true) = state.client.run_script(&controller, &script_name).await {
                                        log::info!("✅ Script: {}/{} ran successfully", controller, script_name);
                                    }
                                }
                                CachedAction::SendCommand { controller, device, cmd_path } => {
                                    match state.client.send_command(&controller, &device, &cmd_path).await {
                                        Ok(true) => log::info!("✅ Sent: {}/{}/{}", controller, device, cmd_path),
                                        Ok(false) => log::warn!("⚠️ Failed to send: {}/{}/{}", controller, device, cmd_path),
                                        Err(e) => log::error!("❌ Error sending command: {}", e),
                                    }
                                }
                                CachedAction::Quit => std::process::exit(0),
                            }
                        });
                    }),
                    ..Default::default()
                })
            }
            CachedMenu::SubMenu { label, items } => {
                MenuItem::SubMenu(SubMenu {
                    label: label.clone(),
                    submenu: items.iter().map(|i| self.create_menu_item(i)).collect(),
                    ..Default::default()
                })
            }
        }
    }
}

impl Tray for BroadlinkTray {
    fn icon_name(&self) -> String {
        let tray_icon = futures::executor::block_on(self.state.tray_icon.read());
        tray_icon.clone().unwrap_or_else(|| "preferences-desktop-peripherals".to_string())
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        self.menu_cache.iter().map(|item| self.create_menu_item(item)).collect()
    }
}
