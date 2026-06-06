use async_trait::async_trait;
use geo::{Coord, LineString, MultiPolygon};
use image::RgbImage;

#[derive(thiserror::Error, Debug)]
pub enum SourceError {
    #[error("Network Error")]
    Network(#[from] reqwest::Error),
}

#[derive(Debug)]
pub struct ImageSet {
    pub pieces: Vec<RgbImage>,
    pub actual_coords: Coord,
}

#[async_trait]
pub trait DataSource: Send + Sync {
    async fn load_bounds(&mut self, bounds: &MultiPolygon) -> Result<(), SourceError>;
    async fn get_at(&self, coords: Coord) -> Result<Option<ImageSet>, SourceError>;
    fn priority(&self) -> usize;
}
