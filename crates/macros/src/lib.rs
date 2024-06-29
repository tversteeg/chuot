#![forbid(unsafe_code)]

//! This crate provides the `assets!` macro for [`chuot`](https://docs.rs/chuot).

mod asset_source;
#[cfg(feature = "embed-assets")]
mod embedded;

use asset_source::Input;
use proc_macro::TokenStream;

/// Define the asset source for `chuot::Game::run`.
#[proc_macro]
pub fn load_assets(input: TokenStream) -> TokenStream {
    match Input::parse(input) {
        Ok(input) => input,
        Err(tokenstream) => return tokenstream,
    }
    .expand_dir()
}
