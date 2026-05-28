mod osm;
pub mod shard;
pub mod sources;

#[cfg(test)]
mod tests {
    use crate::{osm::visual, shard::Shard};
    use geo::Coord;

    #[tokio::test]
    async fn it_works() {
        let shard = Shard::from_coords(Coord {
            x: 43.2173935,
            y: 27.8859658,
        })
        .unwrap();

        println!("{:?}", shard);
        let roads = shard
            .get_paths()
            .await
            .inspect_err(|err| println!("{:?}", err))
            .unwrap_or_default();

        visual::visualize_paths(&roads, "paths_debug_map.html").unwrap();
        println!(
            "All roads found within shard: {} {:?}",
            roads.len(),
            roads.iter().map(|a| a.segments.0.len()).sum::<usize>()
        );
    }
}
