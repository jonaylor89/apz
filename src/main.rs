mod player;
mod ui;
mod controls;
mod waveform;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::env;
use std::io;
use std::process;

use crate::controls::{handle_input, ControlAction};
use crate::player::Player;
use crate::ui::UIState;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let mut enhanced_waveform = false;
    let mut audio_path: Option<&String> = None;

    for arg in args.iter().skip(1) {
        if arg == "--visualizer" || arg == "-v" {
            enhanced_waveform = true;
        } else if !arg.starts_with('-') {
            audio_path = Some(arg);
        }
    }

    let audio_path = audio_path.unwrap_or_else(|| {
        eprintln!("Usage: {} [--visualizer|-v] <audio_file>", args[0]);
        eprintln!("\nSupported formats: MP3, WAV, FLAC, OGG, AAC/M4A");
        eprintln!("\nOptions:");
        eprintln!("  --visualizer, -v    Enable enhanced waveform visualization");
        process::exit(1);
    });

    let player = Player::new(audio_path, enhanced_waveform).map_err(|e| {
        eprintln!("Failed to load audio file: {}", e);
        process::exit(1);
    })?;

    let duration = player.duration();
    let waveform = player.waveform().clone();
    let mut ui_state = UIState::new(audio_path, duration, waveform);

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
