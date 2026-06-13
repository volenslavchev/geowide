use crate::model::MegaLoc;

pub fn get_model_default_paths() -> anyhow::Result<MegaLoc> {
    MegaLoc::new(&std::env::var("MODEL_PATH").unwrap_or("./model_export/megaloc.onnx".to_string()))
}
