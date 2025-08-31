# Python embedding service Dockerfile
FROM python:3.10-slim

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1

# System deps for Pillow and general build
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    libjpeg-dev \
    zlib1g-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY python_service/requirements.txt /app/requirements.txt

# Install CPU-only torch first (smaller and avoids CUDA issues)
RUN pip install --no-cache-dir --upgrade pip \
    && pip install --no-cache-dir torch --index-url https://download.pytorch.org/whl/cpu \
    && pip install --no-cache-dir -r /app/requirements.txt

COPY python_service /app/python_service

EXPOSE 8001

ENV MODEL_NAME=ViT-B-32 \
    PRETRAINED=laion2b_s34b_b79k \
    DEVICE=cpu

CMD ["uvicorn", "python_service.app:app", "--host", "0.0.0.0", "--port", "8001"]
