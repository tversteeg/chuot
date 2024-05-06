//! Dialogue yarn asset loader.

use std::collections::HashMap;

use yarnspinner::{
    compiler::{Compiler, File},
    prelude::Dialogue as YarnDialogue,
    runtime::{MemoryVariableStorage, StringTableTextProvider},
};

use crate::dialogue::Dialogue;

use super::Loader;

/// Yarn Spinner dialogue asset loader.
#[non_exhaustive]
pub struct YarnLoader;

impl Loader<Dialogue> for YarnLoader {
    const EXTENSION: &'static str = "yarn";

    #[inline]
    fn load(bytes: &[u8]) -> Dialogue {
        // Create a fake file, we don't know the filename
        let file_name = "__file__".to_owned();
        let source = String::from_utf8(bytes.to_owned()).expect("Yarn file is not valid UTF-8");

        let yarn_file = File { file_name, source };

        // Compile the yarn source file
        let compilation = Compiler::new()
            .add_file(yarn_file)
            .compile()
            .expect("Error compiling Yarn file");

        // Split the resulting string table into metadata and base language strings
        let (base_language_string_table, metadata): (HashMap<_, _>, HashMap<_, _>) = compilation
            .string_table
            .into_iter()
            .map(|(line_id, string_info)| {
                (
                    (line_id.clone(), string_info.text),
                    (line_id, string_info.metadata),
                )
            })
            .unzip();

        // Use the default text provider for feeding text into the game
        let mut text_provider = StringTableTextProvider::new();
        text_provider.extend_base_language(base_language_string_table);

        // Storage where all the variables defined in the dialogue will be stored
        let variable_storage = MemoryVariableStorage::new();

        // Create the dialogue
        let mut state = YarnDialogue::new(Box::new(variable_storage), Box::new(text_provider));

        // Give the dialogue our compiled program
        state.add_program(compilation.program.expect("Error compiling Yarn dialogue"));

        Dialogue { state, metadata }
    }
}
