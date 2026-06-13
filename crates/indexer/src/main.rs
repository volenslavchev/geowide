use clap::Parser;

use crate::cli::Commands;

pub mod cli;
pub mod commands;
pub mod db;
pub mod model;
pub mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Err(_) = dotenvy::dotenv() {
        println!(".env file was not found.")
    }

    let cli = cli::Cli::parse();

    match cli.command {
        Commands::Visualize(args) => commands::visualize::visualize_command(args).await,
        Commands::Index(args) => commands::index::index_command(args).await,
        Commands::Serve(args) => commands::serve::serve_command(args),
        Commands::Locate(args) => commands::locate::locate_command(args).await,
    }
    // let mut model = MegaLoc::new("./model_export/megaloc.onnx")?;

    // let varna_1 = image::open("test_images/image1.jpg")?.to_rgb8();
    // let varna_2 = image::open("test_images/image1_similar2.png")?.to_rgb8();

    // println!("Embedding varna 1...");
    // let varna_1_embedding = model.transform(&varna_1)?;
    // println!("Embedding varna 2...");
    // let varna_2_embedding = model.transform(&varna_2)?;

    // let sim = cosine_similarity(&varna_1_embedding, &varna_2_embedding);
    // println!("Similarity: {:.4}", sim);
    // Ok(())
}
