use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Args)]
pub struct VisualizeArgs {
    /// Visualize the shards in the given region (city, country, etc.)
    #[arg(long)]
    pub region: Option<String>,
    /// Visualize the shards in the given bbox (geojson).
    #[arg(long, conflicts_with = "region")]
    pub bbox: Option<String>,
    /// Visualize the specific shard
    #[arg(long, conflicts_with = "bbox")]
    pub shard: Option<String>,
    /// Visualize the shard this point is in (geojson).
    #[arg(long, conflicts_with = "shard")]
    pub point: Option<String>,
    /// Visualize all the points inside each shard
    #[arg(long)]
    pub points: bool,
    /// Where to save the visualisation (as an .html file)
    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Args)]
pub struct IndexArgs {
    /// The shard to index
    #[arg(long)]
    pub shard: String,
    /// Google maps API key. If provided, the google maps API
    /// will be used
    #[arg(long)]
    pub google_key: Option<String>,
    /// Mapillary API key. If proivded, the mapillary API
    /// will be used. Both the google and mapillary keys can be
    /// provided. The indexer will prioritize the mapillary API
    /// when possible
    #[arg(long)]
    pub mapillary_key: Option<String>,
}

#[derive(Args)]
pub struct ServeArgs {
    #[arg(long, default_value = "80")]
    port: usize,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Visualize indexer data. By default it shows the indexer's data, however
    /// if any arguments are provided, the data from them will be shown instead
    Visualize(VisualizeArgs),
    Index(IndexArgs),
    Serve(ServeArgs),
}
