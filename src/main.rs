use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::io::{self, stdout};

fn main() -> io::Result<()> {
    enable_raw_mode()?; // Enable raw mode for terminal input
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?; // Switch to alternate screen

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Define menu items
    let menu_items = vec![
        ListItem::new("Linux version"),
        ListItem::new("Apt update"),
        ListItem::new("Apt upgrade"),
        ListItem::new("Quit"),
    ];
    let mut list_state = ListState::default();
    list_state.select(Some(0)); // Select the first item by default

    let mut should_quit = false;
    let mut output_lines: Vec<String> = vec![]; // To store command output

    // Main event loop
    while !should_quit {
        terminal.draw(|f| {
            // Main layout without padding
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Min(20), // Menu area
                        Constraint::Percentage(80),  // Output area
                    ]
                    .as_ref(),
                )
                .split(f.area());

            // Menu block
            let block = Block::default().title(" Menu ").borders(Borders::ALL);
            let list = List::new(menu_items.clone())
                .block(block)
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");
            f.render_stateful_widget(list, chunks[0], &mut list_state);

            // Output block 80% of the area.
            let output = Paragraph::new(output_lines.join("\n"))
                .block(Block::default().title(" Output ").borders(Borders::ALL));
            f.render_widget(output, chunks[1]); // Render inside padded chunk
        })?;

        // Handle key events
        if let Event::Key(key) = event::read()? {
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
                            if i >= menu_items.len() - 1 { 0 } else { i + 1 }
                        }
                        None => 0,
                    };
                    list_state.select(Some(i));
                }
                KeyCode::Up => {
                    // Move selection up
                    let i = match list_state.selected() {
                        Some(i) => {
                            if i == 0 { menu_items.len() - 1 } else { i - 1 }
                        }
                        None => 0,
                    };
                    list_state.select(Some(i));
                }
                KeyCode::Enter => match list_state.selected() {
                    // Execute selected command
                    Some(0) => {
                        // Get Linux version
                        let output = std::process::Command::new("bash")
                            .arg("-c")
                            .arg("cat /etc/os-release")
                            .output()
                            .expect("Failed to execute command");
                        
                        output_lines.clear(); // Clear old output
                        for line in String::from_utf8_lossy(&output.stdout).lines() {
                            output_lines.push(line.to_string()); // Save output to display later
                        }
                    }
                    Some(1) => {
                        // Run apt update
                        let output = std::process::Command::new("bash")
                            .arg("-c")
                            .arg("sudo apt update")
                            .output()
                            .expect("Failed to execute command");

                        output_lines.clear(); // Clear old output
                        for line in String::from_utf8_lossy(&output.stdout).lines() {
                            output_lines.push(line.to_string()); // Save output to display later
                        }
                    }
                    Some(2) => {
                        // Run apt upgrade
                        let child = std::process::Command::new("bash")
                            .arg("-c")
                            .arg("sudo apt upgrade -y")
                            .output()
                            .expect("Failed to execute command");

                        output_lines.clear(); // Clear old output
                        for line in String::from_utf8_lossy(&child.stdout).lines() {
                            output_lines.push(line.to_string()); // Save output to display later
                        }
                    }
                    Some(3) => should_quit = true, // Quit the application
                    _ => {}
                },
                _ => {}
            }
        }
    }

    disable_raw_mode()?; // Disable raw mode before exiting
    terminal.backend_mut().execute(LeaveAlternateScreen)?; // Leave the alternate screen
    Ok(())
}
