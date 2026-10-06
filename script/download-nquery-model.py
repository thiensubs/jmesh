#!/usr/bin/env python3
"""Download the ONNX text-to-SQL model files jmesh nquery needs.

Fetches tokenizer.json plus a plain (non-merged) encoder/decoder ONNX pair
from Xenova/t5-small-awesome-text-to-sql into models/nquery/.
"""

from pathlib import Path

from huggingface_hub import hf_hub_download

REPO_ID = "Xenova/t5-small-awesome-text-to-sql"
MODEL_DIR = Path("models/nquery")

# The engine runs a plain encoder + a non-merged decoder (no past-key-value
# plumbing), so "merged"/"with_past" decoder variants are intentionally skipped.
ONNX_FILES = [
    "onnx/encoder_model_quantized.onnx",
    "onnx/decoder_model_quantized.onnx",
]
META_FILES = ["tokenizer.json", "tokenizer_config.json", "config.json", "spiece.model"]


def main():
    MODEL_DIR.mkdir(parents=True, exist_ok=True)

    for name in ONNX_FILES + META_FILES:
        print(f"⬇️  {name}")
        path = hf_hub_download(repo_id=REPO_ID, filename=name, local_dir=str(MODEL_DIR))
        print(f"   ✓ {path}")

    # Flatten the onnx/ subdirectory next to the other files.
    onnx_dir = MODEL_DIR / "onnx"
    if onnx_dir.exists():
        for f in onnx_dir.glob("*.onnx"):
            target = MODEL_DIR / f.name
            if target.exists():
                f.unlink()
            else:
                f.rename(target)
        onnx_dir.rmdir()

    print(f"\n✅ Done. Files in {MODEL_DIR.resolve()}:")
    for f in sorted(MODEL_DIR.iterdir()):
        if f.is_file():
            size = f.stat().st_size / (1024 * 1024)
            print(f"   {f.name:40s} {size:6.1f} MB")


if __name__ == "__main__":
    main()
