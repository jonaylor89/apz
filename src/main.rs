mod config;
mod controls;
mod player;
mod spectrum;
mod tee_source;
mod ui;
mod waveform;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::process;

use crate::config::Config;
use crate::controls::{ControlAction, handle_input};
use crate::player::Player;
use crate::ui::UIState;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_args();

    let spectrum_config = if config.use_visualizer {
        Some((config.num_bars, config.smoothing, config.bass_boost))
    } else {
        None
    };

    let player = Player::new(
        &config.audio_path,
        false,
        spectrum_config,
        config.volume_step,
        config.seek_step,
    )
    .map_err(|e| {
        eprintln!("Failed to load audio file: {}", e);
        process::exit(1);
    })?;

    let duration = player.duration();
    let waveform = player.waveform().clone();
    let spectrum = player.spectrum();
    let mut ui_state = UIState::new(&config.audio_path, duration, waveform, spectrum);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_event_loop(&mut terminal, &player, &mut ui_state);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    player: &Player,
    ui_state: &mut UIState,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        ui_state.position = player.position();
        ui_state.volume = player.volume();
        ui_state.state = player.state();

        terminal.draw(|f| ui::render(f, ui_state))?;

        match handle_input(player)? {
            ControlAction::Quit => break,
            ControlAction::Continue => {}
        }

        if player.is_finished() {
            break;
        }
    }

    Ok(())
}
