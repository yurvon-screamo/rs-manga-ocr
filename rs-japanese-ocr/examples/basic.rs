use rs_japanese_ocr::JapaneseOCRModel;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Loading model (this may take a moment on first run)...");

    // Load model with default configuration (downloads automatically)
    let mut model = JapaneseOCRModel::load().await?;

    // Or with custom configuration:
    // let config = ModelConfig::new()
    //     .with_base_url("https://huggingface.co")
    //     .with_model_name("l0wgear/manga-ocr-2025-onnx")
    //     .with_cache_dir(".manga-ocr");
    // let mut model = JapaneseOCRModel::with_config(config).await?;

    // Load an image
    let img = image::ImageReader::open("test.png")?.decode()?;

    // Recognize text
    let text = model.run(&img)?;

    println!("Recognized text: {}", text);

    Ok(())
}
