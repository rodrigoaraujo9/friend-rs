mod audio;
mod config;
mod memory;
mod ollama;
mod stt;
mod tts;

use anyhow::{Context, Result};
use audio::{capture_utterance, resample_to_16khz_mono, save_wav_mono_16khz, CaptureSettings};
use config::Config;
use memory::ConversationMemory;
use ollama::OllamaClient;
use std::path::Path;
use std::process;
use std::sync::{Arc, Mutex};
use stt::SttEngine;
use tts::TtsEngine;

#[tokio::main]
async fn main() -> Result<()> {
    let mode = std::env::args()
        .nth(1)
        .context("missing mode argument. use: cargo run --release -- [portuguese|english]")?;

    let cfg = Config::from_arg(&mode)?;

    println!("{} is booting up...", cfg.name);
    println!("he speaks {}.", cfg.mode_name);
    println!("press Ctrl+C to quit.\n");

    install_instant_ctrl_c();

    let stt = SttEngine::new(
        &cfg.whisper_model_path,
        cfg.whisper_language.clone(),
        cfg.whisper_threads,
    )?;

    let ollama = OllamaClient::new(cfg.ollama_base_url.clone(), cfg.ollama_model.clone());

    let tts = TtsEngine::new(
        cfg.piper_executable.clone(),
        cfg.piper_voice_path.clone(),
        cfg.piper_output_path.clone(),
    );

    let capture = CaptureSettings {
        min_record_seconds: cfg.min_record_seconds,
        max_record_seconds: cfg.max_record_seconds,
        silence_seconds: cfg.silence_seconds,
        rms_threshold: cfg.rms_threshold,
    };

    let memory = Arc::new(Mutex::new(ConversationMemory::new(
        cfg.system_prompt.clone(),
        24,
    )));

    loop {
        if let Err(err) = run_turn(&capture, &stt, &ollama, &tts, &memory, &cfg).await {
            eprintln!("error: {err:#}");
        }
    }
}

fn install_instant_ctrl_c() {
    tokio::spawn(async {
        let _ = tokio::signal::ctrl_c().await;
        eprintln!("\nshutting down.");
        process::exit(130);
    });
}

async fn run_turn(
    capture: &CaptureSettings,
    stt: &SttEngine,
    ollama: &OllamaClient,
    tts: &TtsEngine,
    memory: &Arc<Mutex<ConversationMemory>>,
    cfg: &Config,
) -> Result<()> {
    let audio = capture_utterance(capture)?;
    let audio_16k = resample_to_16khz_mono(&audio.samples, audio.sample_rate);

    save_wav_mono_16khz(Path::new("./tmp/last_input.wav"), &audio_16k)
        .context("failed to write debug input WAV")?;

    let transcript = stt.transcribe(&audio_16k)?;
    if transcript.trim().is_empty() {
        println!("heard nothing useful.\n");
        return Ok(());
    }

    println!("you: {transcript}");

    let messages = {
        let mut memory = memory.lock().unwrap();
        memory.push_user(transcript.clone());
        memory.messages_for_model()
    };

    let reply = ollama.chat(&messages).await?;
    println!("{}: {reply}\n", cfg.name);

    {
        let mut memory = memory.lock().unwrap();
        memory.push_assistant(reply.clone());
    }

    tts.speak(&reply)?;

    Ok(())
}
