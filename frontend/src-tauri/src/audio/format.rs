//! Recording output format handling.
//!
//! Recordings used to be hardcoded to an MP4 container (AAC audio inside a
//! *video* container), which meant meeting recordings did not behave like
//! audio files. This module makes the output format a first-class concept so
//! the user's `file_format` preference actually controls what gets written.
//!
//! ## How saving works
//!
//! While recording, audio is written to 30-second checkpoint files so a crash
//! can be recovered. Those checkpoints are **always WAV**, for three reasons:
//!
//! 1. WAV has no encoder delay/padding, so concatenating checkpoints is
//!    sample-exact. Lossy checkpoints (AAC/MP3) add a few milliseconds of
//!    padding per file, which accumulates into seconds of drift over a long
//!    meeting and desynchronises transcript timestamps from the audio.
//! 2. FLAC checkpoints cannot be stream-copy concatenated at all (the merged
//!    file reports only the first checkpoint's duration).
//! 3. A crashed session leaves behind plain, directly playable WAV chunks.
//!
//! At finalize time the checkpoints are concatenated and encoded once into the
//! user's chosen format. When the target *is* WAV the merge is a stream copy,
//! so it stays instant.

use std::path::Path;

/// Extension used for on-disk checkpoint files while recording.
pub const CHECKPOINT_EXTENSION: &str = "wav";

/// Default recording format for new installs.
pub const DEFAULT_RECORDING_FORMAT: &str = "mp3";

/// Formats a recording can be saved as, in the order shown in the UI.
pub const RECORDING_FORMATS: &[&str] = &["mp3", "wav", "m4a", "flac", "mp4"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingFormat {
    /// MPEG Layer III. Small files, plays everywhere.
    Mp3,
    /// Uncompressed PCM. Lossless, large files, instant to save.
    Wav,
    /// AAC in an audio-only MPEG-4 container.
    M4a,
    /// Free Lossless Audio Codec. Lossless, roughly half the size of WAV.
    Flac,
    /// AAC in a video MPEG-4 container. Legacy default, kept for compatibility.
    Mp4,
}

impl RecordingFormat {
    /// Parse a file extension (case-insensitive, leading dot optional).
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.trim_start_matches('.').to_ascii_lowercase().as_str() {
            "mp3" => Some(Self::Mp3),
            "wav" => Some(Self::Wav),
            "m4a" => Some(Self::M4a),
            "flac" => Some(Self::Flac),
            "mp4" => Some(Self::Mp4),
            _ => None,
        }
    }

    /// Parse a stored preference string, falling back to the default when the
    /// value is unknown (e.g. preferences written by a newer build).
    pub fn parse_or_default(value: &str) -> Self {
        Self::from_extension(value).unwrap_or_else(|| {
            Self::from_extension(DEFAULT_RECORDING_FORMAT)
                .expect("DEFAULT_RECORDING_FORMAT must be a valid format")
        })
    }

    /// Infer the format from an output path's extension.
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(Self::from_extension)
    }

    /// File extension, without the leading dot.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Mp3 => "mp3",
            Self::Wav => "wav",
            Self::M4a => "m4a",
            Self::Flac => "flac",
            Self::Mp4 => "mp4",
        }
    }

    /// Human-readable name for logs and UI.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Mp3 => "MP3",
            Self::Wav => "WAV",
            Self::M4a => "M4A",
            Self::Flac => "FLAC",
            Self::Mp4 => "MP4",
        }
    }

    /// True when the merge step can stream-copy the WAV checkpoints instead of
    /// re-encoding them.
    pub fn can_stream_copy_from_checkpoints(&self) -> bool {
        matches!(self, Self::Wav)
    }

    /// FFmpeg output arguments (codec + container) for this format.
    ///
    /// Bitrates are tuned for speech: 128k is transparent enough for voice at
    /// a fraction of the size, while the AAC paths keep the 192k the app used
    /// before so existing recordings stay comparable.
    pub fn ffmpeg_output_args(&self) -> Vec<&'static str> {
        match self {
            Self::Mp3 => vec!["-c:a", "libmp3lame", "-b:a", "128k", "-f", "mp3"],
            Self::Wav => vec!["-c:a", "pcm_s16le", "-f", "wav"],
            Self::Flac => vec!["-c:a", "flac", "-compression_level", "5", "-f", "flac"],
            // `-f ipod` is FFmpeg's audio-only MPEG-4 muxer, which is what an
            // .m4a file should be. `-f mp4` would produce a video container.
            Self::M4a => vec![
                "-c:a", "aac", "-b:a", "192k", "-profile:a", "aac_low",
                "-movflags", "+faststart", "-f", "ipod",
            ],
            Self::Mp4 => vec![
                "-c:a", "aac", "-b:a", "192k", "-profile:a", "aac_low",
                "-movflags", "+faststart", "-f", "mp4",
            ],
        }
    }
}

impl std::fmt::Display for RecordingFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.extension())
    }
}

impl Default for RecordingFormat {
    fn default() -> Self {
        Self::parse_or_default(DEFAULT_RECORDING_FORMAT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parses_extensions_case_insensitively_and_with_dots() {
        assert_eq!(RecordingFormat::from_extension("WAV"), Some(RecordingFormat::Wav));
        assert_eq!(RecordingFormat::from_extension(".Mp3"), Some(RecordingFormat::Mp3));
        assert_eq!(RecordingFormat::from_extension("flac"), Some(RecordingFormat::Flac));
        assert_eq!(RecordingFormat::from_extension("ogg"), None);
    }

    #[test]
    fn unknown_preference_falls_back_to_default() {
        assert_eq!(
            RecordingFormat::parse_or_default("banana"),
            RecordingFormat::Mp3
        );
        // A legacy stored preference must keep working.
        assert_eq!(RecordingFormat::parse_or_default("mp4"), RecordingFormat::Mp4);
    }

    #[test]
    fn infers_format_from_output_path() {
        assert_eq!(
            RecordingFormat::from_path(&PathBuf::from("/tmp/audio.flac")),
            Some(RecordingFormat::Flac)
        );
        assert_eq!(RecordingFormat::from_path(&PathBuf::from("/tmp/audio")), None);
    }

    #[test]
    fn every_advertised_format_is_parseable_and_round_trips() {
        for ext in RECORDING_FORMATS {
            let fmt = RecordingFormat::from_extension(ext)
                .unwrap_or_else(|| panic!("{} should parse", ext));
            assert_eq!(&fmt.extension(), ext);
            assert!(!fmt.ffmpeg_output_args().is_empty());
        }
    }

    #[test]
    fn only_wav_can_stream_copy_from_wav_checkpoints() {
        assert!(RecordingFormat::Wav.can_stream_copy_from_checkpoints());
        for f in [RecordingFormat::Mp3, RecordingFormat::M4a, RecordingFormat::Flac, RecordingFormat::Mp4] {
            assert!(!f.can_stream_copy_from_checkpoints());
        }
    }

    #[test]
    fn checkpoints_are_wav() {
        assert_eq!(
            RecordingFormat::from_extension(CHECKPOINT_EXTENSION),
            Some(RecordingFormat::Wav)
        );
    }
}
