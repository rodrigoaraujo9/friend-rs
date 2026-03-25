### Install

```bash
brew install ollama
brew install pipx
pipx install piper-tts
```

### Start Ollama

```bash
ollama serve
ollama pull qwen2.5:0.5b
```

### Download models

```bash
mkdir -p models voices tmp

curl -L -o models/ggml-base.en.bin \
https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin

curl -L -o voices/en_US-lessac-medium.onnx \
https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium/en_US-lessac-medium.onnx
```

### Run project

```bash
cargo run --release
```
