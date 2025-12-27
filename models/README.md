# Kaido AI Models

This directory contains AI models for the Kaido AI Shell.

## Model Requirements

To use real AI inference, you need to download a GGUF model file:

### Recommended Models

1. **Phi-3.5-mini** (Recommended for beginners)
   - Size: ~2.3GB
   - Download: `wget https://huggingface.co/microsoft/Phi-3.5-mini-instruct-gguf/resolve/main/Phi-3.5-mini-instruct-q4.gguf`
   - Rename to: `phi-3.5-mini.gguf`

2. **Llama-3.2-3B** (Alternative)
   - Size: ~2GB
   - Download: `wget https://huggingface.co/bartowski/Llama-3.2-3B-Instruct-GGUF/resolve/main/Llama-3.2-3B-Instruct-Q4_K_M.gguf`
   - Rename to: `phi-3.5-mini.gguf`

## Current Status

-  Model loading infrastructure ready
-  GGUF support configured
- ️ Model file needed for real inference
-  Currently using mock responses

## Testing Without Model

The shell will work with mock AI responses for testing basic functionality.

