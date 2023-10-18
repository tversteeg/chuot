//! Layout, draw and interact with gui widgets.
//!
//! Requires the `gui` feature flag.

pub mod button;

use std::{any::Any, collections::HashMap};

use miette::{Context, IntoDiagnostic, Result};
use taffy::{
    prelude::{Node, Size},
    style::Style,
    style_helpers::TaffyMaxContent,
    tree::LayoutTree,
    Taffy,
};
use vek::{Extent2, Vec2};

/// Allow calling function on widgets in a simple way.
pub trait Widget {
    /// Update the widget layout position, defines how it must be drawn.
    fn update_layout(&mut self, location: Vec2<f64>, size: Extent2<f64>);

    /// Convert to [`Any`] so we can convert it back to the original type.
    fn as_any(&self) -> &dyn Any;

    /// Convert to mutable [`Any`] so we can convert it back to the original type.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Construct a GUI from a tree of widgets defined by the layout.
pub struct GuiBuilder {
    /// References to all widgets, so they can be updated.
    widgets: HashMap<Node, Box<dyn Widget>>,
    /// Taffy layout, will update the position and sizes of the widgets.
    layout: Taffy,
    /// Root node.
    root: Node,
}

impl GuiBuilder {
    /// Start creating a GUI.
    ///
    /// # Arguments
    ///
    /// * `root_layout` - Layout for the root node. Size will be automatically set by the [`Widget::update_layout`] trait call.
    pub fn new(root_layout: Style) -> Self {
        let widgets = HashMap::new();
        let mut layout = Taffy::new();
        // This shouldn't fail
        let root = layout
            .new_leaf(root_layout)
            .expect("Error adding root node to layout");

        Self {
            widgets,
            layout,
            root,
        }
    }

    /// Register a widget.
    ///
    /// This will automatically update the widget size and location when it changes.
    ///
    /// # Arguments
    ///
    /// * `widget` - Widget implementing the [`Widget`] trait.
    /// * `layout` - How the [`Widget`] behaves as a layout.
    /// * `parent` - Parent Taffy node the widget will be a child of.
    ///
    /// # Returns
    ///
    /// A Taffy node type that can be used to create a hierarchy of nodes.
    /// This can be used in the update or draw loop to get a reference to the widget itself.
    pub fn add_widget<W>(&mut self, widget: W, layout: Style, parent: Node) -> Result<Node>
    where
        W: Widget + 'static,
    {
        // Create a new Taffy layout node
        let node = self
            .layout
            .new_leaf(layout)
            .into_diagnostic()
            .wrap_err("Error adding new leaf to layout tree")?;

        // Insert the widget
        self.widgets.insert(node, Box::new(widget));

        // Attach the child to the parent
        self.layout
            .add_child(parent, node)
            .into_diagnostic()
            .wrap_err("Error adding child layout node to parent")?;

        Ok(node)
    }

    /// Build the GUI.
    pub fn build(self) -> Gui {
        let Self {
            root,
            widgets,
            layout,
        } = self;

        Gui {
            widgets,
            layout,
            root,
        }
    }

    /// The root node so children can be added to it.
    pub fn root(&self) -> Node {
        self.root
    }
}

/// A GUI with a tree of widgets.
///
/// The GUI uses the [`taffy`](https://docs.rs/taffy) crate for layouts, where the size is defined as buffer pixels.
pub struct Gui {
    /// References to all widgets, so they can be updated.
    widgets: HashMap<Node, Box<dyn Widget>>,
    /// Taffy layout, will update the position and sizes of the widgets.
    layout: Taffy,
    /// Root layout node.
    root: Node,
}

impl Gui {
    /// Get a reference to the boxed widget based on the node.
    pub fn widget<W>(&self, widget_node: Node) -> Result<&W>
    where
        W: Widget + 'static,
    {
        self.widgets
            .get(&widget_node)
            .ok_or_else(|| {
                miette::miette!(
            "Error getting the widget based on the node, are you sure you added it to the builder?"
        )
            })
            .and_then(|boxed| {
                boxed.as_any().downcast_ref::<W>().ok_or_else(|| {
                    miette::miette!(
                        "Error casting widget to original type, did you use the proper type?"
                    )
                })
            })
    }

    /// Get a mutable reference to the boxed widget based on the node.
    pub fn widget_mut<W>(&mut self, widget_node: Node) -> Result<&mut W>
    where
        W: Widget + 'static,
    {
        self.widgets
            .get_mut(&widget_node)
            .ok_or_else(|| {
                miette::miette!(
            "Error getting the widget based on the node, are you sure you added it to the builder?"
        )
            })
            .and_then(|boxed| {
                boxed.as_any_mut().downcast_mut::<W>().ok_or_else(|| {
                    miette::miette!(
                        "Error casting widget to original type, did you use the proper type?"
                    )
                })
            })
    }
}

impl Widget for Gui {
    fn update_layout(&mut self, location: Vec2<f64>, size: Extent2<f64>) {
        // Update root node layout
        let mut root_style = self.layout.style(self.root).unwrap().clone();
        root_style.size = Size::from_points(size.w as f32, size.h as f32);
        self.layout.set_style(self.root, root_style).unwrap();

        // Compute the new layout
        self.layout
            .compute_layout(self.root, Size::MAX_CONTENT)
            .expect("Computing layout failed");

        // Update all child widget layouts
        self.widgets
            .iter_mut()
            // We need to calculate the location recursively
            .map(|(node, widget)| {
                // Find the absolute location of this node by traversing the node tree
                let layout = self.layout.layout(*node).unwrap().location;
                let mut widget_location = Vec2::new(layout.x as f64, layout.y as f64);

                // Offset by the root offset
                widget_location += location;

                let mut previous_node = *node;
                while let Some(parent) = self.layout.parent(previous_node) {
                    let layout = self.layout.layout(parent).unwrap().location;
                    widget_location.x += layout.x as f64;
                    widget_location.y += layout.y as f64;

                    previous_node = parent;
                }

                (node, widget, widget_location)
            })
            // Update the layout of all widgets
            .for_each(|(node, widget, location)| {
                // The size is stored
                let size = self
                    .layout
                    .layout(*node)
                    .expect("Could not get layout for widget node")
                    .size;

                // Can't use iter_mut above because of mutable borrow
                widget.update_layout(location, Extent2::new(size.width, size.height).as_());
            });
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
