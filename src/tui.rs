use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use std::{fs, io, time::Duration};

pub enum MenuAction {
    Quit,
}

enum ActivePane {
    Menu,
    History,
}

pub fn run_tui(mut is_running: bool) -> io::Result<MenuAction> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut menu_state = ListState::default();
    menu_state.select(Some(0));
    
    let mut history_state = ListState::default();
    let mut history_items: Vec<String> = Vec::new();
    let mut history_paths: Vec<String> = Vec::new();
    let mut history_contents: Vec<String> = Vec::new();

    let options = vec![
        "[ RECOVER ]  View History & Recover",
        "[  SETUP  ]  Start Tracking Current Directory",
        "[ DAEMON  ]  Start Foreground Watcher",
        "[ GIT SQUASH ]  Clean Commit History",
        "[ SYSTEM  ]  Install Global Background Daemon",
        "[ DISABLE ]  Stop & Disable Auto-Start Daemon",
        "[  HELP   ]  How to use Chronx",
        "[  EXIT   ]  Exit",
    ];

    let mut right_pane_text = String::new();
    let mut active_pane = ActivePane::Menu;

    let action = loop {
        terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(0),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            let status_text = if is_running {
                Span::styled("RUNNING (Watching in background)", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("STOPPED (Not watching)", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))
            };

            let title = Paragraph::new(Line::from(vec![
                Span::styled("Chronx v1.0.2 - Status: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                status_text,
            ]))
            .block(Block::default().borders(Borders::ALL).title("Chronx Dashboard"));
            f.render_widget(title, chunks[0]);

            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
                .split(chunks[1]);

            let menu_items: Vec<ListItem> = options.iter().map(|label| {
                ListItem::new(Line::from(vec![Span::styled(*label, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]))
            }).collect();

            let mut menu_block = Block::default().borders(Borders::ALL).title("Menu");
            if matches!(active_pane, ActivePane::Menu) {
                menu_block = menu_block.border_style(Style::default().fg(Color::Cyan));
            }

            let list = List::new(menu_items)
                .block(menu_block)
                .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ");
            f.render_stateful_widget(list, main_chunks[0], &mut menu_state);

            let mut right_block = Block::default().borders(Borders::ALL).title("Output");
            if matches!(active_pane, ActivePane::History) {
                right_block = right_block.border_style(Style::default().fg(Color::Cyan));
            }

            if menu_state.selected().unwrap_or(0) == 0 && matches!(active_pane, ActivePane::History) {
                let h_items: Vec<ListItem> = history_items.iter().map(|label| {
                    ListItem::new(Line::from(vec![Span::raw(label)]))
                }).collect();
                let h_list = List::new(h_items)
                    .block(right_block.title("Select to Recover (Esc to go back)"))
                    .highlight_style(Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD))
                    .highlight_symbol(">> ");
                f.render_stateful_widget(h_list, main_chunks[1], &mut history_state);
            } else {
                let detail = Paragraph::new(right_pane_text.as_str())
                    .wrap(Wrap { trim: false })
                    .block(right_block);
                f.render_widget(detail, main_chunks[1]);
            }

            let help = Paragraph::new("Up/Down: Navigate | Enter: Select | Esc: Back/Quit")
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(help, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match active_pane {
                        ActivePane::Menu => {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => break MenuAction::Quit,
                                KeyCode::Down | KeyCode::Char('j') => {
                                    let i = menu_state.selected().unwrap_or(0);
                                    menu_state.select(Some(if i >= options.len() - 1 { 0 } else { i + 1 }));
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    let i = menu_state.selected().unwrap_or(0);
                                    menu_state.select(Some(if i == 0 { options.len() - 1 } else { i - 1 }));
                                }
                                KeyCode::Enter => {
                                    let selected = menu_state.selected().unwrap_or(0);
                                    match selected {
                                        0 => {
                                            history_items.clear();
                                            history_paths.clear();
                                            history_contents.clear();
                                            
                                            let history_dir = std::path::Path::new(".chronx/history");
                                            if history_dir.exists() {
                                                let mut entries: Vec<_> = fs::read_dir(history_dir).unwrap().filter_map(|e| e.ok()).collect();
                                                entries.sort_by_key(|e| std::cmp::Reverse(e.metadata().unwrap().modified().unwrap()));
                                                for entry in entries.iter().take(50) {
                                                    if let Ok(content) = fs::read_to_string(entry.path()) {
                                                        if let Ok(snapshot) = serde_json::from_str::<crate::watcher::Snapshot>(&content) {
                                                            history_items.push(format!("[{}] {} - {}", snapshot.timestamp, snapshot.event_type.to_uppercase(), snapshot.path));
                                                            history_paths.push(snapshot.path);
                                                            history_contents.push(snapshot.content);
                                                        }
                                                    }
                                                }
                                            }
                                            
                                            if history_items.is_empty() {
                                                right_pane_text = "No history found.\nRun [ SETUP ] first and make some file changes!".to_string();
                                            } else {
                                                active_pane = ActivePane::History;
                                                history_state.select(Some(0));
                                            }
                                        }
                                        1 => {
                                            right_pane_text = crate::run_init();
                                        }
                                        2 => {
                                            crate::watcher::start_watching_single_thread(std::env::current_dir().unwrap());
                                            is_running = true;
                                            right_pane_text = "Chronx is now silently watching this directory in the background of this session.\n\nYou can keep using the dashboard, and any file saves will be instantly captured!".to_string();
                                        }
                                        3 => {
                                            right_pane_text = crate::run_squash(&None);
                                        }
                                        4 => {
                                            right_pane_text = crate::run_install_daemon();
                                        }
                                        5 => {
                                            right_pane_text = crate::run_uninstall_daemon();
                                        }
                                        6 => {
                                            right_pane_text = "How to use Chronx:\n\n1. Navigate to any folder and select [ SETUP ].\n2. Select [ DAEMON ] to track changes in the foreground.\n3. Whenever you want to undo a mistake, select [ RECOVER ].\n   You will see a timeline of every save on the right.\n   Select one and press Enter to instantly recover it!".to_string();
                                        }
                                        7 => break MenuAction::Quit,
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                        }
                        ActivePane::History => {
                            match key.code {
                                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace | KeyCode::Left | KeyCode::Char('h') => {
                                    active_pane = ActivePane::Menu;
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    let i = history_state.selected().unwrap_or(0);
                                    history_state.select(Some(if i >= history_items.len() - 1 { 0 } else { i + 1 }));
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    let i = history_state.selected().unwrap_or(0);
                                    history_state.select(Some(if i == 0 { history_items.len() - 1 } else { i - 1 }));
                                }
                                KeyCode::Enter => {
                                    if let Some(i) = history_state.selected() {
                                        let path = &history_paths[i];
                                        let content = &history_contents[i];
                                        if let Err(e) = fs::write(path, content) {
                                            right_pane_text = format!("Failed to recover: {}", e);
                                        } else {
                                            right_pane_text = format!("Successfully recovered: {}", path);
                                        }
                                        active_pane = ActivePane::Menu;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    };

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(action)
}
