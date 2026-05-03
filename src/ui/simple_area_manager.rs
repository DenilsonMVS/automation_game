use crate::{ui::{multi_area_conflict_resolver::AreaConflict, simple_area::SimpleArea}};
use nalgebra_glm as glm;

#[derive(Debug)]
pub struct SimpleManager {
    simple_areas: Vec<SimpleArea>,
}

impl SimpleManager {
    pub fn new() -> Self {
        Self {
            simple_areas: vec![],
        }
    }

    pub(super) fn on_frame_start(
        &mut self,
        ndc_mouse_position: glm::Vec2,
    ) -> Vec<AreaConflict> {
        let conflicts = self.simple_areas
            .iter()
            .filter(|area| area.bounding_box.is_point_inside(ndc_mouse_position))
            .enumerate()
            .map(|(i, simple_area)| AreaConflict {
                id: i as u32,
                depth: simple_area.depth,
                cursor: simple_area.cursor,
            })
            .collect();

        self.simple_areas.clear();
        conflicts
    }

    pub fn get(&self, id: u32) -> Option<&SimpleArea> {
        self.simple_areas.get(id as usize)
    }

    pub fn push(&mut self, simple_area: SimpleArea) -> u32 {
        let id = self.simple_areas.len() as u32;
        self.simple_areas.push(simple_area);
        id
    }
}
