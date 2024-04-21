//! Create an embedded asset source.

use std::path::Path;

use proc_macro::TokenStream;
use quote::quote;
use walkdir::WalkDir;

/// Walk through the directory and generate code for each asset.
pub fn parse_dir(asset_dir: &Path) -> TokenStream {
    // Keep a separate entry for all textures
    let mut textures = Vec::new();

    // Walk over each file in each subdirectory
    let assets = WalkDir::new(asset_dir)
        // Make it deterministic
        .sort_by_file_name()
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            // Skip all directories
            if !entry.path().is_file() {
                return None;
            }

            let path = entry.path();

            // Extract the extension, ignore the file if not found
            let extension = path.extension()?.to_string_lossy();

            // Get the path relative to the asset dir
            let relative_path = path.strip_prefix(asset_dir).ok()?;

            // Create an ID from the path
            let id = relative_path
                .iter()
                .map(|path| path.to_string_lossy())
                .collect::<Vec<_>>()
                .join(".");

            // Remove the extension
            let id = id.strip_suffix(&format!(".{}", extension))?;

            // Parse images separately
            if extension == "png" {
                textures.push((id.to_owned(), path.to_path_buf()));

                return None;
            }

            // Convert the path to a string so it can be passed to `include_bytes!()`
            let path = path.display().to_string();

            // Define the asset
            Some(quote!(
                pixel_game_lib::assets::embedded::EmbeddedRawAsset {
                    id: #id,
                    extension: #extension,
                    bytes: include_bytes!(#path)
                }
            ))
        })
        .collect::<Vec<_>>();

    // Create a diced texture atlas
    let atlas = super::atlas::parse_textures(&textures);

    quote! {
        pixel_game_lib::assets::embedded::EmbeddedAssets {
            assets: &[#(#assets),*],
            atlas: #atlas
        }
    }
    .into()
}
