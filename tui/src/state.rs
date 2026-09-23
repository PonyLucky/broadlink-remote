use std::collections::HashMap;
use crate::api_client::{BLControllerInfo, BLNode, BLScript, BLDeviceInfo, BroadlinkClient, BLNodeKind};
use crate::config::Config;

pub struct AppState {
    pub client: BroadlinkClient,
    pub controllers: Vec<BLControllerInfo>,
    pub scripts_cache: HashMap<String, Vec<BLScript>>,
    pub tree_cache: HashMap<String, HashMap<String, BLNode>>,
    pub devices_cache: HashMap<String, Vec<BLDeviceInfo>>,
    pub is_loading: bool,
    pub selected_controllers: std::collections::HashSet<String>,
    // TUI state
    pub current_view: View,
    pub selected_index: usize,
    pub status_message: String,
    pub status_time: std::time::Instant,
    pub show_controllers_popup: bool,
    pub controllers_popup_index: usize,
    pub device_selection: HashMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum View {
    Controllers,
    Devices(String),      // controller name
    Commands(String, String), // controller, device
    Scripts(String),      // controller name
    CommandTree(String, String), // controller, device
}

#[derive(Debug, Clone)]
pub enum CommandListItem {
    Command(BLNode),
    Header(String),
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            client: BroadlinkClient::new(config.host, config.port),
            controllers: Vec::new(),
            scripts_cache: HashMap::new(),
            tree_cache: HashMap::new(),
            devices_cache: HashMap::new(),
            is_loading: false,
            selected_controllers: config.selected_controllers,
            current_view: View::Controllers, // Will be set to first device after refresh
            selected_index: 0,
            status_message: String::new(),
            status_time: std::time::Instant::now(),
            show_controllers_popup: false,
            controllers_popup_index: 0,
            device_selection: HashMap::new(),
        }
    }

    pub async fn refresh_devices(&mut self) {
        self.is_loading = true;
        self.set_status("Loading devices...");

        let ctrls = self.client.fetch_controllers().await.unwrap_or_default();
        let mut new_scripts = HashMap::new();
        let mut new_trees = HashMap::new();
        let mut new_devices = HashMap::new();

        for ctrl in &ctrls {
            let scripts = self.client.fetch_scripts(&ctrl.name).await.unwrap_or_default();
            new_scripts.insert(ctrl.name.clone(), scripts);

            let devs = self.client.fetch_devices(&ctrl.name).await.unwrap_or_default();
            new_devices.insert(ctrl.name.clone(), devs.clone());

            let mut dev_map = HashMap::new();
            for dev in devs {
                if let Ok(tree) = self.client.fetch_command_tree(&ctrl.name, &dev.name).await {
                    dev_map.insert(dev.name, tree);
                }
            }
            new_trees.insert(ctrl.name.clone(), dev_map);
        }

        self.controllers = ctrls;
        self.scripts_cache = new_scripts;
        self.tree_cache = new_trees;
        self.devices_cache = new_devices;
        self.is_loading = false;
        self.set_status("Devices loaded");

        // Auto-navigate to first controller's devices view
        if let Some(first_ctrl) = self.controllers.first() {
            self.current_view = View::Devices(first_ctrl.name.clone());
            self.selected_index = 0;
        }
    }

    pub fn set_status(&mut self, msg: &str) {
        self.status_message = msg.to_string();
        self.status_time = std::time::Instant::now();
    }

    pub fn status_is_fresh(&self) -> bool {
        self.status_time.elapsed().as_secs() < 3
    }

    fn find_first_command_index(&self, controller: &str, device: &str) -> usize {
        let items = self.get_commands_for_device(controller, device);
        for (i, item) in items.iter().enumerate() {
            if let CommandListItem::Command(_) = item {
                return i;
            }
        }
        0
    }

    fn navigate_up(&mut self) {
        if let View::Commands(ctrl, dev) = &self.current_view {
            let items = self.get_commands_for_device(ctrl, dev);
            let mut idx = self.selected_index;
            while idx > 0 {
                idx -= 1;
                if let CommandListItem::Command(_) = &items[idx] {
                    self.selected_index = idx;
                    return;
                }
            }
        } else if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    fn navigate_down(&mut self) {
        if let View::Commands(ctrl, dev) = &self.current_view {
            let items = self.get_commands_for_device(ctrl, dev);
            let mut idx = self.selected_index;
            while idx < items.len() - 1 {
                idx += 1;
                if let CommandListItem::Command(_) = &items[idx] {
                    self.selected_index = idx;
                    return;
                }
            }
        } else {
            self.selected_index += 1;
        }
    }

    pub async fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        // If controllers popup is open, handle its navigation
        if self.show_controllers_popup {
            self.handle_controllers_popup_key(key).await;
            return;
        }

        match key.code {
            // Arrow keys
            crossterm::event::KeyCode::Up => {
                self.navigate_up();
            }
            crossterm::event::KeyCode::Down => {
                self.navigate_down();
            }
            crossterm::event::KeyCode::Left => {
                self.handle_back();
            }
            crossterm::event::KeyCode::Right => {
                self.handle_enter().await;
            }
            // Vim-like navigation
            crossterm::event::KeyCode::Char('j') => {
                self.navigate_down();
            }
            crossterm::event::KeyCode::Char('k') => {
                self.navigate_up();
            }
            crossterm::event::KeyCode::Char('h') => {
                self.handle_back();
            }
            crossterm::event::KeyCode::Char('l') => {
                self.handle_enter().await;
            }
            crossterm::event::KeyCode::Char('g') => {
                self.selected_index = 0;
            }
            crossterm::event::KeyCode::Char('G') => {
                // Jump to end - will be clamped by list length
                self.selected_index = usize::MAX;
            }
            crossterm::event::KeyCode::Enter => {
                self.handle_enter().await;
            }
            crossterm::event::KeyCode::Char('r') => {
                self.refresh_devices().await;
            }
            crossterm::event::KeyCode::Backspace | crossterm::event::KeyCode::Char('b') => {
                self.handle_back();
            }
            crossterm::event::KeyCode::Char('d') => {
                self.show_controllers_popup = true;
                self.controllers_popup_index = 0;
            }
            crossterm::event::KeyCode::Char('s') => {
                // Toggle scripts view for current controller
                if let View::Devices(ctrl_name) = &self.current_view {
                    self.current_view = View::Scripts(ctrl_name.clone());
                    self.selected_index = 0;
                }
            }
            _ => {}
        }
    }

    async fn handle_controllers_popup_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
                if self.controllers_popup_index > 0 {
                    self.controllers_popup_index -= 1;
                }
            }
            crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
                if self.controllers_popup_index < self.controllers.len() - 1 {
                    self.controllers_popup_index += 1;
                }
            }
            crossterm::event::KeyCode::Enter | crossterm::event::KeyCode::Char('l') => {
                if self.controllers_popup_index < self.controllers.len() {
                    let ctrl = &self.controllers[self.controllers_popup_index];
                    self.current_view = View::Devices(ctrl.name.clone());
                    self.selected_index = 0;
                }
                self.show_controllers_popup = false;
            }
            crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Char('h') => {
                self.show_controllers_popup = false;
            }
            _ => {}
        }
    }

    async fn handle_enter(&mut self) {
        match &self.current_view {
            View::Controllers => {
                if self.selected_index < self.controllers.len() {
                    let ctrl = &self.controllers[self.selected_index];
                    self.current_view = View::Devices(ctrl.name.clone());
                    self.selected_index = 0;
                }
            }
            View::Devices(ctrl_name) => {
                let devices = self.get_devices_for_controller(ctrl_name);
                if self.selected_index < devices.len() {
                    let dev = &devices[self.selected_index];
                    let dev_name = dev.name.clone();
                    // Save device selection before entering commands view
                    self.device_selection.insert(ctrl_name.clone(), self.selected_index);
                    // Find first command index before changing view
                    let first_cmd_idx = self.find_first_command_index(ctrl_name, &dev_name);
                    self.current_view = View::Commands(ctrl_name.clone(), dev_name);
                    self.selected_index = first_cmd_idx;
                }
            }
            View::Commands(ctrl_name, dev_name) => {
                // Find the command node and execute it
                if let Some(cmd_node) = self.get_command_at_index(ctrl_name, dev_name, self.selected_index) {
                    if let Some(cmd_path) = &cmd_node.command_path {
                        let result = self.client.send_command(ctrl_name, dev_name, cmd_path).await;
                        match result {
                            Ok(success) => {
                                if success {
                                    let cmd_display = self.get_command_display_name(&cmd_node);
                                    self.set_status(&format!("Command sent: {}", cmd_display));
                                } else {
                                    self.set_status("Command failed");
                                }
                            }
                            Err(e) => {
                                self.set_status(&format!("Error: {}", e));
                            }
                        }
                    }
                }
            }
            View::Scripts(ctrl_name) => {
                if let Some(scripts) = self.scripts_cache.get(ctrl_name) {
                    if self.selected_index < scripts.len() {
                        let script = &scripts[self.selected_index];
                        let script_display = script.friendly_name.clone().unwrap_or_else(|| script.name.clone());
                        let result = self.client.run_script(ctrl_name, &script.name).await;
                        match result {
                            Ok(success) => {
                                if success {
                                    self.set_status(&format!("Script executed: {}", script_display));
                                } else {
                                    self.set_status("Script failed");
                                }
                            }
                            Err(e) => {
                                self.set_status(&format!("Error: {}", e));
                            }
                        }
                    }
                }
            }
            View::CommandTree(_, _) => {}
        }
    }

    fn handle_back(&mut self) {
        match &self.current_view {
            View::Devices(ctrl_name) => {
                // Go to previous controller's devices, or wrap to last
                let idx = self.controllers.iter().position(|c| c.name == *ctrl_name);
                if let Some(i) = idx {
                    if i > 0 {
                        self.current_view = View::Devices(self.controllers[i - 1].name.clone());
                    } else {
                        // Wrap to last controller
                        if let Some(last) = self.controllers.last() {
                            self.current_view = View::Devices(last.name.clone());
                        }
                    }
                }
                self.selected_index = 0;
            }
            View::Commands(ctrl_name, _) | View::CommandTree(ctrl_name, _) => {
                let ctrl = ctrl_name.clone();
                self.current_view = View::Devices(ctrl.clone());
                if let Some(&idx) = self.device_selection.get(&ctrl) {
                    self.selected_index = idx;
                } else {
                    self.selected_index = 0;
                }
            }
            View::Scripts(ctrl_name) => {
                self.current_view = View::Devices(ctrl_name.clone());
                self.selected_index = 0;
            }
            _ => {}
        }
    }

    pub fn get_devices_for_controller(&self, controller: &str) -> Vec<crate::api_client::BLDeviceInfo> {
        self.devices_cache.get(controller).cloned().unwrap_or_default()
    }

    pub fn get_commands_for_device(&self, controller: &str, device: &str) -> Vec<CommandListItem> {
        let mut items = Vec::new();
        if let Some(trees) = self.tree_cache.get(controller) {
            if let Some(root) = trees.get(device) {
                self.collect_commands_with_headers(root, &mut items);
            }
        }
        items
    }

    fn collect_commands_with_headers(&self, node: &BLNode, items: &mut Vec<CommandListItem>) {
        if node.kind == BLNodeKind::Command {
            if !node.disabled {
                items.push(CommandListItem::Command(node.clone()));
            }
        } else if node.kind == BLNodeKind::Group {
            // Collect commands from this group first
            let mut group_items = Vec::new();
            for child in &node.children {
                self.collect_commands_with_headers(child, &mut group_items);
            }
            // Only add header if group has commands
            if !group_items.is_empty() {
                let header_name = node.friendly_name.clone().unwrap_or_else(|| node.name.clone());
                items.push(CommandListItem::Header(header_name));
                items.extend(group_items);
            }
        }
    }

    pub fn get_command_at_index(&self, controller: &str, device: &str, index: usize) -> Option<BLNode> {
        let items = self.get_commands_for_device(controller, device);
        for (i, item) in items.iter().enumerate() {
            if i == index {
                if let CommandListItem::Command(node) = item {
                    return Some(node.clone());
                }
            }
        }
        None
    }

    pub fn get_command_display_name(&self, node: &BLNode) -> String {
        node.friendly_name.clone().unwrap_or_else(|| node.name.clone())
    }
}