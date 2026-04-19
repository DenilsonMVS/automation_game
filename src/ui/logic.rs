use crate::{graphics::renderer::positioning::RenderBox, ui::{Button, Div, UiElement}};
use glfw::Window;
use nalgebra_glm as glm;

struct UpdatePassContext<'a, C> {
    mouse_position: glm::Vec2,
    hovered_on_button: bool,
    mouse_clicked: bool,
    context: &'a mut C,
}

impl RenderBox {
    fn is_point_inside(&self, point: glm::Vec2) -> bool {
        self.min.x <= point.x && self.max.x >= point.x &&
        self.min.y <= point.y && self.max.y >= point.y
    }
}

impl<C> UiElement<C> {
    pub fn update(&mut self, parent_render_div: &RenderBox, mouse_position: glm::Vec2, mouse_clicked: bool, window: &mut Window, context: &mut C) {
        let mut context = UpdatePassContext { mouse_position, hovered_on_button: false, mouse_clicked, context };
        self.update_rec(parent_render_div, &mut context);
        
        let cursor_hand = glfw::Cursor::standard(glfw::StandardCursor::Hand);
        let cursor_arrow = glfw::Cursor::standard(glfw::StandardCursor::Arrow);

        window.set_cursor(Some(if context.hovered_on_button {
            cursor_hand
        } else {
            cursor_arrow
        }));
    }

    fn update_div(ui_div: &mut Div<C>, parent_render_div: &RenderBox, context: &mut UpdatePassContext<C>) -> RenderBox {
        let outer_render_div = parent_render_div.sub_div(ui_div.position, ui_div.size);
        let margin_render_div = outer_render_div.cut(&ui_div.margin);
        let border_render_div = margin_render_div.cut(&ui_div.border);
        let padding_render_div = border_render_div.cut(&ui_div.padding);

        for child in ui_div.children.iter_mut() {
            child.update_rec(&padding_render_div, context);
        }

        margin_render_div
    }

    fn update_button(button: &mut Button<C>, parent_render_div: &RenderBox, context: &mut UpdatePassContext<C>) {
        let div = &mut button.container;
        let rendered_box = Self::update_div(div, parent_render_div, context);
        button.is_hovered = rendered_box.is_point_inside(context.mouse_position);
        context.hovered_on_button |= button.is_hovered;
        if button.is_hovered {
            if context.mouse_clicked {
                (button.on_click)(&mut context.context);
            }
            if let Some(on_hover) = button.on_hover {
                on_hover(&mut context.context);
            }
        }
    }

    fn update_rec(&mut self, parent_render_div: &RenderBox, context: &mut UpdatePassContext<C>) {
        match self {
            Self::Div(ui_div) => {
                Self::update_div(ui_div, parent_render_div, context); }
            Self::Button(button) =>
                Self::update_button(button, parent_render_div, context),
            _ => {},
        }
    }
}
