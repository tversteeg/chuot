//! A simple falling sand game.

use chuot::{Config, Context, Game, KeyCode, MouseButton, RGBA8};

/// Width of the screen but also width of the sandbox simulation.
const WIDTH: f32 = 240.0;
/// Height of the screen but also height of the sandbox simulation.
const HEIGHT: f32 = 192.0;
/// How many cells to fill to draw when clicking.
const BRUSH_SIZE: isize = 2;

/// State of a single pixel in the sand box.
#[derive(Clone, Copy)]
struct Cell {
    /// Type of the cell.
    element: Element,
    /// Whether the cell is visited this update loop.
    visited: bool,
    /// Color of the cell.
    color: RGBA8,
}

impl Default for Cell {
    fn default() -> Self {
        let element = Element::Air;
        let color = element.random_color();
        let visited = false;

        Self {
            element,
            visited,
            color,
        }
    }
}

/// Type of a cell.
#[repr(u8)]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Element {
    /// Empty.
    #[default]
    Air,
    /// Solid powder material.
    Sand,
    /// Solid un-movable material.
    Rock,
    /// Fluid material.
    Water,
}
impl Element {
    /// Create a random color based on the type
    fn random_color(self) -> RGBA8 {
        match self {
            Self::Air => RGBA8::new(0xCC, 0xCC, 0xFF, 0xFF),
            Self::Sand => {
                let variation = chuot::random(0.0, 15.0) as u8;

                RGBA8::new(0xFF, 0xFF - variation, 0x44 - variation, 0xFF - variation)
            }
            Self::Rock => {
                let variation = chuot::random(0.0, 20.0) as u8;

                RGBA8::new(0xAA - variation, 0xAA - variation, 0xAA - variation, 0xFF)
            }
            Self::Water => RGBA8::new(0xAA, 0xAA, 0xFF, 0xFF),
        }
    }
}

/// Define a game state for our example.
struct GameState {
    /// What kind of element to draw when clicking.
    brush: Element,
    /// Grid of pixels for each cell.
    sandbox: Vec<Cell>,
    /// Which horizontal direction to draw, toggles every update tick.
    update_horizontal_cells_forward: bool,
}

impl GameState {
    /// Setup the grid.
    fn new() -> Self {
        // Fill the sandbox with air
        let sandbox = vec![Cell::default(); (WIDTH * HEIGHT) as usize];
        // The default brush is sand
        let brush = Element::Sand;
        // Initial direction, value doesn't matter because it updates every update tick
        let draw_forward = false;

        Self {
            brush,
            sandbox,
            update_horizontal_cells_forward: draw_forward,
        }
    }

    /// Get the sandbox as a vector of pixels to draw.
    fn pixels(&self) -> Vec<RGBA8> {
        self.sandbox.iter().map(|cell| cell.color).collect()
    }

    /// Update the state of a single cell.
    fn update_cell(&mut self, x: usize, y: usize) {
        // Reference to the cell
        let Some(cell) = self.cell(x, y) else {
            return;
        };

        // Do nothing if the cell is already updated this loop or a static type
        if cell.element == Element::Air || cell.element == Element::Rock || cell.visited {
            return;
        }

        match cell.element {
            // Sand is a powder so it falls down in a triangle shape
            Element::Sand => self.handle_powder(x, y),
            // Water is a fluid so it also moves horizontally
            Element::Water => self.handle_fluid(x, y),
            // These have been handled already by the check above
            Element::Rock | Element::Air => unreachable!(),
        }
    }

    /// Update the cell as a powder.
    fn handle_powder(&mut self, x: usize, y: usize) {
        // Check if we can move down
        if let Some(below) = self.cell(x, y + 1) {
            if below.element == Element::Air || below.element == Element::Water {
                self.swap(x, y, x, y + 1);
                return;
            }
        }

        // Choose a random diagonal direction to see if we can move there
        let direction = chuot::random(-1.0, 1.0).signum() as isize;
        if let Some(diagonal_next) = self.cell(x.wrapping_add_signed(direction), y + 1) {
            if diagonal_next.element == Element::Air || diagonal_next.element == Element::Water {
                self.swap(x, y, x.wrapping_add_signed(direction), y + 1);
                return;
            }
        }

        // Choose the other diagonal direction to see if we can move there
        if let Some(diagonal_mirror) = self.cell(x.wrapping_add_signed(-direction), y + 1) {
            if diagonal_mirror.element == Element::Air || diagonal_mirror.element == Element::Water
            {
                self.swap(x, y, x.wrapping_add_signed(-direction), y + 1);
            }
        }
    }

    /// Update the cell as a fluid.
    fn handle_fluid(&mut self, x: usize, y: usize) {
        // Check if we can move down
        if let Some(below) = self.cell(x, y + 1) {
            if below.element == Element::Air {
                self.swap(x, y, x, y + 1);
                return;
            }
        }

        // Choose a random diagonal direction to see if we can move there
        let direction = chuot::random(-1.0, 1.0).signum() as isize;
        if let Some(diagonal_next) = self.cell(x.wrapping_add_signed(direction), y + 1) {
            if diagonal_next.element == Element::Air {
                self.swap(x, y, x.wrapping_add_signed(direction), y + 1);
                return;
            }
        }

        // Choose the other diagonal direction to see if we can move there
        if let Some(diagonal_mirror) = self.cell(x.wrapping_add_signed(-direction), y + 1) {
            if diagonal_mirror.element == Element::Air {
                self.swap(x, y, x.wrapping_add_signed(-direction), y + 1);
                return;
            }
        }

        // Choose a random diagonal direction to see if we can move there
        let direction = chuot::random(-1.0, 1.0).signum() as isize;
        if let Some(horizontal_next) = self.cell(x.wrapping_add_signed(direction), y) {
            if horizontal_next.element == Element::Air {
                self.swap(x, y, x.wrapping_add_signed(direction), y);
                return;
            }
        }

        // Choose the other diagonal direction to see if we can move there
        if let Some(horizontal_mirror) = self.cell(x.wrapping_add_signed(-direction), y) {
            if horizontal_mirror.element == Element::Air {
                self.swap(x, y, x.wrapping_add_signed(-direction), y);
            }
        }
    }

    /// Get the cell at the position.
    fn cell(&self, x: usize, y: usize) -> Option<Cell> {
        // Ignore cells at the edges
        if x >= WIDTH as usize || y >= HEIGHT as usize {
            return None;
        }

        Some(self.sandbox[Self::cell_index(x, y)])
    }

    /// Swap two cells.
    fn swap(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) {
        let index1 = Self::cell_index(x1, y1);
        let index2 = Self::cell_index(x2, y2);

        // Mark them as visited
        self.sandbox[index1].visited = true;
        self.sandbox[index2].visited = true;

        // Swap the elements in the array
        self.sandbox.swap(index1, index2);
    }

    /// Calculate the cell index for a coordinate.
    const fn cell_index(x: usize, y: usize) -> usize {
        x + y * WIDTH as usize
    }
}

impl Game for GameState {
    /// Create the texture once at startup.
    fn init(&mut self, ctx: Context) {
        // Create a new sprite with the size of the screen
        ctx.sprite("sandbox").create((WIDTH, HEIGHT), self.pixels());
    }

    /// Update the sandbox and handle input in the update loop.
    fn update(&mut self, ctx: Context) {
        // Only handle the mouse when it's on the buffer
        if let Some((mouse_x, mouse_y)) = ctx.mouse() {
            // Set the pixels to the selected element
            if ctx.mouse_held(MouseButton::Left) {
                // Draw a square of pixels
                for y in -BRUSH_SIZE..BRUSH_SIZE {
                    for x in -BRUSH_SIZE..BRUSH_SIZE {
                        // Convert mouse coordinates to index into the sandbox
                        let cell_index = Self::cell_index(
                            (mouse_x + x as f32).clamp(0.0, WIDTH - 1.0) as usize,
                            (mouse_y + y as f32).clamp(0.0, HEIGHT - 1.0) as usize,
                        );

                        // Create a cell for the brush
                        let element = self.brush;

                        // Create a color from the element
                        let color = element.random_color();

                        // Set the cell under the mouse
                        self.sandbox[cell_index] = Cell {
                            element,
                            color,
                            ..Default::default()
                        };
                    }
                }
            }
        }

        // Handle changing the brush with the keyboard
        if ctx.key_released(KeyCode::Digit1) || ctx.key_released(KeyCode::KeyS) {
            self.brush = Element::Sand;
        }
        if ctx.key_released(KeyCode::Digit2) || ctx.key_released(KeyCode::KeyW) {
            self.brush = Element::Water;
        }
        if ctx.key_released(KeyCode::Digit3) || ctx.key_released(KeyCode::KeyR) {
            self.brush = Element::Rock;
        }
        if ctx.key_released(KeyCode::Digit4) || ctx.key_released(KeyCode::KeyA) {
            self.brush = Element::Air;
        }

        // Reset the state for each cell to start the simulation
        self.sandbox
            .iter_mut()
            .for_each(|cell| cell.visited = false);

        // Perform the simulation
        for y in 0..(HEIGHT as usize) {
            if self.update_horizontal_cells_forward {
                for x in 0..(WIDTH as usize) {
                    self.update_cell(x, y);
                }
            } else {
                for x in (0..(WIDTH as usize)).rev() {
                    self.update_cell(x, y);
                }
            }
        }

        // Toggle the forward updates so it switches every tick
        self.update_horizontal_cells_forward = !self.update_horizontal_cells_forward;
    }

    /// Render the game.
    fn render(&mut self, ctx: Context) {
        // Update the sandbox texture's pixels
        ctx.sprite("sandbox")
            .update_pixels((0.0, 0.0, WIDTH, HEIGHT), self.pixels());

        // Draw the sandbox
        ctx.sprite("sandbox")
            // Use the UI camera which draws the center in the top left
            .use_ui_camera()
            .draw();

        // Show the keyboard buttons for the brush as text on the screen
        ctx.text(
            "Beachball",
            &format!(
                "FPS: {:.0}\nActive: {}\n1: Sand\n2: Water\n3: Rock\n4: Air",
                ctx.frames_per_second(),
                match self.brush {
                    Element::Air => "Air",
                    Element::Sand => "Sand",
                    Element::Rock => "Rock",
                    Element::Water => "Water",
                }
            ),
        )
        // Use the UI camera which draws the center in the top left
        .use_ui_camera()
        .translate((1.0, 1.0))
        .draw();
    }
}

/// Open an empty window.
fn main() {
    // Game configuration
    let config = Config {
        buffer_width: WIDTH,
        buffer_height: HEIGHT,
        scaling: 3.0,
        // Use a slightly darker viewport color than the color of air
        viewport_color: RGBA8::new(0xAA, 0xAA, 0xFF, 0xFF),
        // Update 100 times per second
        update_delta_time: 100.0_f32.recip(),
        ..Default::default()
    };

    // Spawn the window and run the game
    GameState::new().run(chuot::load_assets!(), config);
}
