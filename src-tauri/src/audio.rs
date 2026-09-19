use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample};
use serde::Serialize;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

struct Captured {
    samples: Vec<f32>,
    sample_rate: u32,
}

pub struct Recorder {
    stop_tx: Sender<()>,
    handle: JoinHandle<Captured>,
}

#[derive(Serialize)]
pub struct RecordingResult {
    pub path: String,
    pub seconds: f32,
    pub peak: f32,
}

impl Recorder {
    /// Starts capturing from the default microphone on a dedicated thread
    /// (cpal streams are not Send on Windows, so the stream lives there).
    pub fn start() -> Result<Self, String> {
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();

        let handle = std::thread::spawn(move || {
            let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
            let mut sample_rate = 0;
            match open_stream(buffer.clone()) {
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
            Ok(Ok(())) => Ok(Recorder { stop_tx, handle }),
            Ok(Err(e)) => {
                let _ = handle.join();
                Err(e)
            }
            Err(_) => Err("Microphone thread failed to start.".into()),
        }
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

fn open_stream(buffer: Arc<Mutex<Vec<f32>>>) -> Result<(cpal::Stream, u32), String> {
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
        SampleFormat::F32 => build::<f32>(&device, &stream_config, channels, buffer, err_fn),
        SampleFormat::I16 => build::<i16>(&device, &stream_config, channels, buffer, err_fn),
        SampleFormat::U16 => build::<u16>(&device, &stream_config, channels, buffer, err_fn),
        SampleFormat::I32 => build::<i32>(&device, &stream_config, channels, buffer, err_fn),
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
                for frame in data.chunks(channels) {
                    let sum: f32 = frame.iter().map(|s| f32::from_sample(*s)).sum();
                    buf.push(sum / frame.len() as f32);
                }
            },
            err_fn,
            None,
        )
        .map_err(|e| format!("Could not open the microphone: {e}"))
}

fn write_wav(c: &Captured) -> Result<RecordingResult, String> {
    let path = std::env::temp_dir().join("flow_last_recording.wav");
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: c.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(&path, spec).map_err(|e| e.to_string())?;
    let mut peak = 0f32;
    for &s in &c.samples {
        peak = peak.max(s.abs());
        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        writer.write_sample(v).map_err(|e| e.to_string())?;
    }
    writer.finalize().map_err(|e| e.to_string())?;
    Ok(RecordingResult {
        path: path.to_string_lossy().into_owned(),
        seconds: c.samples.len() as f32 / c.sample_rate as f32,
        peak,
    })
}
