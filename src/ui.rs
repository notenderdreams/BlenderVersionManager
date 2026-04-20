use crate::app::{App, ViewMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Gauge, Clear},
    Frame,
};

const BLENDER_ORANGE: Color = Color::Rgb(232, 123, 44);
const ACCENT: Color = Color::Rgb(0, 150, 255);

pub fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(size);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" BVM ", Style::default().fg(Color::Black).bg(BLENDER_ORANGE).add_modifier(Modifier::BOLD)),
        Span::styled(" Blender Version Manager ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(BLENDER_ORANGE)));
    f.render_widget(title, chunks[0]);

    // Content area
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(chunks[1]);

    // Tabs
    let titles = vec![" [1] Available Versions ", " [2] Installed Versions "];
    let tabs = Tabs::new(titles.clone().into_iter().map(Line::from).collect::<Vec<_>>())
        .block(Block::default().borders(Borders::ALL).title(" Views "))
        .select(match app.view_mode {
            ViewMode::Available => 0,
            _ => 1,
        })
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(Style::default().fg(BLENDER_ORANGE).add_modifier(Modifier::BOLD));

    let main_block = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(content_chunks[0]);

    f.render_widget(tabs, main_block[0]);

    match app.view_mode {
        ViewMode::Available => {
            let items: Vec<ListItem> = app.available
                .iter()
                .map(|v| {
                    let is_installed = app.installed.iter().any(|i| i.version == v.version);
                    let icon = if is_installed {
                         Span::styled(" 󰄬 ", Style::default().fg(Color::Green))
                    } else {
                         Span::styled(" 󱓞 ", Style::default().fg(BLENDER_ORANGE))
                    };
                    
                    let content = vec![
                        Line::from(vec![
                            icon,
                            Span::styled(format!("{:<10}", v.version), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                            if is_installed {
                                Span::styled(" (Installed)", Style::default().fg(Color::Green).add_modifier(Modifier::DIM))
                            } else {
                                Span::styled(format!(" ({})", v.url), Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))
                            },
                        ])
                    ];
                    ListItem::new(content)
                })
                .collect();
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(" 󰬚 Available for Install "))
                .highlight_style(Style::default().bg(Color::Rgb(45, 45, 45)).fg(BLENDER_ORANGE))
                .highlight_symbol("  ");
            f.render_stateful_widget(list, main_block[1], &mut app.available_state);
        }
        ViewMode::Installed | ViewMode::ConfirmDelete(_) => {
            let items: Vec<ListItem> = app.installed
                .iter()
                .map(|v| {
                    let content = vec![
                        Line::from(vec![
                            Span::styled(" 󰂖 ", Style::default().fg(Color::Green)),
                            Span::styled(format!("{:<10}", v.version), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                            Span::styled(format!("  {:?}", v.path), Style::default().fg(Color::DarkGray)),
                        ])
                    ];
                    ListItem::new(content)
                })
                .collect();
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(" 󰄬 Installed Versions "))
                .highlight_style(Style::default().bg(Color::Rgb(45, 45, 45)).fg(Color::Cyan))
                .highlight_symbol("  ");
            f.render_stateful_widget(list, main_block[1], &mut app.installed_state);
        }
    }

    // Info panel
    let info = Paragraph::new(vec![
        Line::from(vec![Span::styled(" 󰘳 CONTROLS ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))]),
        Line::from("  ↑/↓   Navigate"),
        Line::from("  Tab   Switch Tab"),
        Line::from("  1 / 2 Switch Direct"),
        Line::from("  Enter Install/Launch"),
        Line::from("  'd'   Remove"),
        Line::from("  'f'   Refresh List"),
        Line::from("  'q'   Quit"),
        Line::from(""),
        Line::from(vec![Span::styled(" 󰝰 PATHS ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))]),
        Line::from(format!("  Root:   {}", app.manager.base_path.display())),
        Line::from(format!("  Shared: {}", app.manager.get_shared_config_dir().display())),
        Line::from(""),
        Line::from(vec![Span::styled(" 󰀼 SHARED CONFIG ", Style::default().fg(BLENDER_ORANGE).add_modifier(Modifier::BOLD))]),
        Line::from("  Addons & Preferences are"),
        Line::from("  shared across all versions."),
    ])
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(info, content_chunks[1]);

    // Footer / Status
    let footer_text = if let Some(prog) = app.downloading {
        format!(" 󰉍 Downloading: [{:.1}%] ", prog * 100.0)
    } else {
        format!(" 󰊚 Status: {} ", app.status)
    };

    if let Some(prog) = app.downloading {
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(" Progress "))
            .gauge_style(Style::default().fg(BLENDER_ORANGE).bg(Color::Rgb(30, 30, 30)).add_modifier(Modifier::BOLD))
            .ratio(prog)
            .label(format!("{:.1}%", prog * 100.0));
        f.render_widget(gauge, chunks[2]);
    } else {
        let footer = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
        f.render_widget(footer, chunks[2]);
    }

    if let ViewMode::ConfirmDelete(v) = &app.view_mode {
        render_popup(f, &format!(" Are you sure you want to delete Blender {}? (y/n) ", v));
    }
}

fn render_popup(f: &mut Frame, message: &str) {
    let block = Block::default()
        .title(" Confirmation ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let area = centered_rect(60, 20, f.size());
    f.render_widget(Clear, area); 
    f.render_widget(block, area);

    let text = Paragraph::new(Line::from(vec![
        Span::styled(message, Style::default().add_modifier(Modifier::BOLD))
    ]))
    .block(Block::default().borders(Borders::NONE))
    .alignment(ratatui::layout::Alignment::Center);

    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    f.render_widget(text, popup_chunks[1]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
