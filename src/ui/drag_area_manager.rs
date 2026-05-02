use crate::{stable_index_vector::StableIndexVector, ui::{AreaManager, multi_area_conflict_resolver::Area, simple_area::SimpleArea}};
use nalgebra_glm as glm;

#[derive(Debug)]
pub struct DragAreaManager {
    drag_areas: StableIndexVector<SimpleArea>,
    pub grabbed_id: Option<usize>,
}

impl DragAreaManager {
    pub fn new() -> Self {
        Self { 
            drag_areas: StableIndexVector::new(),
            grabbed_id: None 
        }
    }

    pub(super) fn on_frame_start(
        &mut self,
        areas_containing_mouse: &mut Vec<Area>,
        ndc_mouse_position: glm::Vec2,
    ) {
        for (id, area) in self.drag_areas.iter_enumerate_unordered_mut() {
            area.alive = false;
            if area.bounding_box.is_point_inside(ndc_mouse_position) {
                areas_containing_mouse.push(Area::DragArea(id, area.depth));
            }
        }
    } 

    pub(super) fn on_frame_end(&mut self) {
        let dead_ids: Vec<usize> = self.drag_areas
            .iter_enumerate_unordered()
            .filter_map(|(id, area)|
                if !area.alive { Some(id) } else { None }
            ).collect();
        
        for dead_id in dead_ids.iter() {
            self.drag_areas.remove(*dead_id);
        }
    }
}

impl AreaManager for DragAreaManager {
    fn push(&mut self, drag_area: SimpleArea) -> usize {
        self.drag_areas.push(drag_area)
    }

    fn update(&mut self, id: usize, drag_area: SimpleArea) {
        if let Some(area) = self.drag_areas.get_mut(id) {
            *area = drag_area;
        }
    }

    fn keep_alive(&mut self, id: usize) {
        if let Some(area) = self.drag_areas.get_mut(id) {
            area.alive = true;
        }
    }

    fn get(&self, id: usize) -> Option<&SimpleArea> {
        self.drag_areas.get(id)
    }
}
