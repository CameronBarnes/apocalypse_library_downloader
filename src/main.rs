mod download;
mod parsing;
mod term;
mod types;

use std::{env, path::Path};

use anyhow::{anyhow, Result};
use clap::Parser;
use once_cell::sync::Lazy;
use ratatui::{backend::CrosstermBackend, Terminal};
use term::{app::App, event::EventHandler, tui::Tui, update::update};

static IS_WINDOWS: bool = cfg!(windows);
static HAS_RSYNC: Lazy<bool> = Lazy::new(download::check_for_rsync);

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
            return Err(anyhow!(
                "Plugin path: {} is not a directory",
                args.plugin_path
            ));
        }
    }

    // Get library index
    let library = parsing::load_library(path, args.direct_json)?;

    // Build app object
    let mut app = App::new(library);

    // Init term ui
    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.enter()?;

    // Do man program loop
    while !app.should_quit {
        tui.draw(&mut app)?;

        match tui.events.next()? {
            term::event::Event::Tick => app.tick(),
            term::event::Event::Key(key_event) => update(&mut app, key_event),
            term::event::Event::Mouse(_) => {}
            term::event::Event::Resize(_, _) => {}
            term::event::Event::FocusGained => {}
            term::event::Event::FocusLost => {}
        }
    }

    // Close down the term ui stuff cleanly
    tui.exit()?;

    // Download stuff
    let path = args.out_path;
    if app.download {
        download::setup_folder(&path)?;
        app.category.items.iter().for_each(|item| {
            download::download_item(&path, item, args.prefer_http).unwrap(); // Ignore for now
        });
    }

    Ok(())
}
