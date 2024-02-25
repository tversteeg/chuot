//! Hot-reloadable wrapper for [Yarn Spinner](https://www.yarnspinner.dev/).

use std::{borrow::Cow, collections::HashMap};

use assets_manager::{loader::Loader, Asset, BoxedError};
use miette::{Context, IntoDiagnostic, Result};
use yarnspinner::{
    compiler::{Compiler, File},
    core::{IntoYarnValueFromNonYarnValue, LineId, YarnFn, YarnValue},
    prelude::Dialogue as YarnDialogue,
    runtime::{MemoryVariableStorage, StringTableTextProvider},
};

/// Dialogue system based on Yarn Spinner.
///
/// Can be hot reloaded, the starting node is always called `start`.
#[derive(Debug)]
pub struct Dialogue {
    /// Yarnspinner dialogue internal.
    pub state: YarnDialogue,
    /// Per-line metadata.
    pub metadata: HashMap<LineId, Vec<String>>,
}

impl Dialogue {
    /// Register a function that can be triggered from the dialogue state.
    pub fn register_function<M, F>(&mut self, name: impl Into<Cow<'static, str>>, function: F)
    where
        M: 'static,
        F: YarnFn<M> + 'static + Clone,
        F::Out: IntoYarnValueFromNonYarnValue + 'static + Clone,
    {
        self.state.library_mut().add_function(name, function);
    }

    /// Set the value of a dialogue variable.
    ///
    /// Name must start with `"$.."`
    pub fn set_variable(
        &mut self,
        name: impl Into<String>,
        value: impl Into<YarnValue>,
    ) -> Result<()> {
        let name = name.into();

        self.state
            .variable_storage_mut()
            .set(name.clone(), value.into())
            .into_diagnostic()
            .wrap_err_with(|| format!("Error setting dialogue variable '{name}'"))
    }

    /// Get the current value as a number of a dialogue variable.
    ///
    /// Name must start with `"$.."`
    pub fn number(&self, name: &str) -> Result<f32> {
        let YarnValue::Number(var) = self.variable(name)? else {
            miette::bail!("Yarn variable '{name}' is not a number");
        };

        Ok(var)
    }

    /// Get the current value as a string of a dialogue variable.
    ///
    /// Name must start with `"$.."`
    pub fn string(&self, name: &str) -> Result<String> {
        let YarnValue::String(var) = self.variable(name)? else {
            miette::bail!("Yarn variable '{name}' is not a string");
        };

        Ok(var)
    }

    /// Get the current value as a boolean of a dialogue variable.
    ///
    /// Name must start with `"$.."`
    pub fn boolean(&self, name: &str) -> Result<bool> {
        let YarnValue::Boolean(var) = self.variable(name)? else {
            miette::bail!("Yarn variable '{name}' is not a boolean");
        };

        Ok(var)
    }

    /// Get the current raw Yarn value of a dialogue variable.
    ///
    /// Name must start with `"$.."`
    fn variable(&self, name: &str) -> Result<YarnValue> {
        self.state
            .variable_storage()
            .get(name)
            .into_diagnostic()
            .wrap_err_with(|| format!("Error loading dialogue variable '{name}'"))
    }
}

impl Asset for Dialogue {
    const EXTENSION: &'static str = "yarn";

    type Loader = DialogueLoader;
}

/// Yarn Spinner dialogue asset loader.
///
/// Currently only supports loading a single file.
pub struct DialogueLoader;

impl Loader<Dialogue> for DialogueLoader {
    fn load(content: Cow<[u8]>, ext: &str) -> Result<Dialogue, BoxedError> {
        assert_eq!(ext.to_lowercase(), "yarn");

        // Create a fake file, we don't know the filename
        let file_name = "dialogue.yarn".to_string();
        let source = String::from_utf8(content.into_owned())?;

        let yarn_file = File { file_name, source };

        // Compile the yarn source file
        let compilation = Compiler::new().add_file(yarn_file).compile()?;

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
        state.add_program(
            compilation
                .program
                .ok_or_else(|| miette::miette!("Error compiling Yarn dialogue"))?,
        );

        Ok(Dialogue { state, metadata })
    }
}
