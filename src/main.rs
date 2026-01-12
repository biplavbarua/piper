use anyhow::Result;
use std::{io, time::Duration};
use std::path::PathBuf;
use clap::Parser;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

mod app;
mod compressor;
mod scanner;
mod ui;
mod config;

use app::App;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to scan for optimization (default: ~/Developer)
    #[arg(short, long)]
    scan: Option<String>,

    /// Path to configuration file
    #[arg(short, long)]
    config: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let scan_path = if let Some(path_str) = args.scan {
        PathBuf::from(path_str)
    } else {
        // Default: ~/Developer
        match dirs::home_dir() {
            Some(mut p) => {
                p.push("Developer");
                p
            },
            None => PathBuf::from("."), // Fallback
        }
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app with path
    let mut app = App::new(scan_path);

    // Run app
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> 
where
    <B as Backend>::Error: Send + Sync + 'static,
{
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
                // Handle other keys
                app.handle_input(key.code);
            }
        }
        
        // Handle background updates here if needed
        app.tick();
    }
}
