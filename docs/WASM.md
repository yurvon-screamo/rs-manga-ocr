# WASM Usage

This document explains how to use the Japanese OCR library in WebAssembly environments.

## Browser Setup

The library automatically uses the Cache API for model storage in browser environments, providing offline capabilities after the initial download.

### Installation

```bash
npm install rs-japanese-ocr
```

### Basic Usage

```javascript
import init, { JapaneseOCRModel, ModelConfig } from 'rs-japanese-ocr';

async function recognizeText(imageData) {
    // Initialize the WASM module
    await init();
    
    // Load the model (downloads and caches automatically)
    const config = new ModelConfig();
    const model = await JapaneseOCRModel.withConfigAsync(config);
    
    // Recognize text from image
    const text = model.run(imageData);
    
    return text;
}
```

### Custom Configuration

```javascript
import { JapaneseOCRModel, ModelConfig } from 'rs-japanese-ocr';

const config = new ModelConfig()
    .withBaseUrl('https://your-mirror.com')
    .withModelName('your-org/your-model')
    .withCacheDir('custom-cache');

const model = await JapaneseOCRModel.withConfigAsync(config);
```

### Working with Canvas

```javascript
import init, { JapaneseOCRModel } from 'rs-japanese-ocr';

async function recognizeFromCanvas(canvas) {
    await init();
    
    const model = await JapaneseOCRModel.loadAsync();
    
    // Get image data from canvas
    const ctx = canvas.getContext('2d');
    const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
    
    // Convert to format expected by the library
    const pixels = new Uint8Array(imageData.data.buffer);
    
    // Recognize
    const text = model.runFromPixels(
        pixels,
        canvas.width,
        canvas.height
    );
    
    return text;
}
```

### React Example

```jsx
import React, { useState, useEffect } from 'react';
import init, { JapaneseOCRModel } from 'rs-japanese-ocr';

function OCRComponent() {
    const [model, setModel] = useState(null);
    const [result, setResult] = useState('');
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        async function loadModel() {
            await init();
            const model = await JapaneseOCRModel.loadAsync();
            setModel(model);
            setLoading(false);
        }
        loadModel();
    }, []);

    const handleImageUpload = async (event) => {
        const file = event.target.files[0];
        if (!file || !model) return;

        const img = new Image();
        img.onload = async () => {
            const canvas = document.createElement('canvas');
            canvas.width = img.width;
            canvas.height = img.height;
            const ctx = canvas.getContext('2d');
            ctx.drawImage(img, 0, 0);
            
            const imageData = ctx.getImageData(0, 0, img.width, img.height);
            const pixels = new Uint8Array(imageData.data.buffer);
            
            const text = model.runFromPixels(pixels, img.width, img.height);
            setResult(text);
        };
        img.src = URL.createObjectURL(file);
    };

    if (loading) return <div>Loading model...</div>;

    return (
        <div>
            <input type="file" accept="image/*" onChange={handleImageUpload} />
            {result && <div>Recognized: {result}</div>}
        </div>
    );
}
```

## Browser Compatibility

The library uses modern browser APIs:

- **Cache API**: For persistent model storage
- **Fetch API**: For downloading models
- **WebAssembly**: For running the OCR model

Supported browsers:
- Chrome 66+
- Firefox 57+
- Safari 11.1+
- Edge 79+

## Performance Considerations

1. **Initial Load**: The model is ~140MB and downloads on first use. Consider showing a progress indicator.

2. **Memory**: The WASM module requires approximately 200MB of RAM.

3. **Processing Time**: Typical recognition takes 100-500ms depending on image size and device performance.

## Caching

Models are cached using the Cache API with a unique cache name based on the model identifier. To clear the cache:

```javascript
async function clearModelCache() {
    const cacheName = 'manga-ocr-model-l0wgear-manga-ocr-2025-onnx';
    const cache = await caches.delete(cacheName);
    console.log('Cache cleared:', cache);
}
```

## Offline Support

After the initial download, the library works completely offline. You can verify offline capability:

```javascript
if ('serviceWorker' in navigator) {
    // Your app can work offline
    const model = await JapaneseOCRModel.loadAsync();
}
```

## Troubleshooting

### CORS Issues

If you're serving the WASM files from a different origin, ensure proper CORS headers:

```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET
```

### Memory Issues

If you encounter memory errors, try:
1. Reducing image size before processing
2. Clearing browser cache
3. Using a smaller model variant

### Cache Issues

If models aren't caching properly:
1. Check if Cache API is enabled in your browser
2. Verify HTTPS is being used (required for Cache API)
3. Check browser storage quotas
