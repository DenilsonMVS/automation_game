use crate::{graphics::renderer::positioning::RenderBox, ui::{Button, Div, DragBox, UiElement}};
use glfw::Window;
use nalgebra_glm as glm;

#[derive(Debug, Default)]
struct ReturnData {
    grabbed_descendant: bool,
    child_render_box: RenderBox,
    hovered_button: bool,
}

struct UpdatePassContext<'a, C> {
    mouse_position: glm::Vec2,
    mouse_clicked: bool,
    mouse_released: bool,
    context: &'a mut C,
    previous_mouse_position: glm::Vec2,
}

impl RenderBox {
    fn is_point_inside(&self, point: glm::Vec2) -> bool {
        self.min.x <= point.x && self.max.x >= point.x &&
        self.min.y <= point.y && self.max.y >= point.y
    }
}

impl<C> UiElement<C> {
    pub fn update(
        &mut self,
        parent_render_div: &RenderBox,
        mouse_position: glm::Vec2,
        previous_mouse_position: glm::Vec2,
        mouse_clicked: bool,
        mouse_released: bool,
        window: &mut Window,
        context: &mut C,
    ) {
        let mut context = UpdatePassContext {
            mouse_position,
            mouse_clicked,
            mouse_released,
            context,
            previous_mouse_position,
        };
        let rtn = self.update_rec(parent_render_div, &mut context);
        
        // let cursor_hand = glfw::Cursor::standard(glfw::StandardCursor::Hand);
        // let cursor_arrow = glfw::Cursor::standard(glfw::StandardCursor::Arrow);

        // window.set_cursor(Some(if rtn.hovered_button {
        //     cursor_hand
        // } else {
        //     cursor_arrow
        // }));
    }

    fn update_div(ui_div: &mut Div<C>, parent_render_div: &RenderBox, context: &mut UpdatePassContext<C>) -> ReturnData {
        let outer_render_div = parent_render_div.sub_div(ui_div.position, ui_div.size);
        let margin_render_div = outer_render_div.cut(&ui_div.margin);
        let border_render_div = margin_render_div.cut(&ui_div.border);
        let padding_render_div = border_render_div.cut(&ui_div.padding);

        let mut rtn = ReturnData {
            grabbed_descendant: false,
            hovered_button: false,
            child_render_box: margin_render_div,
        };
        for child in ui_div.children.iter_mut() {
            let child_rtn = child.update_rec(&padding_render_div, context);
            rtn.grabbed_descendant |= child_rtn.grabbed_descendant;
            rtn.hovered_button |= child_rtn.hovered_button;
        }

        rtn
    }

    fn update_button(button: &mut Button<C>, parent_render_div: &RenderBox, context: &mut UpdatePassContext<C>) -> ReturnData {
        let div = &mut button.container;
        let mut rtn = Self::update_div(div, parent_render_div, context);
        
        button.is_hovered = rtn.child_render_box.is_point_inside(context.mouse_position);
        rtn.hovered_button |= button.is_hovered;

        if button.is_hovered {
            if context.mouse_clicked {
                (button.on_click)(&mut context.context);
            }
            if let Some(on_hover) = button.on_hover {
                on_hover(&mut context.context);
            }
        }

        rtn
    }

    fn update_drag_box(drag_box: &mut DragBox<C>, parent_render_div: &RenderBox, context: &mut UpdatePassContext<C>) -> ReturnData {
        let div = &mut drag_box.container;
        let mut rtn = Self::update_div(
            div,
            &parent_render_div.apply_offset(drag_box.offset),
            context,
        );

        if context.mouse_clicked {
            if !rtn.grabbed_descendant {
                drag_box.selected = true;
                rtn.grabbed_descendant = true;
            }
        } else if context.mouse_released {
            drag_box.selected = false;
        } else if drag_box.selected {
            let diff = context.mouse_position - context.previous_mouse_position;
            drag_box.offset += diff;
        }

        rtn
    }

    fn update_rec(&mut self, parent_render_div: &RenderBox, context: &mut UpdatePassContext<C>) -> ReturnData {
        match self {
            Self::Div(ui_div) =>
                Self::update_div(ui_div, parent_render_div, context),
            Self::Button(button) =>
                Self::update_button(button, parent_render_div, context),
            Self::DragBox(drag_box) =>
                Self::update_drag_box(drag_box, parent_render_div, context),
            _ => ReturnData {
                grabbed_descendant: false,
                hovered_button: false,
                child_render_box: Default::default(),
            },
        }
    }
}
