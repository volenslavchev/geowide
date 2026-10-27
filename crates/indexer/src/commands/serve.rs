use std::sync::Arc;

use axum::{
    Json, Router, body::Bytes, extract::{DefaultBodyLimit, State}, http::{HeaderValue, StatusCode, header}, routing::post,
};
use colored::Colorize;
use serde::Serialize;

use tower_http::cors::{Any, CorsLayer};

use crate::{
    cli::ServeArgs,
    db::IndexDatabase,
    model::{MegaLoc, Model},
    utils::get_model_default_paths,
};

struct ApiState {
    model: MegaLoc,
    db: IndexDatabase,
}

pub async fn serve_command(args: ServeArgs) -> anyhow::Result<()> {
    let state = Arc::new(ApiState {
        model: get_model_default_paths()?,
        db: IndexDatabase::new().await?,
    });

    let cors_layer = CorsLayer::new()
        .allow_methods(Any)
        // TODO: Disable CORS is front-end URL is not provided
        .allow_origin(
            (std::env::var("FRONTEND_URL").expect("Frontend URL not provided."))
                .parse::<HeaderValue>()
                .unwrap(),
        )
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/", post(location_route))
        .layer(cors_layer)
        .layer(DefaultBodyLimit::max(5_242_880)) // 5 MB
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Serialize)]
struct LocationImageOutput {
    lon: f64,
    lat: f64,
    score: f32,
}

#[derive(Serialize)]
struct LocationOutput {
    time: u128,
    locations: Vec<LocationImageOutput>,
}

async fn location_route(
    State(state): State<Arc<ApiState>>,
    image_bytes: Bytes,
) -> Result<Json<LocationOutput>, StatusCode> {
    let start = std::time::Instant::now();
    let state_for_embedding = state.clone();
    let embedding_result = tokio::task::spawn_blocking(move || {
        let img = image::load_from_memory(&image_bytes)
            .map_err(|_| StatusCode::BAD_REQUEST)?
            .to_rgb8();

        let embeddings = state_for_embedding
            .model
            .transform(&img)
            .inspect_err(|err| println!("{}", err.to_string().red()))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok::<Vec<f32>, StatusCode>(embeddings)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)??;

    let result = state
        .db
        .search(embedding_result, 7)
        .await
        .inspect_err(|err| println!("{}", err.to_string().red()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|img| LocationImageOutput {
            lon: img.data.lon,
            lat: img.data.lat,
            score: img.score,
        })
        .collect();

    let elapsed = start.elapsed();

    Ok(Json(LocationOutput {
        time: elapsed.as_millis(),
        locations: result,
    }))
}
