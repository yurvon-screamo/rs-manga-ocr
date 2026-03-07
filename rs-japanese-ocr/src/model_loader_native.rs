use crate::config::ModelConfig;
use crate::error::JapaneseOCRError;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncReadExt;

pub struct ModelFiles {
    pub encoder: Vec<u8>,
    pub decoder: Vec<u8>,
    pub tokenizer: Vec<u8>,
}

pub struct ModelLoader {
    config: ModelConfig,
    client: reqwest::Client,
}

impl ModelLoader {
    pub fn new(config: ModelConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self { config, client }
    }

    pub async fn load_or_download_model(&self) -> Result<ModelFiles, JapaneseOCRError> {
        let cache_path = self.get_cache_path().await?;

        if self.is_model_cached(&cache_path).await {
            self.load_from_cache(&cache_path).await
        } else {
            self.download_and_cache_model(&cache_path).await
        }
    }

    async fn get_cache_path(&self) -> Result<PathBuf, JapaneseOCRError> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
            .join(&self.config.cache_dir);

        fs::create_dir_all(&cache_dir).await?;

        Ok(cache_dir)
    }

    async fn is_model_cached(&self, cache_path: &Path) -> bool {
        for file in ModelConfig::required_files() {
            if fs::metadata(cache_path.join(file)).await.is_err() {
                return false;
            }
        }
        true
    }

    async fn load_from_cache(&self, cache_path: &Path) -> Result<ModelFiles, JapaneseOCRError> {
        let encoder = self
            .load_file(&cache_path.join("encoder_model.onnx"))
            .await?;
        let decoder = self
            .load_file(&cache_path.join("decoder_model.onnx"))
            .await?;
        let tokenizer = self.load_file(&cache_path.join("tokenizer.json")).await?;

        Ok(ModelFiles {
            encoder,
            decoder,
            tokenizer,
        })
    }

    async fn load_file(&self, path: &Path) -> Result<Vec<u8>, JapaneseOCRError> {
        let mut file = fs::File::open(path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        Ok(buffer)
    }

    async fn download_and_cache_model(
        &self,
        cache_path: &Path,
    ) -> Result<ModelFiles, JapaneseOCRError> {
        let all_files: Vec<&str> = ModelConfig::required_files()
            .iter()
            .chain(ModelConfig::optional_files().iter())
            .copied()
            .collect();

        for filename in &all_files {
            self.download_file_with_retry(filename, cache_path).await?;
        }

        self.load_from_cache(cache_path).await
    }

    async fn download_file_with_retry(
        &self,
        filename: &str,
        cache_path: &Path,
    ) -> Result<(), JapaneseOCRError> {
        const MAX_RETRIES: u32 = 3;

        for attempt in 0..MAX_RETRIES {
            match self.try_download_file(filename, cache_path).await {
                Ok(()) => return Ok(()),
                Err(e) if attempt < MAX_RETRIES - 1 => {
                    let delay = Duration::from_secs(2u64.pow(attempt));
                    tokio::time::sleep(delay).await;
                }
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    async fn try_download_file(
        &self,
        filename: &str,
        cache_path: &Path,
    ) -> Result<(), JapaneseOCRError> {
        let url = self.config.model_file_url(filename);
        let dest_path = cache_path.join(filename);

        let response = self.client.get(&url).send().await.map_err(|e| {
            JapaneseOCRError::Download(format!("Failed to download {}: {}", filename, e))
        })?;

        if !response.status().is_success() {
            return Err(JapaneseOCRError::Download(format!(
                "Failed to download {}: HTTP {}",
                filename,
                response.status()
            )));
        }

        let buffer = response.bytes().await.map_err(|e| {
            JapaneseOCRError::Download(format!("Failed to read {}: {}", filename, e))
        })?;

        fs::write(&dest_path, &buffer).await?;

        Ok(())
    }
}
