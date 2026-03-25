use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub ollama_base_url: String,
    pub ollama_model: String,
    pub system_prompt: String,
    pub whisper_model_path: PathBuf,
    pub whisper_language: Option<String>,
    pub whisper_threads: usize,
    pub piper_executable: String,
    pub piper_voice_path: PathBuf,
    pub piper_output_path: PathBuf,
    pub min_record_seconds: f32,
    pub max_record_seconds: f32,
    pub silence_seconds: f32,
    pub rms_threshold: f32,
}

impl Config {
    pub fn from_env() -> Self {
        let _ = dotenvy::from_filename_override(".env");

        Self {
            ollama_base_url: env_or("OLLAMA_BASE_URL", "http://localhost:11434"),
            ollama_model: env_or("OLLAMA_MODEL", "qwen2.5:0.5b"),
            system_prompt: env_or(
                "SYSTEM_PROMPT",
                "You are Friend, a local voice companion. Speak naturally, warmly, and clearly, like a thoughtful friend having a real conversation. Keep answers concise unless asked to explain more. Avoid sounding robotic, formal, or like a generic assistant. Show curiosity, emotional intelligence, and natural conversational rhythm.",
            ),
            whisper_model_path: PathBuf::from(env_or(
                "WHISPER_MODEL_PATH",
                "./models/ggml-base.en.bin",
            )),
            whisper_language: env::var("WHISPER_LANGUAGE").ok(),
            whisper_threads: env_or("WHISPER_THREADS", "4").parse().unwrap_or(4),
            piper_executable: env_or("PIPER_EXECUTABLE", "piper"),
            piper_voice_path: PathBuf::from(env_or(
                "PIPER_VOICE_PATH",
                "./voices/en_US-lessac-medium.onnx",
            )),
            piper_output_path: PathBuf::from(env_or("PIPER_OUTPUT_PATH", "./tmp/tts.wav")),
            min_record_seconds: env_or("MIN_RECORD_SECONDS", "0.8").parse().unwrap_or(0.8),
            max_record_seconds: env_or("MAX_RECORD_SECONDS", "20.0").parse().unwrap_or(20.0),
            silence_seconds: env_or("SILENCE_SECONDS", "0.9").parse().unwrap_or(0.9),
            rms_threshold: env_or("RMS_THRESHOLD", "0.015").parse().unwrap_or(0.015),
        }
    }
}

fn env_or(key: &str, fallback: &str) -> String {
    env::var(key).unwrap_or_else(|_| fallback.to_string())
}
