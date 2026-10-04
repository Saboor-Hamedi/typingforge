use rodio::{OutputStream, OutputStreamHandle, Source};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

struct VoiceGuard {
    voices: Arc<AtomicUsize>,
}

impl Drop for VoiceGuard {
    fn drop(&mut self) {
        self.voices.fetch_sub(1, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SoundPreset {
    Mechanical,
    DeepThock,
    BubblePop,
    Typewriter,
    CyberBlip,
    Off,
}

impl SoundPreset {
    pub const ALL: &'static [SoundPreset] = &[
        SoundPreset::Mechanical,
        SoundPreset::DeepThock,
        SoundPreset::BubblePop,
        SoundPreset::Typewriter,
        SoundPreset::CyberBlip,
        SoundPreset::Off,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            SoundPreset::Mechanical => "Mechanical Tactile",
            SoundPreset::DeepThock => "Creamy Deep Thock",
            SoundPreset::BubblePop => "Juicy Bubble Pop",
            SoundPreset::Typewriter => "Vintage Typewriter",
            SoundPreset::CyberBlip => "Cyberpunk Blip",
            SoundPreset::Off => "Mute (Silent)",
        }
    }
}

struct FnSource<F> {
    generator: F,
}

impl<F: FnMut() -> Option<f32>> Iterator for FnSource<F> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        (self.generator)()
    }
}

impl<F: FnMut() -> Option<f32>> Source for FnSource<F> {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        44100
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

fn from_fn<F: FnMut() -> Option<f32>>(init: impl FnOnce() -> F) -> FnSource<F> {
    FnSource { generator: init() }
}

pub struct AudioManager {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    pub volume: f32,
    pub sound_preset: SoundPreset,
    pub enabled: bool,
    active_voices: Arc<AtomicUsize>,
    last_click_ms: Arc<AtomicU64>,
}

impl AudioManager {
    pub fn new() -> Self {
        let (stream, handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(e) => {
                eprintln!("[Audio] Warning: Audio output device unavailable: {e}");
                (None, None)
            }
        };

        Self {
            _stream: stream,
            stream_handle: handle,
            volume: 0.5,
            sound_preset: SoundPreset::Mechanical,
            enabled: true,
            active_voices: Arc::new(AtomicUsize::new(0)),
            last_click_ms: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn play_click(&self) {
        if !self.enabled || self.sound_preset == SoundPreset::Off || self.volume <= 0.0 {
            return;
        }
        let Some(handle) = &self.stream_handle else { return };

        // Voice concurrency limiting: max 6 overlapping clicks and minimum 14ms interval
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let last = self.last_click_ms.load(Ordering::Relaxed);
        if now_ms.saturating_sub(last) < 14 {
            return;
        }

        if self.active_voices.load(Ordering::Relaxed) >= 6 {
            return;
        }

        self.last_click_ms.store(now_ms, Ordering::Relaxed);
        self.active_voices.fetch_add(1, Ordering::Relaxed);

        let vol = self.volume;
        let preset = self.sound_preset;
        let guard = VoiceGuard {
            voices: Arc::clone(&self.active_voices),
        };

        let _ = handle.play_raw(
            from_fn(move || {
                let _voice_guard = guard;
                let mut sample_idx = 0;
                let sample_rate = 44100;

                let duration_samples = match preset {
                    SoundPreset::Mechanical => (sample_rate as f32 * 0.032) as usize,
                    SoundPreset::DeepThock => (sample_rate as f32 * 0.058) as usize,
                    SoundPreset::BubblePop => (sample_rate as f32 * 0.040) as usize,
                    SoundPreset::Typewriter => (sample_rate as f32 * 0.065) as usize,
                    SoundPreset::CyberBlip => (sample_rate as f32 * 0.022) as usize,
                    SoundPreset::Off => 0,
                };

                move || {
                    if sample_idx >= duration_samples {
                        return None;
                    }
                    let t = (sample_idx as f32) / (sample_rate as f32);
                    sample_idx += 1;
                    let decay = (1.0 - (sample_idx as f32 / duration_samples as f32)).powi(2);

                    let val = match preset {
                        SoundPreset::Mechanical => {
                            let freq = 1250.0 * (1.0 - t * 28.0).max(0.12);
                            (t * freq * std::f32::consts::TAU).sin() * decay * vol * 0.45
                        }
                        SoundPreset::DeepThock => {
                            let freq = 240.0 * (1.0 - t * 14.0).max(0.18);
                            let sub = 120.0 * (1.0 - t * 10.0).max(0.2);
                            ((t * freq * std::f32::consts::TAU).sin() * 0.7
                                + (t * sub * std::f32::consts::TAU).sin() * 0.3)
                                * decay
                                * vol
                                * 0.55
                        }
                        SoundPreset::BubblePop => {
                            let freq = 400.0 + t * 900.0;
                            (t * freq * std::f32::consts::TAU).sin() * decay * vol * 0.4
                        }
                        SoundPreset::Typewriter => {
                            let click = if t < 0.008 { (t * 2200.0 * std::f32::consts::TAU).sin() * 0.8 } else { 0.0 };
                            let clack = (t * 450.0 * std::f32::consts::TAU).sin() * 0.4;
                            (click + clack) * decay * vol * 0.5
                        }
                        SoundPreset::CyberBlip => {
                            let freq = 1800.0 - t * 800.0;
                            let phase = (t * freq) % 1.0;
                            (phase * 2.0 - 1.0) * decay * vol * 0.25
                        }
                        SoundPreset::Off => 0.0,
                    };

                    Some(val)
                }
            })
            .convert_samples(),
        );
    }

    pub fn play_error(&self) {
        if !self.enabled || self.volume <= 0.0 {
            return;
        }
        let Some(handle) = &self.stream_handle else { return };
        let vol = self.volume;

        let _ = handle.play_raw(
            from_fn(move || {
                let mut sample_idx = 0;
                let sample_rate = 44100;
                let duration_samples = (sample_rate as f32 * 0.14) as usize;

                move || {
                    if sample_idx >= duration_samples {
                        return None;
                    }
                    let t = (sample_idx as f32) / (sample_rate as f32);
                    sample_idx += 1;
                    let decay = (1.0 - (sample_idx as f32 / duration_samples as f32)).powi(2);
                    let freq = 95.0 * (1.0 - t * 4.0).max(0.3);
                    let phase = (t * freq) % 1.0;
                    let val = (phase * 2.0 - 1.0) * decay * vol * 0.4;
                    Some(val)
                }
            })
            .convert_samples(),
        );
    }

    pub fn play_milestone(&self, _combo: usize) {
        if !self.enabled || self.volume <= 0.0 {
            return;
        }
        let Some(handle) = &self.stream_handle else { return };
        let vol = self.volume;

        let _ = handle.play_raw(
            from_fn(move || {
                let mut sample_idx = 0;
                let sample_rate = 44100;
                let duration_samples = (sample_rate as f32 * 0.3) as usize;

                move || {
                    if sample_idx >= duration_samples {
                        return None;
                    }
                    let t = (sample_idx as f32) / (sample_rate as f32);
                    sample_idx += 1;
                    let decay = (1.0 - (sample_idx as f32 / duration_samples as f32)).powi(2);
                    let val = (t * 523.25 * std::f32::consts::TAU).sin() * 0.35
                        + (t * 659.25 * std::f32::consts::TAU).sin() * 0.35
                        + (t * 783.99 * std::f32::consts::TAU).sin() * 0.3;
                    Some(val * decay * vol * 0.35)
                }
            })
            .convert_samples(),
        );
    }
}
