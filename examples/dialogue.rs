use std::time::{Duration, Instant};

use miette::{Context, IntoDiagnostic, Result};
use pixel_game_lib::{
    dialogue::Dialogue,
    font::Font,
    vek::Vec2,
    window::{KeyCode, WindowConfig},
};
use yarnspinner::runtime::DialogueEvent;

/// Simple state for managing the dialogue.
#[derive(Debug)]
pub struct GameState {
    /// Actual dialogue state itself.
    dialogue: Dialogue,
    /// Current available options for the user.
    user_options: Vec<String>,
    /// Current line to print.
    ///
    /// Will be updated in the update loop.
    line: String,
    /// How long we still need to sleep before updating the dialogue.
    sleep_until: Option<Instant>,
}

impl GameState {
    /// Compile the dialogue and setup the initial values.
    pub fn new() -> Result<Self> {
        // Compile and load new dialogue state
        let mut dialogue: Dialogue = pixel_game_lib::asset_owned("example_dialogue");
        // Choose the initial node to start
        dialogue
            .state
            .set_node("Example")
            .into_diagnostic()
            .wrap_err("Error setting initial node")?;

        // There's nothing to choose yet for the user
        let user_options = Vec::new();

        // Feed the line from the dialogue
        let line = String::new();

        // Don't sleep yet
        let sleep_until = None;

        Ok(Self {
            dialogue,
            user_options,
            line,
            sleep_until,
        })
    }
}

/// Open an empty window.
fn main() {
    // Window configuration with default pixel size and scaling
    let window_config = WindowConfig {
        ..Default::default()
    };

    // Load a font for the text
    let font = Font::default();

    // Setup the game state shared between the update and the render loop
    let state = GameState::new().expect("Error setting up dialogue state");

    // Open the window and start the game-loop
    pixel_game_lib::window(
        state,
        window_config.clone(),
        // Update loop exposing input events we can handle, this is where you would handle the game logic
        |state, input, _mouse, _dt| {
            if let Some(sleep_until) = state.sleep_until {
                // Wait a bit before going to the next line

                if sleep_until <= Instant::now() {
                    // Our sleeping time is over, unset it
                    state.sleep_until = None;
                }
            } else if state.user_options.is_empty() {
                // The user has no options to choose from, this means we can advance the dialogue

                // Get the next dialogue event
                let events = state
                    .dialogue
                    .state
                    .continue_()
                    .expect("Error advancing dialogue state");

                for event in events {
                    match event {
                        // Update the line
                        DialogueEvent::Line(line) => {
                            // The sleep duration is a variable set in the dialogue script itself
                            let next_sleep_duration = state
                                .dialogue
                                .number("$sleep")
                                .expect("Error getting variable for sleep duration");

                            // A new line is chosen, sleep for a short duration to show the line
                            state.sleep_until =
                                Some(Instant::now() + Duration::from_secs_f32(next_sleep_duration));

                            // Set the current line as the newly selected Yarn line
                            state.line = line.text
                        }
                        // Set the options as our blocking options
                        DialogueEvent::Options(options) => {
                            state.user_options = options
                                .into_iter()
                                .map(|dialogue_option| dialogue_option.line.text)
                                .collect()
                        }
                        // Exit when the dialogue is complete
                        DialogueEvent::DialogueComplete => return true,
                        _ => (),
                    }
                }
            } else {
                // The user can select an option, wait for that action
            }

            // Exit when escape is pressed
            input.key_pressed(KeyCode::Escape)
        },
        // Render loop exposing the pixel buffer we can mutate
        move |state, canvas, _dt| {
            canvas.fill(0xFFFFFFFF);

            // Draw the current dialogue line at the center of the screen
            font.render_centered(
                &state.line,
                Vec2::new(
                    window_config.buffer_size.w / 2,
                    window_config.buffer_size.h / 2,
                )
                .as_(),
                canvas,
            );
        },
    )
    .expect("Error opening window");
}
