use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::app::{App, InputMode, Panel};

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3), Constraint::Length(1)])
        .split(area);

    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(vertical[0]);

    render_panel(f, app, panels[0], &Panel::Left);
    render_panel(f, app, panels[1], &Panel::Right);
    render_preview(f, app, vertical[1]);
    render_status(f, app, vertical[2]);
}

fn render_panel(f: &mut Frame, app: &App, area: Rect, side: &Panel) {
    let panel = match side { Panel::Left => &app.left, Panel::Right => &app.right };
    let is_active = app.active == *side;

    let border_style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = format!(" {} ", panel.path.display());
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));

    let visible_height = area.height.saturating_sub(2) as usize;
    let items: Vec<ListItem> = panel.entries
        .iter()
        .skip(panel.scroll_offset)
        .take(visible_height)
        .enumerate()
        .map(|(i, path)| {
            let actual_idx = i + panel.scroll_offset;
            let name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let (icon, style) = if path.is_dir() {
                ("📁 ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
            } else if is_executable(path) {
                ("⚙  ", Style::default().fg(Color::Green))
            } else {
                ("   ", Style::default().fg(Color::White))
            };

            let selected_style = if actual_idx == panel.selected && is_active {
                style.add_modifier(Modifier::REVERSED)
            } else if actual_idx == panel.selected {
                style.bg(Color::DarkGray)
            } else {
                style
            };

            ListItem::new(Line::from(vec![
                Span::raw(icon),
                Span::styled(name, selected_style),
            ]))
        })
        .collect();

    let mut list_state = ListState::default();
    if is_active {
        list_state.select(Some(panel.selected.saturating_sub(panel.scroll_offset)));
    }

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_preview(f: &mut Frame, app: &App, area: Rect) {
    let panel = app.active_panel();
    let content = match panel.selected_path() {
        None => String::from("(empty)"),
        Some(p) if p.is_dir() => {
            let count = std::fs::read_dir(p).map(|d| d.count()).unwrap_or(0);
            format!("Directory — {} items", count)
        }
        Some(p) => {
            let size = p.metadata().map(|m| m.len()).unwrap_or(0);
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            let preview = if is_text_file(ext) {
                std::fs::read_to_string(p)
                    .unwrap_or_default()
                    .lines()
                    .take(2)
                    .collect::<Vec<_>>()
                    .join(" │ ")
            } else {
                String::new()
            };
            if preview.is_empty() {
                format!("{} bytes  .{}", size, ext)
            } else {
                format!("{} bytes  │  {}", size, preview)
            }
        }
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Preview ");

    // Show input prompt if in input mode
    let display = match &app.input_mode {
        InputMode::Rename(s)  => format!("Rename → {} (Enter to confirm, Esc to cancel)", s),
        InputMode::NewDir(s)  => format!("New dir → {} (Enter to confirm, Esc to cancel)", s),
        InputMode::Normal     => content,
    };

    let p = Paragraph::new(display)
        .block(block)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(p, area);
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let panel = app.active_panel();
    let count = panel.entries.len();
    let idx = if count > 0 { panel.selected + 1 } else { 0 };
    let text = format!(" {}/{} │ {}", idx, count, app.status);
    let p = Paragraph::new(text)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(p, area);
}

fn is_executable(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn is_text_file(ext: &str) -> bool {
    matches!(ext, "txt" | "md" | "rs" | "go" | "py" | "js" | "ts" | "json" | "yaml" | "yml"
             | "toml" | "sh" | "bash" | "html" | "css" | "xml" | "csv" | "log" | "conf" | "env")
}
