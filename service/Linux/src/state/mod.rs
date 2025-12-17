use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::api_client::{BLControllerInfo, BLNode, BLScript, BroadlinkClient};
use crate::config::{Config, MprisConfig};

#[derive(Clone, Debug)]
pub struct RecentCommand {
    pub controller: String,
    pub device: String,
    pub device_label: String,
    pub command_path: String,
    pub label: String,
}

pub struct AppState {
    pub client: BroadlinkClient,
    pub controllers: RwLock<Vec<BLControllerInfo>>,
    pub scripts_cache: RwLock<HashMap<String, Vec<BLScript>>>,
    pub tree_cache: RwLock<HashMap<String, HashMap<String, BLNode>>>,
    pub is_loading: RwLock<bool>,
    pub selected_controllers: RwLock<HashSet<String>>,
    pub recent_commands: RwLock<Vec<RecentCommand>>,
    pub mpris_config: RwLock<MprisConfig>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            client: BroadlinkClient::new(config.host, config.port),
            controllers: RwLock::new(Vec::new()),
            scripts_cache: RwLock::new(HashMap::new()),
            tree_cache: RwLock::new(HashMap::new()),
            is_loading: RwLock::new(false),
            selected_controllers: RwLock::new(config.selected_controllers),
            recent_commands: RwLock::new(Vec::new()),
            mpris_config: RwLock::new(config.mpris),
        }
    }

    pub async fn toggle_controller(&self, name: String) {
        let mut selected = self.selected_controllers.write().await;
        if selected.contains(&name) {
            selected.remove(&name);
        } else {
            selected.insert(name);
        }

        let mpris = self.mpris_config.read().await;

        // Save config
        let config = Config {
            host: self.client.host().to_string(),
            port: self.client.port(),
            selected_controllers: selected.clone(),
            mpris: mpris.clone(),
        };
        if let Err(e) = config.save() {
            log::error!("Failed to save config: {}", e);
        }
    }

    pub async fn add_recent_command(&self, cmd: RecentCommand) {
        let mut recent = self.recent_commands.write().await;
        // Remove if already exists to move it to the top
        recent.retain(|r| !(r.controller == cmd.controller && r.device == cmd.device && r.command_path == cmd.command_path));
        recent.insert(0, cmd);
        if recent.len() > 10 {
            recent.truncate(10);
        }
    }

    pub async fn clear_recent_commands(&self) {
        let mut recent = self.recent_commands.write().await;
        recent.clear();
    }

    pub async fn refresh_devices(self: Arc<Self>) {
        {
            let mut loading = self.is_loading.write().await;
            *loading = true;
        }

        let ctrls = self.client.fetch_controllers().await.unwrap_or_default();
        let mut new_scripts = HashMap::new();
        let mut new_trees = HashMap::new();

        for ctrl in &ctrls {
            let scripts = self.client.fetch_scripts(&ctrl.name).await.unwrap_or_default();
            new_scripts.insert(ctrl.name.clone(), scripts);

            let devs = self.client.fetch_devices(&ctrl.name).await.unwrap_or_default();
            let mut dev_map = HashMap::new();
            for dev in devs {
                if let Ok(tree) = self.client.fetch_command_tree(&ctrl.name, &dev.name).await {
                    dev_map.insert(dev.name, tree);
                }
            }
            new_trees.insert(ctrl.name.clone(), dev_map);
        }

        {
            let mut controllers = self.controllers.write().await;
            *controllers = ctrls;
            let mut scripts = self.scripts_cache.write().await;
            *scripts = new_scripts;
            let mut trees = self.tree_cache.write().await;
            *trees = new_trees;
            let mut loading = self.is_loading.write().await;
            *loading = false;
        }
    }

    pub async fn find_command(&self, controller: &str, device: &str, target: &str) -> Option<String> {
        let trees = self.tree_cache.read().await;
        let ctrl_tree = trees.get(controller)?;
        let device_root = ctrl_tree.get(device)?;
        
        // Normalize the target by replacing common separators with dots
        let normalized_target = target.replace(',', ".").replace('/', ".");
        
        self.search_node(device_root, &normalized_target)
    }

    fn search_node(&self, node: &BLNode, target: &str) -> Option<String> {
        if let Some(path) = &node.command_path {
            if path == target || node.name == target {
                return Some(path.clone());
            }
        }
        for child in &node.children {
            if let Some(found) = self.search_node(child, target) {
                return Some(found);
            }
        }
        None
    }
}
