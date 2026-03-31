use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{InputCallbackInfo, SampleFormat, StreamConfig};
use std::sync::mpsc::{self, SyncSender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CaptureSettings {
    pub min_record_seconds: f32,
    pub max_record_seconds: f32,
    pub silence_seconds: f32,
    pub rms_threshold: f32,
}

#[derive(Debug, Clone)]
pub struct CapturedAudio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

struct CaptureState {
    started: bool,
    finished: bool,
    channels: usize,
    samples: Vec<f32>,
    speech_frames: u64,
    silence_frames: u64,
    min_frames: u64,
    max_frames: u64,
    silence_limit_frames: u64,
    rms_threshold: f32,
    done_tx: Option<SyncSender<()>>,
}

impl CaptureState {
    fn push_input(&mut self, input: &[f32]) {
        if self.finished || input.is_empty() {
            return;
        }

        let mono: Vec<f32> = if self.channels == 1 {
            input.to_vec()
        } else {
            input
                .chunks(self.channels)
                .map(|frame| frame.iter().copied().sum::<f32>() / self.channels as f32)
                .collect()
        };

        let level = rms(&mono);
        let frames = mono.len() as u64;

        if !self.started {
            if level >= self.rms_threshold {
                self.started = true;
                self.samples.extend_from_slice(&mono);
                self.speech_frames += frames;
                self.silence_frames = 0;
            }
            return;
        }

        self.samples.extend_from_slice(&mono);
        self.speech_frames += frames;

        if level < self.rms_threshold {
            self.silence_frames += frames;
        } else {
            self.silence_frames = 0;
        }

        let min_ok = self.speech_frames >= self.min_frames;
        let silence_ok = self.silence_frames >= self.silence_limit_frames;
        let too_long = self.speech_frames >= self.max_frames;

        if (min_ok && silence_ok) || too_long {
            self.finished = true;
            if let Some(tx) = self.done_tx.take() {
                let _ = tx.send(());
            }
        }
    }
}

pub fn capture_utterance(settings: &CaptureSettings) -> Result<CapturedAudio> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| anyhow!("no default input device found"))?;

    let supported = device
        .default_input_config()
        .context("failed to get default input config")?;

    let sample_rate = supported.sample_rate();
    let channels = supported.channels() as usize;

    let config = StreamConfig {
        channels: supported.channels(),
        sample_rate,
        buffer_size: cpal::BufferSize::Default,
    };

    let min_frames = (settings.min_record_seconds * sample_rate as f32) as u64;
    let max_frames = (settings.max_record_seconds * sample_rate as f32) as u64;
    let silence_limit_frames = (settings.silence_seconds * sample_rate as f32) as u64;

    let (done_tx, done_rx) = mpsc::sync_channel::<()>(1);

    let state = Arc::new(Mutex::new(CaptureState {
        started: false,
        finished: false,
        channels,
        samples: Vec::with_capacity(max_frames as usize),
        speech_frames: 0,
        silence_frames: 0,
        min_frames,
        max_frames,
        silence_limit_frames,
        rms_threshold: settings.rms_threshold,
        done_tx: Some(done_tx),
    }));

    let err_fn = |err| eprintln!("audio stream error: {err}");

    let stream = match supported.sample_format() {
        SampleFormat::F32 => build_input_stream::<f32>(&device, &config, state.clone(), err_fn)?,
        SampleFormat::I16 => build_input_stream::<i16>(&device, &config, state.clone(), err_fn)?,
        SampleFormat::U16 => build_input_stream::<u16>(&device, &config, state.clone(), err_fn)?,
        other => return Err(anyhow!("unsupported input sample format: {other:?}")),
    };

    println!("listening... speak now.");
    stream.play().context("failed to start input stream")?;

    done_rx
        .recv_timeout(Duration::from_secs_f32(settings.max_record_seconds + 5.0))
        .context("timed out while waiting for audio")?;

    drop(stream);

    let state = Arc::try_unwrap(state)
        .map_err(|_| anyhow!("failed to reclaim capture state"))?
        .into_inner()
        .map_err(|_| anyhow!("capture state mutex poisoned"))?;

    if state.samples.is_empty() {
        return Err(anyhow!("no speech detected"));
    }

    Ok(CapturedAudio {
        samples: state.samples,
        sample_rate,
    })
}

fn build_input_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    state: Arc<Mutex<CaptureState>>,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<cpal::Stream>
where
    T: cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &InputCallbackInfo| {
            let chunk: Vec<f32> = data.iter().map(|s| (*s).to_sample::<f32>()).collect();
            if let Ok(mut state) = state.lock() {
                state.push_input(&chunk);
            }
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let power = samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32;
    power.sqrt()
}

pub fn resample_to_16khz_mono(input: &[f32], src_rate: u32) -> Vec<f32> {
    const TARGET_RATE: u32 = 16_000;

    if src_rate == TARGET_RATE {
        return input.to_vec();
    }

    let ratio = src_rate as f32 / TARGET_RATE as f32;
    let out_len = ((input.len() as f32) / ratio).floor() as usize;
    let mut out = Vec::with_capacity(out_len);

    for i in 0..out_len {
        let pos = i as f32 * ratio;
        let idx = pos.floor() as usize;
        let frac = pos - idx as f32;

        let a = input.get(idx).copied().unwrap_or(0.0);
        let b = input.get(idx + 1).copied().unwrap_or(a);

        out.push(a + (b - a) * frac);
    }

    out
}

pub fn save_wav_mono_16khz(path: &std::path::Path, samples: &[f32]) -> Result<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut writer = hound::WavWriter::create(path, spec)?;
    for &sample in samples {
        let s = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        writer.write_sample(s)?;
    }
    writer.finalize()?;

    Ok(())
}
