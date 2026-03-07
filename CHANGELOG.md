# Japanese OCR - Changelog

## [Unreleased] - 2025-03-07

### Changed
- **BREAKING**: Removed model files from repository and Git LFS
- Models are now downloaded automatically from HuggingFace on first use
- Added model caching system:
  - Native platforms: `~/.cache/manga-ocr/` (or OS-specific cache directory)
  - WASM/Browser: Cache API for offline support
- Added configurable model URLs via `ModelConfig`
- Added async model loading for WASM (`load_async()`, `with_config_async()`)
- Updated dependencies:
  - Added `ureq` for HTTP downloads (native)
  - Added `dirs` for cache directory detection (native)
  - Added `wasm-bindgen`, `web-sys` for browser support (WASM)
  - Added `serde`, `serde_json` for configuration (WASM)

### Added
- `ModelConfig` struct for customizing model source and cache location
- `ModelLoader` for handling model downloads and caching
- Automatic model download from HuggingFace
- Cache validation to avoid re-downloading existing models
- Browser Cache API support for WASM builds
- Example code in `examples/basic.rs`
- WASM usage documentation in `docs/WASM.md`

### Migration Guide

#### For Library Users

**Before:**
```rust
let model = JapaneseOCRModel::load()?;
```

**After:**
```rust
// Same API - now downloads automatically on first use
let model = JapaneseOCRModel::load()?;
```

**With Custom Configuration:**
```rust
let config = ModelConfig::new()
    .with_base_url("https://your-mirror.com")
    .with_model_name("your-org/your-model")
    .with_cache_dir(".custom-cache");

let model = JapaneseOCRModel::with_config(config)?;
```

#### For WASM Users

**Before:**
```rust
let model = JapaneseOCRModel::load()?;
```

**After:**
```rust
// Async API for WASM
let model = JapaneseOCRModel::load_async().await?;
```

#### For Offline Builds

If you need offline builds with bundled models:

1. Download model files manually:
   ```bash
   python download_model.py
   ```

2. Copy files to `rs-japanese-ocr/src/model/`

3. Build with static-model feature:
   ```bash
   cargo build --features static-model
   ```

### Model Information

Default model: [`l0wgear/manga-ocr-2025-onnx`](https://huggingface.co/l0wgear/manga-ocr-2025-onnx)

Model files (~140MB total):
- `encoder_model.onnx` (~45MB)
- `decoder_model.onnx` (~95MB)
- `tokenizer.json` (~500KB)
- Additional config files

### Performance

- First run: Downloads model (~140MB, time depends on connection)
- Subsequent runs: Loads from cache (~1-2 seconds)
- Recognition: 100-500ms per image (depends on hardware)

### Known Limitations

1. WASM builds require HTTPS for Cache API
2. Initial model download may be slow on first use
3. Model files require ~200MB of disk/memory

### Future Improvements

- [ ] Progress callbacks for model downloads
- [ ] Model versioning and updates
- [ ] Multiple model support
- [ ] Compression for faster downloads
- [ ] WebWorker support for background processing
