mod audio;
mod config;
mod ollama;
mod stt;
mod tts;

use anyhow::{Context, Result};
use audio::{capture_utterance, resample_to_16khz_mono, save_wav_mono_16khz, CaptureSettings};
use config::Config;
use ollama::{ChatMessage, OllamaClient};
use std::sync::{Arc, Mutex};
use stt::SttEngine;
use tokio::signal;
use tts::TtsEngine;

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Config::from_env();

    println!("Local voice assistant starting...");
    println!("Ollama model: {}", cfg.ollama_model);
    println!("Whisper model: {}", cfg.whisper_model_path.display());
    println!("Piper voice: {}", cfg.piper_voice_path.display());
    println!("Press Ctrl+C to quit.\n");

    let stt = Arc::new(SttEngine::new(
        &cfg.whisper_model_path,
        cfg.whisper_language.clone(),
        cfg.whisper_threads,
    )?);

    let ollama = Arc::new(OllamaClient::new(&cfg.ollama_base_url, &cfg.ollama_model));

    let tts = Arc::new(TtsEngine::new(
        cfg.piper_executable.clone(),
        cfg.piper_voice_path.clone(),
        cfg.piper_output_path.clone(),
    ));

    let capture = CaptureSettings {
        min_record_seconds: cfg.min_record_seconds,
        max_record_seconds: cfg.max_record_seconds,
        silence_seconds: cfg.silence_seconds,
        rms_threshold: cfg.rms_threshold,
    };

    let history = Arc::new(Mutex::new(vec![ChatMessage {
        role: "system".to_string(),
        content: cfg.system_prompt.clone(),
    }]));

    loop {
        let stt = Arc::clone(&stt);
        let ollama = Arc::clone(&ollama);
        let tts = Arc::clone(&tts);
        let history = Arc::clone(&history);
        let capture = capture.clone();

        tokio::select! {
            _ = signal::ctrl_c() => {
                println!("\nShutting down.");
                break;
            }
            res = tokio::task::spawn_blocking(move || {
                let rt = tokio::runtime::Handle::current();
                rt.block_on(run_turn(&capture, &stt, &ollama, &tts, &history))
            }) => {
                match res {
                    Ok(Ok(())) => {}
                    Ok(Err(err)) => eprintln!("error: {err:#}"),
                    Err(err) => eprintln!("task error: {err}"),
                }
            }
        }
    }

    Ok(())
}

async fn run_turn(
    capture: &CaptureSettings,
    stt: &SttEngine,
    ollama: &OllamaClient,
    tts: &TtsEngine,
    history: &Arc<Mutex<Vec<ChatMessage>>>,
) -> Result<()> {
    let audio = capture_utterance(capture)?;
    let audio_16k = resample_to_16khz_mono(&audio.samples, audio.sample_rate);

    let debug_path = std::path::Path::new("./tmp/last_input.wav");
    save_wav_mono_16khz(debug_path, &audio_16k).context("failed to write debug input WAV")?;

    let transcript = stt.transcribe(&audio_16k)?;
    if transcript.trim().is_empty() {
        println!("Heard nothing useful.\n");
        return Ok(());
    }

    println!("You: {transcript}");

    let messages = {
        let mut history = history.lock().unwrap();
        history.push(ChatMessage {
            role: "user".to_string(),
            content: transcript,
        });
        history.clone()
    };

    let reply = ollama.chat(&messages).await?;
    println!("Assistant: {reply}\n");

    {
        let mut history = history.lock().unwrap();
        history.push(ChatMessage {
            role: "assistant".to_string(),
            content: reply.clone(),
        });
        trim_history(&mut history, 12);
    }

    tts.speak(&reply)?;

    Ok(())
}

fn trim_history(history: &mut Vec<ChatMessage>, keep: usize) {
    if history.is_empty() {
        return;
    }

    let system = history[0].clone();
    let mut tail = history[1..].to_vec();

    if tail.len() > keep {
        let cut = tail.len() - keep;
        tail = tail.split_off(cut);
    }

    history.clear();
    history.push(system);
    history.extend(tail);
}
