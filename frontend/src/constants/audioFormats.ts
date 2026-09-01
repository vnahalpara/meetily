/**
 * Supported audio file extensions for import and retranscription.
 * IMPORTANT: Keep in sync with Rust constant in src-tauri/src/audio/constants.rs
 *
 * Includes:
 * - Native formats: MP4, M4A, WAV, MP3, FLAC, OGG, AAC
 * - FFmpeg-backed: MKV, WebM, WMA
 */
export const AUDIO_EXTENSIONS = [
  'mp4', 'm4a', 'wav', 'mp3', 'flac', 'ogg', 'aac', 'mkv', 'webm', 'wma'
] as const;

export type AudioExtension = typeof AUDIO_EXTENSIONS[number];

export const isAudioExtension = (ext: string): ext is AudioExtension =>{
  return (AUDIO_EXTENSIONS as readonly string[]).includes(ext);
}

/**
 * Human-readable format names for display
 */
export const AUDIO_FORMAT_DISPLAY_NAMES: Record<AudioExtension, string> = {
  mp4: 'MP4',
  m4a: 'M4A',
  wav: 'WAV',
  mp3: 'MP3',
  flac: 'FLAC',
  ogg: 'OGG',
  aac: 'AAC',
  mkv: 'MKV',
  webm: 'WebM',
  wma: 'WMA',
};

/**
 * Get comma-separated list for UI display
 * Example: "MP4, M4A, WAV, MP3, FLAC, OGG, AAC, MKV, WebM, WMA"
 */
export function getAudioFormatsDisplayList(): string {
  return AUDIO_EXTENSIONS.map(ext => AUDIO_FORMAT_DISPLAY_NAMES[ext]).join(', ');
}

/**
 * Formats a NEW recording can be saved in.
 * IMPORTANT: Keep in sync with the Rust enum in
 * src-tauri/src/audio/format.rs (RECORDING_FORMATS / RecordingFormat).
 *
 * This is deliberately narrower than AUDIO_EXTENSIONS above: that list covers
 * every format the app can *import and read*, this one covers what the
 * recorder can *write*.
 */
export const RECORDING_FORMATS = ['mp3', 'wav', 'm4a', 'flac', 'mp4'] as const;

export type RecordingFormat = typeof RECORDING_FORMATS[number];

export const DEFAULT_RECORDING_FORMAT: RecordingFormat = 'mp3';

export interface RecordingFormatInfo {
  /** File extension, also the value stored in preferences */
  id: RecordingFormat;
  /** Short label shown in the dropdown */
  label: string;
  /** One-line explanation of the trade-off */
  description: string;
  /** Rough file size for one hour of mono meeting audio */
  sizePerHour: string;
}

export const RECORDING_FORMAT_INFO: Record<RecordingFormat, RecordingFormatInfo> = {
  mp3: {
    id: 'mp3',
    label: 'MP3',
    description: 'Compressed audio that plays on virtually anything. Best all-round choice.',
    sizePerHour: '~55 MB / hour',
  },
  wav: {
    id: 'wav',
    label: 'WAV',
    description: 'Uncompressed, perfect quality, and saves instantly. Large files.',
    sizePerHour: '~345 MB / hour',
  },
  m4a: {
    id: 'm4a',
    label: 'M4A',
    description: 'AAC audio. Same quality and size as the old MP4 files, but a real audio file.',
    sizePerHour: '~85 MB / hour',
  },
  flac: {
    id: 'flac',
    label: 'FLAC',
    description: 'Lossless compression. Perfect quality at about half the size of WAV.',
    sizePerHour: '~170 MB / hour',
  },
  mp4: {
    id: 'mp4',
    label: 'MP4 (legacy)',
    description: 'Audio inside a video container. Kept for older recordings; not recommended.',
    sizePerHour: '~85 MB / hour',
  },
};

export const isRecordingFormat = (value: string): value is RecordingFormat =>
  (RECORDING_FORMATS as readonly string[]).includes(value);

/** Normalise a stored preference so the UI never renders an unknown format. */
export const toRecordingFormat = (value: string | null | undefined): RecordingFormat =>
  value && isRecordingFormat(value) ? value : DEFAULT_RECORDING_FORMAT;
