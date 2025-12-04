// GhostWire Client - Main Entry Point
// This implements the CRITICAL async/sync split architecture:
// - Main thread: Runs the Ratatui UI loop (synchronous)
// - Network thread: Runs the WebSocket task (asynchronous via tokio::spawn)
// - Communication: mpsc unbounded channels

mod app;
mod network;
mod ui;

use app::{App, ChatMessage, InputMode, User};
use chrono::Utc;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use network::{NetworkCommand, NetworkEvent};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Default server URL (can be overridden via CLI args)
const DEFAULT_SERVER_URL: &str = "wss://ghost.jcyrus.com/ws";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    let username = if args.len() > 1 {
        args[1].clone()
    } else {
        // Generate a random username if none provided
        format!("ghost_{}", &uuid::Uuid::new_v4().to_string()[..8])
    };
    
    let server_url = if args.len() > 2 {
        args[2].clone()
    } else {
        DEFAULT_SERVER_URL.to_string()
    };

    // Create the application state
    let mut app = App::new(username.clone());

    // Create channels for communication between UI and network task
    // event_rx: UI receives events from network
    // command_tx: UI sends commands to network
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<NetworkEvent>();
    let (command_tx, command_rx) = mpsc::unbounded_channel::<NetworkCommand>();

    // Spawn the network task in a separate async runtime
    // This is the CRITICAL async/sync split!
    let network_handle = tokio::spawn(network::network_task(
        server_url,
        username.clone(),
        event_tx,
        command_rx,
    ));

    // Setup terminal for TUI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main UI loop (synchronous, runs on main thread)
    let result = run_ui_loop(&mut terminal, &mut app, &mut event_rx, &command_tx);

    // Cleanup: Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Shutdown network task
    let _ = command_tx.send(NetworkCommand::Disconnect);
    let _ = network_handle.await;

    // Print any errors
    if let Err(err) = result {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

/// Main UI event loop - runs synchronously on the main thread
fn run_ui_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    event_rx: &mut mpsc::UnboundedReceiver<NetworkEvent>,
    command_tx: &mpsc::UnboundedSender<NetworkCommand>,
) -> anyhow::Result<()> {
    // Track uptime
    let mut last_uptime_update = Instant::now();
    
    loop {
        // Render the UI
        terminal.draw(|f| ui::render(f, app))?;

        // Check for network events (non-blocking)
        while let Ok(event) = event_rx.try_recv() {
            handle_network_event(app, event);
        }

        // Check for terminal events (blocking with timeout)
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                handle_key_event(app, key.code, key.modifiers, command_tx)?;
            }
        }

        // Update uptime every second
        if last_uptime_update.elapsed() >= Duration::from_secs(1) {
            app.increment_uptime(1);
            app.update_network_activity();
            last_uptime_update = Instant::now();
        }
        
        // Check if we should quit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

/// Handle keyboard events
fn handle_key_event(
    app: &mut App,
    key: KeyCode,
    _modifiers: KeyModifiers,
    command_tx: &mpsc::UnboundedSender<NetworkCommand>,
) -> anyhow::Result<()> {
    match app.input_mode {
        InputMode::Normal => {
            match key {
                // Quit
                KeyCode::Char('q') | KeyCode::Esc => {
                    app.quit();
                }
                // Enter edit mode
                KeyCode::Char('i') | KeyCode::Enter => {
                    app.enter_edit_mode();
                }
                // Scroll chat
                KeyCode::Char('j') | KeyCode::Down => {
                    app.scroll_down();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    app.scroll_up();
                }
                // Scroll to bottom
                KeyCode::Char('G') => {
                    app.scroll_to_bottom();
                }
                
                // Channel navigation
                KeyCode::Char('h') | KeyCode::Left => app.select_previous_channel(),
                KeyCode::Char('l') | KeyCode::Right => app.select_next_channel(),
                KeyCode::Tab => app.activate_selected_channel(),
                KeyCode::Char('#') => app.switch_channel("global".to_string()),
                
                // Create DM
                KeyCode::Char('d') => {
                    // Prompt for username (simple implementation)
                    if !app.users.is_empty() {
                        // Use selected user
                        if let Some(user) = app.users.get(app.selected_user) {
                            app.open_dm(user.username.clone());
                        }
                    }
                }
                
                // User selection (for DM creation)
                KeyCode::Char('J') => app.select_next_user(),
                KeyCode::Char('K') => app.select_previous_user(),
                
                _ => {}
            }
        }
        InputMode::Editing => {
            match key {
                // Exit edit mode
                KeyCode::Esc => {
                    app.exit_edit_mode();
                }
                // Send message
                KeyCode::Enter => {
                    let input = app.take_input();
                    if !input.is_empty() {
                        let channel_id = app.active_channel.clone();
                        
                        // Send to network task
                        let _ = command_tx.send(NetworkCommand::SendMessage {
                            content: input.clone(),
                            channel_id: channel_id.clone(),
                        });
                        
                        // Add to local chat immediately (optimistic update)
                        app.add_message(ChatMessage::new(
                            app.username.clone(),
                            input,
                            false,
                        ));
                        
                        // Update telemetry
                        app.telemetry.messages_sent += 1;
                    }
                    app.exit_edit_mode();
                }
                // Character input
                KeyCode::Char(c) => {
                    app.input_char(c);
                }
                // Backspace
                KeyCode::Backspace => {
                    app.input_backspace();
                }
                // Cursor movement
                KeyCode::Left => {
                    app.input_cursor_left();
                }
                KeyCode::Right => {
                    app.input_cursor_right();
                }
                _ => {}
            }
        }
    }

    Ok(())
}

/// Handle network events from the async task
fn handle_network_event(app: &mut App, event: NetworkEvent) {
    match event {
        NetworkEvent::Connected => {
            app.set_connected(true);
        }
        NetworkEvent::Disconnected => {
            app.set_connected(false);
        }
        NetworkEvent::Message { sender, content, timestamp, channel_id } => {
            // Convert Unix timestamp to DateTime
            let datetime = chrono::DateTime::from_timestamp(timestamp, 0)
                .unwrap_or_else(Utc::now);
            
            // Create message with actual timestamp
            let mut msg = ChatMessage::new(sender.clone(), content, false);
            msg.timestamp = datetime;
            
            // Add user to roster if not already there (for user discovery)
            if !app.users.iter().any(|u| u.username == sender) && sender != app.username {
                app.add_user(User::new(sender.clone()));
            }
            
            // Route to the correct channel
            app.add_message_to_channel(&channel_id, msg);
            app.telemetry.messages_received += 1;
            
            // Update user activity
            app.update_user_activity(&sender);
        }
        NetworkEvent::UserJoined { username } => {
            app.add_user(User::new(username));
        }
        NetworkEvent::UserLeft { username } => {
            app.remove_user(&username);
        }
        NetworkEvent::SystemMessage { content } => {
            app.add_message(ChatMessage::system(content));
        }
        NetworkEvent::Error { message } => {
            app.add_message(ChatMessage::system(format!("Error: {}", message)));
        }
    }
}
