use std::any::Any;

use crate::{
    assets::{AssetOrPath, LoadedAsset},
    canvas::Canvas,
    font::Font,
    sprite::{Sprite, SpriteMetadata},
};

use blit::slice::Slice;
use taffy::NodeId;
use vek::{Extent2, Rect, Vec2};
use winit::event::MouseButton;
use winit_input_helper::WinitInputHelper;

use super::{Widget, WidgetRef};

/// A simple button widget.
#[derive(Debug)]
pub struct Button {
    /// Top-left position of the widget in pixels.
    pub offset: Vec2<f64>,
    /// Size of the button in pixels.
    pub size: Extent2<f64>,
    /// Extra size of the click region in pixels.
    ///
    /// Relative to the offset.
    pub click_region: Option<Rect<f64, f64>>,
    /// A custom label with text centered at the button.
    pub label: Option<String>,
    /// Current button state.
    pub state: State,
    /// Taffy layout node.
    pub node: NodeId,
    /// Where to load the assets.
    pub assets: ButtonAssetPaths,
}

impl Button {
    /// Handle the input.
    ///
    /// Return when the button is released.
    pub fn update(&mut self, input: &WinitInputHelper, mouse_pos: Option<Vec2<usize>>) -> bool {
        let mut rect = Rect::new(self.offset.x, self.offset.y, self.size.w, self.size.h);
        if let Some(mut click_region) = self.click_region {
            click_region.x += self.offset.x;
            click_region.y += self.offset.y;
            rect = rect.union(click_region);
        }

        match self.state {
            State::Normal => {
                if let Some(mouse_pos) = mouse_pos {
                    if !input.mouse_held(MouseButton::Left) && rect.contains_point(mouse_pos.as_())
                    {
                        self.state = State::Hover;
                    }
                }

                false
            }
            State::Hover => {
                if let Some(mouse_pos) = mouse_pos {
                    if !rect.contains_point(mouse_pos.as_()) {
                        self.state = State::Normal;
                    } else if input.mouse_pressed(MouseButton::Left) {
                        self.state = State::Down;
                    }
                }

                false
            }
            State::Down => {
                if input.mouse_released(MouseButton::Left) {
                    self.state = State::Normal;

                    true
                } else {
                    false
                }
            }
        }
    }

    /// Render the button.
    pub fn render(&self, canvas: &mut Canvas) {
        let button: LoadedAsset<Sprite> = match self.state {
            State::Normal => &self.assets.normal,
            State::Hover => &self.assets.hover,
            State::Down => &self.assets.down,
        }
        .into();
        button.render_area(self.offset, self.size.as_(), canvas);

        if let Some(label) = &self.label {
            let font: LoadedAsset<Font> = (&self.assets.font).into();
            font.render_centered(
                label,
                self.offset + (self.size.w / 2.0, self.size.h / 2.0),
                canvas,
            );
        }
    }
}

impl Widget for Button {
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

#[cfg(feature = "default-gui")]
impl Default for Button {
    fn default() -> Self {
        Self {
            offset: Vec2::zero(),
            size: Extent2::zero(),
            label: None,
            state: State::default(),
            click_region: None,
            node: NodeId::new(0),
            assets: ButtonAssetPaths::default(),
        }
    }
}

/// In which state the button can be.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Button is doing nothing.
    #[default]
    Normal,
    /// Button is hovered over by the mouse.
    Hover,
    /// Button is hold down.
    Down,
}

/// Asset paths for the sprites used for drawing a button.
#[derive(Debug)]
pub struct ButtonAssetPaths {
    /// Normal background, when not hovering or pressing.
    pub normal: AssetOrPath<Sprite>,
    /// Hover background, when the mouse is over the button but not pressing it.
    pub hover: AssetOrPath<Sprite>,
    /// Down background, when the mouse is pressing on the button.
    pub down: AssetOrPath<Sprite>,
    /// Font asset path.
    pub font: AssetOrPath<Font>,
}

#[cfg(feature = "default-gui")]
impl Default for ButtonAssetPaths {
    fn default() -> Self {
        // Sprite metadata is the same for each button
        let sprite_metadata = SpriteMetadata {
            vertical_slice: Some(Slice::Ternary {
                split_first: 3,
                split_last: 4,
            }),
            horizontal_slice: Some(Slice::Ternary {
                split_first: 3,
                split_last: 6,
            }),
            ..Default::default()
        };

        let normal = AssetOrPath::Owned(
            Sprite::from_png_bytes(
                include_bytes!("../../assets/button-normal.png"),
                sprite_metadata.clone(),
            )
            .unwrap(),
        );
        let hover = AssetOrPath::Owned(
            Sprite::from_png_bytes(
                include_bytes!("../../assets/button-hover.png"),
                sprite_metadata.clone(),
            )
            .unwrap(),
        );
        let down = AssetOrPath::Owned(
            Sprite::from_png_bytes(
                include_bytes!("../../assets/button-down.png"),
                sprite_metadata,
            )
            .unwrap(),
        );

        Self {
            normal,
            hover,
            down,
            #[cfg(feature = "default-font")]
            font: AssetOrPath::Owned(Font::default()),
            #[cfg(not(feature = "default-font"))]
            font: "font".into(),
        }
    }
}

/// Gui reference for retrieving constructed buttons.
///
/// See [`WidgetRef`].
#[derive(Copy, Clone)]
pub struct ButtonRef(NodeId);

impl WidgetRef for ButtonRef {
    type Widget = Button;
}

impl From<ButtonRef> for NodeId {
    fn from(val: ButtonRef) -> Self {
        val.0
    }
}

impl From<NodeId> for ButtonRef {
    fn from(value: NodeId) -> Self {
        Self(value)
    }
}
