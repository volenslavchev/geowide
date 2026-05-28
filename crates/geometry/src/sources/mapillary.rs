use async_trait::async_trait;
use geo::Coord;
use image::RgbImage;

use crate::sources::SourceError;

use super::DataSource;

pub struct MapillaryDataSource {
    api_key: String,
}

impl MapillaryDataSource {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }
}

#[async_trait]
impl DataSource for MapillaryDataSource {
    async fn load_bounds(&mut self, bounds: geo::MultiPolygon) -> Result<(), SourceError> {
        todo!()
    }

    async fn get_at(&self, coords: Coord) -> Result<Option<RgbImage>, SourceError> {
        todo!()
    }

    fn priority(&self) -> usize {
        1
    }
}

