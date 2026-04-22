
use nalgebra_glm as glm;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    position: glm::Vec2,
    zoom: f32,
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

    pub fn get_position(&self) -> glm::Vec2 {
        self.position
    }

    pub fn get_zoom(&self) -> f32 {
        self.zoom
    }

    pub fn ndc_to_world_pos(&self, ndc_pos: glm::Vec2) -> glm::Vec2 {
        ndc_pos / self.zoom + self.position
    }

    pub fn world_to_ndc_pos(&self, world_pos: glm::Vec2) -> glm::Vec2 {
        (world_pos - self.position) * self.zoom
    }
}
