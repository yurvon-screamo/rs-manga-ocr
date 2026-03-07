use candle_core::{Device, IndexOp, Tensor};
use candle_onnx::onnx::ModelProto;
use candle_onnx::simple_eval;
use image::{DynamicImage, imageops::FilterType};
use prost::Message;
use std::collections::HashMap;
use tokenizers::Tokenizer;

use crate::error::JapaneseOCRError;

const MAX_SEQ_LEN: usize = 300;
const IMAGE_RESIZE_W: u32 = 224;
const IMAGE_RESIZE_H: u32 = 224;
const PIXEL_NORM_FACTOR: f32 = 255.0;
const NORM_MEAN: [f32; 3] = [0.5, 0.5, 0.5];
const NORM_STD: [f32; 3] = [0.5, 0.5, 0.5];
// [CLS] token - начало последовательности
const DEFAULT_BOS_TOKEN_ID: u32 = 2;
// [SEP] token - конец последовательности
const DEFAULT_EOS_TOKEN_ID: u32 = 3;

#[cfg(feature = "static-model")]
static ENCODER_BYTES: &[u8] = include_bytes!("model/encoder_model.onnx");
#[cfg(feature = "static-model")]
static DECODER_BYTES: &[u8] = include_bytes!("model/decoder_model.onnx");
#[cfg(feature = "static-model")]
static TOKENIZER_BYTES: &[u8] = include_bytes!("model/tokenizer.json");

pub struct JapaneseOCRModel {
    encoder: ModelProto,
    decoder: ModelProto,
    tokenizer: Tokenizer,
    device: Device,
}

impl JapaneseOCRModel {
    #[cfg(feature = "static-model")]
    pub fn load() -> Result<Self, JapaneseOCRError> {
        let device = Device::Cpu;

        let tokenizer = Tokenizer::from_bytes(TOKENIZER_BYTES)
            .map_err(|e| JapaneseOCRError::Tokenizer(format!("Failed to load tokenizer: {}", e)))?;

        let encoder = ModelProto::decode(ENCODER_BYTES)
            .map_err(|e| JapaneseOCRError::Model(format!("Failed to decode encoder: {}", e)))?;

        let decoder = ModelProto::decode(DECODER_BYTES)
            .map_err(|e| JapaneseOCRError::Model(format!("Failed to decode decoder: {}", e)))?;

        Ok(Self {
            encoder,
            decoder,
            tokenizer,
            device,
        })
    }

    #[cfg(all(not(feature = "static-model"), target_arch = "wasm32"))]
    pub async fn load() -> Result<Self, JapaneseOCRError> {
        use crate::ModelConfig;

        let config = ModelConfig::new();
        Self::load_with_config_internal(config).await
    }

    #[cfg(all(not(feature = "static-model"), not(target_arch = "wasm32")))]
    pub async fn load() -> Result<Self, JapaneseOCRError> {
        use crate::ModelConfig;

        let config = ModelConfig::new();
        Self::load_with_config_internal(config).await
    }

    pub fn from_bytes(
        encoder_bytes: Vec<u8>,
        decoder_bytes: Vec<u8>,
        tokenizer_bytes: Vec<u8>,
    ) -> Result<Self, JapaneseOCRError> {
        let device = Device::Cpu;

        let tokenizer = Tokenizer::from_bytes(&tokenizer_bytes)
            .map_err(|e| JapaneseOCRError::Tokenizer(format!("Failed to load tokenizer: {}", e)))?;

        let encoder = ModelProto::decode(encoder_bytes.as_slice())
            .map_err(|e| JapaneseOCRError::Model(format!("Failed to decode encoder: {}", e)))?;

        let decoder = ModelProto::decode(decoder_bytes.as_slice())
            .map_err(|e| JapaneseOCRError::Model(format!("Failed to decode decoder: {}", e)))?;

        Ok(Self {
            encoder,
            decoder,
            tokenizer,
            device,
        })
    }

    #[cfg(feature = "static-model")]
    pub fn with_config(_config: crate::ModelConfig) -> Result<Self, JapaneseOCRError> {
        Self::load()
    }

    #[cfg(all(not(feature = "static-model"), target_arch = "wasm32"))]
    pub async fn with_config(config: crate::ModelConfig) -> Result<Self, JapaneseOCRError> {
        Self::load_with_config_internal(config).await
    }

    #[cfg(all(not(feature = "static-model"), not(target_arch = "wasm32")))]
    pub async fn with_config(config: crate::ModelConfig) -> Result<Self, JapaneseOCRError> {
        Self::load_with_config_internal(config).await
    }

    #[cfg(not(feature = "static-model"))]
    async fn load_with_config_internal(
        config: crate::ModelConfig,
    ) -> Result<Self, JapaneseOCRError> {
        use crate::ModelLoader;

        let loader = ModelLoader::new(config);
        let model_files = loader.load_or_download_model().await?;

        Self::from_bytes(
            model_files.encoder,
            model_files.decoder,
            model_files.tokenizer,
        )
    }

    pub fn run(&mut self, img: &DynamicImage) -> Result<String, JapaneseOCRError> {
        let pixel_values = self.preprocess_image(img)?;

        let mut encoder_inputs: HashMap<String, Tensor> = HashMap::new();
        encoder_inputs.insert("pixel_values".to_string(), pixel_values);

        let encoder_outputs = simple_eval(&self.encoder, encoder_inputs)?;

        let encoder_hidden_states = encoder_outputs
            .get("last_hidden_state")
            .ok_or_else(|| {
                JapaneseOCRError::Model("Missing last_hidden_state from encoder".into())
            })?
            .clone();

        let bos_token_id = self
            .tokenizer
            .token_to_id("[CLS]")
            .unwrap_or(DEFAULT_BOS_TOKEN_ID);
        let eos_token_id = self
            .tokenizer
            .token_to_id("[SEP]")
            .unwrap_or(DEFAULT_EOS_TOKEN_ID);

        let mut input_ids = vec![bos_token_id as i64];

        for _ in 0..MAX_SEQ_LEN {
            let input_tensor = Tensor::from_slice(&input_ids, (1, input_ids.len()), &self.device)?;

            let mut decoder_inputs: HashMap<String, Tensor> = HashMap::new();
            decoder_inputs.insert("input_ids".to_string(), input_tensor);
            decoder_inputs.insert(
                "encoder_hidden_states".to_string(),
                encoder_hidden_states.clone(),
            );

            let decoder_outputs = simple_eval(&self.decoder, decoder_inputs)?;

            let logits = decoder_outputs
                .get("logits")
                .ok_or_else(|| JapaneseOCRError::Model("Missing logits from decoder".into()))?;

            let seq_len = logits.dim(1)?;
            let last_logits = logits.i((0, seq_len - 1, ..))?;
            let next_token = last_logits.argmax(0)?.to_scalar::<u32>()?;

            if next_token == eos_token_id {
                break;
            }

            input_ids.push(next_token as i64);
        }

        let decoded = self
            .tokenizer
            .decode(
                &input_ids.iter().map(|&id| id as u32).collect::<Vec<_>>(),
                true,
            )
            .map_err(|e| JapaneseOCRError::Tokenizer(format!("Failed to decode: {}", e)))?;

        Ok(decoded.replace(" ", ""))
    }

    fn preprocess_image(&self, img: &DynamicImage) -> Result<Tensor, JapaneseOCRError> {
        let resized = img.resize_exact(IMAGE_RESIZE_W, IMAGE_RESIZE_H, FilterType::Nearest);
        let rgb = resized.to_rgb8();
        let (width, height) = rgb.dimensions();

        let mut data = Vec::with_capacity((width * height * 3) as usize);
        for pixel in rgb.pixels() {
            data.push(pixel[0] as f32 / PIXEL_NORM_FACTOR);
            data.push(pixel[1] as f32 / PIXEL_NORM_FACTOR);
            data.push(pixel[2] as f32 / PIXEL_NORM_FACTOR);
        }

        let tensor = Tensor::from_vec(data, (height as usize, width as usize, 3), &self.device)?;
        let mean = Tensor::new(&NORM_MEAN, &self.device)?;
        let std = Tensor::new(&NORM_STD, &self.device)?;

        let normalized = tensor.broadcast_sub(&mean)?.broadcast_div(&std)?;
        let pixel_values = normalized.permute((2, 0, 1))?.unsqueeze(0)?;

        Ok(pixel_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ocr() -> Result<(), JapaneseOCRError> {
        let img = image::ImageReader::open("test.png")?.decode()?;

        let mut model = JapaneseOCRModel::load().await?;

        let output = model.run(&img)?;
        assert_eq!(&output, "悪魔との戦い");
        Ok(())
    }
}
