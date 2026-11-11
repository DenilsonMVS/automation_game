
use nalgebra_glm as glm;

use crate::graphics::renderer::positioning::RenderBox;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub position: glm::Vec2,
    pub zoom: f32,
}

impl Camera {
    pub fn new(position: glm::Vec2, zoom: f32) -> Self {
        Self {
            position,
            zoom,
        }
    }

    pub fn get_view_matrix(&self) -> glm::Mat4 {
        let identity = glm::identity();
        let scaled = glm::scale(&identity, &glm::vec3(self.zoom, self.zoom, 1.0));
        let view = glm::translate(&scaled, &glm::vec3(-self.position.x, -self.position.y, 0.0));
        return view;
    }

    pub fn get_vp_matrix(&self, projection_matrix: &glm::Mat4) -> glm::Mat4 {
        projection_matrix * self.get_view_matrix()
    }

    pub fn ndc_to_world_pos(&self, ndc_pos: glm::Vec2) -> glm::Vec2 {
        ndc_pos / self.zoom + self.position
    }

    pub fn ndc_to_world_box(&self, ndc_box: &RenderBox) -> RenderBox {
        RenderBox {
            min: self.ndc_to_world_pos(ndc_box.min),
            max: self.ndc_to_world_pos(ndc_box.max),
        }
    }

    pub fn world_to_ndc_pos(&self, world_pos: glm::Vec2) -> glm::Vec2 {
        (world_pos - self.position) * self.zoom
    }

    pub fn world_to_ndc_box(&self, ndc_box: &RenderBox) -> RenderBox {
        RenderBox {
            min: self.world_to_ndc_pos(ndc_box.min),
            max: self.world_to_ndc_pos(ndc_box.max),
        }
    }
}
