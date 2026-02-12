use rodio::{Decoder, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Clone)]
pub struct WaveformData {
    pub samples: Vec<f32>,
    pub enhanced: bool,
}

impl WaveformData {
    pub fn new(samples: Vec<f32>, enhanced: bool) -> Self {
        Self { samples, enhanced }
    }
}

pub fn generate_waveform<P: AsRef<Path>>(
    path: P,
    target_width: usize,
    enhanced: bool,
) -> Result<WaveformData, Box<dyn std::error::Error>> {
    let width = if enhanced { target_width * 2 } else { target_width };
    generate_waveform_internal(path, width, enhanced)
}

fn generate_waveform_internal<P: AsRef<Path>>(
    path: P,
    target_width: usize,
    enhanced: bool,
) -> Result<WaveformData, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let source = Decoder::new(BufReader::new(file))?;

    let channels = source.channels();
    let samples: Vec<i16> = source.convert_samples().collect();

    if samples.is_empty() {
        return Ok(WaveformData::new(vec![0.0; target_width], enhanced));
    }

    let total_samples = samples.len() / channels as usize;
    let samples_per_bar = (total_samples / target_width).max(1);

    let mut waveform = Vec::with_capacity(target_width);

    for i in 0..target_width {
        let start_idx = i * samples_per_bar * channels as usize;
        let end_idx = ((i + 1) * samples_per_bar * channels as usize).min(samples.len());

        if start_idx >= samples.len() {
            waveform.push(0.0);
            continue;
        }

        let mut sum = 0.0;
        let mut count = 0;

        for idx in (start_idx..end_idx).step_by(channels as usize) {
            if idx < samples.len() {
                let sample = samples[idx] as f32 / i16::MAX as f32;
                sum += sample.abs();
                count += 1;
            }
        }

        let avg = if count > 0 { sum / count as f32 } else { 0.0 };
        waveform.push(avg);
    }

    normalize_waveform(&mut waveform);

    Ok(WaveformData::new(waveform, enhanced))
}

fn normalize_waveform(waveform: &mut [f32]) {
    if waveform.is_empty() {
        return;
    }

    let max = waveform
        .iter()
        .copied()
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(1.0);

    if max > 0.0 {
        for sample in waveform.iter_mut() {
            *sample /= max;
        }
    }
}
