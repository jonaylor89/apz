use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::player::PlaybackState;
use crate::spectrum::SpectrumAnalyzer;
use crate::waveform::WaveformData;

pub struct UIState {
    pub filename: String,
    pub position: Duration,
    pub duration: Duration,
    pub volume: f32,
    pub state: PlaybackState,
    pub waveform: WaveformData,
    pub spectrum: Option<Arc<Mutex<SpectrumAnalyzer>>>,
}

impl UIState {
    pub fn new<P: AsRef<Path>>(
        path: P,
        duration: Duration,
        waveform: WaveformData,
        spectrum: Option<Arc<Mutex<SpectrumAnalyzer>>>,
    ) -> Self {
        let filename = path
            .as_ref()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Self {
            filename,
            position: Duration::from_secs(0),
            duration,
            volume: 1.0,
            state: PlaybackState::Paused,
            waveform,
            spectrum,
        }
    }
}

pub fn render(frame: &mut Frame, state: &UIState) {
    let area = frame.area();

    let viz_height = if state.spectrum.is_some() {
        area.height.saturating_sub(12).max(10)
    } else if state.waveform.enhanced {
        9
    } else {
        5
    };

    let chunks = Layout::vertical([
        Constraint::Length(3),          // Title
        Constraint::Length(viz_height), // Waveform/Spectrum
        Constraint::Length(3),          // Progress
        Constraint::Length(3),          // Volume
        Constraint::Min(0),             // Spacer
        Constraint::Length(3),          // Controls
    ])
    .split(area);

    render_title(frame, chunks[0], state);
    render_visualization(frame, chunks[1], state);
    render_progress(frame, chunks[2], state);
    render_volume(frame, chunks[3], state);
    render_controls(frame, chunks[5]);
}

fn render_visualization(frame: &mut Frame, area: Rect, state: &UIState) {
    if let Some(spectrum) = &state.spectrum {
        render_spectrum_bars(frame, area, state, spectrum);
    } else if state.waveform.enhanced {
        render_enhanced_waveform(frame, area, state);
    } else {
        render_simple_waveform(frame, area, state);
    }
}

fn render_spectrum_bars(
    frame: &mut Frame,
    area: Rect,
    state: &UIState,
    spectrum: &Arc<Mutex<SpectrumAnalyzer>>,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Spectrum Analyzer");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut analyzer = spectrum.lock().unwrap();
    analyzer.update();
    let bars = analyzer.bars();
    let num_bars = analyzer.num_bars();

    let width = inner.width as usize;
    let height = inner.height as usize;
    let bar_width = (width / num_bars).max(1);

    let waveform_color = match state.state {
        PlaybackState::Playing => Color::Cyan,
        PlaybackState::Paused => Color::Yellow,
    };

    for (i, &amplitude) in bars.iter().enumerate() {
        let x_pos = i * bar_width;
        if x_pos >= width {
            break;
        }

        let bar_height = (amplitude * height as f32 * 0.5) as usize;
        let bar_height = bar_height.min(height);

        for h in 0..bar_height {
            let y = height.saturating_sub(h + 1);

            let hue_factor = i as f32 / num_bars as f32;
            let intensity = h as f32 / bar_height.max(1) as f32;

            let color = if intensity > 0.8 {
                Color::Red
            } else if intensity > 0.5 {
                if hue_factor < 0.33 {
                    Color::Magenta
                } else if hue_factor < 0.66 {
                    waveform_color
                } else {
                    Color::Green
                }
            } else {
                waveform_color
            };

            for w in 0..bar_width {
                let x = x_pos + w;
                if x < width {
                    let cell = &mut frame.buffer_mut()[(inner.x + x as u16, inner.y + y as u16)];
                    cell.set_symbol("█");
                    cell.set_fg(color);
                }
            }
        }
    }
}

fn render_simple_waveform(frame: &mut Frame, area: Rect, state: &UIState) {
    let width = area.width.saturating_sub(2) as usize;
    let waveform_data: Vec<u64> = if state.waveform.samples.len() >= width {
        state.waveform.samples[..width]
            .iter()
            .map(|&v| (v * 100.0) as u64)
            .collect()
    } else {
        let scale = width as f32 / state.waveform.samples.len() as f32;
        (0..width)
            .map(|i| {
                let idx = (i as f32 / scale) as usize;
                if idx < state.waveform.samples.len() {
                    (state.waveform.samples[idx] * 100.0) as u64
                } else {
                    0
                }
            })
            .collect()
    };

    let waveform_color = match state.state {
        PlaybackState::Playing => Color::Cyan,
        PlaybackState::Paused => Color::Yellow,
    };

    let sparkline = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title("Waveform"))
        .data(&waveform_data)
        .style(Style::default().fg(waveform_color));

    frame.render_widget(sparkline, area);
}

fn render_enhanced_waveform(frame: &mut Frame, area: Rect, state: &UIState) {
    let waveform_color = match state.state {
        PlaybackState::Playing => Color::Cyan,
        PlaybackState::Paused => Color::Yellow,
    };

    let position_secs = state.position.as_secs();
    let duration_secs = state.duration.as_secs().max(1);
    let progress_ratio = position_secs as f64 / duration_secs as f64;

    let block = Block::default().borders(Borders::ALL).title("Waveform");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let width = inner.width as usize;
    let height = inner.height as usize;
    let center = height / 2;

    let waveform_data: Vec<f32> = if state.waveform.samples.len() >= width {
        state.waveform.samples[..width].to_vec()
    } else {
        let scale = width as f32 / state.waveform.samples.len() as f32;
        (0..width)
            .map(|i| {
                let idx = (i as f32 / scale) as usize;
                if idx < state.waveform.samples.len() {
                    state.waveform.samples[idx]
                } else {
                    0.0
                }
            })
            .collect()
    };

    let cursor_pos = (progress_ratio * width as f64) as usize;

    for (x, &amplitude) in waveform_data.iter().enumerate() {
        let bar_height = (amplitude * center as f32) as usize;
        let color = if x <= cursor_pos {
            waveform_color
        } else {
            Color::DarkGray
        };

        for y in 0..bar_height.min(center) {
            let top_y = center.saturating_sub(y + 1);
            let bottom_y = center + y;

            if top_y < height {
                let cell = &mut frame.buffer_mut()[(inner.x + x as u16, inner.y + top_y as u16)];
                cell.set_symbol("█");
                cell.set_fg(color);
            }
            if bottom_y < height {
                let cell = &mut frame.buffer_mut()[(inner.x + x as u16, inner.y + bottom_y as u16)];
                cell.set_symbol("█");
                cell.set_fg(color);
            }
        }
    }

    if center < height {
        for x in 0..width {
            let cell = &mut frame.buffer_mut()[(inner.x + x as u16, inner.y + center as u16)];
            cell.set_symbol("─");
            cell.set_fg(Color::DarkGray);
        }
    }
}

fn render_title(frame: &mut Frame, area: Rect, state: &UIState) {
    let status_symbol = match state.state {
        PlaybackState::Playing => "▶",
        PlaybackState::Paused => "⏸",
    };

    let status_color = match state.state {
        PlaybackState::Playing => Color::Green,
        PlaybackState::Paused => Color::Yellow,
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            status_symbol,
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            &state.filename,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(
        Block::default().borders(Borders::ALL).title(Span::styled(
            "apz",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
    );

    frame.render_widget(title, area);
}

fn render_progress(frame: &mut Frame, area: Rect, state: &UIState) {
    let position_secs = state.position.as_secs();
    let duration_secs = state.duration.as_secs().max(1);
    let ratio = (position_secs as f64 / duration_secs as f64).min(1.0);

    let position_str = format_duration(state.position);
    let duration_str = format_duration(state.duration);
    let label = format!("{} / {}", position_str, duration_str);

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .label(label)
        .ratio(ratio);

    frame.render_widget(gauge, area);
}

fn render_volume(frame: &mut Frame, area: Rect, state: &UIState) {
    let volume_percent = (state.volume * 100.0) as u16;
    let label = format!("{}%", volume_percent);

    let volume_color = if state.volume > 0.7 {
        Color::Green
    } else if state.volume > 0.3 {
        Color::Yellow
    } else {
        Color::Red
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Volume"))
        .gauge_style(Style::default().fg(volume_color).bg(Color::DarkGray))
        .label(label)
        .ratio(state.volume as f64);

    frame.render_widget(gauge, area);
}

fn render_controls(frame: &mut Frame, area: Rect) {
    let controls = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                "[Space]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" play/pause  "),
            Span::styled(
                "[Q]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" quit  "),
            Span::styled(
                "[R]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" restart"),
        ]),
        Line::from(vec![
            Span::styled(
                "[←/→]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" seek ±5s  "),
            Span::styled(
                "[↑/↓]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" volume ±5%"),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title("Controls"));

    frame.render_widget(controls, area);
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let minutes = secs / 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}", minutes, seconds)
}
