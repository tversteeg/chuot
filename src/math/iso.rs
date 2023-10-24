use vek::Vec2;

use super::Rotation;

/// Position with a rotation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Iso {
    /// Position before being rotated.
    pub pos: Vec2<f64>,
    /// Rotation.
    pub rot: Rotation,
}

impl Iso {
    /// Construct from a position and a rotation.
    pub fn new<P, R>(pos: P, rot: R) -> Self
    where
        P: Into<Vec2<f64>>,
        R: Into<Rotation>,
    {
        let pos = pos.into();
        let rot = rot.into();

        Self { pos, rot }
    }

    /// Construct from a position with a rotation of zero.
    pub fn from_pos<P>(pos: P) -> Self
    where
        P: Into<Vec2<f64>>,
    {
        let pos = pos.into();
        let rot = Rotation::zero();

        Self { pos, rot }
    }

    /// Rotate a relative point and add the position.
    #[inline]
    pub fn translate(&self, point: Vec2<f64>) -> Vec2<f64> {
        self.pos + self.rot.rotate(point)
    }
}

impl From<(Vec2<f64>, Rotation)> for Iso {
    fn from((pos, rot): (Vec2<f64>, Rotation)) -> Self {
        Self { pos, rot }
    }
}

#[cfg(feature = "physics")]
impl From<Iso> for parry2d_f64::na::Isometry2<f64> {
    fn from(value: Iso) -> Self {
        parry2d_f64::na::Isometry2::new(
            parry2d_f64::na::Vector2::new(value.pos.x, value.pos.y),
            value.rot.to_radians(),
        )
    }
}
