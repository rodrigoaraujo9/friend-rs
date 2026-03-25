<br />
<div align="center">
  <h3 align="center">friend-rs</h3>
  <p align="center">
      A terminal based friend in Rust.
  </p>
</div>

<div align="center">
    <img src="./assets/demo.gif" alt="Demo">
</div>

## About

This is a personal project built out of curiousity to explore what a small local companion can feel like when speech recognition, language models, and voice synthesis run entirely on-device. How human can we make this models sound and, really, how convincing are they?

I built this in Rust and implemented none of the core features. you can check the libs I used bellow. I am currently using qwen since the model is great



### Features

- [x] Capture live microphone input.
- [x] Transcribe speech locally with Whisper.
- [x] Generate responses with Ollama.
- [x] Speak replies using Piper.

### Built With

These are some of the tools used to build this project.

- `cpal` for audio capture.
- `whisper-rs` for speech recognition.
- `ollama` for local language inference.
- `piper` for voice synthesis.

### Install

```bash
# install
brew install ollama
brew install pipx
pipx install piper-tts
# start Ollama
ollama serve
ollama pull qwen2.5:0.5b
# download models
mkdir -p models voices tmp
# english setup
curl --fail --location --output ./models/ggml-base.en.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin
curl --fail --location --output ./voices/en_US-lessac-medium.onnx \
  https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium/en_US-lessac-medium.onnx
curl --fail --location --output ./voices/en_US-lessac-medium.onnx.json \
  https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium/en_US-lessac-medium.onnx.json

# portuguese setup
curl --fail --location --output ./models/ggml-base.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
curl --fail --location --output ./voices/pt_PT-tugao-medium.onnx \
  "https://huggingface.co/rhasspy/piper-voices/resolve/main/pt/pt_PT/tug%C3%A3o/medium/pt_PT-tug%C3%A3o-medium.onnx"
curl --fail --location --output ./voices/pt_PT-tugao-medium.onnx.json \
  "https://huggingface.co/rhasspy/piper-voices/resolve/main/pt/pt_PT/tug%C3%A3o/medium/pt_PT-tug%C3%A3o-medium.onnx.json"
```

### Run!

```bash
cp .env.example .env
cargo run --release
```
<p align="right">(<a href="#readme-top">back to top</a>)</p>
