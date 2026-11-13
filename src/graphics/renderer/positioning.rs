
use nalgebra_glm as glm;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionMode {
	TopLeft,    TopCenter,    TopRight,
	CenterLeft, Center,       CenterRight,
	BottomLeft, BottomCenter, BottomRight,
}

#[derive(Debug, Clone, Copy)]
pub struct RenderBox {
    pub min: glm::Vec2,
    pub max: glm::Vec2,
}

impl RenderBox {
    pub fn new(
        position: glm::Vec2,
        size: glm::Vec2,
        position_mode: PositionMode,
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

    pub fn sub_box(&self, offset: glm::Vec2, size: glm::Vec2, anchor: PositionMode, parent_anchor: PositionMode) -> RenderBox {
        let parent_anchor_position = self.get_point(parent_anchor);
        RenderBox::new(parent_anchor_position + offset, size, anchor)
    }

    pub fn offset(&self, amount: glm::Vec2) -> RenderBox {
        RenderBox {
            min: self.min + amount,
            max: self.max + amount,
        }
    }

    pub fn get_size(&self) -> glm::Vec2 {
        self.max - self.min
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

    pub fn get_bottom_left(&self) -> glm::Vec2 {
        self.min
    }

    pub fn is_point_inside(&self, point: glm::Vec2) -> bool {
        self.min.x <= point.x && point.x <= self.max.x &&
        self.min.y <= point.y && point.y <= self.max.y
    }

    pub fn get_center(&self) -> glm::Vec2 {
        (self.min + self.max) / 2.0
    }

    pub fn get_point(&self, mode: PositionMode) -> glm::Vec2 {
        let center = self.get_center();
        match mode {
            PositionMode::TopLeft      => glm::vec2(self.min.x, self.max.y),
            PositionMode::TopCenter    => glm::vec2(center.x,   self.max.y),
            PositionMode::TopRight     => glm::vec2(self.max.x, self.max.y),

            PositionMode::CenterLeft   => glm::vec2(self.min.x, center.y),
            PositionMode::Center       => center,
            PositionMode::CenterRight  => glm::vec2(self.max.x, center.y),

            PositionMode::BottomLeft   => glm::vec2(self.min.x, self.min.y),
            PositionMode::BottomCenter => glm::vec2(center.x,   self.min.y),
            PositionMode::BottomRight  => glm::vec2(self.max.x, self.min.y),
        }
    }
}

pub fn get_screen_bbox(window_size: glm::Vec2) -> RenderBox {
    let size = 2.0 * if window_size.x > window_size.y {
        glm::vec2(window_size.x / window_size.y, 1.0)
    } else {
        glm::vec2(1.0, window_size.y / window_size.x)
    };

    RenderBox::new(glm::vec2(0.0, 0.0), size, PositionMode::Center)
}

pub struct OrientedBox {
    pub top_left: glm::Vec2,
    pub top_right: glm::Vec2,
    pub bot_left: glm::Vec2,
    pub bot_right: glm::Vec2,
}

impl OrientedBox {
    pub fn new_rotated(
        position: glm::Vec2,
        position_mode: PositionMode,
        size: glm::Vec2,
        forward: glm::Vec2,
    ) -> Self {
        let right = glm::normalize(&forward);
        let up = glm::vec2(-right.y, right.x);

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
}

pub fn screen_position_to_ndc(screen_position: glm::Vec2, window_size: glm::Vec2) -> glm::Vec2 {
    let screen_bbox = get_screen_bbox(window_size);
    let norm = screen_position.component_div(&window_size);
    let inv_y = glm::vec2(norm.x, -norm.y + 1.0);

    screen_bbox.min.component_mul(&(glm::vec2(1.0, 1.0) - inv_y)) + screen_bbox.max.component_mul(&inv_y)
}
