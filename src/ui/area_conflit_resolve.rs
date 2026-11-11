use crate::ui::ListeningArea;
use nalgebra_glm as glm;
use winit::window::CursorIcon;

pub fn resolve_conflicts(areas: &mut [ListeningArea], mouse_position: glm::Vec2) -> (Vec<u64>, CursorIcon) {
    areas.sort_by(|a, b| b.depth.cmp(&a.depth));
    let areas = areas.iter()
        .filter(|area| area.bbox.is_point_inside(mouse_position));
    (
        areas.clone().map(|area| area.id).collect(),
        areas.last().map(|area| area.cursor).unwrap_or(CursorIcon::Default),
    )
}
