use crate::config::ModelConfig;
use crate::error::JapaneseOCRError;
use js_sys::{Array, Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Cache, CacheStorage, Request, RequestInit, RequestMode, Response, Window};

pub struct ModelLoader {
    config: ModelConfig,
}

impl ModelLoader {
    pub fn new(config: ModelConfig) -> Self {
        Self { config }
    }

    pub async fn load_or_download_model(&self) -> Result<ModelFiles, JapaneseOCRError> {
        let window = web_sys::window()
            .ok_or_else(|| JapaneseOCRError::Cache("No window object available".to_string()))?;

        let cache = self.get_or_create_cache(&window).await?;

        if self.is_model_cached(&cache).await? {
            self.load_from_cache(&cache).await
        } else {
            self.download_and_cache_model(&cache).await
        }
    }

    async fn get_or_create_cache(&self, window: &Window) -> Result<Cache, JapaneseOCRError> {
        let cache_storage = window.caches().map_err(|e| {
            JapaneseOCRError::Cache(format!("Failed to get cache storage: {:?}", e))
        })?;

        let safe_model_name: String = self
            .config
            .model_name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let cache_name = format!("manga-ocr-model-{}", safe_model_name);

        let cache_exists = JsFuture::from(cache_storage.has(&cache_name))
            .await
            .map_err(|e| JapaneseOCRError::Cache(format!("Failed to check cache: {:?}", e)))?
            .as_bool()
            .ok_or_else(|| {
                JapaneseOCRError::Cache("Failed to convert cache exists to bool".to_string())
            })?;

        let cache = if cache_exists {
            JsFuture::from(cache_storage.open(&cache_name))
                .await
                .map_err(|e| JapaneseOCRError::Cache(format!("Failed to open cache: {:?}", e)))?
        } else {
            JsFuture::from(cache_storage.open(&cache_name))
                .await
                .map_err(|e| JapaneseOCRError::Cache(format!("Failed to create cache: {:?}", e)))?
        };

        cache
            .dyn_into::<Cache>()
            .map_err(|e| JapaneseOCRError::Cache(format!("Failed to cast to Cache: {:?}", e)))
    }

    async fn is_model_cached(&self, cache: &Cache) -> Result<bool, JapaneseOCRError> {
        for file in ModelConfig::required_files() {
            let request = Request::new_with_str(file).map_err(|e| {
                JapaneseOCRError::Cache(format!("Failed to create request: {:?}", e))
            })?;

            let has_response = JsFuture::from(cache.match_with_request(&request))
                .await
                .map_err(|e| JapaneseOCRError::Cache(format!("Failed to check cache: {:?}", e)))?
                .as_bool()
                .ok_or_else(|| {
                    JapaneseOCRError::Cache("Failed to convert match result to bool".to_string())
                })?;

            if !has_response {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn load_from_cache(&self, cache: &Cache) -> Result<ModelFiles, JapaneseOCRError> {
        let encoder = self
            .load_file_from_cache(cache, "encoder_model.onnx")
            .await?;
        let decoder = self
            .load_file_from_cache(cache, "decoder_model.onnx")
            .await?;
        let tokenizer = self.load_file_from_cache(cache, "tokenizer.json").await?;

        Ok(ModelFiles {
            encoder,
            decoder,
            tokenizer,
        })
    }

    async fn load_file_from_cache(
        &self,
        cache: &Cache,
        filename: &str,
    ) -> Result<Vec<u8>, JapaneseOCRError> {
        let request = Request::new_with_str(filename)
            .map_err(|e| JapaneseOCRError::Cache(format!("Failed to create request: {:?}", e)))?;

        let response = JsFuture::from(cache.match_with_request(&request))
            .await
            .map_err(|e| JapaneseOCRError::Cache(format!("Failed to get from cache: {:?}", e)))?
            .ok_or_else(|| JapaneseOCRError::Cache(format!("File {} not in cache", filename)))?
            .dyn_into::<Response>()
            .map_err(|e| JapaneseOCRError::Cache(format!("Failed to cast to Response: {:?}", e)))?;

        let array_buffer = JsFuture::from(response.array_buffer().map_err(|e| {
            JapaneseOCRError::Cache(format!("Failed to get array buffer: {:?}", e))
        })?)
        .await
        .map_err(|e| JapaneseOCRError::Cache(format!("Failed to read array buffer: {:?}", e)))?;

        let uint8_array = Uint8Array::new(&array_buffer);
        Ok(uint8_array.to_vec())
    }

    async fn download_and_cache_model(
        &self,
        cache: &Cache,
    ) -> Result<ModelFiles, JapaneseOCRError> {
        let all_files: Vec<&str> = ModelConfig::required_files()
            .iter()
            .chain(ModelConfig::optional_files().iter())
            .copied()
            .collect();

        for filename in &all_files {
            self.download_and_cache_file(cache, filename).await?;
        }

        self.load_from_cache(cache).await
    }

    async fn download_and_cache_file(
        &self,
        cache: &Cache,
        filename: &str,
    ) -> Result<(), JapaneseOCRError> {
        let url = self.config.model_file_url(filename);

        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(&url, &opts).map_err(|e| {
            JapaneseOCRError::Download(format!("Failed to create request: {:?}", e))
        })?;

        let window = web_sys::window()
            .ok_or_else(|| JapaneseOCRError::Download("No window object available".to_string()))?;

        let response = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| {
                JapaneseOCRError::Download(format!("Failed to fetch {}: {:?}", filename, e))
            })?
            .dyn_into::<Response>()
            .map_err(|e| {
                JapaneseOCRError::Download(format!("Failed to cast to Response: {:?}", e))
            })?;

        if !response.ok() {
            return Err(JapaneseOCRError::Download(format!(
                "Failed to download {}: HTTP {}",
                filename,
                response.status()
            )));
        }

        let array_buffer = JsFuture::from(response.array_buffer().map_err(|e| {
            JapaneseOCRError::Download(format!("Failed to get array buffer: {:?}", e))
        })?)
        .await
        .map_err(|e| {
            JapaneseOCRError::Download(format!("Failed to read response body: {:?}", e))
        })?;

        let cache_request = Request::new_with_str(filename).map_err(|e| {
            JapaneseOCRError::Cache(format!("Failed to create cache request: {:?}", e))
        })?;

        let cache_response_init = web_sys::ResponseInit::new();
        let cache_response = Response::new_with_opt_u8_array_and_init(
            Some(&Uint8Array::new(&array_buffer).to_vec()),
            &cache_response_init,
        )
        .map_err(|e| {
            JapaneseOCRError::Cache(format!("Failed to create cache response: {:?}", e))
        })?;

        JsFuture::from(cache.put_with_request_response(&cache_request, &cache_response))
            .await
            .map_err(|e| {
                JapaneseOCRError::Cache(format!("Failed to cache {}: {:?}", filename, e))
            })?;

        Ok(())
    }
}

pub struct ModelFiles {
    pub encoder: Vec<u8>,
    pub decoder: Vec<u8>,
    pub tokenizer: Vec<u8>,
}
