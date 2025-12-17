use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BLControllerInfo {
    pub name: String,
    pub friendly_name: Option<String>,
    pub ip: String,
    pub port: u16,
    pub r#type: Option<String>,
    pub mac: Option<String>,
    pub model: Option<String>,
    pub devices: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BLDeviceInfo {
    pub name: String,
    pub friendly_name: Option<String>,
    pub r#type: String,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BLScriptStep {
    pub r#type: String,
    pub device: Option<String>,
    pub command: Option<String>,
    pub time: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BLScript {
    pub name: String,
    pub friendly_name: Option<String>,
    pub steps: Option<Vec<BLScriptStep>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BLNode {
    pub kind: BLNodeKind,
    pub name: String,
    pub friendly_name: Option<String>,
    pub disabled: bool,
    pub children: Vec<BLNode>,
    pub command_path: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BLNodeKind {
    Group,
    Command,
}

pub struct BroadlinkClient {
    host: String,
    port: u16,
    client: Client,
}

impl BroadlinkClient {
    pub fn new(host: String, port: u16) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(4))
            .build()
            .unwrap_or_default();
        Self { host, port, client }
    }

    fn get_url(&self, endpoint: &str) -> String {
        format!("http://{}:{}/api{}", self.host, self.port, endpoint)
    }

    pub async fn fetch_controllers(&self) -> Result<Vec<BLControllerInfo>, reqwest::Error> {
        self.client.get(self.get_url("/controller"))
            .send()
            .await?
            .json()
            .await
    }

    pub async fn fetch_devices(&self, controller: &str) -> Result<Vec<BLDeviceInfo>, reqwest::Error> {
        self.client.get(self.get_url(&format!("/{}/device", controller)))
            .send()
            .await?
            .json()
            .await
    }

    pub async fn fetch_scripts(&self, controller: &str) -> Result<Vec<BLScript>, reqwest::Error> {
        self.client.get(self.get_url(&format!("/{}/scripts", controller)))
            .send()
            .await?
            .json()
            .await
    }

    pub async fn run_script(&self, controller: &str, name: &str) -> Result<bool, reqwest::Error> {
        let resp = self.client.post(self.get_url(&format!("/{}/scripts/{}", controller, name)))
            .send()
            .await?;
        Ok(resp.status().is_success())
    }

    pub async fn send_command(&self, controller: &str, device: &str, command_path: &str) -> Result<bool, reqwest::Error> {
        let resp = self.client.post(self.get_url(&format!("/{}/{}/{}", controller, device, command_path)))
            .send()
            .await?;
        Ok(resp.status().is_success())
    }

    pub async fn fetch_command_tree(&self, controller: &str, device: &str) -> Result<BLNode, Box<dyn std::error::Error + Send + Sync>> {
        let resp = self.client.get(self.get_url(&format!("/{}/{}", controller, device)))
            .send()
            .await?;
        
        let json: serde_json::Value = resp.json().await?;
        
        let root_friendly = json.get("friendly_name").and_then(|v| v.as_str()).map(|s| s.to_string());
        let mut root = BLNode {
            kind: BLNodeKind::Group,
            name: device.to_string(),
            friendly_name: root_friendly,
            disabled: false,
            children: Vec::new(),
            command_path: None,
        };

        self.build_nodes(&mut root, &json, "");
        Ok(root)
    }

    fn build_nodes(&self, parent: &mut BLNode, json: &serde_json::Value, path: &str) {
        // Commands
        if let Some(commands) = json.get("commands") {
            if let Some(obj) = commands.as_object() {
                let mut sorted_cmds: Vec<_> = obj.iter().collect();
                sorted_cmds.sort_by_key(|a| a.0);
                for (name, val) in sorted_cmds {
                    let disabled = val.get("disabled").and_then(|v| v.as_bool()).unwrap_or(false);
                    let friendly = val.get("friendly_name").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let new_path = if path.is_empty() { name.clone() } else { format!("{}.{}", path, name) };
                    parent.children.push(BLNode {
                        kind: BLNodeKind::Command,
                        name: name.clone(),
                        friendly_name: friendly,
                        disabled,
                        children: Vec::new(),
                        command_path: Some(new_path),
                    });
                }
            } else if let Some(arr) = commands.as_array() {
                for cmd in arr {
                    if let Some(name) = cmd.get("name").and_then(|v| v.as_str()) {
                        let disabled = cmd.get("disabled").and_then(|v| v.as_bool()).unwrap_or(false);
                        let friendly = cmd.get("friendly_name").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let new_path = if path.is_empty() { name.to_string() } else { format!("{}.{}", path, name) };
                        parent.children.push(BLNode {
                            kind: BLNodeKind::Command,
                            name: name.to_string(),
                            friendly_name: friendly,
                            disabled,
                            children: Vec::new(),
                            command_path: Some(new_path),
                        });
                    }
                }
            }
        }

        // Groups
        if let Some(groups) = json.get("groups") {
            if let Some(obj) = groups.as_object() {
                let mut sorted_groups: Vec<_> = obj.iter().collect();
                sorted_groups.sort_by_key(|a| a.0);
                for (name, val) in sorted_groups {
                    let disabled = val.get("disabled").and_then(|v| v.as_bool()).unwrap_or(false);
                    let friendly = val.get("friendly_name").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let mut node = BLNode {
                        kind: BLNodeKind::Group,
                        name: name.clone(),
                        friendly_name: friendly,
                        disabled,
                        children: Vec::new(),
                        command_path: None,
                    };
                    self.build_nodes(&mut node, val, &if path.is_empty() { name.clone() } else { format!("{}.{}", path, name) });
                    parent.children.push(node);
                }
            } else if let Some(arr) = groups.as_array() {
                for g in arr {
                    if let Some(name) = g.get("name").and_then(|v| v.as_str()) {
                        let disabled = g.get("disabled").and_then(|v| v.as_bool()).unwrap_or(false);
                        let friendly = g.get("friendly_name").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let mut node = BLNode {
                            kind: BLNodeKind::Group,
                            name: name.to_string(),
                            friendly_name: friendly,
                            disabled,
                            children: Vec::new(),
                            command_path: None,
                        };
                        self.build_nodes(&mut node, g, &if path.is_empty() { name.to_string() } else { format!("{}.{}", path, name) });
                        parent.children.push(node);
                    }
                }
            }
        }
    }
}
