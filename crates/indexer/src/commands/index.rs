use crate::{
    cli::IndexArgs,
    db::{ImagePoint, IndexDatabase},
    model::{MegaLoc, Model},
};

pub async fn index_command(args: IndexArgs) -> anyhow::Result<()> {
    let client = IndexDatabase::new().await?;
    let mut model = MegaLoc::new(
        &std::env::var("MODEL_PATH").unwrap_or("./model_export/megaloc.onnx".to_string()),
    )?;

    println!("Loaded model...");

    let varna_1 = image::open("test_images/image1.jpg")?.to_rgb8();
    let genned = model.transform(&varna_1)?;

    println!("Transformed image...");

    client
        .bulk_insert_points(vec![(
            ImagePoint {
                source: "any".to_string(),
                lon: 1.0,
                lat: 1.0,
                ilon: 2.0,
                ilat: 2.0,
                way_id: 123,
            },
            genned,
        )])
        .await?;

    println!("Bulk insert finished");

    println!("{:?}", client.get_all_points().await);

    println!("DONE! Index database initialized, model loaded.");

    Ok(())
}
