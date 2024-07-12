//! Asset source for loading assets.

use std::path::{Path, PathBuf};

use litrs::Literal;
use proc_macro::TokenStream;

/// Asset source for [`chuot::load_assets`].
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
            Err(err) => panic!("Asset path '{path_str}' could not be canonicalized: {err}",),
        };

        Ok(Self(canonicalized))
    }

    /// Read the directory and create the Rust code to load everything for it.
    #[allow(unused_mut)]
    pub fn expand_dir(&self) -> TokenStream {
        let asset_path = self.0.to_string_lossy();

        // Just return the asset directory, all files will be loaded from there during runtime
        let mut source: TokenStream = quote::quote! {
            chuot::AssetSource::new().with_runtime_dir(#asset_path)
        }
        .into();

        // Add the embedded assets if the feature flag is enabled
        #[cfg(feature = "embed-assets")]
        source.extend(crate::embedded::asset_source::parse_dir(&self.0));

        source
    }
}
