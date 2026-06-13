use std::sync::Arc;

use colored::Colorize;
use futures::StreamExt;
use geo::Coord;
use geometry::{
    shard::Shard,
    sources::{
        DataSource, ImageSet, mapillary::MapillaryDataSource, street_view::GoogleStreetViewSource,
    },
};
use tokio::sync::mpsc;

use crate::{
    cli::IndexArgs,
    db::{ImagePoint, IndexDatabase},
    model::{MegaLoc, Model},
};

pub async fn index_command(args: IndexArgs) -> anyhow::Result<()> {
    let mut sources: Vec<Box<dyn DataSource>> = Vec::new();

    if let Some(mapillary) = args.mapillary_key {
        sources.push(Box::new(MapillaryDataSource::new(&mapillary)));
    }

    if let Some(google) = args.google_key {
        sources.push(Box::new(GoogleStreetViewSource::new(&google)));
    }

    if sources.len() == 0 {
        anyhow::bail!("At least one API key needs to be provided");
    }

    sources.sort_by(|a, b| a.priority().cmp(&b.priority()));

    println!("{}", "Fetching shard data...".blue());
    let shard = Shard::from_cell_str(&args.shard).await?;

    println!("{}", "Initializing sources...".blue());
    for source in &mut sources {
        source.load_bounds(&shard.cell.into()).await?;
    }

    println!("{}", "Connecting to qdrant database...".blue());
    let client = IndexDatabase::new().await?;

    println!("{}", "Loading MegaLoc model...".blue());
    let model = MegaLoc::new(
        &std::env::var("MODEL_PATH").unwrap_or("./model_export/megaloc.onnx".to_string()),
    )?;

    println!("{}", "Getting all saved points".blue());
    let all_saved_images = client.get_all_points().await?;
    let all_coords: Vec<(Coord, u64)> = shard
        .paths
        .iter()
        .flat_map(|p| p.segments.0.iter().map(|c| (*c, p.id)))
        .collect();

    let all_coords_len = all_coords.len();

    let coords_that_need_fetching: Vec<(Coord, u64)> = all_coords
        .into_iter()
        .filter(|(coord, _)| all_saved_images.get_at(*coord, 6.0).is_none())
        .collect();

    let coords_that_need_fetching_len = coords_that_need_fetching.len();

    println!(
        "Total saved points (est): {} / Total shard points: {} / Points to fetch: {} / Images to fetch: {}",
        (all_saved_images.size() / 4).to_string().blue(),
        all_coords_len.to_string().blue(),
        coords_that_need_fetching_len.to_string().blue(),
        (coords_that_need_fetching_len * 4).to_string().blue()
    );

    let (image_tx, mut image_rx) = mpsc::channel::<(String, Coord, u64, ImageSet)>(30);
    let sources_shared = Arc::new(sources);

    tokio::spawn(async move {
        futures::stream::iter(coords_that_need_fetching)
            .for_each_concurrent(4, |(coord, way)| {
                let tx_clone = image_tx.clone();
                let sources = sources_shared.clone();

                async move {
                    for source in sources.iter() {
                        match source.get_at(coord, 14.0).await {
                            Ok(Some(found_image)) => {
                                let _ = tx_clone
                                    .send((source.id().to_owned(), coord, way, found_image))
                                    .await;
                                break;
                            }
                            Ok(None) => continue,
                            Err(err) => {
                                println!(
                                    "Encountered an error while fetching {:?}: {:?}",
                                    coord, err
                                );
                            }
                        }
                    }
                }
            })
            .await;
    });

    let mut current_batch = Vec::new();
    let mut total_images_processed = 0;

    let model = Arc::new(model);
    while let Some((source, coord, way, image_set)) = image_rx.recv().await {
        let mut inference_tasks = Vec::new();

        for (ind, image) in image_set.pieces.into_iter().enumerate() {
            let model_clone = model.clone();

            inference_tasks.push(tokio::task::spawn_blocking(move || {
                let embeddings = model_clone.transform(&image)?;
                Ok::<_, anyhow::Error>((ind, embeddings))
            }));
        }

        total_images_processed += inference_tasks.len();

        for task in inference_tasks {
            match task.await.unwrap() {
                Ok((ind, embeddings)) => current_batch.push((
                    ImagePoint {
                        way_id: way,
                        ilon: coord.x,
                        ilat: coord.y,
                        lon: image_set.actual_coords.x,
                        lat: image_set.actual_coords.y,
                        seq_id: ind as u32,
                        source: source.clone(),
                    },
                    embeddings,
                )),
                Err(err) => println!("Error embedding image at {:?}: {:?}", coord, err),
            }
        }

        if current_batch.len() >= args.batch_size {
            client
                .bulk_insert_points(std::mem::take(&mut current_batch))
                .await?;

            println!(
                "Processed batch. Total processed images: {} / images left total: {}",
                total_images_processed.to_string().blue(),
                ((coords_that_need_fetching_len as i32) - (total_images_processed as i32))
                    .max(0)
                    .to_string()
                    .blue()
            );

            if let Some(limit) = args.amount {
                if total_images_processed >= limit {
                    println!(
                        "Requested amount limit ({}) reached. Shutting down gracefully...",
                        limit
                    );
                    break;
                }
            }
        }
    }

    if !current_batch.is_empty() {
        client
            .bulk_insert_points(std::mem::take(&mut current_batch))
            .await?;
    }

    println!(
        "{} Processed {} valid images.",
        "Indexing complete!".green(),
        total_images_processed.to_string().blue()
    );

    Ok(())
}
