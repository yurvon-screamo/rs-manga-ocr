use crate::error::JapaneseOCRError;

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub base_url: String,
    pub model_name: String,
    pub cache_dir: String,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            base_url: "https://huggingface.co".to_string(),
            model_name: "l0wgear/manga-ocr-2025-onnx".to_string(),
            cache_dir: ".manga-ocr".to_string(),
        }
    }
}

impl ModelConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Result<Self, JapaneseOCRError> {
        let url = base_url.into();
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(JapaneseOCRError::Config(
                "base_url must start with http:// or https://".to_string(),
            ));
        }
        self.base_url = url;
        Ok(self)
    }

    pub fn with_model_name(
        mut self,
        model_name: impl Into<String>,
    ) -> Result<Self, JapaneseOCRError> {
        let name = model_name.into();
        if name.contains("..") || name.contains('\\') {
            return Err(JapaneseOCRError::Config(
                "model_name cannot contain '..' or '\\'".to_string(),
            ));
        }
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '/')
        {
            return Err(JapaneseOCRError::Config(
                "model_name can only contain alphanumeric characters, '-', '_', and '/'"
                    .to_string(),
            ));
        }
        self.model_name = name;
        Ok(self)
    }

    pub fn with_cache_dir(mut self, cache_dir: impl Into<String>) -> Self {
        self.cache_dir = cache_dir.into();
        self
    }

    pub fn model_file_url(&self, filename: &str) -> String {
        format!(
            "{}/{}/resolve/main/{}",
            self.base_url.trim_end_matches('/'),
            self.model_name
                .trim_start_matches('/')
                .trim_end_matches('/'),
            filename.trim_start_matches('/')
        )
    }

    pub fn required_files() -> &'static [&'static str] {
        &["encoder_model.onnx", "decoder_model.onnx", "tokenizer.json"]
    }

    pub fn optional_files() -> &'static [&'static str] {
        &[
            "config.json",
            "preprocessor_config.json",
            "special_tokens_map.json",
            "tokenizer_config.json",
            "vocab.txt",
            "generation_config.json",
        ]
    }
}
