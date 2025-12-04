// GhostWire Client - UI Components
// This module handles all Ratatui rendering logic

use crate::app::{App, InputMode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Gauge, List, ListItem, Paragraph,
    },
    Frame,
};

/// Main UI render function
pub fn render(f: &mut Frame, app: &App) {
    // Create the main layout: Left sidebar | Middle chat | Right sidebar
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Left: Channels
            Constraint::Percentage(60), // Middle: Chat
            Constraint::Percentage(20), // Right: Telemetry
        ])
        .split(f.size());

    // Render each section
    render_channel_list(f, app, chunks[0]);
    render_chat_area(f, app, chunks[1]);
    render_telemetry(f, app, chunks[2]);
}

/// Render the channel list (left sidebar)
fn render_channel_list(f: &mut Frame, app: &App, area: Rect) {
    // Split into channels (top) and users (bottom)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60), // Channels
            Constraint::Percentage(40), // Users
        ])
        .split(area);
    
    // Render channels
    render_channels(f, app, chunks[0]);
    
    // Render users
    render_users(f, app, chunks[1]);
}

/// Render channels section
fn render_channels(f: &mut Frame, app: &App, area: Rect) {
    // Get sorted channel list
    let channel_ids = app.get_channel_list();
    
    // Create channel list items
    let channels: Vec<ListItem> = channel_ids
        .iter()
        .map(|channel_id| {
            if let Some(channel) = app.channels.get(channel_id) {
                let display_name = channel.display_name();
                
                // Add unread count if any
                let content = if channel.unread_count > 0 {
                    format!("{} ({})", display_name, channel.unread_count)
                } else {
                    display_name
                };
                
                // Highlight active channel
                let style = if channel_id == &app.active_channel {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else if channel.unread_count > 0 {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green)
                };
                
                ListItem::new(content).style(style)
            } else {
                ListItem::new("???").style(Style::default().fg(Color::Red))
            }
        })
        .collect();

    let title = format!(" Channels ({}) ", app.channels.len());
    let channel_list = List::new(channels)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green)),
        )
        .style(Style::default().fg(Color::Green));

    f.render_widget(channel_list, area);
}

/// Render users section
fn render_users(f: &mut Frame, app: &App, area: Rect) {
    use chrono::Utc;
    
    // Create user list items
    let users: Vec<ListItem> = app
        .users
        .iter()
        .enumerate()
        .map(|(i, user)| {
            // Determine user status: online, idle, or offline
            let (status_icon, status_color) = if !user.is_online {
                ("○", Color::DarkGray) // Offline
            } else if user.is_idle() {
                ("◐", Color::Yellow) // Idle (half-circle)
            } else {
                ("●", Color::Green) // Online and active
            };
            
            // Calculate time since last seen for offline/idle users
            let last_seen_text = if !user.is_online {
                let duration = Utc::now().signed_duration_since(user.last_seen);
                let mins = duration.num_minutes();
                let hours = duration.num_hours();
                let days = duration.num_days();
                
                if days > 0 {
                    format!(" ({}d)", days)
                } else if hours > 0 {
                    format!(" ({}h)", hours)
                } else if mins > 0 {
                    format!(" ({}m)", mins)
                } else {
                    "".to_string()
                }
            } else if user.is_idle() {
                // Show idle time for idle users
                let duration = Utc::now().signed_duration_since(user.last_seen);
                let mins = duration.num_minutes();
                format!(" (idle {}m)", mins)
            } else {
                String::new()
            };
            
            let content = format!("{} {}{}", status_icon, user.username, last_seen_text);
            
            let style = if i == app.selected_user {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(status_color)
            };
            
            ListItem::new(content).style(style)
        })
        .collect();

    let title = format!(" Users ({}) [J/K to select, d for DM] ", app.users.len());
    let users_list = List::new(users)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::Green));

    f.render_widget(users_list, area);
}

/// Render the chat area (middle section)
fn render_chat_area(f: &mut Frame, app: &App, area: Rect) {
    // Split chat area into messages and input
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),      // Chat messages
            Constraint::Length(3),   // Input box
        ])
        .split(area);

    render_messages(f, app, chunks[0]);
    render_input(f, app, chunks[1]);
}

/// Render chat messages
fn render_messages(f: &mut Frame, app: &App, area: Rect) {
    // Get messages from active channel
    let messages: Vec<ListItem> = if let Some(channel) = app.channels.get(&app.active_channel) {
        channel.messages
            .iter()
            .map(|msg| {
                let timestamp = msg.timestamp.format("%H:%M:%S");
                
                let content = if msg.is_system {
                    // System messages in red
                    Line::from(vec![
                        Span::styled(
                            format!("[{}] ", timestamp),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(
                            format!("⚠ {}", msg.content),
                            Style::default()
                                .fg(Color::Red)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ])
                } else {
                    // Regular messages
                    let sender_style = if msg.sender == app.username {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    };
                    
                    Line::from(vec![
                        Span::styled(
                            format!("[{}] ", timestamp),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(format!("{}: ", msg.sender), sender_style),
                        Span::styled(&msg.content, Style::default().fg(Color::White)),
                    ])
                };
                
                ListItem::new(content)
            })
            .collect()
    } else {
        Vec::new()
    };

    let connection_status = if app.is_connected {
        Span::styled(" ● CONNECTED ", Style::default().fg(Color::Green))
    } else {
        Span::styled(" ○ DISCONNECTED ", Style::default().fg(Color::Red))
    };
    
    // Get active channel display name
    let channel_name = app.channels.get(&app.active_channel)
        .map(|ch| ch.display_name())
        .unwrap_or_else(|| "Unknown".to_string());

    let title = Line::from(vec![
        Span::raw(" "),
        Span::styled(channel_name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        connection_status,
    ]);

    let messages_list = List::new(messages)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green)),
        )
        .style(Style::default().fg(Color::Green));

    f.render_widget(messages_list, area);
}

/// Render input box
fn render_input(f: &mut Frame, app: &App, area: Rect) {
    let input_style = match app.input_mode {
        InputMode::Normal => Style::default().fg(Color::Green),
        InputMode::Editing => Style::default().fg(Color::Yellow),
    };

    let mode_indicator = match app.input_mode {
        InputMode::Normal => " [NORMAL] ",
        InputMode::Editing => " [EDIT] ",
    };

    let input = Paragraph::new(app.input.as_str())
        .style(input_style)
        .block(
            Block::default()
                .title(mode_indicator)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(input_style),
        );

    f.render_widget(input, area);

    // Show cursor in edit mode
    if app.input_mode == InputMode::Editing {
        // Calculate cursor position
        f.set_cursor(
            area.x + app.input_cursor as u16 + 1,
            area.y + 1,
        );
    }
}

/// Render telemetry (right sidebar)
fn render_telemetry(f: &mut Frame, app: &App, area: Rect) {
    // Split telemetry area into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Connection uptime
            Constraint::Length(3),   // Latency
            Constraint::Length(7),   // Statistics (expanded)
            Constraint::Min(3),      // Network activity chart
            Constraint::Length(3),   // Server time
        ])
        .split(area);

    // Connection uptime
    let uptime_hours = app.telemetry.connection_uptime / 3600;
    let uptime_mins = (app.telemetry.connection_uptime % 3600) / 60;
    let uptime_secs = app.telemetry.connection_uptime % 60;
    
    let uptime = Paragraph::new(format!(
        "{}h {}m {}s",
        uptime_hours, uptime_mins, uptime_secs
    ))
    .style(Style::default().fg(Color::Green))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .title(" Uptime ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Green)),
    );
    f.render_widget(uptime, chunks[0]);

    // Latency gauge
    let latency_percent = (app.telemetry.latency_ms.min(500) as f64 / 500.0 * 100.0) as u16;
    let latency_color = if app.telemetry.latency_ms < 50 {
        Color::Green
    } else if app.telemetry.latency_ms < 150 {
        Color::Yellow
    } else {
        Color::Red
    };

    let latency = Gauge::default()
        .block(
            Block::default()
                .title(format!(" Latency: {}ms ", app.telemetry.latency_ms))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green)),
        )
        .gauge_style(Style::default().fg(latency_color))
        .percent(latency_percent);
    f.render_widget(latency, chunks[1]);

    // Expanded statistics
    let active_channel_name = app.channels.get(&app.active_channel)
        .map(|ch| ch.display_name())
        .unwrap_or_else(|| "Unknown".to_string());
    
    let stats_text = format!(
        "↑ Sent: {}\n↓ Recv: {}\n📊 Bytes: {} / {}\n📺 Channel: {}\n👥 Users: {} | Channels: {}",
        app.telemetry.messages_sent,
        app.telemetry.messages_received,
        format_bytes(app.telemetry.bytes_sent),
        format_bytes(app.telemetry.bytes_received),
        active_channel_name,
        app.users.len(),
        app.channels.len(),
    );
    
    let stats = Paragraph::new(stats_text)
        .style(Style::default().fg(Color::Green))
        .block(
            Block::default()
                .title(" Statistics ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green)),
        );
    f.render_widget(stats, chunks[2]);

    // Compact network activity chart
    let activity_data: Vec<u64> = app.telemetry.network_activity.clone();
    let max_activity = *activity_data.iter().max().unwrap_or(&1).max(&1);
    
    // Take last 15 data points
    let recent_data: Vec<(&str, u64)> = activity_data
        .iter()
        .rev()
        .take(15)
        .rev()
        .map(|&val| ("", val))
        .collect();
    
    let title = format!(" Activity (max: {}/s) ", max_activity);
    
    let barchart = ratatui::widgets::BarChart::default()
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green)),
        )
        .data(&recent_data)
        .bar_width(2)
        .bar_gap(0)
        .bar_style(Style::default().fg(Color::Green))
        .value_style(Style::default().fg(Color::DarkGray));
    
    f.render_widget(barchart, chunks[3]);
    
    // Server time
    use chrono::Utc;
    let now = Utc::now();
    let time_str = now.format("%H:%M:%S UTC").to_string();
    
    let time_widget = Paragraph::new(time_str)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(" Server Time ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green)),
        );
    f.render_widget(time_widget, chunks[4]);
}

/// Format bytes into human-readable format
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}
