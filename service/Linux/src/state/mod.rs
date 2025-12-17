use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::api_client::{BLControllerInfo, BLNode, BLScript, BroadlinkClient};

pub struct AppState {
    pub client: BroadlinkClient,
    pub controllers: RwLock<Vec<BLControllerInfo>>,
    pub scripts_cache: RwLock<HashMap<String, Vec<BLScript>>>,
    pub tree_cache: RwLock<HashMap<String, HashMap<String, BLNode>>>,
    pub is_loading: RwLock<bool>,
    pub show_disabled: RwLock<bool>,
}

impl AppState {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            client: BroadlinkClient::new(host, port),
            controllers: RwLock::new(Vec::new()),
            scripts_cache: RwLock::new(HashMap::new()),
            tree_cache: RwLock::new(HashMap::new()),
            is_loading: RwLock::new(false),
            show_disabled: RwLock::new(true),
        }
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
}
