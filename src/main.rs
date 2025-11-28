use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};
use std::{
    io::{self, stdout, Write},
    process::{Command, Stdio},
};

const PASSWORD_PROMPT: &str = "sudo password: ";

#[derive(Clone, Copy)]
enum MenuAction {
    LinuxVersion,
    PackageManager,
    AptUpdate,
    AptUpgrade,
    Quit,
}

enum UiMode {
    Normal,
    PasswordPrompt { action: MenuAction, input: String },
}

fn title() -> &'static str {
    r#"Linux menu"#
}

fn main() -> io::Result<()> {
    enable_raw_mode()?; // Enable raw mode for terminal input
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?; // Switch to alternate screen

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?; // Hide cursor until we need to accept text input

    let menu_entries = [("Linux version", MenuAction::LinuxVersion),
        ("Package manager", MenuAction::PackageManager),
        ("Apt update", MenuAction::AptUpdate),
        ("Apt upgrade", MenuAction::AptUpgrade),
        ("Quit", MenuAction::Quit)];
    let menu_items: Vec<ListItem> = menu_entries
        .iter()
        .map(|(label, _)| ListItem::new(*label))
        .collect();
    let mut list_state = ListState::default();
    list_state.select(Some(0)); // Select the first item by default

    let mut should_quit = false;
    let mut output_lines: Vec<String> = vec![]; // To store command output
    let mut ui_mode = UiMode::Normal;

    // Main event loop
    while !should_quit {
        let wants_cursor = matches!(ui_mode, UiMode::PasswordPrompt { .. });
        terminal.draw(|f| {
            // First split: Vertical split between the Info (top 20%) and the rest (80%)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3), // Info area (1 line)
                        Constraint::Min(0),    // Bottom area (Menu and Output)
                    ]
                    .as_ref(),
                )
                .split(f.area());

            // Second split: Horizontal split between the Menu and Output (after Info)
            let bottom_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(20), // Menu area (left side)
                        Constraint::Percentage(80), // Output area (right side)
                    ]
                    .as_ref(),
                )
                .split(chunks[1]); // Split the bottom 80% of the screen

            // Info block at the top
            let info_block = Paragraph::new(title())
                .style(
                    Style::default()
                        .bg(Color::Cyan)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().bg(Color::Cyan).fg(Color::Black))
                        .border_type(BorderType::Thick),
                ) // Use thick border type
                .alignment(Alignment::Center);
            f.render_widget(info_block, chunks[0]);

            // Menu block on the left side
            let menu_block = Block::default()
                .title(" Menu ")
                .borders(Borders::ALL)
                .border_type(BorderType::Thick);
            let list = List::new(menu_items.clone())
                .block(menu_block)
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("  ");
            f.render_stateful_widget(list, bottom_chunks[0], &mut list_state);

            // Output block on the right side
            let mut output_text = output_lines.join("\n");
            if let UiMode::PasswordPrompt { input, .. } = &ui_mode {
                if !output_text.is_empty() {
                    output_text.push('\n');
                }
                output_text.push_str(PASSWORD_PROMPT);
                output_text.push_str(&"*".repeat(input.chars().count()));
            }
            let output_block = Paragraph::new(output_text).block(
                Block::default()
                    .title(" Output ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick),
            );
            f.render_widget(output_block, bottom_chunks[1]);

            if let UiMode::PasswordPrompt { input, .. } = &ui_mode {
                let cursor_x = bottom_chunks[1].x
                    + PASSWORD_PROMPT.len() as u16
                    + input.chars().count() as u16
                    + 1;
                let cursor_y = bottom_chunks[1].y + output_lines.len() as u16 + 1;
                f.set_cursor_position((cursor_x, cursor_y));
            }
        })?;

        if wants_cursor {
            terminal.show_cursor()?;
        } else {
            terminal.hide_cursor()?;
        }

        // Handle key events
        if let Event::Key(key) = event::read()? {
            if let UiMode::PasswordPrompt { action, input } = &mut ui_mode {
                let mut submit: Option<(MenuAction, String)> = None;
                match key.code {
                    KeyCode::Enter => {
                        submit = Some((*action, input.clone()));
                    }
                    KeyCode::Esc => {
                        input.clear();
                        ui_mode = UiMode::Normal;
                        output_lines.push("Cancelled sudo prompt.".into());
                    }
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Char(c) => {
                        input.push(c);
                    }
                    _ => {}
                }

                if let Some((action_to_run, password)) = submit {
                    ui_mode = UiMode::Normal;
                    run_menu_action(action_to_run, Some(password), &mut output_lines);
                }

                continue;
            }

            match key.code {
                KeyCode::Char('q') => {
                    should_quit = true; // Quit the application
                }
                KeyCode::Char('c') => {
                    output_lines.clear(); // Clear old output
                }
                KeyCode::Down => {
                    // Move selection down
                    let i = match list_state.selected() {
                        Some(i) => {
                            if i >= menu_items.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    list_state.select(Some(i));
                }
                KeyCode::Up => {
                    // Move selection up
                    let i = match list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                menu_items.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    list_state.select(Some(i));
                }
                KeyCode::Enter => {
                    if let Some(index) = list_state.selected() {
                        let action = menu_entries[index].1;
                        match action {
                            MenuAction::Quit => should_quit = true,
                            _ if requires_sudo(action) => {
                                output_lines.clear();
                                ui_mode = UiMode::PasswordPrompt {
                                    action,
                                    input: String::new(),
                                };
                            }
                            _ => {
                                run_menu_action(action, None, &mut output_lines);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?; // Disable raw mode before exiting
    terminal.backend_mut().execute(LeaveAlternateScreen)?; // Leave the alternate screen
    Ok(())
}

fn requires_sudo(action: MenuAction) -> bool {
    matches!(
        action,
        MenuAction::LinuxVersion | MenuAction::AptUpdate | MenuAction::AptUpgrade
    )
}

fn run_menu_action(action: MenuAction, password: Option<String>, output_lines: &mut Vec<String>) {
    output_lines.clear();
    match execute_action(action, password.as_deref()) {
        Ok(lines) => {
            output_lines.extend(lines);
        }
        Err(err) => {
            output_lines.push(format!("Failed to execute command: {}", err));
        }
    }
}

fn execute_action(action: MenuAction, password: Option<&str>) -> io::Result<Vec<String>> {
    match action {
        MenuAction::LinuxVersion => run_command("./scripts/check_distro.sh", true, password),
        MenuAction::PackageManager => run_command("sh ./scripts/check_pmg.sh", false, None),
        MenuAction::AptUpdate => run_command("apt update 2>&1", true, password),
        MenuAction::AptUpgrade => run_command("apt upgrade -y 2>&1", true, password),
        MenuAction::Quit => Ok(vec![]),
    }
}

fn run_command(
    command: &str,
    requires_sudo: bool,
    password: Option<&str>,
) -> io::Result<Vec<String>> {
    let output = if requires_sudo {
        let password =
            password.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Password required"))?;
        let mut child = Command::new("sudo")
            .arg("-S")
            .arg("-p")
            .arg("")
            .arg("bash")
            .arg("-c")
            .arg(command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(password.as_bytes())?;
            stdin.write_all(b"\n")?;
        }

        child.wait_with_output()?
    } else {
        Command::new("bash")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?
    };

    let mut lines: Vec<String> = Vec::new();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        lines.push(line.to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    for line in stderr.lines() {
        lines.push(line.to_string());
    }

    if lines.is_empty() {
        lines.push(String::from("Command finished with no output."));
    }

    Ok(lines)
}
