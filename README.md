# apz

Minimal terminal audio player with live spectrum analyzer.

```
┌apz──────────────────────────────────────────────────────────────────────┐
│⏸ sample8.mp3                                                            │
└─────────────────────────────────────────────────────────────────────────┘
┌Waveform─────────────────────────────────────────────────────────────────┐
│▃ ▂▂ ▄▄ ▂▁ ▂ ▄▃ ▄▁ ▄ ▁▅ ▂▃ ▂  ▃ ▂▂ ▃▁ ▃▂ ▅ ▂   █ ▃     ▁ ▃▁ ▃  ▃ ▁  ▁▂ ▁ │
│█▃██▂██▅██▇█▅██▆██▅█▆██▁██▃█▇▄█▇██▅██▅████▆█▂▅▇█ ██▂█ ██ ██▂█▄▅█▁██ ██▁█▆│
│████████████████████████████████████████████████▆████▆█████████████▆█████│
└─────────────────────────────────────────────────────────────────────────┘
┌Progress─────────────────────────────────────────────────────────────────┐
│                              00:00 / 00:32                              │
└─────────────────────────────────────────────────────────────────────────┘
┌Volume───────────────────────────────────────────────────────────────────┐
│██████████████████████████████████100% ██████████████████████████████████│
└─────────────────────────────────────────────────────────────────────────┘








┌Controls─────────────────────────────────────────────────────────────────┐
│[Space] play/pause  [Q] quit  [R] restart                                │
└─────────────────────────────────────────────────────────────────────────┘
```

## Build

```bash
cargo build --release
```

## Usage

```bash
./target/release/apz [OPTIONS] <audio_file>
```

## Options

```
--visualizer           Enable live spectrum analyzer
--bars <n>             Number of frequency bars (default: 100)
--smoothing <f>        Smoothing factor 0.0-1.0 (default: 0.7)
--bass-boost <f>       Bass boost multiplier (default: 1.5)
--volume-step <f>      Volume adjustment step (default: 0.05)
--seek-step <n>        Seek step in seconds (default: 5)
-h, --help             Show help message
```

### Examples

```bash
# Basic playback
./target/release/apz song.mp3

# With visualizer
./target/release/apz --visualizer song.mp3

# Custom visualizer settings
./target/release/apz --visualizer --bars 50 --bass-boost 2.0 song.mp3

# Custom seek/volume steps
./target/release/apz --seek-step 10 --volume-step 0.1 song.mp3
```

## Supported Formats

MP3, WAV, FLAC, OGG, AAC/M4A

## Controls

- `Space` - Play/pause
- `←/→` - Seek ±5 seconds
- `↑/↓` - Volume ±5%
- `R` - Restart
- `Q` - Quit

