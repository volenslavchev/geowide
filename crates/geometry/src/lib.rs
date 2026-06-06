mod osm;
pub mod shard;
pub mod sources;
pub mod visualize;

#[cfg(test)]
mod tests {
    use crate::{
        shard::Shard,
        sources::{self, DataSource},
        visualize::{MapVisualizer, Style},
    };
    use geo::{BallTreePoint, Coord};

    #[tokio::test]
    async fn it_works() {
        // Varna
        let shard = Shard::from_coords(Coord {
            x: 27.9096873,
            y: 43.2145027,
        })
        .unwrap();

        // Frankfurt
        // let shard = Shard::from_coords(Coord {
        //     // 50.1201244,8.6704001
        //     x: 8.6704001,
        //     y: 50.1201244,
        // })
        // .unwrap();

        let roads = shard
            .get_paths()
            .await
            .inspect_err(|err| println!("{:?}", err))
            .unwrap_or_default();

        let mut mapillary = sources::mapillary::MapillaryDataSource::new(
            "MLY|26044754875173911|9c90a62549d628513a9edb99ab0f4160",
        );
        mapillary.load_bounds(&shard.bounds).await.unwrap();

        let points_with_imgs = roads
            .iter()
            .flat_map(|r| r.segments.clone().into_points())
            .map(|p| p.coord())
            .filter(|point| mapillary.get_images_within_radius(*point, 6.0).is_some())
            .collect::<Vec<_>>();

        let required_imgs: usize = roads.iter().map(|r| r.segments.0.len()).sum();
        println!(
            "Required images: {}, Mapillary provided: {}, Google Maps needed: {}",
            required_imgs,
            points_with_imgs.len(),
            required_imgs - points_with_imgs.len()
        );

        MapVisualizer::new()
            .draw(&shard)
            .draw(&roads)
            .add_coords(&points_with_imgs, Style::default().fill_color("#49ff30"))
            .write_html("./paths_debug_map.html")
            .unwrap();
    }
}
