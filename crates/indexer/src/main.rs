use clap::Parser;

use crate::cli::Commands;

pub mod cli;
pub mod commands;
pub mod db;
pub mod model;
pub mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = rustls::crypto::ring::default_provider().install_default();

    if let Err(_) = dotenvy::dotenv() {
        println!(".env file was not found.")
    }

    let cli = cli::Cli::parse();

    match cli.command {
        Commands::Visualize(args) => commands::visualize::visualize_command(args).await,
        Commands::Index(args) => commands::index::index_command(args).await,
        Commands::Serve(args) => commands::serve::serve_command(args).await,
        Commands::Locate(args) => commands::locate::locate_command(args).await,
    }
}
