use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample};
use serde::Serialize;
use std::sync::mpsc::{self, Sender};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

struct Captured {
    samples: Vec<f32>,
    sample_rate: u32,
}

pub struct Recorder {
    /// Loudness of the latest audio chunk (0.0 - 1.0 as f32 bits), for the live waveform.
    level: Arc<AtomicU32>,
    stop_tx: Sender<()>,
    handle: JoinHandle<Captured>,
}

#[derive(Serialize)]
pub struct RecordingResult {
    pub path: String,
    pub seconds: f32,
    pub original_seconds: f32,
    pub peak: f32,
}

impl Recorder {
    /// Starts capturing from the default microphone on a dedicated thread
    /// (cpal streams are not Send on Windows, so the stream lives there).
    pub fn start() -> Result<Self, String> {
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();

        let level = Arc::new(AtomicU32::new(0));
        let thread_level = level.clone();
        let handle = std::thread::spawn(move || {
            let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
            let mut sample_rate = 0;
            match open_stream(buffer.clone(), thread_level) {
                Ok((stream, rate)) => {
                    sample_rate = rate;
                    let _ = ready_tx.send(Ok(()));
                    let _ = stop_rx.recv();
                    drop(stream);
                }
                Err(e) => {
                    let _ = ready_tx.send(Err(e));
                }
            }
            let samples = std::mem::take(&mut *buffer.lock().unwrap());
            Captured { samples, sample_rate }
        });

        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Recorder { level, stop_tx, handle }),
            Ok(Err(e)) => {
                let _ = handle.join();
                Err(e)
            }
            Err(_) => Err("Microphone thread failed to start.".into()),
        }
    }

    /// Current input loudness, roughly 0.0 (silence) to 1.0 (loud speech).
    pub fn level(&self) -> f32 {
        f32::from_bits(self.level.load(Ordering::Relaxed))
    }

    pub fn stop(self) -> Result<RecordingResult, String> {
        let _ = self.stop_tx.send(());
        let captured = self
            .handle
            .join()
            .map_err(|_| "Microphone thread crashed.".to_string())?;
        if captured.samples.is_empty() {
            return Err("No audio was captured.".into());
        }
        write_wav(&captured)
    }
}

fn open_stream(buffer: Arc<Mutex<Vec<f32>>>, level: Arc<AtomicU32>) -> Result<(cpal::Stream, u32), String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or("No microphone found. Plug one in and check Windows Settings > Sound > Input.")?;
    let config = device
        .default_input_config()
        .map_err(|e| format!("Could not read microphone settings: {e}"))?;
    let rate = config.sample_rate();
    let channels = config.channels() as usize;
    let format = config.sample_format();
    let stream_config: cpal::StreamConfig = config.into();

    let err_fn = |e| eprintln!("microphone stream error: {e}");
    let stream = match format {
        SampleFormat::F32 => build::<f32>(&device, &stream_config, channels, buffer, level.clone(), err_fn),
        SampleFormat::I16 => build::<i16>(&device, &stream_config, channels, buffer, level.clone(), err_fn),
        SampleFormat::U16 => build::<u16>(&device, &stream_config, channels, buffer, level.clone(), err_fn),
        SampleFormat::I32 => build::<i32>(&device, &stream_config, channels, buffer, level.clone(), err_fn),
        other => return Err(format!("Unsupported microphone format: {other:?}")),
    }?;
    stream.play().map_err(|e| {
        format!("Could not start the microphone (is it blocked in Windows privacy settings?): {e}")
    })?;
    Ok((stream, rate))
}

fn build<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    buffer: Arc<Mutex<Vec<f32>>>,
    level: Arc<AtomicU32>,
    err_fn: impl FnMut(cpal::Error) + Send + 'static,
) -> Result<cpal::Stream, String>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    device
        .build_input_stream(
            *config,
            move |data: &[T], _| {
                let mut buf = buffer.lock().unwrap();
                let start = buf.len();
                for frame in data.chunks(channels) {
                    let sum: f32 = frame.iter().map(|s| f32::from_sample(*s)).sum();
                    buf.push(sum / frame.len() as f32);
                }
                level.store(display_level(&buf[start..]).to_bits(), Ordering::Relaxed);
            },
            err_fn,
            None,
        )
        .map_err(|e| format!("Could not open the microphone: {e}"))
}

/// RMS loudness mapped to 0..1 on a curve that makes normal speech fill most of the range.
fn display_level(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
    (rms * 6.0).sqrt().min(1.0)
}

/// Whisper models work at 16 kHz mono internally, so this loses nothing
/// and keeps uploads small (about 32 KB per second).
const TARGET_RATE: u32 = 16_000;
/// Keeps the upload well under typical 25 MB service limits.
const MAX_SECONDS: f32 = 600.0;
/// Speech kept before/after the detected start/end so fast starts and trailing words aren't clipped.
const TRIM_PADDING_SECONDS: f32 = 0.35;

/// Averages each group of input samples into one output sample (a simple low-pass
/// that avoids aliasing when going from e.g. 48 kHz down to 16 kHz).
fn resample(input: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == to {
        return input.to_vec();
    }
    let ratio = from as f64 / to as f64;
    let out_len = (input.len() as f64 / ratio) as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let start = (i as f64 * ratio) as usize;
        let end = (((i + 1) as f64 * ratio) as usize).clamp(start + 1, input.len());
        let slice = &input[start..end];
        out.push(slice.iter().sum::<f32>() / slice.len() as f32);
    }
    out
}

/// Removes silence at the start and end only (never in the middle), keeping padding.
fn trim_silence(samples: &[f32], rate: u32) -> &[f32] {
    let frame = (rate as usize / 50).max(1); // 20 ms
    let rms: Vec<f32> = samples
        .chunks(frame)
        .map(|f| (f.iter().map(|s| s * s).sum::<f32>() / f.len() as f32).sqrt())
        .collect();
    let loudest = rms.iter().cloned().fold(0.0, f32::max);
    if loudest < 0.005 {
        return samples; // essentially silent; let the caller warn instead
    }
    let threshold = (loudest * 0.05).max(0.003);
    let first = rms.iter().position(|&r| r >= threshold);
    let last = rms.iter().rposition(|&r| r >= threshold);
    match (first, last) {
        (Some(f), Some(l)) => {
            let pad = (TRIM_PADDING_SECONDS * rate as f32) as usize;
            let start = (f * frame).saturating_sub(pad);
            let end = ((l + 1) * frame + pad).min(samples.len());
            &samples[start..end]
        }
        _ => samples,
    }
}

fn write_wav(c: &Captured) -> Result<RecordingResult, String> {
    let original_seconds = c.samples.len() as f32 / c.sample_rate as f32;
    if original_seconds > MAX_SECONDS {
        return Err(format!(
            "That recording is {:.0} minutes long. The limit is {:.0} minutes - please record in shorter pieces.",
            original_seconds / 60.0,
            MAX_SECONDS / 60.0
        ));
    }
    let peak = c.samples.iter().fold(0f32, |m, s| m.max(s.abs()));
    let resampled = resample(&c.samples, c.sample_rate, TARGET_RATE);
    let trimmed = trim_silence(&resampled, TARGET_RATE);

    let path = std::env::temp_dir().join("yapp_last_recording.wav");
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: TARGET_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(&path, spec).map_err(|e| e.to_string())?;
    for &s in trimmed {
        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        writer.write_sample(v).map_err(|e| e.to_string())?;
    }
    writer.finalize().map_err(|e| e.to_string())?;
    Ok(RecordingResult {
        path: path.to_string_lossy().into_owned(),
        seconds: trimmed.len() as f32 / TARGET_RATE as f32,
        original_seconds,
        peak,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn display_level_is_bounded_and_monotonic() {
        assert_eq!(super::display_level(&[]), 0.0);
        assert_eq!(super::display_level(&[0.0; 100]), 0.0);
        let quiet = super::display_level(&[0.01; 100]);
        let loud = super::display_level(&[0.2; 100]);
        assert!(quiet > 0.0 && quiet < loud && loud <= 1.0);
        assert_eq!(super::display_level(&[1.0; 100]), 1.0);
    }

    use super::*;

    #[test]
    fn resample_48k_to_16k_has_third_the_samples() {
        let input = vec![0.5f32; 48_000];
        let out = resample(&input, 48_000, 16_000);
        assert_eq!(out.len(), 16_000);
        assert!(out.iter().all(|s| (s - 0.5).abs() < 1e-6));
    }

    #[test]
    fn trim_removes_edge_silence_but_keeps_padding() {
        let rate = 16_000usize;
        let mut s = vec![0.0f32; rate]; // 1 s silence
        s.extend(std::iter::repeat(0.3).take(rate)); // 1 s "speech"
        s.extend(vec![0.0f32; rate]); // 1 s silence
        let out = trim_silence(&s, rate as u32);
        let secs = out.len() as f32 / rate as f32;
        assert!(secs > 1.0 && secs < 1.8, "got {secs}");
    }

    #[test]
    fn trim_leaves_silent_clip_alone() {
        let s = vec![0.0f32; 16_000];
        assert_eq!(trim_silence(&s, 16_000).len(), 16_000);
    }

    #[test]
    fn trim_never_cuts_middle_pauses() {
        let rate = 16_000usize;
        let mut s = vec![0.3f32; rate / 2];
        s.extend(vec![0.0f32; rate]); // 1 s pause in the middle
        s.extend(vec![0.3f32; rate / 2]);
        assert_eq!(trim_silence(&s, rate as u32).len(), s.len());
    }
}