use std::time::{Duration, Instant};

use miette::{Context, IntoDiagnostic, Result};
use pixel_game_lib::{
    dialogue::Dialogue,
    font::Font,
    gui::{
        button::{Button, ButtonRef},
        Gui, GuiBuilder, Widget,
    },
    vek::Vec2,
    window::{KeyCode, WindowConfig},
};
use taffy::{FlexDirection, LengthPercentage, Size, Style};
use yarnspinner::runtime::{DialogueEvent, OptionId};

/// Simple state for managing the dialogue.
pub struct GameState {
    /// Actual dialogue state itself.
    dialogue: Dialogue,
    /// Current line to print.
    ///
    /// Will be updated in the update loop.
    line: String,
    /// How long we still need to sleep before updating the dialogue.
    sleep_until: Option<Instant>,
    /// GUI for the buttons.
    gui: Gui,
    /// Buttons and the IDs of the dialogue to trigger.
    buttons: Vec<(ButtonRef, OptionId)>,
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

        // Feed the line from the dialogue
        let line = String::new();

        // Don't sleep yet
        let sleep_until = None;

        // Setup the GUI for the buttons
        let gui = GuiBuilder::new(Style::DEFAULT).build();
        let buttons = Vec::new();

        Ok(Self {
            gui,
            dialogue,
            line,
            sleep_until,
            buttons,
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
        move |state, input, mouse_pos, _dt| {
            if let Some(sleep_until) = state.sleep_until {
                // Wait a bit before going to the next line

                if sleep_until <= Instant::now() {
                    // Our sleeping time is over, unset it
                    state.sleep_until = None;
                }
            } else if state.buttons.is_empty() {
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
                            // Create a base GUI canvas
                            let mut gui = GuiBuilder::new(Style {
                                size: Size::from_lengths(
                                    window_config.buffer_size.w as f32,
                                    window_config.buffer_size.h as f32,
                                ),
                                flex_direction: FlexDirection::Column,
                                gap: Size {
                                    width: LengthPercentage::Length(0.0),
                                    height: LengthPercentage::Length(10.0),
                                },
                                ..Default::default()
                            });

                            // Create a button for each option
                            for option in options.into_iter() {
                                let button_node = gui
                                    .add_widget::<ButtonRef>(
                                        Button {
                                            // Set the line text from the option as the label of the button
                                            label: Some(option.line.text),
                                            ..Default::default()
                                        },
                                        Style {
                                            size: Size::from_lengths(180.0, 20.0),
                                            ..Default::default()
                                        },
                                        gui.root(),
                                    )
                                    .unwrap();

                                // Push the button reference with the option reference
                                state.buttons.push((button_node, option.id));
                            }

                            // Build the GUI
                            state.gui = gui.build();
                        }
                        // Exit when the dialogue is complete
                        DialogueEvent::DialogueComplete => return true,
                        _ => (),
                    }
                }
            } else {
                // The user can select an option, wait for that action

                // Update the button GUI in the meantime
                state
                    .gui
                    .update_layout(Vec2::zero(), window_config.buffer_size.as_());

                // Update each button
                let mut clear_gui = false;
                for (button_node, option_id) in &state.buttons {
                    let button: &mut Button = state.gui.widget_mut(*button_node).unwrap();
                    // Handle the button press
                    if button.update(input, mouse_pos) {
                        // Select the option in the dialogue
                        state
                            .dialogue
                            .state
                            .set_selected_option(*option_id)
                            .unwrap();

                        clear_gui = true;
                    }
                }

                if clear_gui {
                    // Reset the GUI
                    state.gui = GuiBuilder::new(Style::DEFAULT).build();
                    state.buttons.clear();
                }
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

            // Draw the buttons
            for button_node in &state.buttons {
                let button: &Button = state.gui.widget(button_node.0).unwrap();
                button.render(canvas);
            }
        },
    )
    .expect("Error opening window");
}
