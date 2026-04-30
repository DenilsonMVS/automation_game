use crate::{stable_index_vector::StableIndexVector, ui::{multi_area_conflict_resolver::Area, simple_area::SimpleArea}};
use nalgebra_glm as glm;

pub struct SimpleManager {
    simple_areas: StableIndexVector<SimpleArea>,
}

impl SimpleManager {
    pub fn new() -> Self {
        Self {
            simple_areas: StableIndexVector::new(),
        }
    }

    pub fn keep_alive(&mut self, id: usize) {
        if let Some(simple_area) = self.simple_areas.get_mut(id) {
            simple_area.alive = true;
        }
    }

    pub fn get_area(&self, id: usize) -> Option<&SimpleArea> {
        self.simple_areas.get(id)
    }

    pub fn push_simple_area(&mut self, simple_area: SimpleArea) -> usize {
        self.simple_areas.push(simple_area)
    }

    pub fn update_simple_area(&mut self, id: usize, simple_area: SimpleArea) {
        if let Some(area) = self.simple_areas.get_mut(id) {
            *area = simple_area;
        }
    }

    pub(super) fn on_frame_start(
        &mut self,
        areas_containing_mouse: &mut Vec<Area>,
        ndc_mouse_position: glm::Vec2,
    ) {
        for (id, simple_area) in self.simple_areas.iter_enumerate_unordered_mut() {
            simple_area.alive = false;
            if simple_area.bounding_box.is_point_inside(ndc_mouse_position) {
                areas_containing_mouse.push(Area::SimpleArea(id, simple_area.depth, simple_area.is_button));
            }
        }
    }

    pub(super) fn on_frame_end(&mut self) {
        let dead_ids: Vec<usize> = self.simple_areas
            .iter_enumerate_unordered()
            .filter_map(|(id, area)|
                if !area.alive { Some(id) } else { None }
            ).collect();
        
        for dead_id in dead_ids.iter() {
            self.simple_areas.remove(*dead_id);
        }
    }
}