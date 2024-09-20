// src/main.rs
use clap::{Parser, Subcommand};
use colored::*;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs},
    Terminal,
};

#[derive(Parser)]
#[command(name = "Commitaura")]
#[command(about = "Intelligent Git Commit Assistant with README Integration", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Commit with a message
    Commit {
        /// Commit message
        #[arg(short, long)]
        message: String,
    },
    /// Update README based on changes
    UpdateReadme,
    /// Commit and update README
    CommitAndUpdate {
        /// Commit message
        #[arg(short, long)]
        message: String,
    },
}

enum InputMode {
    Normal,
    Editing,
}

struct App<'a> {
    tabs: Vec<&'a str>,
    selected_tab: usize,
    input: String,
    input_mode: InputMode,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            tabs: vec!["Commit", "Update README", "Commit & Update"],
            selected_tab: 0,
            input: String::new(),
            input_mode: InputMode::Normal,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Commit { message } => {
            // Implement commit logic here
            println!(
                "{}",
                format!("Committing with message: {}", message).green()
            );
        }
        Commands::UpdateReadme => {
            // Implement README update logic here
            println!("{}", "Updating README based on changes...".blue());
        }
        Commands::CommitAndUpdate { message } => {
            // Implement commit and README update logic here
            // Initialize TUI
            enable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;

            let app = App::new();
            let res = run_app(&mut terminal, app, &message);

            // Restore terminal
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;

            if let Err(err) = res {
                println!("Error: {:?}", err);
            }

            println!(
                "{}",
                "Commit and README update completed successfully!".green()
            );
        }
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App, message: &str) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            // Define layout
            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            // Header with Tabs
            let titles = app
                .tabs
                .iter()
                .map(|t| {
                    Spans::from(Span::styled(
                        *t,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ))
                })
                .collect::<Vec<Spans>>();

            let tabs = Tabs::new(titles)
                .select(app.selected_tab)
                .block(Block::default().borders(Borders::ALL).title("Commitaura"))
                .highlight_style(
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )
                .style(Style::default().fg(Color::White))
                .divider(Span::raw("|"));

            f.render_widget(tabs, chunks[0]);

            // Main Content
            let info = match app.selected_tab {
                0 => format!("Commit Message:\n[bold green]{}[/bold green]", message),
                1 => "Updating README based on changes...".to_string(),
                2 => format!(
                    "Committing and updating README with message:\n[bold green]{}[/bold green]",
                    message
                ),
                _ => "".to_string(),
            };

            let paragraph = Paragraph::new(info)
                .block(Block::default().borders(Borders::ALL).title("Info"))
                .alignment(Alignment::Left);

            f.render_widget(paragraph, chunks[1]);

            // Footer
            let footer = Paragraph::new("Press 'q' to exit")
                .block(Block::default().borders(Borders::ALL).title("Footer"))
                .alignment(Alignment::Center);

            f.render_widget(footer, chunks[2]);
        })?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let CEvent::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Left => {
                        if app.selected_tab > 0 {
                            app.selected_tab -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if app.selected_tab < app.tabs.len() - 1 {
                            app.selected_tab += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
