use crate::{ui::{multi_area_conflict_resolver::AreaConflict, simple_area::SimpleArea}};
use nalgebra_glm as glm;

#[derive(Debug)]
pub struct SimpleManager {
    simple_areas: Vec<SimpleArea>,
}

const EMPTY_ID: u32 = 0xffffffff;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AreaId {
    id: u32,
}

impl AreaId {
    const NONE: AreaId = Self::new(EMPTY_ID);
    const fn new(id: u32) -> Self {
        Self { id }
    }

    pub const fn is_some(&self) -> bool {
        self.id != EMPTY_ID
    }

    pub const fn is_none(&self) -> bool {
        self.id == EMPTY_ID
    }
}

impl Default for AreaId {
    fn default() -> Self {
        Self::NONE
    }
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
            .enumerate()
            .filter(|(_, area)| area.bounding_box.is_point_inside(ndc_mouse_position))
            .map(|(i, simple_area)| AreaConflict {
                id: AreaId::new(i as u32),
                depth: simple_area.depth,
                cursor: simple_area.cursor,
            })
            .collect();

        self.simple_areas.clear();
        conflicts
    }

    pub fn get(&self, id: AreaId) -> Option<&SimpleArea> {
        self.simple_areas.get(id.id as usize)
    }

    pub fn push(&mut self, simple_area: SimpleArea) -> AreaId {
        let id = self.simple_areas.len() as u32;
        self.simple_areas.push(simple_area);
        AreaId::new(id)
    }
}
