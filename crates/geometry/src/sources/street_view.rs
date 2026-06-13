use std::time::Duration;

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
        let client = Client::builder()
            .tcp_keepalive(Duration::from_secs(60))
            .pool_idle_timeout(Duration::from_secs(60))
            .pool_max_idle_per_host(50)
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            api_key: api_key.to_owned(),
            client,
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

        let fetch_tasks = [0, 90, 180, 270].iter().map(|heading| {
            let img_url = format!(
                "https://maps.googleapis.com/maps/api/streetview?size=640x640&pano={}&heading={}&pitch=0&fov=90&key={}",
                pano_id, heading, self.api_key
            );

            let client_clone = self.client.clone();

            async move {
                let response = client_clone.get(&img_url).send().await?;
                response.bytes().await
            }
        });

        let all_image_bytes = futures::future::try_join_all(fetch_tasks).await?;

        let pieces = tokio::task::spawn_blocking(move || {
            let mut decoded = Vec::with_capacity(4);
            for bytes in all_image_bytes {
                let img = image::load_from_memory(&bytes)?.into_rgb8();
                decoded.push(img);
            }
            Ok::<_, image::ImageError>(decoded)
        })
        .await
        .unwrap()?;

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
