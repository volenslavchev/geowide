use clap::{Args, Parser, Subcommand};
use geo::{Coord, Rect};
use std::path::PathBuf;
use std::str::FromStr;

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
    pub bbox: Option<CliBBox>,
    /// Visualize the specific shard
    #[arg(long, conflicts_with = "bbox")]
    pub shard: Option<String>,
    /// Visualize the shard this point is in (geojson).
    #[arg(long, conflicts_with = "shard")]
    pub point: Option<CliPoint>,
    // Visualize the already indexed points - they will be in green.
    #[arg(long, default_value = "false")]
    pub indexed: bool,
    /// Where to save the visualisation (as an .html file)
    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Args)]
pub struct IndexArgs {
    /// The shard to index. Can be found via the "visualize" command.
    #[arg(long)]
    pub shard: String,
    /// Only index a specific amount of points. Keep in mind every point
    /// contains 4 images.
    #[arg(long)]
    pub amount: Option<usize>,
    /// Amount of images to index per batch.
    #[arg(long, default_value = "80")]
    pub batch_size: usize,
    /// Google maps API key. If provided, the google maps API
    /// will be used. Can also be in environment (GOOGLE_API_KEY)
    #[arg(long)]
    pub google_key: Option<String>,
    /// Mapillary API key. If proivded, the mapillary API
    /// will be used. Both the google and mapillary keys can be
    /// provided. The indexer will prioritize the mapillary API
    /// when possible. Can also be in environment (MAPILLARY_API_KEY)
    #[arg(long)]
    pub mapillary_key: Option<String>,
}

#[derive(Args)]
pub struct ServeArgs {
    #[arg(long, default_value = "80")]
    pub port: usize,
}

#[derive(Args)]
pub struct LocateArgs {
    pub path: PathBuf,
    /// If provided, a html file will be saved with the top 10 locations
    /// mapped out
    #[arg(long)]
    pub out: Option<PathBuf>
}

#[derive(Subcommand)]
pub enum Commands {
    Visualize(VisualizeArgs),
    Index(IndexArgs),
    Serve(ServeArgs),
    Locate(LocateArgs)
}

#[derive(Debug, Clone)]
pub struct CliPoint(pub Coord);

impl FromStr for CliPoint {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err("Point must be in format 'lon,lat'".to_string());
        }

        let x = parts[0].parse::<f64>().map_err(|_| "Invalid longitude")?;
        let y = parts[1].parse::<f64>().map_err(|_| "Invalid latitude")?;

        Ok(CliPoint(Coord { x, y }))
    }
}

#[derive(Debug, Clone)]
pub struct CliBBox(pub Rect);

impl FromStr for CliBBox {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 4 {
            return Err("BBox must be in format 'min_lon,min_lat,max_lon,max_lat'".to_string());
        }

        let min_x = parts[0].parse::<f64>().map_err(|_| "Invalid min_lon")?;
        let min_y = parts[1].parse::<f64>().map_err(|_| "Invalid min_lat")?;
        let max_x = parts[2].parse::<f64>().map_err(|_| "Invalid max_lon")?;
        let max_y = parts[3].parse::<f64>().map_err(|_| "Invalid max_lat")?;

        let min = Coord { x: min_x, y: min_y };
        let max = Coord { x: max_x, y: max_y };

        Ok(CliBBox(Rect::new(min, max)))
    }
}
