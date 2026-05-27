use geo::MultiPolygon;
use geo_traits::CoordTrait;
use h3o::{CellIndex, LatLng, Resolution, error::InvalidLatLng};

use crate::osm::{self, OverpassError, Path};

pub const SHARD_RESOLUTION: Resolution = Resolution::Eight;

#[derive(Clone, Debug)]
pub struct Shard {
    cell: CellIndex,
}

impl Shard {
    pub fn from_coords<V: CoordTrait<T = f64>>(value: V) -> Result<Self, InvalidLatLng> {
        Ok(Self {
            cell: LatLng::from_coord(&value)?.to_cell(SHARD_RESOLUTION),
        })
    }

    pub async fn get_paths(&self) -> Result<Vec<Path>, OverpassError> {
        let bounds = MultiPolygon::from(self.cell);
        osm::get_all_paths(bounds).await
    }
}
