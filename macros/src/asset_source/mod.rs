//! Asset source for loading assets.

#[cfg(any(
    target_arch = "wasm32",
    not(feature = "hot-reloading-assets"),
    not(doctest)
))]
mod embedded;

use litrs::Literal;
use proc_macro::TokenStream;
use std::path::{Path, PathBuf};

/// Asset source for [`pixel_game_lib::load_assets`].
pub struct Input(pub PathBuf);

impl Input {
    /// Parse from a token stream.
    pub fn parse(input: TokenStream) -> Result<Self, TokenStream> {
        // Read the literal as a path
        let path_str = match input.into_iter().next() {
            Some(tt) => {
                let lit = Literal::try_from(tt).map_err(|err| err.to_compile_error())?;

                match lit {
                    Literal::String(asset_path_str) => asset_path_str,
                    _ => panic!("input has to be a string literal, but this is not: {lit}"),
                }
                .value()
                .to_owned()
            }
            // Use the default
            None => "assets/".to_owned(),
        };

        // Get the full path on disk
        let canonicalized = match Path::new(&path_str).canonicalize() {
            Ok(path) => path,
            Err(err) => panic!(
                "Asset path '{}' could not be canonicalized: {err}",
                path_str
            ),
        };

        Ok(Self(canonicalized))
    }

    /// Read the directory and create the Rust code to load everything for it.
    #[cfg(any(
        target_arch = "wasm32",
        not(feature = "hot-reloading-assets"),
        not(doctest)
    ))]
    pub fn expand_dir(&self) -> TokenStream {
        embedded::expand_dir(&self.0)
    }

    /// Create the Rust code to load from the directory.
    #[cfg(not(any(
        target_arch = "wasm32",
        not(feature = "hot-reloading-assets"),
        not(doctest)
    )))]
    pub fn expand_dir(&self) -> TokenStream {
        let path = self.0.display().to_string();
        quote::quote! {
            pixel_game_lib::assets::source::FileSystem::new(#path).expect("Error setting up asset filesystem from path")
        }
        .into()
    }
}
