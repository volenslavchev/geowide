use qdrant_client::{
    Payload, Qdrant, QdrantError,
    qdrant::{
        CreateCollectionBuilder, Distance, PointId, PointStruct, ScrollPointsBuilder,
        UpsertPointsBuilder, VectorParamsBuilder,
    },
};
use rapidhash::v3::rapidhash_v3;
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
}

#[derive(Clone, Debug)]
pub struct InsertedImagePoint {
    pub value: ImagePoint,
    pub id: u64,
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
            let point_id = hash_image(image_data.way_id, image_data.ilon, image_data.ilat);
            let payload: Payload = serde_json::to_value(&image_data)?.try_into()?;
            qdrant_points.push(PointStruct::new(point_id, embedding, payload));
        }

        self.client
            .upsert_points(UpsertPointsBuilder::new(COLLECTION_NAME, qdrant_points))
            .await?;
        Ok(())
    }

    pub async fn get_all_points(&self) -> Result<Vec<InsertedImagePoint>, QdrantError> {
        let mut all_data = Vec::new();
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
                let point_id = match record
                    .id
                    .as_ref()
                    .and_then(|id| id.point_id_options.as_ref())
                {
                    Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(id)) => *id,
                    _ => continue,
                };

                if let Ok(value) = serde_json::to_value(record.payload) {
                    if let Ok(data) = serde_json::from_value::<ImagePoint>(value) {
                        all_data.push(InsertedImagePoint {
                            value: data,
                            id: point_id,
                        })
                    }
                }
            }

            if response.next_page_offset.is_some() {
                offset = response.next_page_offset;
            } else {
                break;
            }
        }

        Ok(all_data)
    }
}

pub fn hash_image(way_id: u64, lon: f64, lat: f64) -> u64 {
    let mut buffer = [0u8; 24];

    buffer[0..8].copy_from_slice(&way_id.to_le_bytes());
    buffer[8..16].copy_from_slice(&lon.to_bits().to_le_bytes());
    buffer[16..24].copy_from_slice(&lat.to_bits().to_le_bytes());

    rapidhash_v3(&buffer)
}
