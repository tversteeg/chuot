//! Hot-reloadable wrapper for [Yarn Spinner](https://www.yarnspinner.dev/).

use std::{borrow::Cow, collections::HashMap};

use miette::{Context, IntoDiagnostic, Result};
use yarnspinner::{
    core::{IntoYarnValueFromNonYarnValue, LineId, YarnFn, YarnValue},
    prelude::Dialogue as YarnDialogue,
};

use crate::assets::{loader::yarn::YarnLoader, AssetSource, Id, Loadable};

/// Dialogue system based on Yarn Spinner.
///
/// Can be hot reloaded, the starting node is always called `start`.
#[derive(Debug)]
#[non_exhaustive]
pub struct Dialogue {
    /// Yarnspinner dialogue internal.
    pub state: YarnDialogue,
    /// Per-line metadata.
    pub metadata: HashMap<LineId, Vec<String>>,
}

impl Dialogue {
    /// Register a function that can be triggered from the dialogue state.
    #[inline]
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
    ///
    /// # Errors
    ///
    /// - When dialogue variable could not be set.
    #[inline]
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
    ///
    /// # Errors
    ///
    /// - When dialogue variable is not a number
    #[inline]
    pub fn number(&self, name: &str) -> Result<f32> {
        let YarnValue::Number(var) = self.variable(name)? else {
            miette::bail!("Yarn variable '{name}' is not a number");
        };

        Ok(var)
    }

    /// Get the current value as a string of a dialogue variable.
    ///
    /// Name must start with `"$.."`
    ///
    /// # Errors
    ///
    /// - When dialogue variable is not a string.
    #[inline]
    pub fn string(&self, name: &str) -> Result<String> {
        let YarnValue::String(var) = self.variable(name)? else {
            miette::bail!("Yarn variable '{name}' is not a string");
        };

        Ok(var)
    }

    /// Get the current value as a boolean of a dialogue variable.
    ///
    /// Name must start with `"$.."`
    ///
    /// # Errors
    ///
    /// - When dialogue variable is not a boolean.
    #[inline]
    pub fn boolean(&self, name: &str) -> Result<bool> {
        let YarnValue::Boolean(var) = self.variable(name)? else {
            miette::bail!("Yarn variable '{name}' is not a boolean");
        };

        Ok(var)
    }

    /// Get the current raw Yarn value of a dialogue variable.
    ///
    /// Name must start with `"$.."`
    ///
    /// # Errors
    ///
    /// - When dialogue variable does not exist
    fn variable(&self, name: &str) -> Result<YarnValue> {
        self.state
            .variable_storage()
            .get(name)
            .into_diagnostic()
            .wrap_err_with(|| format!("Error loading dialogue variable '{name}'"))
    }
}

impl Loadable for Dialogue {
    #[inline]
    fn load_if_exists(id: &Id, assets: &AssetSource) -> Option<Self>
    where
        Self: Sized,
    {
        assets.load_if_exists::<YarnLoader, _>(id)
    }
}
