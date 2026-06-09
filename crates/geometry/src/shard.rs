use std::str::FromStr;

use geo::{MultiPolygon, Rect};
use geo_traits::CoordTrait;
use h3o::{CellIndex, LatLng, Resolution};

use crate::osm::{self, MultiLinePath, OsmConfig, Path};

const SHARD_RESOLUTION: Resolution = Resolution::Eight;
const OSM_CONFIG: OsmConfig = OsmConfig {
    segment_min_distance: 17.0,
    segment_ideal_distance: 25.0,
};

#[derive(Debug)]
pub struct Shard {
    pub cell: CellIndex,
    pub paths: Vec<Path>,
}

impl Shard {
    async fn from_cell_index(cell: CellIndex) -> anyhow::Result<Self> {
        let bounds = MultiPolygon::from(cell);
        let raw_paths = osm::get_all_paths(&bounds, &OSM_CONFIG).await?;
        let paths = osm::clip_paths(&bounds, raw_paths.iter());
        Ok(Self { cell, paths })
    }

    pub async fn from_coords<V: CoordTrait<T = f64>>(value: V) -> anyhow::Result<Self> {
        let cell: CellIndex = LatLng::from_coord(&value)?.to_cell(SHARD_RESOLUTION);
        Self::from_cell_index(cell).await
    }

    pub async fn from_cell_str(cell_str: &str) -> anyhow::Result<Self> {
        let cell = CellIndex::from_str(cell_str)?;
        Self::from_cell_index(cell).await
    }

    pub fn into_shard_cluster(self) -> ShardCluster {
        ShardCluster(vec![self])
    }

    fn from_cell_index_with_data(cell: CellIndex, element_data: &Vec<MultiLinePath>) -> Self {
        let bounds = MultiPolygon::from(cell);
        let paths = osm::clip_paths(&bounds, element_data.iter());
        Self { cell, paths }
    }
}

#[derive(Debug)]
pub struct ShardCluster(pub Vec<Shard>);

impl ShardCluster {
    pub fn new(shards: Vec<Shard>) -> Self {
        Self(shards)
    }

    pub async fn from_bbox(bbox: Rect) -> anyhow::Result<Self> {
        let poly: MultiPolygon = bbox.clone().into();
        let osm_values = osm::get_all_paths(&poly, &OSM_CONFIG).await?;
        let mut tiler = h3o::geom::TilerBuilder::new(SHARD_RESOLUTION).build();
        tiler.add(bbox.to_polygon()).unwrap();
        Ok(Self(
            tiler
                .into_coverage()
                .map(|cell| Shard::from_cell_index_with_data(cell, &osm_values))
                .collect(),
        ))
    }

    pub async fn from_query(query: &str) -> anyhow::Result<Self> {
        let mut tiler = h3o::geom::TilerBuilder::new(SHARD_RESOLUTION).build();
        let poly = match osm::search_coordinates(query).await? {
            geo::Geometry::Polygon(poly) => {
                tiler
                    .add(poly.clone())
                    .map_err(|_| anyhow::anyhow!("Failed to tile polygon"))?;
                MultiPolygon(vec![poly])
            }
            geo::Geometry::MultiPolygon(mpoly) => {
                for poly in &mpoly {
                    tiler
                        .add(poly.clone())
                        .map_err(|_| anyhow::anyhow!("Failed to tile polygon"))?;
                }
                mpoly
            }
            _ => return Err(anyhow::anyhow!("Failed to tile polygon")),
        };

        let osm_values = osm::get_all_paths(&poly, &OSM_CONFIG).await?;

        Ok(Self(
            tiler
                .into_coverage()
                .map(|cell| Shard::from_cell_index_with_data(cell, &osm_values))
                .collect(),
        ))
    }
}
