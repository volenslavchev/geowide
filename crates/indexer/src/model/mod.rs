use image::{ImageBuffer, Rgb, imageops::FilterType};
use ndarray::Array4;
use ort::session::{Session, builder::GraphOptimizationLevel};

pub trait Model: Send + 'static {
    fn transform(&mut self, image: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> anyhow::Result<Vec<f32>>;
}

pub struct MegaLoc {
    model: Session,
}

impl MegaLoc {
    pub fn new(model_source: &str) -> anyhow::Result<Self> {
        let model = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?
            .with_intra_threads(10)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?
            .commit_from_file(model_source)?;

        Ok(Self { model })
    }
}

impl Model for MegaLoc {
    fn transform(&mut self, image: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> anyhow::Result<Vec<f32>> {
        let resized = image::imageops::resize(image, 322, 322, FilterType::Triangle);
        let mean = [0.485, 0.456, 0.406];
        let std = [0.229, 0.224, 0.225];

        let mut input_tensor = Array4::<f32>::zeros((1, 3, 322, 322));

        for (x, y, pixel) in resized.enumerate_pixels() {
            for c in 0..3 {
                let val = pixel[c] as f32 / 255.0;
                input_tensor[[0, c, y as usize, x as usize]] = (val - mean[c]) / std[c];
            }
        }

        let tensor = ort::value::Value::from_array(input_tensor)?;
        let outputs = self.model.run(ort::inputs!["input_image" => tensor])?;

        let descriptor_tensor = outputs["descriptor"].try_extract_tensor::<f32>()?;
        Ok(descriptor_tensor.1.to_vec())
    }
}
