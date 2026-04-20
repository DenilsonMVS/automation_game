use crate::{graphics::{input_state::InputState, renderer::positioning::RenderBox}, ui::{Button, Div, DragBox, UiElement, render::ui_coord_inversion}};
use nalgebra_glm as glm;
use winit::window::{CursorIcon, Window};

#[derive(Debug, Default)]
struct ReturnData {
    hovered_grab_box: bool,
    grabbed_descendant: bool,
    child_render_box: RenderBox,
    hovered_button: bool,
}

struct UiPassContext<'a, C> {
    input_state: &'a InputState,
    context: &'a mut C,
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
        input_state: &InputState,
        context: &mut C,
        window: &Window,
    ) {
        let mut pass_context = UiPassContext { input_state, context };
        let rtn = self.update_rec(&input_state.get_root_ui_render_box(), &mut pass_context);
        
        let icon = if rtn.hovered_button {
            CursorIcon::Pointer
        } else if rtn.grabbed_descendant {
            CursorIcon::Grabbing
        } else if rtn.hovered_grab_box {
            CursorIcon::Grab
        } else {
            CursorIcon::Default
        };

        window.set_cursor(icon);
    }

    fn update_div(ui_div: &mut Div<C>, parent_render_div: &RenderBox, context: &mut UiPassContext<C>) -> ReturnData {
        let outer_render_div = parent_render_div.sub_div(ui_div.position, ui_div.size);
        let margin_render_div = outer_render_div.cut(&ui_div.margin);
        let border_render_div = margin_render_div.cut(&ui_div.border);
        let padding_render_div = border_render_div.cut(&ui_div.padding);

        let mut rtn = ReturnData {
            grabbed_descendant: false,
            hovered_button: false,
            hovered_grab_box: false,
            child_render_box: margin_render_div,
        };
        for child in ui_div.children.iter_mut() {
            let child_rtn = child.update_rec(&padding_render_div, context);
            rtn.grabbed_descendant |= child_rtn.grabbed_descendant;
            rtn.hovered_button |= child_rtn.hovered_button;
            rtn.hovered_grab_box |= child_rtn.hovered_grab_box;
        }

        rtn
    }

    fn update_button(button: &mut Button<C>, parent_render_div: &RenderBox, context: &mut UiPassContext<C>) -> ReturnData {
        let div = &mut button.container;
        let mut rtn = Self::update_div(div, parent_render_div, context);
        
        button.is_hovered = rtn.child_render_box.is_point_inside(context.input_state.get_mouse_position());
        rtn.hovered_button |= button.is_hovered;

        if button.is_hovered {
            if context.input_state.mouse_down() {
                (button.on_click)(&mut context.context);
            }
            if let Some(on_hover) = button.on_hover {
                on_hover(&mut context.context);
            }
        }

        rtn
    }

    fn update_drag_box(drag_box: &mut DragBox<C>, parent_render_div: &RenderBox, context: &mut UiPassContext<C>) -> ReturnData {
        let div = &mut drag_box.container;
        let mut rtn = Self::update_div(
            div,
            &parent_render_div.apply_offset(drag_box.offset),
            context,
        );

        rtn.hovered_grab_box |= rtn.child_render_box.is_point_inside(context.input_state.get_mouse_position());

        if context.input_state.mouse_down() {
            if !rtn.grabbed_descendant && !rtn.hovered_button {
                drag_box.selected = true;
                rtn.grabbed_descendant = true;
            }
        } else if context.input_state.mouse_up() {
            if drag_box.selected {
                if let Some(on_release) = drag_box.on_release {
                    on_release(&mut context.context, ui_coord_inversion(drag_box.offset));
                }
                drag_box.selected = false;
            }
        } else if drag_box.selected {
            let diff = context.input_state.get_mouse_position() - context.input_state.get_previous_mouse_position();
            drag_box.offset += diff;
            rtn.grabbed_descendant = true;
        }

        rtn
    }

    fn update_rec(&mut self, parent_render_div: &RenderBox, context: &mut UiPassContext<C>) -> ReturnData {
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
                hovered_grab_box: false,
                child_render_box: Default::default(),
            },
        }
    }
}
