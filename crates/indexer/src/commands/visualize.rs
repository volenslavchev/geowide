use geometry::{
    shard::{Shard, ShardCluster},
    visualize::MapVisualizer,
};

use crate::cli::VisualizeArgs;

// cargo run visualize --out ./test.html --bbox 27.84536361694336,43.19541792979731,27.935142517089847,43.2490775089937
pub async fn visualize_command(args: VisualizeArgs) -> anyhow::Result<()> {
    // TODO: Handle case where we already have the roads in the DB
    // - Get all the segments we already have in the database
    // - Color them differently to show that they've been finished

    let shards = if let Some(point) = args.point {
        Shard::from_coords(point.0).await?.into_shard_cluster()
    } else if let Some(bbox) = args.bbox {
        ShardCluster::from_bbox(bbox.0).await?
    } else if let Some(shard_idx) = args.shard {
        Shard::from_cell_str(&shard_idx).await?.into_shard_cluster()
    } else if let Some(query) = args.region {
        ShardCluster::from_query(&query).await?
    } else {
        panic!("No shard argument provided.")
    };

    MapVisualizer::new().draw(&shards).write_html(&args.out)?;

    let total_roads: usize = shards.0.iter().map(|s| s.paths.len()).sum();
    let total_segments: usize = shards
        .0
        .iter()
        .flat_map(|s| s.paths.iter().map(|p| p.segments.0.len()))
        .sum();
    let total_pics_needed = total_segments * 4;

    println!(
        "Visualization file: {:?}\nTotal number of roads: {total_roads}\nTotal number of segments: {total_segments}\nTotal pictures needed: {total_pics_needed}",
        args.out.canonicalize().unwrap()
    );

    Ok(())
}
