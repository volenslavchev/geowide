use crate::{cli::LocateArgs, db::IndexDatabase, model::Model, utils::get_model_default_paths};
use colored::Colorize;
use geo::Coord;
use geometry::visualize::{MapVisualizer, Style};
use image::io::Reader;

pub async fn locate_command(args: LocateArgs) -> anyhow::Result<()> {
    let client = IndexDatabase::new().await?;
    let model = get_model_default_paths()?;

    let input_image = Reader::open(args.path)?
        .with_guessed_format()?
        .decode()?
        .into_rgb8();
    let image_embeddings = model.transform(&input_image)?;

    println!("Performing search...");
    let found_results = client.search(image_embeddings, 10).await?;

    println!("Found {} results", found_results.len().to_string().blue());

    for result in &found_results {
        println!(
            "Score: {}, Coordinates: {},{}",
            if result.score > 0.3 {
                result.score.to_string().bright_green()
            } else {
                result.score.to_string().blue()
            },
            result.data.lat.to_string().blue(),
            result.data.lon.to_string().blue()
        );
    }

    if let Some(out_path) = args.out {
        let mut visualizer = MapVisualizer::new();

        for result in &found_results {
            visualizer.add_point(
                &Coord {
                    x: result.data.lon,
                    y: result.data.lat,
                },
                Style::default()
                    .fill_color(if result.score > 0.3 { "#00f518" } else { "#006cbf" })
                    .tooltip(&format!("Score: {:.4}", result.score)),
            );
        }

        visualizer.write_html(out_path)?;
    }

    Ok(())
}
