//! Layout, draw and interact with gui widgets.
//!
//! Requires the `gui` feature flag.

use taffy::{prelude::Node, Taffy};
use vek::{Extent2, Vec2};

/// Allow calling function on widgets in a simple way.
pub trait Widget {
    /// Attach a layout node to the widget when constructing the widget.
    fn with_layout(self, layout_node: &Node) -> Self
    where
        Self: Sized;

    /// Update the widget layout position, defines how it must be drawn.
    fn update_layout(&mut self, location: Vec2<f64>, size: Extent2<f64>);

    /// Returns a refeference to the layout node of this widget.
    fn layout_node(&self) -> &Node;
}

/// A GUI with a tree of widgets.
pub struct Gui {
    /// References to all widgets, so they can be updated.
    widgets: Vec<Box<dyn Widget>>,
    /// Taffy layout, will update the position and sizes of the widgets.
    layout: Taffy,
    /// Root layout node, must be set before anything can be drawn.
    root: Option<Node>,
}

impl Gui {
    /// Construct a new Gui section where widgets can be added and the layout set.
    pub fn new() -> Self {
        let widgets = Vec::new();
        let layout = Taffy::new();
        let root = None;

        Self {
            widgets,
            layout,
            root,
        }
    }

    /// Register a widget.
    ///
    /// This will automatically update the widget size and location when it changes.
    pub fn add_widget<W>(&mut self, widget: W, layout_node: &Node)
    where
        W: Widget + 'static,
    {
        self.widgets.push(Box::new(widget.with_layout(layout_node)));
    }

    /// Set the root node.
    ///
    /// Must be set before anything can be drawn or updated.
    pub fn root(&mut self, root_layout_node: Node) {
        self.root = Some(root_layout_node);
    }

    /// Get a mutable reference to the taffy layout so children can be added.
    pub fn layout_mut(&mut self) -> &mut Taffy {
        &mut self.layout
    }
}

impl Default for Gui {
    fn default() -> Self {
        Self::new()
    }
}
