mod config;
mod error;
mod model;

#[cfg(not(target_arch = "wasm32"))]
mod model_loader_native;

#[cfg(target_arch = "wasm32")]
mod model_loader_wasm;

pub use config::ModelConfig;
pub use error::JapaneseOCRError;
pub use model::JapaneseOCRModel;

#[cfg(not(target_arch = "wasm32"))]
pub use model_loader_native::{ModelFiles, ModelLoader};

#[cfg(target_arch = "wasm32")]
pub use model_loader_wasm::{ModelFiles, ModelLoader};
