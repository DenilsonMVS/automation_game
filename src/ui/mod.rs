

use nalgebra_glm as glm;
use crate::graphics::renderer::positioning::PositionMode;

pub mod render;

#[derive(Debug)]
pub enum UiElement {
    Box(Box),
    Text(Text),
}

#[derive(Debug)]
pub struct DirectionInfo<T> {
    top: T,
    bottom: T,
    left: T,
    right: T,
}

impl<T: Clone + Default> DirectionInfo<T> {
    pub fn all(value: T) -> Self {
        Self {
            top: value.clone(),
            bottom: value.clone(),
            left: value.clone(),
            right: value.clone(),
        }
    }

    pub fn vertical(vertical: T) -> Self {
        Self {
            top: vertical.clone(),
            bottom: vertical,
            left: Default::default(),
            right: Default::default(),
        }
    }

    pub fn horizontal(horizontal: T) -> Self {
        Self {
            top: Default::default(),
            bottom: Default::default(),
            left: horizontal.clone(),
            right: horizontal,
        }
    }

    pub fn vertical_horizontal(vertical: T, horizontal: T) -> Self {
        Self {
            top: vertical.clone(),
            bottom: vertical,
            left: horizontal.clone(),
            right: horizontal,
        }
    }

    pub fn with_top(mut self, top: T) -> Self {
        self.top = top;
        self
    }

    pub fn with_bottom(mut self, bottom: T) -> Self {
        self.bottom = bottom;
        self
    }

    pub fn with_left(mut self, left: T) -> Self {
        self.left = left;
        self
    }

    pub fn with_right(mut self, right: T) -> Self {
        self.right = right;
        self
    }
}

impl<T: Default> Default for DirectionInfo<T> {
    fn default() -> Self {
        Self {
            top: Default::default(),
            bottom: Default::default(),
            left: Default::default(),
            right: Default::default(),
        }
    }
}

pub trait WrapIntoUiElement {
    fn wrap_ui(self) -> UiElement;
}


#[derive(Debug)]
pub struct Box {
    margin: DirectionInfo<f32>,
    border: DirectionInfo<f32>,
    border_color: glm::U8Vec4,
    padding: DirectionInfo<f32>,
    background_color: glm::U8Vec4,
    size: glm::Vec2,
    position: glm::Vec2,
    position_mode: PositionMode,
    children: Vec<UiElement>,
}

impl Box {
    pub fn new(position: glm::Vec2, size: glm::Vec2, background_color: glm::U8Vec4,) -> Self {
        Self {
            margin: Default::default(),
            border: Default::default(),
            border_color: Default::default(),
            padding: Default::default(),
            background_color,
            size,
            position,
            position_mode: PositionMode::Center,
            children: vec![]
        }
    }

    pub fn with_margin(mut self, margin: DirectionInfo<f32>) -> Self {
        self.margin = margin;
        self
    }

    pub fn with_padding(mut self, padding: DirectionInfo<f32>) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_border(mut self, width: DirectionInfo<f32>, color: glm::U8Vec4) -> Self {
        self.border = width;
        self.border_color = color;
        self
    }

    pub fn with_position_mode(mut self, mode: PositionMode) -> Self {
        self.position_mode = mode;
        self
    }

    pub fn with_children(mut self, children: Vec<UiElement>) -> Self {
        self.children = children;
        self
    }
}

impl WrapIntoUiElement for Box {
    fn wrap_ui(self) -> UiElement {
        UiElement::Box(self)
    }
}

#[derive(Debug)]
pub struct Text {
    content: String,
    line_height: f32,
    position: glm::Vec2,
    position_mode: PositionMode,
    color: glm::U8Vec4,
}

fn space_evenly(amount: i32, width: f32, total_width: f32) -> impl Iterator<Item = f32> {
    let amount_f = amount as f32;
    let space_remaining = total_width - amount_f * width;
    let gap = space_remaining / (amount_f - 1.0);
    let step = width + gap;
    (0..amount).map(move |i| (i as f32) * step)
}

pub fn first_test() -> UiElement {
    let width = 0.08;
    let children: Vec<UiElement> =
        space_evenly(10, width, 0.88)
        .map(|position|
            Box::new(
                glm::vec2(position, 0.0),
                glm::vec2(width, width),
                glm::vec4(200, 100, 30, 255)
            )
            .with_position_mode(PositionMode::TopLeft)
            .wrap_ui()
        )
        .collect();

    let root = Box::new(
        glm::vec2(0.5, -0.5),
        glm::vec2(1.0, 0.2),
        glm::vec4(20, 20, 40, 255)
    )
    .with_margin(DirectionInfo::all(0.02))
    .with_border(DirectionInfo::all(0.02), glm::vec4(200, 200, 30, 255))
    .with_padding(DirectionInfo::all(0.02))
    .with_children(children)
    .wrap_ui();

    root
}


