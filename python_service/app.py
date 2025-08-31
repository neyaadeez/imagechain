import io
import os
import sys
from typing import List

import numpy as np
from fastapi import FastAPI, File, UploadFile, HTTPException
from fastapi.responses import JSONResponse
from PIL import Image

# Lazy import torch/open_clip to allow the app to start even if not installed yet
try:
    import torch
    import open_clip
except Exception as e:
    print(f"[startup] Warning: failed to import torch/open_clip: {e}", file=sys.stderr)
    open_clip = None
    torch = None

app = FastAPI(title="Embedding Service", version="0.1.0")

MODEL_NAME = os.getenv("MODEL_NAME", "ViT-B-32")
PRETRAINED = os.getenv("PRETRAINED", "laion2b_s34b_b79k")
DEVICE = os.getenv("DEVICE", "cuda" if (torch and torch.cuda.is_available()) else "cpu")

MODEL = None
PREPROCESS = None


def _load_model():
    global MODEL, PREPROCESS
    if open_clip is None or torch is None:
        raise RuntimeError("torch/open_clip not available. Please install dependencies.")
    if MODEL is None:
        print(f"[startup] Loading model name={MODEL_NAME} pretrained={PRETRAINED} device={DEVICE}")
        model, preprocess, _ = open_clip.create_model_and_transforms(
            MODEL_NAME, pretrained=PRETRAINED, device=DEVICE
        )
        model.eval()
        MODEL = model
        PREPROCESS = preprocess


@app.on_event("startup")
async def on_startup():
    try:
        _load_model()
    except Exception as e:
        # Defer hard failure until first request, but log now
        print(f"[startup] Model load failed: {e}", file=sys.stderr)


@app.get("/health")
async def health():
    return {"ok": True, "model": MODEL_NAME, "pretrained": PRETRAINED, "device": DEVICE}

@app.get("/models")
async def models():
    if open_clip is None:
        raise HTTPException(status_code=500, detail="open_clip not available")
    try:
        # Returns dict: {model_name: [list of pretrained tags]}
        return open_clip.list_pretrained()
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to list models: {e}")

@app.get("/config")
async def config():
    return {"MODEL_NAME": MODEL_NAME, "PRETRAINED": PRETRAINED, "DEVICE": DEVICE}


@app.post("/embed")
async def embed(image: UploadFile = File(...)):
    try:
        _load_model()
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Model not available: {e}")

    try:
        data = await image.read()
        pil = Image.open(io.BytesIO(data)).convert("RGB")
    except Exception as e:
        raise HTTPException(status_code=400, detail=f"Invalid image: {e}")

    try:
        x = PREPROCESS(pil).unsqueeze(0)
        if DEVICE == "cuda":
            x = x.cuda(non_blocking=True)
        with torch.no_grad():
            feats = MODEL.encode_image(x)
            feats = feats / feats.norm(dim=-1, keepdim=True)
            vec = feats.squeeze(0).float().cpu().numpy().astype(np.float32)
        return JSONResponse({
            "embedding": vec.tolist(),
            "dim": int(vec.shape[0]),
            "model": MODEL_NAME,
            "pretrained": PRETRAINED,
        })
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Embedding failed: {e}")
