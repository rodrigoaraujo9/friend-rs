use anyhow::{anyhow, Context, Result};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct SttEngine {
    ctx: WhisperContext,
    language: Option<String>,
    threads: usize,
}

impl SttEngine {
    pub fn new(
        model_path: &std::path::Path,
        language: Option<String>,
        threads: usize,
    ) -> Result<Self> {
        let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
            .with_context(|| {
                format!("failed to load Whisper model from {}", model_path.display())
            })?;

        Ok(Self {
            ctx,
            language,
            threads,
        })
    }

    pub fn transcribe(&self, pcm: &[f32]) -> Result<String> {
        if pcm.is_empty() {
            return Ok(String::new());
        }

        let mut state = self
            .ctx
            .create_state()
            .context("failed to create Whisper state")?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(self.threads as i32);
        params.set_translate(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_special(false);
        params.set_no_context(true);
        params.set_single_segment(false);

        if let Some(language) = &self.language {
            params.set_language(Some(language));
        }

        state
            .full(params, pcm)
            .context("Whisper transcription failed")?;

        let n = state.full_n_segments();
        let mut out = String::new();

        for i in 0..n {
            let seg = state
                .get_segment(i)
                .ok_or_else(|| anyhow!("failed to read Whisper segment {i}"))?;

            out.push_str(seg.to_str().context("failed to decode Whisper segment")?);
        }

        Ok(out.trim().to_string())
    }
}
