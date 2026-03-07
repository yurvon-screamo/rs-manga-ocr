# Japanese OCR

> WASM Support

High-performance OCR for recognizing Japanese text from japanese images, written in Rust.

## Description

Japanese OCR is a tool for optical character recognition of Japanese text, optimized for japanese content. The project uses the VisionEncoderDecoderModel architecture with ONNX models for efficient inference on CPU.

### Model Architecture

- **Encoder**: ViT (Vision Transformer) based on `facebook/deit-tiny-patch16-224`
- **Decoder**: BERT based on `tohoku-nlp/bert-base-japanese-char-v2`
- **Format**: ONNX for cross-platform compatibility

## Features

- Recognition of Japanese text from images
- Two operating modes: file and clipboard monitoring
- Automatic copying of the result to the clipboard
- CPU inference without external dependencies
- **Automatic model downloading and caching**
- **Configurable model URLs**
- **WASM support with browser caching**

## Installation

### CLI

```bash
cargo install --git https://github.com/yurvon-screamo/japanese-ocr japanese-ocr
```

### Library

```bash
cargo add --git https://github.com/yurvon-screamo/japanese-ocr rs-japanese-ocr
```

## Model Management

By default, the library automatically downloads the ONNX model from HuggingFace on first use and caches it locally:

- **Native platforms**: Models are cached in `~/.cache/manga-ocr/` (or equivalent cache directory on your system)
- **WASM/Browser**: Models are cached using the Cache API for offline use

The default model is [`l0wgear/manga-ocr-2025-onnx`](https://huggingface.co/l0wgear/manga-ocr-2025-onnx), an ONNX version of the manga OCR model fine-tuned by jzhang533. You can find more details in the [model's README](https://huggingface.co/l0wgear/manga-ocr-2025-onnx/blob/main/README.md).

### Custom Model URLs

You can override the model source URL and cache directory:

```rust
use rs_japanese_ocr::{JapaneseOCRModel, ModelConfig};

// Custom model configuration
let config = ModelConfig::new()
    .with_base_url("https://your-mirror.com")
    .with_model_name("your-org/your-model")
    .with_cache_dir(".custom-cache");

let model = JapaneseOCRModel::with_config(config)?;
```

### WASM/Async Usage

For WASM builds, use the async API:

```rust
use rs_japanese_ocr::{JapaneseOCRModel, ModelConfig};

// Async loading for WASM
let model = JapaneseOCRModel::load_async().await?;

// Or with custom config
let config = ModelConfig::new()
    .with_model_name("l0wgear/manga-ocr-2025-onnx");

let model = JapaneseOCRModel::with_config_async(config).await?;
```

### Static Model (Optional)

For offline builds or faster startup, you can bundle the model statically:

1. Download model files using the provided script:
   ```bash
   python download_model.py
   ```

2. Copy downloaded files to `rs-japanese-ocr/src/model/`:
   ```bash
   cp -r ~/.cache/manga-ocr/* rs-japanese-ocr/src/model/
   ```

3. Build with the `static-model` feature:
   ```bash
   cargo build --features static-model
   ```

## Usage

### Clipboard Mode (Default)

The program monitors the clipboard and automatically recognizes text from appearing images:

```bash
japanese-ocr
```

The recognition result is automatically copied back to the clipboard.

### File Mode

```bash
japanese-ocr --mode file --image path/to/image.png
```

### Command Line Arguments

| Argument                      | Description                          | Default      |
| ----------------------------- | ------------------------------------ | ------------ |
| `-i, --image <PATH>`          | Path to the image for recognition    | —            |
| `--mode <MODE>`               | Operating mode: `file` or `clipboard`| `clipboard`  |
| `--refresh-timeout <SECONDS>` | Clipboard polling interval           | `1.0`        |

## Project Structure

```tree
japanese-ocr/
├── rs-japanese-ocr/           # OCR Library
│   ├── src/
│   │   ├── lib.rs             # Public API
│   │   ├── model.rs           # Model Implementation
│   │   ├── config.rs          # Model Configuration
│   │   ├── model_loader_native.rs  # Native Model Loader
│   │   ├── model_loader_wasm.rs    # WASM Model Loader
│   │   └── error.rs           # Error Handling
├── japanese-ocr/              # CLI Application
│   └── src/
│       ├── main.rs            # Entry Point
│       ├── clipboard.rs       # Clipboard Operations
│       └── error.rs           # Error Handling
└── Cargo.toml                 # Workspace Configuration
```

## Technical Details

### Image Preprocessing

- Input image size: 224×224 pixels
- Normalization: mean=[0.5, 0.5, 0.5], std=[0.5, 0.5, 0.5]
- Resize method: Nearest Neighbor

### Text Generation

- Maximum sequence length: 300 tokens
- Autocorrection with `[CLS]` (start) and `[SEP]` (end) tokens
- Removal of spaces from the final result

### Dependencies

| Library       | Purpose                          |
| ------------- | -------------------------------- |
| `candle-core` | Tensor computations              |
| `candle-onnx` | Working with ONNX models         |
| `tokenizers`  | Text tokenization (HuggingFace)  |
| `image`       | Image processing                 |
| `clap`        | Parsing CLI arguments            |
| `arboard`     | Clipboard operations             |
| `ureq`        | HTTP client for model downloads  |
| `dirs`        | Cache directory detection         |

## Usage as a Library

```rust
use rs_japanese_ocr::JapaneseOCRModel;
use image;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Model will be automatically downloaded on first use
    let mut model = JapaneseOCRModel::load()?;
    
    let img = image::ImageReader::open("japanese.png")?.decode()?;
    let text = model.run(&img)?;
    
    println!("Recognized text: {}", text);
    Ok(())
}
```

Add to `Cargo.toml`:

```bash
cargo add --git https://github.com/yurvon-screamo/japanese-ocr rs-japanese-ocr
```

## License

The project is distributed under the GNU AGPL v3 license. Details in the [LICENSE](LICENSE) file.
