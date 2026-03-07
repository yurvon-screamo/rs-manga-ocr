#!/usr/bin/env python3
"""Script to download model files from HuggingFace"""

import os
import urllib.request
from pathlib import Path

MODEL_NAME = "l0wgear/manga-ocr-2025-onnx"
BASE_URL = f"https://huggingface.co/{MODEL_NAME}/resolve/main"
CACHE_DIR = Path.home() / ".cache" / "manga-ocr"

REQUIRED_FILES = [
    "encoder_model.onnx",
    "decoder_model.onnx",
    "tokenizer.json",
]

OPTIONAL_FILES = [
    "config.json",
    "preprocessor_config.json",
    "special_tokens_map.json",
    "tokenizer_config.json",
    "vocab.txt",
    "generation_config.json",
]


def download_file(filename: str, dest_dir: Path):
    """Download a single file from HuggingFace"""
    url = f"{BASE_URL}/{filename}"
    dest_path = dest_dir / filename

    if dest_path.exists():
        print(f"✓ {filename} already exists, skipping")
        return

    print(f"↓ Downloading {filename}...")
    try:
        urllib.request.urlretrieve(url, dest_path)
        print(f"✓ Downloaded {filename}")
    except Exception as e:
        print(f"✗ Failed to download {filename}: {e}")
        if dest_path.exists():
            dest_path.unlink()


def main():
    """Download all model files"""
    print(f"Model: {MODEL_NAME}")
    print(f"Cache directory: {CACHE_DIR}")

    # Create cache directory
    CACHE_DIR.mkdir(parents=True, exist_ok=True)

    # Download required files
    print("\nDownloading required files:")
    for filename in REQUIRED_FILES:
        download_file(filename, CACHE_DIR)

    # Download optional files
    print("\nDownloading optional files:")
    for filename in OPTIONAL_FILES:
        download_file(filename, CACHE_DIR)

    print("\n✓ Model download complete!")


if __name__ == "__main__":
    main()
