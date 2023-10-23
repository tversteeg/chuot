use std::any::Any;

use crate::{
    assets::{AssetOrPath, LoadedAsset},
    canvas::Canvas,
    font::Font,
};

use super::{Widget, WidgetRef};

use taffy::prelude::Node;
use vek::{Extent2, Vec2};

/// A simple text label widget.
#[derive(Debug)]
pub struct Label {
    /// Top-left position of the widget in pixels.
    pub offset: Vec2<f64>,
    /// Size of the label area in pixels.
    pub size: Extent2<f64>,
    /// The text to draw.
    pub label: String,
    /// Taffy layout node.
    pub node: Node,
    /// Where to load the font asset.
    pub font_asset: AssetOrPath<Font>,
}

impl Label {
    /// Render the label.
    pub fn render(&self, canvas: &mut Canvas) {
        let font: LoadedAsset<Font> = (&self.font_asset).into();
        font.render_centered(
            &self.label,
            self.offset + (self.size.w / 2.0, self.size.h / 2.0),
            canvas,
        );
    }
}

impl Widget for Label {
    fn update_layout(&mut self, location: Vec2<f64>, size: Extent2<f64>) {
        self.offset = location;
        self.size = size;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for Label {
    fn default() -> Self {
        Self {
            offset: Vec2::zero(),
            label: String::new(),
            node: Node::default(),
            size: Extent2::default(),
            #[cfg(feature = "default-font")]
            font_asset: AssetOrPath::Owned(Font::default()),
            #[cfg(not(feature = "default-font"))]
            font_asset: "font".into(),
        }
    }
}

/// Gui reference for retrieving constructed labels.
///
/// See [`WidgetRef`].
#[derive(Copy, Clone)]
pub struct LabelRef(Node);

impl WidgetRef for LabelRef {
    type Widget = Label;
}

impl From<LabelRef> for Node {
    fn from(val: LabelRef) -> Self {
        val.0
    }
}

impl From<Node> for LabelRef {
    fn from(value: Node) -> Self {
        Self(value)
    }
}
