use glam;
use macroquad::math;

impl From<math::Vec4> for glam::Vec4 {
    fn from(mv: math::Vec4) -> Self {
        glam::Vec4::new(mv.x, mv.y, mv.z, mv.w)
    }
}

impl From<glam::Vec4> for math::Vec4 {
    fn from(gv: glam::Vec4) -> Self {
        math::Vec4::new(gv.x, gv.y, gv.z, gv.w)
    }
}

impl From<math::Vec2> for glam::Vec2 {
    fn from(mv: math::Vec2) -> Self {
        glam::Vec2::new(mv.x, mv.y)
    }
}

impl From<glam::Vec2> for math::Vec2 {
    fn from(gv: glam::Vec2) -> Self {
        math::Vec2::new(gv.x, gv.y)
    }
}