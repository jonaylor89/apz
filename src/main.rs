mod player;
mod ui;
mod controls;

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

    if args.len() != 2 {
        eprintln!("Usage: {} <audio_file>", args[0]);
        eprintln!("\nSupported formats: MP3, WAV, FLAC, OGG, AAC/M4A");
        process::exit(1);
    }

    let audio_path = &args[1];

    let player = Player::new(audio_path).map_err(|e| {
        eprintln!("Failed to load audio file: {}", e);
        process::exit(1);
    })?;

    let duration = player.duration();
    let mut ui_state = UIState::new(audio_path, duration);

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
