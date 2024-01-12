mod term;
mod download;
mod types;
mod parsing;

use anyhow::Result;
use clap::Parser;

static IS_WINDOWS: bool = cfg!(windows);

#[derive(Parser, Debug)]
#[command(author = "Cameron Barnes", version = "1.0", about = None, long_about = None)]
struct Args {
    /// Path to output the downloaded content
    #[arg(short, long, default_value_t = String::from("./library"))]
    path: String,
    #[arg(short, long, default_value_t = false)]
    prefer_http: bool,
}

fn main() -> Result<()> {

    let args = Args::parse();
    
    // Get library index
    let library = parsing::load_library()?;
    // Init term ui

    // Build app object

    // Do man program loop
    
    // Close down the term ui stuff cleanly

    // Download stuff
    let path = args.path;
    download::setup_folder(&path)?;
    library.iter().for_each(|cat| {
        download::download_category(&path, cat, args.prefer_http).unwrap(); // Ignore for now
    });

    Ok(())

}
