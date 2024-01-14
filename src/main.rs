mod term;
mod download;
mod types;
mod parsing;

use std::{path::Path, env};

use anyhow::{Result, anyhow};
use clap::Parser;
use ratatui::{backend::CrosstermBackend, Terminal};
use term::{event::EventHandler, tui::Tui};

static IS_WINDOWS: bool = cfg!(windows);

#[derive(Parser, Debug)]
#[command(author = "Cameron Barnes", version = "1.0", about = None, long_about = None)]
struct Args {
    /// Path to output the downloaded content
    #[arg(short, long, default_value_t = String::from("./library"))]
    out_path: String,
    #[arg(short, long, default_value_t = false)]
    prefer_http: bool,
    #[arg(short, long, default_value_t = String::from(""))]
    plugin_path: String,
    #[arg(short, long, default_value_t = false)]
    direct_json: bool,
}

fn main() -> Result<()> {

    let args = Args::parse();

    // Validate the plugin path
    let mut root = env::current_exe().unwrap();
    root.push("./plugins");
    let mut path = root.as_path();
    if !args.plugin_path.is_empty() {
        path = Path::new(&args.plugin_path);
        if !path.exists() {
            return Err(anyhow!("Plugin path: {} does not exist!", args.plugin_path));
        } else if !path.is_dir() {
            return Err(anyhow!("Plugin path: {} is not a directory", args.plugin_path));
        }
    }
    
    // Get library index
    let library = parsing::load_library(path, args.direct_json)?;

    // Build app object

    // Init term ui
    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.enter()?;

    // Do man program loop
    
    // Close down the term ui stuff cleanly
    tui.exit()?;

    // Download stuff
    let path = args.out_path;
    download::setup_folder(&path)?;
    library.iter().for_each(|cat| {
        download::download_category(&path, cat, args.prefer_http).unwrap(); // Ignore for now
    });

    Ok(())

}
