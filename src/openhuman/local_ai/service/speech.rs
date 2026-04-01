use std::path::PathBuf;

use log::{debug, warn};

use crate::openhuman::config::Config;
use crate::openhuman::local_ai::model_ids;
use crate::openhuman::local_ai::paths::{
    config_root_dir, resolve_piper_binary, resolve_stt_model_path, resolve_tts_voice_path,
    resolve_whisper_binary,
};
use crate::openhuman::local_ai::types::{LocalAiSpeechResult, LocalAiTtsResult};

use super::whisper_engine;
use super::LocalAiService;

const LOG_PREFIX: &str = "[speech]";

impl LocalAiService {
    pub async fn transcribe(
        &self,
        config: &Config,
        audio_path: &str,
    ) -> Result<LocalAiSpeechResult, String> {
        if !config.local_ai.enabled {
            return Err("local ai is disabled".to_string());
        }

        // Try in-process whisper engine first.
        if whisper_engine::is_loaded(&self.whisper) {
            debug!("{LOG_PREFIX} using in-process whisper engine for {audio_path}");
            match self.transcribe_in_process(audio_path) {
                Ok(text) => {
                    self.status.lock().stt_state = "ready".to_string();
                    return Ok(LocalAiSpeechResult {
                        text,
                        model_id: model_ids::effective_stt_model_id(config),
                    });
                }
                Err(e) => {
                    warn!("{LOG_PREFIX} in-process transcription failed, falling back to CLI: {e}");
                }
            }
        }

        // Fallback: subprocess per call (original behavior).
        debug!("{LOG_PREFIX} using whisper-cli subprocess for {audio_path}");
        self.transcribe_subprocess(config, audio_path).await
    }

    /// Transcribe using the in-process whisper-rs engine. Reads the audio file
    /// and runs inference without spawning a subprocess.
    fn transcribe_in_process(&self, audio_path: &str) -> Result<String, String> {
        let path = std::path::Path::new(audio_path);
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        if ext == "wav" {
            whisper_engine::transcribe_wav_file(&self.whisper, path, None)
        } else {
            // For non-WAV formats, read raw bytes and try to decode.
            // This works when the file is already raw PCM, but for encoded
            // formats (webm, ogg, mp3) the audio pipeline (Phase 3) will
            // convert to WAV via ffmpeg before reaching here.
            warn!(
                "{LOG_PREFIX} non-WAV input ({ext}), attempting WAV decode anyway \
                 (may fail — use ffmpeg conversion for best results)"
            );
            whisper_engine::transcribe_wav_file(&self.whisper, path, None)
        }
    }

    /// Original subprocess-based transcription via whisper-cli.
    async fn transcribe_subprocess(
        &self,
        config: &Config,
        audio_path: &str,
    ) -> Result<LocalAiSpeechResult, String> {
        let whisper_bin = resolve_whisper_binary().ok_or_else(|| {
            "whisper.cpp binary not found. Set WHISPER_BIN or install whisper-cli.".to_string()
        })?;
        let model_path = resolve_stt_model_path(config)?;
        let output = tokio::process::Command::new(whisper_bin)
            .args(["-m", &model_path, "-f", audio_path])
            .output()
            .await
            .map_err(|e| format!("failed to run whisper.cpp: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "whisper.cpp failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            return Err("whisper.cpp returned empty transcript".to_string());
        }
        self.status.lock().stt_state = "ready".to_string();
        Ok(LocalAiSpeechResult {
            text,
            model_id: model_ids::effective_stt_model_id(config),
        })
    }

    pub async fn tts(
        &self,
        config: &Config,
        text: &str,
        output_path: Option<&str>,
    ) -> Result<LocalAiTtsResult, String> {
        if !config.local_ai.enabled {
            return Err("local ai is disabled".to_string());
        }
        let piper_bin = resolve_piper_binary()
            .ok_or_else(|| "piper binary not found. Set PIPER_BIN or install piper.".to_string())?;
        let model_path = resolve_tts_voice_path(config)?;
        let out_path = output_path
            .map(std::string::ToString::to_string)
            .unwrap_or_else(|| {
                config_root_dir(config)
                    .join("models")
                    .join("local-ai")
                    .join("tts-output.wav")
                    .display()
                    .to_string()
            });
        let parent = PathBuf::from(&out_path)
            .parent()
            .map(PathBuf::from)
            .ok_or_else(|| "invalid output_path".to_string())?;
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("failed to create TTS output directory: {e}"))?;

        let mut child = tokio::process::Command::new(piper_bin)
            .args(["--model", &model_path, "--output_file", &out_path])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("failed to launch piper: {e}"))?;

        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            stdin
                .write_all(text.as_bytes())
                .await
                .map_err(|e| format!("failed to write text to piper stdin: {e}"))?;
        }
        let output = child
            .wait_with_output()
            .await
            .map_err(|e| format!("failed to wait for piper: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "piper failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        self.status.lock().tts_state = "ready".to_string();
        Ok(LocalAiTtsResult {
            output_path: out_path,
            voice_id: model_ids::effective_tts_voice_id(config),
        })
    }
}
