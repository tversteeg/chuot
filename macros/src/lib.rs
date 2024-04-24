//! This crate provides the `assets!` macro for [`pixel_game_lib`](https://docs.rs/pixel_game_lib).

mod asset_source;
#[cfg(feature = "embed-assets")]
mod embedded;

use asset_source::Input;
use proc_macro::TokenStream;

/// Define the asset source for `pixel_game_lib::PixelGame::run`.
#[proc_macro]
pub fn load_assets(input: TokenStream) -> TokenStream {
    match Input::parse(input) {
        Ok(input) => input,
        Err(tokenstream) => return tokenstream,
    }
    .expand_dir()
}
