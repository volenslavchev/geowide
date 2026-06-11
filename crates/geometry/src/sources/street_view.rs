use async_trait::async_trait;
use geo::{Coord, Distance, Haversine, MultiPolygon, Point};
use reqwest::Client;
use serde::Deserialize;

use crate::sources::{DataSource, ImageSet, SourceError};

#[derive(Deserialize)]
struct StreetViewMetadata {
    status: String,
    location: Option<Location>,
    pano_id: Option<String>,
}

#[derive(Deserialize)]
struct Location {
    lat: f64,
    lng: f64,
}

pub struct GoogleStreetViewSource {
    api_key: String,
    client: Client,
}

impl GoogleStreetViewSource {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_owned(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl DataSource for GoogleStreetViewSource {
    async fn load_bounds(&mut self, _bounds: &MultiPolygon) -> Result<(), SourceError> {
        Ok(())
    }

    async fn get_at(
        &self,
        coords: Coord,
        search_radius: f64,
    ) -> Result<Option<ImageSet>, SourceError> {
        let meta_url = format!(
            "https://maps.googleapis.com/maps/api/streetview/metadata?location={},{}&key={}",
            coords.y, coords.x, self.api_key
        );

        let metadata: StreetViewMetadata = self.client.get(&meta_url).send().await?.json().await?;

        if metadata.status != "OK" {
            return Ok(None);
        }

        let actual_loc = metadata.location.unwrap();
        let pano_id = metadata.pano_id.unwrap();

        let target_point = Point::new(coords.x, coords.y);
        let actual_point = Point::new(actual_loc.lng, actual_loc.lat);

        if Haversine.distance(target_point, actual_point) > search_radius {
            return Ok(None);
        }

        let mut pieces = Vec::with_capacity(4);
        let headings = [0, 90, 180, 270];
        let mut fetch_tasks = Vec::new();

        for heading in headings {
            let img_url = format!(
                "https://maps.googleapis.com/maps/api/streetview?size=640x640&pano={}&heading={}&pitch=0&fov=90&key={}",
                pano_id, heading, self.api_key
            );

            let client_clone = self.client.clone();

            fetch_tasks.push(tokio::spawn(async move {
                let response = client_clone.get(&img_url).send().await?;
                response.bytes().await
            }));
        }

        for task in fetch_tasks {
            let image_bytes = task.await.unwrap()?;

            let decoded_img = image::load_from_memory(&image_bytes)?.into_rgb8();
            pieces.push(decoded_img);
        }

        Ok(Some(ImageSet {
            pieces,
            actual_coords: Coord {
                x: actual_loc.lng,
                y: actual_loc.lat,
            },
        }))
    }

    fn priority(&self) -> usize {
        100
    }

    fn id(&self) -> &str {
        "street_view_static"
    }
}
