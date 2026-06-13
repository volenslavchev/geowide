use async_trait::async_trait;
use geo::{BoundingRect, Coord, MultiPolygon};
use geozero::{
    ColumnValue, FeatureProcessor, GeomProcessor, PropertyProcessor,
    mvt::{Message, Tile},
};
use std::f64::consts::PI;

use crate::{sources::SourceError, utils::SpatialTree};

use super::DataSource;

#[derive(Debug, Clone)]
pub struct MapillaryImage {
    pub id: u64,
    pub coords: Coord,
    pub is_pano: bool,
    pub compass_angle: Option<f64>,
    pub quality: Option<f32>,
}

struct MapillaryExtractor {
    pub images: Vec<MapillaryImage>,

    current_id: Option<u64>,
    current_coords: Coord,
    is_pano: bool,
    quality: Option<f32>,
    compass_angle: Option<f64>,

    tile_x: u32,
    tile_y: u32,
    extent: u32,
}

impl MapillaryExtractor {
    pub fn new(tile_x: u32, tile_y: u32, extent: u32) -> Self {
        Self {
            images: Vec::new(),
            current_id: None,
            current_coords: Coord { x: 0.0, y: 0.0 },
            quality: None,
            compass_angle: None,
            is_pano: false,
            tile_x,
            tile_y,
            extent,
        }
    }
}

impl FeatureProcessor for MapillaryExtractor {
    fn feature_end(&mut self, _idx: u64) -> geozero::error::Result<()> {
        if let Some(id) = &self.current_id {
            self.images.push(MapillaryImage {
                id: *id,
                coords: self.current_coords,
                is_pano: self.is_pano,
                compass_angle: self.compass_angle,
                quality: self.quality,
            });
        }
        self.current_id = None;
        self.is_pano = false;
        self.compass_angle = None;
        self.quality = None;
        Ok(())
    }
}

impl PropertyProcessor for MapillaryExtractor {
    fn property(
        &mut self,
        _i: usize,
        name: &str,
        value: &ColumnValue,
    ) -> geozero::error::Result<bool> {
        match name {
            "id" => match value {
                ColumnValue::ULong(id) => self.current_id = Some(*id),
                ColumnValue::Long(id) => self.current_id = Some(*id as u64),
                ColumnValue::String(s) => self.current_id = s.parse().ok(),
                _ => {}
            },
            "is_pano" => {
                if let ColumnValue::Bool(b) = value {
                    self.is_pano = *b;
                }
            }
            "quality" => {
                if let ColumnValue::Float(s) = value {
                    self.quality = Some(*s);
                }
            }
            "compass_angle" => {
                if let ColumnValue::Double(s) = value {
                    self.compass_angle = Some(*s);
                }
            }
            _ => {}
        }

        Ok(false)
    }
}

impl GeomProcessor for MapillaryExtractor {
    fn xy(&mut self, x: f64, y: f64, _idx: usize) -> geozero::error::Result<()> {
        let n = f64::powi(2.0, ZOOM_LEVEL);
        let lon_deg = (self.tile_x as f64 + (x / self.extent as f64)) / n * 360.0 - 180.0;
        let lat_rad = (PI * (1.0 - 2.0 * (self.tile_y as f64 + (y / self.extent as f64)) / n))
            .sinh()
            .atan();

        self.current_coords = Coord {
            x: lon_deg,
            y: lat_rad.to_degrees(),
        };
        Ok(())
    }
}

pub struct MapillaryDataSource {
    api_key: String,
    images: SpatialTree<MapillaryImage>,
}

impl MapillaryDataSource {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.replace("|", "%7C"),
            images: SpatialTree::new(),
        }
    }

    pub fn get_points(&self) -> Vec<Coord> {
        self.images.iter().map(|v| v.coords).collect()
    }

    pub fn get_images_within_radius(&self, coords: Coord, radius: f64) -> Option<&MapillaryImage> {
        self.images
            .get_at_filter(coords, radius, |item| !item.is_pano)
    }
}

#[async_trait]
impl DataSource for MapillaryDataSource {
    async fn load_bounds(&mut self, bounds: &MultiPolygon) -> Result<(), SourceError> {
        let bbox = bounds.bounding_rect().unwrap();
        let (min_x, min_y) = lon_lat_to_tile(bbox.min().x, bbox.max().y);
        let (max_x, max_y) = lon_lat_to_tile(bbox.max().x, bbox.min().y);

        let mut images: Vec<(Coord, MapillaryImage)> = vec![];

        let client = reqwest::Client::builder()
            .user_agent("Geowide/1.0")
            .build()
            .map_err(SourceError::Network)?;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                let url = format!(
                    "https://tiles.mapillary.com/maps/vtp/mly1_public/2/{}/{}/{}?access_token={}",
                    ZOOM_LEVEL, x, y, self.api_key
                );

                let response = client
                    .execute(client.get(url).build().unwrap())
                    .await
                    .map_err(SourceError::Network)?;
                let bytes = response.bytes().await.map_err(SourceError::Network)?;
                let tile = Tile::decode(bytes).unwrap();

                if let Some(image_layer) = tile.layers.into_iter().find(|l| l.name == "image") {
                    let extent = image_layer.extent.unwrap_or(4096);
                    let mut extractor = MapillaryExtractor::new(x, y, extent);
                    geozero::mvt::process(&image_layer, &mut extractor)
                        .expect("Failed to process MVT Layer");

                    for image in extractor.images {
                        images.push((image.coords, image));
                    }
                }
            }
        }

        self.images = SpatialTree::from_bulk(images);

        Ok(())
    }

    async fn get_at(
        &self,
        coords: Coord,
        radius: f64,
    ) -> Result<Option<super::ImageSet>, SourceError> {
        if let Some(_found) = self.get_images_within_radius(coords, radius) {
            // TODO: Splitting logic...
            todo!()
        } else {
            Ok(None)
        }
    }

    fn priority(&self) -> usize {
        1
    }

    fn id(&self) -> &str {
        "mapillary"
    }
}

const ZOOM_LEVEL: i32 = 14;

pub fn lon_lat_to_tile(lon: f64, lat: f64) -> (u32, u32) {
    let n = f64::powi(2.0, ZOOM_LEVEL);

    let x = (n * ((lon + 180.0) / 360.0)).floor() as u32;

    let lat_rad = lat.to_radians();
    let y = (n * (1.0 - (lat_rad.tan() + (1.0 / lat_rad.cos())).ln() / PI) / 2.0).floor() as u32;

    (x, y)
}
