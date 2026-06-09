mod imageutils;
mod osm;
pub mod shard;
pub mod sources;
pub mod visualize;

#[cfg(test)]
mod tests {
    use std::{str::FromStr, time::Duration};

    use crate::{
        osm::OsmConfig,
        shard::{Shard, ShardCluster},
        sources::{self, DataSource, street_view::GoogleStreetViewSource},
        visualize::{Label, MapVisualizer, Style},
    };
    use geo::{BallTreePoint, Centroid, Coord, Rect};
    use h3o::CellIndex;

    #[tokio::test]
    async fn it_works2() {
        // let shards = Shard::from_query("Varna, Bulgaria").await.unwrap();

        // (Almost) entire Varna
        let shards = ShardCluster::from_bbox(Rect::new(
            Coord {
                x: 27.84536361694336,
                y: 43.19541792979731,
            },
            Coord {
                x: 27.935142517089847,
                y: 43.2490775089937,
            },
        ))
        .await
        .unwrap();

        // let shards = Shard::from_cell_str("881eebb829fffff")
        //     .await
        //     .unwrap()
        //     .into_shard_cluster();

        let mut total_count = 0;
        for shard in &shards.0 {
            println!("Fetching road...");
            total_count += shard
                .paths
                .iter()
                .map(|p| p.segments.0.len())
                .sum::<usize>();
        }
        println!("Total images: {}", total_count);

        MapVisualizer::new()
            .draw(&shards)
            //.add_coords(&points_with_imgs, Style::default().fill_color("#49ff30"))
            .write_html("./paths_debug_map.html")
            .unwrap();
    }

    #[tokio::test]
    async fn it_works() {
        // Varna original
        // let shard = Shard::from_coords(Coord {
        //     x: 27.9096873,
        //     y: 43.2145027,
        // })
        // .unwrap();

        // Varna center 2
        // let shard = Shard::from_coords(
        //     Coord {
        //         x: 27.912309,
        //         y: 43.204428,
        //     },
        //     &OsmConfig {
        //         segment_ideal_distance: 25.0,
        //         segment_min_distance: 15.0,
        //     },
        // )
        // .await
        // .unwrap();

        // Frankfurt
        // let shard = Shard::from_coords(Coord {
        //     // 50.1201244,8.6704001
        //     x: 8.6704001,
        //     y: 50.1201244,
        // })
        // .unwrap();

        // let mut mapillary = sources::mapillary::MapillaryDataSource::new(
        //     "MLY|26044754875173911|9c90a62549d628513a9edb99ab0f4160",
        // );
        // mapillary.load_bounds(&shard.bounds).await.unwrap();

        // let points_with_imgs = roads
        //     .iter()
        //     .flat_map(|r| r.segments.clone().into_points())
        //     .map(|p| p.coord())
        //     .filter(|point| mapillary.get_images_within_radius(*point, 6.0).is_some())
        //     .collect::<Vec<_>>();

        // let required_imgs: usize = roads.iter().map(|r| r.segments.0.len()).sum();
        // println!(
        //     "Required images: {}, Mapillary provided: {}, Google Maps needed: {}",
        //     required_imgs,
        //     points_with_imgs.len(),
        //     required_imgs - points_with_imgs.len()
        // );

        // MapVisualizer::new()
        //     .draw(&shard)
        //     //.add_coords(&points_with_imgs, Style::default().fill_color("#49ff30"))
        //     .write_html("./paths_debug_map.html")
        //     .unwrap();

        // let google_api = GoogleStreetViewSource::new("AIzaSyDBAuSHlsFkZVz0fcvucDD6G0C3xwveEvI");
        // let first_image_coord = roads[0].segments.0.first().unwrap();

        // let returned_source = google_api
        //     .get_at(*first_image_coord, 4.0)
        //     .await
        //     .inspect_err(|err| println!("{}", err))
        //     .expect("Google API error")
        //     .expect("No image found!");

        // println!("Returned a valid source, saving to disk.");

        // for (ind, img) in returned_source.pieces.iter().enumerate() {
        //     img.save(format!("./{ind}_img.jpg")).unwrap();
        // }
    }
}
