use std::time::Duration;

use geo::Coord;
use geometry::utils::SpatialTree;
use qdrant_client::{
    Payload, Qdrant, QdrantError,
    qdrant::{
        CreateCollectionBuilder, Distance, PointId, PointStruct, ScrollPointsBuilder,
        UpsertPointsBuilder, VectorParamsBuilder,
    },
};
use serde::{Deserialize, Serialize};

const COLLECTION_NAME: &'static str = "images";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImagePoint {
    pub way_id: u64,
    pub lon: f64,
    pub lat: f64,
    pub ilon: f64,
    pub ilat: f64,
    pub source: String,
    pub seq_id: u32,
}

pub struct IndexDatabase {
    client: Qdrant,
}

impl IndexDatabase {
    pub async fn new() -> Result<Self, QdrantError> {
        let client = Qdrant::from_url("http://localhost:6334").build()?;
        if !client.collection_exists(COLLECTION_NAME).await? {
            client
                .create_collection(
                    CreateCollectionBuilder::new(COLLECTION_NAME)
                        .vectors_config(VectorParamsBuilder::new(8448, Distance::Cosine)),
                )
                .await?;
        }
        Ok(Self { client })
    }

    pub async fn bulk_insert_points(
        &self,
        data: Vec<(ImagePoint, Vec<f32>)>,
    ) -> anyhow::Result<()> {
        let mut qdrant_points = Vec::with_capacity(data.len());

        for (image_data, embedding) in data {
            let point_id = hash_image(
                image_data.way_id,
                image_data.ilon,
                image_data.ilat,
                image_data.seq_id,
            );
            let payload: Payload = serde_json::to_value(&image_data)?.try_into()?;
            qdrant_points.push(PointStruct::new(point_id, embedding, payload));
        }

        let max_attempts = 2;
        let mut attempts = 0;

        loop {
            attempts += 1;

            match self
                .client
                .upsert_points(UpsertPointsBuilder::new(
                    COLLECTION_NAME,
                    qdrant_points.clone(),
                ))
                .await
            {
                Ok(_) => return Ok(()),
                Err(err) => {
                    if attempts >= max_attempts {
                        return Err(anyhow::anyhow!(
                            "Fatal: Failed to insert batch after {} failed attempts.",
                            max_attempts
                        ));
                    }
                    println!("Error inserting batch: {}. Retrying in 2 seconds...", err);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }

    pub async fn get_all_points(&self) -> Result<SpatialTree<ImagePoint>, QdrantError> {
        let mut points: Vec<(Coord, ImagePoint)> = vec![];
        let mut offset: Option<PointId> = None;

        loop {
            let mut request_builder = ScrollPointsBuilder::new(COLLECTION_NAME)
                .limit(10_000)
                .with_payload(true)
                .with_vectors(false);

            if let Some(valid_offset) = offset {
                request_builder = request_builder.offset(valid_offset);
            }

            let response = self.client.scroll(request_builder).await?;

            for record in response.result {
                if let Ok(value) = serde_json::to_value(record.payload) {
                    if let Ok(data) = serde_json::from_value::<ImagePoint>(value) {
                        points.push((
                            Coord {
                                x: data.ilon,
                                y: data.ilat,
                            },
                            data,
                        ));
                    }
                }
            }

            if response.next_page_offset.is_some() {
                offset = response.next_page_offset;
            } else {
                break;
            }
        }

        Ok(SpatialTree::from_bulk(points))
    }
}

pub fn hash_image(way_id: u64, lon: f64, lat: f64, image_index: u32) -> u64 {
    let mut buffer = [0u8; 32];

    let lon_fixed = (lon * 10_000_000.0).round() as i64;
    let lat_fixed = (lat * 10_000_000.0).round() as i64;

    buffer[0..8].copy_from_slice(&way_id.to_le_bytes());
    buffer[8..16].copy_from_slice(&lon_fixed.to_le_bytes());
    buffer[16..24].copy_from_slice(&lat_fixed.to_le_bytes());
    buffer[24..28].copy_from_slice(&image_index.to_le_bytes());

    rapidhash::v3::rapidhash_v3(&buffer)
}
