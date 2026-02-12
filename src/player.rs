use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

pub struct Player {
    _stream: OutputStream,
    sink: Arc<Sink>,
    state: Arc<Mutex<PlaybackState>>,
    duration: Duration,
}

impl Player {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        let file = File::open(&path)?;
        let source = Decoder::new(BufReader::new(file))?;

        let duration = source.total_duration().unwrap_or(Duration::from_secs(0));

        sink.append(source);
        sink.pause();

        Ok(Player {
            _stream,
            sink: Arc::new(sink),
            state: Arc::new(Mutex::new(PlaybackState::Paused)),
            duration,
        })
    }

    pub fn play(&self) {
        self.sink.play();
        *self.state.lock().unwrap() = PlaybackState::Playing;
    }

    pub fn pause(&self) {
        self.sink.pause();
        *self.state.lock().unwrap() = PlaybackState::Paused;
    }

    pub fn toggle_play_pause(&self) {
        let state = *self.state.lock().unwrap();
        match state {
            PlaybackState::Playing => self.pause(),
            PlaybackState::Paused | PlaybackState::Stopped => self.play(),
        }
    }

    pub fn stop(&self) {
        self.sink.stop();
        *self.state.lock().unwrap() = PlaybackState::Stopped;
    }

    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume.clamp(0.0, 1.0));
    }

    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }

    pub fn seek(&self, offset: i64) {
        let current = self.position().as_secs() as i64;
        let new_position = (current + offset).max(0) as u64;
        let duration = self.duration.as_secs();

        if new_position < duration {
            self.sink.try_seek(Duration::from_secs(new_position)).ok();
        }
    }

    pub fn restart(&self) {
        self.sink.try_seek(Duration::from_secs(0)).ok();
        self.play();
    }

    pub fn position(&self) -> Duration {
        self.sink.get_pos()
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn state(&self) -> PlaybackState {
        *self.state.lock().unwrap()
    }

    pub fn is_finished(&self) -> bool {
        self.sink.empty()
    }
}
