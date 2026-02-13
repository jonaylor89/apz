use rustfft::{FftPlanner, num_complex::Complex};
use std::sync::{Arc, Mutex};

const SAMPLE_SIZE: usize = 2048;
const NUM_BARS: usize = 50;

pub struct SpectrumAnalyzer {
    samples: Arc<Mutex<Vec<f32>>>,
    bars: Vec<f32>,
    smoothing: f32,
}

impl SpectrumAnalyzer {
    pub fn new() -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            bars: vec![0.0; NUM_BARS],
            smoothing: 0.7,
        }
    }

    pub fn get_sample_buffer(&self) -> Arc<Mutex<Vec<f32>>> {
        Arc::clone(&self.samples)
    }

    pub fn update(&mut self) {
        let samples = self.samples.lock().unwrap();
        if samples.len() < SAMPLE_SIZE {
            return;
        }

        let mut buffer: Vec<Complex<f32>> = samples[..SAMPLE_SIZE]
            .iter()
            .map(|&s| Complex::new(s, 0.0))
            .collect();

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(SAMPLE_SIZE);
        fft.process(&mut buffer);

        let spectrum: Vec<f32> = buffer[..SAMPLE_SIZE / 2]
            .iter()
            .map(|c| (c.re * c.re + c.im * c.im).sqrt())
            .collect();

        for (i, bar) in self.bars.iter_mut().enumerate() {
            let freq_index = ((i as f32 / NUM_BARS as f32).powf(1.3) * (spectrum.len() - 1) as f32) as usize;
            let freq_index = freq_index.min(spectrum.len() - 1);

            let bass_boost = 1.5 * (1.0 - i as f32 / NUM_BARS as f32);
            let amplitude = spectrum[freq_index] * (1.0 + bass_boost);

            *bar = *bar * self.smoothing + amplitude * (1.0 - self.smoothing);
        }
    }

    pub fn bars(&self) -> &[f32] {
        &self.bars
    }

    pub fn num_bars(&self) -> usize {
        NUM_BARS
    }
}
