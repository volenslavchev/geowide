use std::collections::HashSet;

use colored::Colorize;
use geo::Coord;
use geometry::{
    shard::{Shard, ShardCellIndex, ShardCluster},
    visualize::{MapVisualizer, Style},
};

use crate::{cli::VisualizeArgs, db::IndexDatabase};

pub async fn visualize_command(args: VisualizeArgs) -> anyhow::Result<()> {
    let db_image_points = if args.indexed {
        println!(
            "{}",
            "Initializing qdrant and getting saved points...".blue()
        );
        let db = IndexDatabase::new().await?.get_all_points().await?;
        let points = db.iter().cloned().collect::<Vec<_>>();
        Some(points)
    } else {
        None
    };

    println!("{}", "Fetching shard data...".blue());

    let shards = if let Some(point) = args.point {
        Shard::from_coords(point.0).await?.into_shard_cluster()
    } else if let Some(bbox) = args.bbox {
        ShardCluster::from_bbox(bbox.0).await?
    } else if let Some(shard_idx) = args.shard {
        Shard::from_cell_str(&shard_idx).await?.into_shard_cluster()
    } else if let Some(query) = args.region {
        ShardCluster::from_query(&query).await?
    } else if let Some(indexed_points) = &db_image_points {
        let mut shards: HashSet<ShardCellIndex> = HashSet::new();
        for point in indexed_points {
            shards.insert(ShardCellIndex::from_coords(Coord {
                x: point.ilon,
                y: point.ilat,
            })?);
        }
        ShardCluster::from_cells(&shards.into_iter().collect::<Vec<_>>()).await?
    } else {
        panic!("No shard argument provided.")
    };

    println!("{}", "Fetched shards, drawing visualization...".blue());

    let mut visualizer = MapVisualizer::new();
    visualizer.draw(&shards);

    let total_roads: usize = shards.0.iter().map(|s| s.paths.len()).sum();
    let total_segments: usize = shards
        .0
        .iter()
        .flat_map(|s| s.paths.iter().map(|p| p.segments.0.len()))
        .sum();
    let total_pics_needed = total_segments * 4;

    println!(
        "Total number of roads: {}\nTotal number of points: {}\nTotal images needed: {}",
        total_roads.to_string().blue(),
        total_segments.to_string().blue(),
        total_pics_needed.to_string().blue()
    );

    if let Some(points) = db_image_points {
        println!(
            "Points indexed: {}\nPoints left to index: {}",
            (points.len() / 4).to_string().bright_green(),
            (total_pics_needed - points.len()).to_string().bright_red()
        );

        for point in points {
            visualizer.add_point(
                &Coord {
                    x: point.ilon,
                    y: point.ilat,
                },
                Style::default().fill_color("#00f518"),
            );
        }
    }

    visualizer.write_html(&args.out)?;

    Ok(())
}
