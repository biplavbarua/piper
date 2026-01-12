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

mod ui;
mod config;
mod spyder;

use app::App;
use config::Config;

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
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let config = if let Some(config_path) = &args.config {
        Config::load_from_file(config_path).ok()
    } else {
        None
    };

    let scan_path = args.scan
        .or_else(|| config.as_ref().and_then(|c| c.scan.clone()))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            // Default: ~/Developer
            match dirs::home_dir() {
                Some(mut p) => {
                    p.push("Developer");
                    p
                },
                None => PathBuf::from("."), // Fallback
            }
        });

    let compression_level = config.as_ref()
        .and_then(|c| c.compression_level)
        .unwrap_or(15); // Default Middle-Out Level

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app with path
    let mut app = App::new(scan_path, compression_level);

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
