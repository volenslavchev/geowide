use std::str::FromStr;

use geo::{MultiPolygon, Rect};
use geo_traits::CoordTrait;
use h3o::{
    CellIndex, LatLng, Resolution,
    error::{InvalidCellIndex, InvalidLatLng},
};

use crate::osm::{self, OsmConfig, OverpassError, Path};

pub const SHARD_RESOLUTION: Resolution = Resolution::Eight;
pub const OSM_CONFIG: OsmConfig = OsmConfig {
    segment_min_distance: 15.0,
    segment_ideal_distance: 25.0,
};

#[derive(Clone, Debug)]
pub struct Shard {
    pub cell: CellIndex,
    pub bounds: MultiPolygon,
}

impl Shard {
    pub fn from_coords<V: CoordTrait<T = f64>>(value: V) -> Result<Self, InvalidLatLng> {
        let cell: CellIndex = LatLng::from_coord(&value)?.to_cell(SHARD_RESOLUTION);
        let bounds = MultiPolygon::from(cell);
        Ok(Self { cell, bounds })
    }

    pub fn from_cell_index(cell: CellIndex) -> Self {
        let bounds = MultiPolygon::from(cell);
        Self { cell, bounds }
    }

    pub fn from_cell_str(cell_str: &str) -> Result<Self, InvalidCellIndex> {
        let cell = CellIndex::from_str(cell_str)?;
        let bounds = MultiPolygon::from(cell);
        Ok(Self { cell, bounds })
    }

    pub fn from_bbox(bbox: Rect) -> Vec<Self> {
        let mut tiler = h3o::geom::TilerBuilder::new(SHARD_RESOLUTION).build();
        tiler.add(bbox.to_polygon()).unwrap();
        tiler.into_coverage().map(Self::from_cell_index).collect()
    }

    pub async fn from_query(query: &str) -> anyhow::Result<Vec<Self>> {
        let mut tiler = h3o::geom::TilerBuilder::new(SHARD_RESOLUTION).build();
        match osm::search_coordinates(query).await? {
            geo::Geometry::Polygon(poly) => {
                tiler.add(poly).expect("Failed to tile polygon");
            }
            geo::Geometry::MultiPolygon(mpoly) => {
                for poly in mpoly {
                    tiler.add(poly).expect("Failed to tile polygon");
                }
            }
            _ => {}
        };

        Ok(tiler.into_coverage().map(Self::from_cell_index).collect())
    }

    pub async fn get_paths(&self) -> Result<Vec<Path>, OverpassError> {
        osm::get_all_paths(&self.bounds, &OSM_CONFIG).await
    }
}
