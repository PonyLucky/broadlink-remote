use ratatui::Frame;
use ratatui::text::{Span, Line};
use ratatui::layout::{Layout, Direction, Constraint, Rect};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Clear, Tabs};
use ratatui::style::{Style, Color, Modifier};
use crate::state::{AppState, View, CommandListItem};

fn get_view_title(view: &View, state: &AppState) -> String {
    match view {
        View::Controllers => "Controllers".to_string(),
        View::Devices(ctrl) => {
            let friendly = state.controllers.iter()
                .find(|c| c.name == *ctrl)
                .and_then(|c| c.friendly_name.clone())
                .unwrap_or_else(|| ctrl.clone());
            format!("Devices - {}", friendly)
        }
        View::Commands(ctrl, dev) => {
            let dev_friendly = state.get_devices_for_controller(ctrl).iter()
                .find(|d| d.name == *dev)
                .and_then(|d| d.friendly_name.clone())
                .unwrap_or_else(|| dev.clone());
            format!("Commands - {}/{}", ctrl, dev_friendly)
        }
        View::Scripts(ctrl) => format!("Scripts - {}", ctrl),
        View::CommandTree(ctrl, dev) => format!("Tree - {}/{}", ctrl, dev),
    }
}

fn render_tab_bar(frame: &mut Frame, state: &AppState, area: Rect) {
    let tabs: Vec<String> = state.controllers.iter().map(|ctrl| {
        ctrl.friendly_name.clone().unwrap_or_else(|| ctrl.name.clone())
    }).collect();

    let current_idx = match &state.current_view {
        View::Devices(ctrl) | View::Commands(ctrl, _) | View::Scripts(ctrl) | View::CommandTree(ctrl, _) => {
            state.controllers.iter().position(|c| c.name == *ctrl).unwrap_or(0)
        }
        _ => 0,
    };

    let tabs_widget = Tabs::new(tabs)
        .select(current_idx)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::BOTTOM));

    frame.render_widget(tabs_widget, area);
}

pub fn render(frame: &mut Frame, state: &AppState) {
    // Clear screen
    frame.render_widget(Clear, frame.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),    // Tab bar (controller tabs)
            Constraint::Min(10),      // Main content
            Constraint::Length(1),    // Status bar
            Constraint::Length(1),    // Help bar
        ])
        .split(frame.area());

    // Controller tab bar
    render_tab_bar(frame, state, chunks[0]);

    // Main content based on view
    match &state.current_view {
        View::Controllers => render_controllers(frame, state, chunks[1]),
        View::Devices(ctrl) => render_devices(frame, state, ctrl, chunks[1]),
        View::Commands(ctrl, dev) => render_commands(frame, state, ctrl, dev, chunks[1]),
        View::Scripts(ctrl) => render_scripts(frame, state, ctrl, chunks[1]),
        View::CommandTree(ctrl, dev) => render_command_tree(frame, state, ctrl, dev, chunks[1]),
    }

    // Status bar
    let status_text = if state.is_loading {
        "Loading..."
    } else if state.status_is_fresh() {
        &state.status_message
    } else {
        ""
    };
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default());
    frame.render_widget(status, chunks[2]);

    // Help bar with navigation legend
    let key_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let help = Paragraph::new(Line::from(vec![
        Span::styled("↑↓", key_style),
        Span::raw("/"),
        Span::styled("jk", key_style),
        Span::raw(" Nav | "),
        Span::styled("←→", key_style),
        Span::raw("/"),
        Span::styled("hl", key_style),
        Span::raw(" Back/Enter | "),
        Span::styled("Tab", key_style),
        Span::raw(" Next Ctrl | "),
        Span::styled("d", key_style),
        Span::raw(" Controllers | "),
        Span::styled("s", key_style),
        Span::raw(" Scripts | "),
        Span::styled("r", key_style),
        Span::raw(" Refresh | "),
        Span::styled("q", key_style),
        Span::raw(" Quit"),
    ]))
    .style(Style::default().fg(Color::Gray));
    frame.render_widget(help, chunks[3]);

    // Render controllers popup if open
    if state.show_controllers_popup {
        render_controllers_popup(frame, state);
    }
}

fn render_controllers_popup(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    // Center the popup
    let popup_width = 40;
    let popup_height = state.controllers.len() as u16 + 4;
    let x = (area.width - popup_width) / 2;
    let y = (area.height - popup_height) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let items: Vec<ListItem> = state.controllers.iter().map(|ctrl| {
        let name = ctrl.friendly_name.clone().unwrap_or_else(|| ctrl.name.clone());
        let selected = state.current_view == View::Devices(ctrl.name.clone())
            || matches!(&state.current_view, View::Commands(c, _) | View::Scripts(c) | View::CommandTree(c, _) if c == &ctrl.name);
        let prefix = if selected { "[active] " } else { "" };
        ListItem::new(format!("{}{} ({})", prefix, name, ctrl.ip))
    }).collect();

    let list = List::new(items)
        .block(Block::default()
            .title("Controllers (press Enter to switch)")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black)))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, popup_area, &mut ListState::default().with_selected(Some(state.controllers_popup_index)));
}

fn render_controllers(frame: &mut Frame, state: &AppState, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = state.controllers.iter().map(|ctrl| {
        let name = ctrl.friendly_name.clone().unwrap_or_else(|| ctrl.name.clone());
        let selected = state.selected_controllers.contains(&ctrl.name);
        let prefix = if selected { "[✓] " } else { "[ ] " };
        ListItem::new(format!("{}{} ({})", prefix, name, ctrl.ip))
    }).collect();

    let list = List::new(items)
        .block(Block::default()
            .title("Controllers")
            .borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut ListState::default().with_selected(Some(state.selected_index)));
}

fn render_devices(frame: &mut Frame, state: &AppState, controller: &str, area: ratatui::layout::Rect) {
    let devices = state.get_devices_for_controller(controller);
    let items: Vec<ListItem> = devices.iter().map(|dev| {
        let name = dev.friendly_name.clone().unwrap_or_else(|| dev.name.clone());
        ListItem::new(name)
    }).collect();

    let list = List::new(items)
        .block(Block::default()
            .title(get_view_title(&state.current_view, state))
            .borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut ListState::default().with_selected(Some(state.selected_index)));
}

fn render_commands(frame: &mut Frame, state: &AppState, controller: &str, device: &str, area: ratatui::layout::Rect) {
    let items = state.get_commands_for_device(controller, device);
    let list_items: Vec<ListItem> = items.iter().map(|item| {
        match item {
            CommandListItem::Command(node) => {
                let name = state.get_command_display_name(node);
                let prefix = if node.disabled { "[disabled] " } else { "" };
                ListItem::new(format!("{}{}", prefix, name))
            }
            CommandListItem::Header(header) => {
                ListItem::new(format!("─── {} ───", header))
                    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            }
        }
    }).collect();

    let list = List::new(list_items)
        .block(Block::default()
            .title(get_view_title(&state.current_view, state))
            .borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut ListState::default().with_selected(Some(state.selected_index)));
}

fn render_scripts(frame: &mut Frame, state: &AppState, controller: &str, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = state.scripts_cache.get(controller)
        .map(|scripts| scripts.iter().map(|s| {
            let name = s.friendly_name.clone().unwrap_or_else(|| s.name.clone());
            ListItem::new(name)
        }).collect())
        .unwrap_or_default();
    
    let list = List::new(items)
        .block(Block::default()
            .title(get_view_title(&state.current_view, state))
            .borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut ListState::default().with_selected(Some(state.selected_index)));
}

fn render_command_tree(frame: &mut Frame, _state: &AppState, controller: &str, device: &str, area: ratatui::layout::Rect) {
    let text = "Command tree view";
    let paragraph = Paragraph::new(text)
        .block(Block::default()
            .title(format!("Tree - {}/{}", controller, device))
            .borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}