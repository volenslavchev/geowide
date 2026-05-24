use crate::model::{MegaLoc, Model};

pub mod model;

fn main() -> anyhow::Result<()> {
    let mut model = MegaLoc::new("./model_export/megaloc.onnx")?;

    let varna_1 = image::open("test_images/image1.jpg")?.to_rgb8();
    let varna_2 = image::open("test_images/image1_similar2.png")?.to_rgb8();

    println!("Embedding varna 1...");
    let varna_1_embedding = model.transform(&varna_1)?;
    println!("Embedding varna 2...");
    let varna_2_embedding = model.transform(&varna_2)?;

    let sim = cosine_similarity(&varna_1_embedding, &varna_2_embedding);
    println!("Similarity: {:.4}", sim);
    Ok(())
}

fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for i in 0..v1.len() {
        dot_product += v1[i] * v2[i];
        norm_a += v1[i] * v1[i];
        norm_b += v2[i] * v2[i];
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a.sqrt() * norm_b.sqrt())
}
