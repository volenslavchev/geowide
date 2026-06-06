use geo::MultiPolygon;
use geo_traits::CoordTrait;
use h3o::{CellIndex, LatLng, Resolution, error::InvalidLatLng};

use crate::osm::{self, OverpassError, Path};

pub const SHARD_RESOLUTION: Resolution = Resolution::Eight;

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

    pub async fn get_paths(&self) -> Result<Vec<Path>, OverpassError> {
        osm::get_all_paths(&self.bounds).await
    }
}
