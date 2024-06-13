use chuot::{Context, EmbeddedAssets, GameConfig, PixelGame};

#[derive(Default)]
struct GameState;

impl PixelGame for GameState {
    fn render(&mut self, ctx: Context) {}

    fn update(&mut self, _ctx: Context) {}
}

fn main() {
    GameState.run(EmbeddedAssets {}, GameConfig {});
}
