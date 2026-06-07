use async_trait::async_trait;
use geo::{Coord, MultiPolygon};
use image::RgbImage;

#[derive(thiserror::Error, Debug)]
pub enum SourceError {
    #[error("Network Error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Image Error: {0}")]
    Image(#[from] image::ImageError),
}

#[derive(Debug)]
pub struct ImageSet {
    pub pieces: Vec<RgbImage>,
    pub actual_coords: Coord,
}

#[async_trait]
pub trait DataSource: Send + Sync {
    async fn load_bounds(&mut self, bounds: &MultiPolygon) -> Result<(), SourceError>;
    async fn get_at(&self, coords: Coord, radius: f64) -> Result<Option<ImageSet>, SourceError>;
    fn priority(&self) -> usize;
}
