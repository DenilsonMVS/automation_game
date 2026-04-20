

use nalgebra_glm as glm;
use crate::graphics::renderer::fonts::TextAlign;

pub mod render;
pub mod logic;

#[derive(Debug)]
pub enum UiElement<C> {
    Div(Box<Div<C>>),
    Text(Box<Text>),
    Button(Box<Button<C>>),
    DragBox(Box<DragBox<C>>),
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

pub trait WrapIntoUiElement<C> {
    fn wrap_ui(self) -> UiElement<C>;
}


#[derive(Debug)]
pub struct Div<C> {
    margin: DirectionInfo<f32>,
    border: DirectionInfo<f32>,
    border_color: glm::U8Vec4,
    padding: DirectionInfo<f32>,
    background_color: glm::U8Vec4,
    size: glm::Vec2,
    position: glm::Vec2,
    children: Vec<UiElement<C>>,
}

impl<C> Div<C> {
    pub fn new(position: glm::Vec2, size: glm::Vec2, background_color: glm::U8Vec4,) -> Self {
        Self {
            margin: Default::default(),
            border: Default::default(),
            border_color: Default::default(),
            padding: Default::default(),
            background_color,
            size,
            position,
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

    pub fn with_children(mut self, children: Vec<UiElement<C>>) -> Self {
        self.children = children;
        self
    }
}

impl<C> WrapIntoUiElement<C> for Div<C> {
    fn wrap_ui(self) -> UiElement<C> {
        UiElement::Div(Box::new(self))
    }
}

#[derive(Debug)]
pub struct Button<C> {
    container: Div<C>,
    hover_color: glm::U8Vec4,
    hover_border_color: glm::U8Vec4,
    on_click: fn(&mut C),
    on_hover: Option<fn(&mut C)>,
    is_hovered: bool,
}

impl<C> Button<C> {
    pub fn new(container: Div<C>, on_click: fn(&mut C)) -> Self {
        Self {
            hover_color: container.background_color,
            hover_border_color: container.border_color,
            container: container,
            on_click,
            on_hover: None,
            is_hovered: false,
        }
    }

    pub fn with_hover_color(mut self, color: glm::U8Vec4) -> Self {
        self.hover_color = color;
        self
    }

    pub fn with_hover_border_color(mut self, color: glm::U8Vec4) -> Self {
        self.hover_border_color = color;
        self
    }

    pub fn with_on_hover(mut self, on_hover: fn(&mut C)) -> Self {
        self.on_hover = Some(on_hover);
        self
    }
}

impl<C> WrapIntoUiElement<C> for Button<C> {
    fn wrap_ui(self) -> UiElement<C> {
        UiElement::Button(Box::new(self))
    }
}

#[derive(Debug)]
pub struct DragBox<C> {
    container: Div<C>,
    offset: glm::Vec2,
    selected: bool,
    on_release: Option<fn (&mut C, glm::Vec2)>,
}

impl<C> DragBox<C> {
    pub fn new(container: Div<C>) -> Self {
        Self { container, offset: glm::vec2(0.0, 0.0), selected: false, on_release: None }
    }

    pub fn with_on_release(mut self, on_release: fn (&mut C, glm::Vec2)) -> Self {
        self.on_release = Some(on_release);
        self
    }
}

impl<C> WrapIntoUiElement<C> for DragBox<C> {
    fn wrap_ui(self) -> UiElement<C> {
        UiElement::DragBox(Box::new(self))
    }
}

#[derive(Debug)]
pub struct Text {
    content: String,
    line_height: f32,
    color: glm::U8Vec4,
    align: TextAlign,
}

impl Text {
    pub fn new(content: String, line_height: f32, color: glm::U8Vec4) -> Self {
        Self {
            content,
            line_height,
            color,
            align: TextAlign::Left,
        }
    }

    pub fn set_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }
}

impl<C> WrapIntoUiElement<C> for Text {
    fn wrap_ui(self) -> UiElement<C> {
        UiElement::Text(Box::new(self))
    }
}


fn space_evenly(amount: i32, width: f32, total_width: f32) -> impl Iterator<Item = f32> {
    let amount_f = amount as f32;
    let space_remaining = total_width - amount_f * width;
    let gap = space_remaining / (amount_f - 1.0);
    let step = width + gap;
    (0..amount).map(move |i| (i as f32) * step)
}

pub fn first_test() -> UiElement<()> {
    let width = 0.08;
    let children: Vec<UiElement<()>> =
        space_evenly(10, width, 0.88)
        .enumerate()
        .map(|(i, position)| {
            let div = Div::new(
                glm::vec2(position, 0.0),
                glm::vec2(width, width),
                glm::vec4(200, 100, 30, 150)
            )
            .with_padding(DirectionInfo::all(0.005))
            .with_children(vec![
                Text::new("A".to_owned(), 0.07, glm::vec4(20, 20, 40, 200))
                    .set_align(TextAlign::Center)
                    .wrap_ui(),
            ]);

            if i == 0 {
                Button::new(div, |_| println!("Click!"))
                    .with_hover_color(glm::vec4(220, 120, 50, 255))
                    .with_on_hover(|_| println!("yay"))
                    .wrap_ui()
            } else {
                div.wrap_ui()
            }
        })
        .collect();

    let root = DragBox::new(Div::new(
        glm::vec2(0.5, 0.5),
        glm::vec2(1.0, 0.2),
        glm::vec4(20, 20, 40, 255)
    )
    .with_margin(DirectionInfo::all(0.02))
    .with_border(DirectionInfo::all(0.02), glm::vec4(200, 200, 30, 255))
    .with_padding(DirectionInfo::all(0.02))
    .with_children(children))
    .with_on_release(|_, pos| println!("{:?}", pos))
    .wrap_ui();

    root
}
