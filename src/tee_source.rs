use rodio::Source;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct TeeSource<I> {
    input: I,
    sample_buffer: Arc<Mutex<Vec<f32>>>,
    buffer_size: usize,
}

impl<I> TeeSource<I> {
    pub fn new(input: I, sample_buffer: Arc<Mutex<Vec<f32>>>) -> Self {
        Self {
            input,
            sample_buffer,
            buffer_size: 2048,
        }
    }
}

impl<I> Iterator for TeeSource<I>
where
    I: Source<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sample) = self.input.next() {
            let mut buffer = self.sample_buffer.lock().unwrap();
            buffer.push(sample);
            let len = buffer.len();
            if len > self.buffer_size {
                buffer.drain(0..len - self.buffer_size);
            }
            Some(sample)
        } else {
            None
        }
    }
}

impl<I> Source for TeeSource<I>
where
    I: Source<Item = f32>,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.input.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.input.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }
}
