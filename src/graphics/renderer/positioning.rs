
use nalgebra_glm as glm;

use crate::graphics::camera::Camera;

pub const MAXIMUM_ABS_SPACE: f32 = 8.0;
pub const SCALE_FACTOR: f32 = i16::MAX as f32 / MAXIMUM_ABS_SPACE;

pub const fn f32_to_short(v: f32) -> i16 {
    (v * SCALE_FACTOR).ceil().clamp(i16::MIN as f32, i16::MAX as f32) as i16
}

pub fn vec_to_short<const S: usize>(v: &glm::TVec<f32, S>) -> glm::TVec<i16, S> {
    v.map(|x| f32_to_short(x))
}

pub fn vec_to_short_recenter(v: glm::Vec2, camera: &Camera) -> glm::I16Vec2 {
    vec_to_short(&((v - camera.get_position()) * camera.get_zoom()))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionMode {
	TopLeft,    TopCenter,    TopRight,
	CenterLeft, Center,       CenterRight,
	BottomLeft, BottomCenter, BottomRight,
}

#[derive(Debug, Clone, Copy)]
pub enum BaseDimensions {
    Width(f32),
    Height(f32),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RenderBox {
    pub min: glm::Vec2,
    pub max: glm::Vec2,
}

impl RenderBox {
    pub fn new(
        position: glm::Vec2,
        position_mode: PositionMode,
        original_size: glm::Vec2,
        base_dimension: BaseDimensions,
    ) -> Self {
        let aspect_ratio = original_size.x / original_size.y;
    
        let size = match base_dimension {
            BaseDimensions::Width(w) => glm::vec2(w, w / aspect_ratio),
            BaseDimensions::Height(h) => glm::vec2(h * aspect_ratio, h),
        };

        let offset = match position_mode {
            PositionMode::TopLeft      => glm::vec2(0.0, -size.y),
            PositionMode::TopCenter    => glm::vec2(-size.x / 2.0, -size.y),
            PositionMode::TopRight     => glm::vec2(-size.x, -size.y),
            
            PositionMode::CenterLeft   => glm::vec2(0.0, -size.y / 2.0),
            PositionMode::Center       => glm::vec2(-size.x / 2.0, -size.y / 2.0),
            PositionMode::CenterRight  => glm::vec2(-size.x, -size.y / 2.0),
            
            PositionMode::BottomLeft   => glm::vec2(0.0, 0.0),
            PositionMode::BottomCenter => glm::vec2(-size.x / 2.0, 0.0),
            PositionMode::BottomRight  => glm::vec2(-size.x, 0.0),
        };

        let min = position + offset;
        let max = min + size;

        return Self { min, max };
    }

    pub fn new_simple(
        position: glm::Vec2,
        position_mode: PositionMode,
        size: glm::Vec2,
    ) -> Self {
        let offset = match position_mode {
            PositionMode::TopLeft      => glm::vec2(0.0, -size.y),
            PositionMode::TopCenter    => glm::vec2(-size.x / 2.0, -size.y),
            PositionMode::TopRight     => glm::vec2(-size.x, -size.y),
            
            PositionMode::CenterLeft   => glm::vec2(0.0, -size.y / 2.0),
            PositionMode::Center       => glm::vec2(-size.x / 2.0, -size.y / 2.0),
            PositionMode::CenterRight  => glm::vec2(-size.x, -size.y / 2.0),
            
            PositionMode::BottomLeft   => glm::vec2(0.0, 0.0),
            PositionMode::BottomCenter => glm::vec2(-size.x / 2.0, 0.0),
            PositionMode::BottomRight  => glm::vec2(-size.x, 0.0),
        };

        let min = position + offset;
        let max = min + size;

        return Self { min, max };
    }

    pub fn get_top_right(&self) -> glm::Vec2 {
        self.max
    }

    pub fn get_top_left(&self) -> glm::Vec2 {
        glm::vec2(self.min.x, self.max.y)
    }

    pub fn get_bottom_right(&self) -> glm::Vec2 {
        glm::vec2(self.max.x, self.min.y)
    }
}

pub struct OrientedBox {
    pub top_left: glm::Vec2,
    pub top_right: glm::Vec2,
    pub bot_left: glm::Vec2,
    pub bot_right: glm::Vec2,
}

impl OrientedBox {
    pub fn new(
        position: glm::Vec2,
        position_mode: PositionMode,
        original_size: glm::Vec2,
        base_dimension: BaseDimensions,
        up_vector: glm::Vec2,
    ) -> OrientedBox {
        let up = glm::normalize(&up_vector);
        let right = glm::vec2(up.y, -up.x);

        let aspect_ratio = original_size.x / original_size.y;
        let size = match base_dimension {
            BaseDimensions::Width(w) => glm::vec2(w, w / aspect_ratio),
            BaseDimensions::Height(h) => glm::vec2(h * aspect_ratio, h),
        };

        let half_w = size.x / 2.0;
        let half_h = size.y / 2.0;

        let center_offset = match position_mode {
            PositionMode::TopLeft      => right * half_w - up * half_h,
            PositionMode::TopCenter    => -up * half_h,
            PositionMode::TopRight     => -right * half_w - up * half_h,
            PositionMode::CenterLeft   => right * half_w,
            PositionMode::Center       => glm::vec2(0.0, 0.0),
            PositionMode::CenterRight  => -right * half_w,
            PositionMode::BottomLeft   => right * half_w + up * half_h,
            PositionMode::BottomCenter => up * half_h,
            PositionMode::BottomRight  => -right * half_w + up * half_h,
        };

        let center = position + center_offset;

        let half_up = up * half_h;
        let half_right = right * half_w;

        return OrientedBox {
            top_left:  center + half_up - half_right,
            top_right: center + half_up + half_right,
            bot_left:  center - half_up - half_right,
            bot_right: center - half_up + half_right,
        };
    }

    pub fn new_rotated(
        position: glm::Vec2,
        position_mode: PositionMode,
        size: glm::Vec2,
        up_vector: glm::Vec2,
    ) -> Self {
        let up = glm::normalize(&up_vector);
        let right = glm::vec2(up.y, -up.x);

        let half_w = size.x / 2.0;
        let half_h = size.y / 2.0;

        let center_offset = match position_mode {
            PositionMode::TopLeft      => right * half_w - up * half_h,
            PositionMode::TopCenter    => -up * half_h,
            PositionMode::TopRight     => -right * half_w - up * half_h,
            PositionMode::CenterLeft   => right * half_w,
            PositionMode::Center       => glm::vec2(0.0, 0.0),
            PositionMode::CenterRight  => -right * half_w,
            PositionMode::BottomLeft   => right * half_w + up * half_h,
            PositionMode::BottomCenter => up * half_h,
            PositionMode::BottomRight  => -right * half_w + up * half_h,
        };

        let center = position + center_offset;

        let half_up = up * half_h;
        let half_right = right * half_w;

        return Self {
            top_left:  center + half_up - half_right,
            top_right: center + half_up + half_right,
            bot_left:  center - half_up - half_right,
            bot_right: center - half_up + half_right,
        };
    }

    pub fn get_top_right(&self) -> glm::Vec2 {
        self.top_right
    }

    pub fn get_bottom_left(&self) -> glm::Vec2 {
        self.bot_left
    }
}


pub fn scale_dimension(original_size: glm::Vec2, base_dimension: BaseDimensions) -> f32 {
    match base_dimension {
        BaseDimensions::Height(new_h) => (new_h / original_size.y) * original_size.x,
        BaseDimensions::Width(new_w) => (new_w / original_size.x) * original_size.y,
    }
}

pub fn screen_pos_to_world_pos(
    screen_pos: glm::Vec2,
    window_size: glm::Vec2,
    i_proj_matrix: &glm::Mat4,
) -> glm::Vec2 {
    let width = window_size.x;
    let height = window_size.y;
    
    let ndc_x = (2.0 * screen_pos.x as f32) / width as f32 - 1.0;
    let ndc_y = 1.0 - (2.0 * screen_pos.y as f32) / height as f32;
    let ndc_vec = glm::vec4(ndc_x, ndc_y, 0.0, 1.0);
    let transformed = i_proj_matrix * ndc_vec;

    return glm::vec2(transformed.x, transformed.y);
}

pub enum SimpleTransform {
    None,
    FlipHorizontal,
    FlipVertical,
    Rotate180,
}