use anyhow::{Context, Result};
use rodio::{Decoder, OutputStreamBuilder, Sink};
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct TtsEngine {
    executable: String,
    voice_path: PathBuf,
    output_path: PathBuf,
}

impl TtsEngine {
    pub fn new(executable: impl Into<String>, voice_path: PathBuf, output_path: PathBuf) -> Self {
        Self {
            executable: executable.into(),
            voice_path,
            output_path,
        }
    }

    pub fn speak(&self, text: &str) -> Result<()> {
        if text.trim().is_empty() {
            return Ok(());
        }

        if let Some(parent) = self.output_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        self.generate_wav(text, &self.output_path)?;
        play_wav(&self.output_path)?;
        Ok(())
    }

    fn generate_wav(&self, text: &str, out: &Path) -> Result<()> {
        let mut child = Command::new(&self.executable)
            .args([
                "--model",
                &self.voice_path.to_string_lossy(),
                "--output_file",
                &out.to_string_lossy(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("failed to spawn Piper executable '{}'", self.executable))?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(text.as_bytes())
                .context("failed to write text to Piper stdin")?;
        }

        let status = child.wait().context("failed while waiting for Piper")?;
        if !status.success() {
            anyhow::bail!("Piper exited with status {status}");
        }

        Ok(())
    }
}

fn play_wav(path: &Path) -> Result<()> {
    let stream = OutputStreamBuilder::open_default_stream()
        .context("failed to open default output device")?;
    let sink = Sink::connect_new(stream.mixer());

    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let source = Decoder::try_from(BufReader::new(file)).context("failed to decode WAV")?;

    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}
