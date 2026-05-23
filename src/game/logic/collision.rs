use nalgebra_glm as glm;
use crate::{game::logic::{fabricator::Fabricator, robot::Robot, world::{Chunk, ChunkPosition, GridPosition}}, graphics::renderer::positioning::{PositionMode, RenderBox}, kd_tree::KdTree};

fn resolve_static_aabb(locked: &RenderBox, movable: &RenderBox, prev_movable: &RenderBox) -> RenderBox {
    let mut resolved = *movable;

    if !movable.intersects(locked) {
        return resolved;
    }
    let was_overlapping_x = prev_movable.max.x > locked.min.x && prev_movable.min.x < locked.max.x;
    let was_overlapping_y = prev_movable.max.y > locked.min.y && prev_movable.min.y < locked.max.y;

    if was_overlapping_x && !was_overlapping_y {
        apply_static_y(&mut resolved, locked, movable, prev_movable);
    } else if was_overlapping_y && !was_overlapping_x {
        apply_static_x(&mut resolved, locked, movable, prev_movable);
    } else {
        let overlap_x = (movable.max.x.min(locked.max.x) - movable.min.x.max(locked.min.x)).max(0.0);
        let overlap_y = (movable.max.y.min(locked.max.y) - movable.min.y.max(locked.min.y)).max(0.0);

        if overlap_x < overlap_y {
            apply_static_x(&mut resolved, locked, movable, prev_movable);
        } else {
            apply_static_y(&mut resolved, locked, movable, prev_movable);
        }
    }

    resolved
}

fn resolve_dynamic_aabb(
    bb1: &RenderBox, 
    prev_bb1: &RenderBox, 
    bb2: &RenderBox, 
    prev_bb2: &RenderBox
) -> (RenderBox, RenderBox) {
    let mut res1 = *bb1;
    let mut res2 = *bb2;

    if !bb1.intersects(bb2) {
        return (res1, res2);
    }

    let was_overlapping_x = prev_bb1.max.x > prev_bb2.min.x && prev_bb1.min.x < prev_bb2.max.x;
    let was_overlapping_y = prev_bb1.max.y > prev_bb2.min.y && prev_bb1.min.y < prev_bb2.max.y;

    if was_overlapping_x && !was_overlapping_y {
        let oy = (res1.max.y.min(res2.max.y) - res1.min.y.max(res2.min.y)).max(0.0);
        resolve_dynamic_y(&mut res1, &mut res2, prev_bb1, prev_bb2, oy);
    } else if was_overlapping_y && !was_overlapping_x {
        let ox = (res1.max.x.min(res2.max.x) - res1.min.x.max(res2.min.x)).max(0.0);
        resolve_dynamic_x(&mut res1, &mut res2, prev_bb1, prev_bb2, ox);
    } else {
        let ox = (res1.max.x.min(res2.max.x) - res1.min.x.max(res2.min.x)).max(0.0);
        let oy = (res1.max.y.min(res2.max.y) - res1.min.y.max(res2.min.y)).max(0.0);

        if ox < oy {
            resolve_dynamic_x(&mut res1, &mut res2, prev_bb1, prev_bb2, ox);
        } else {
            resolve_dynamic_y(&mut res1, &mut res2, prev_bb1, prev_bb2, oy);
        }
    }

    (res1, res2)
}

fn apply_static_x(res: &mut RenderBox, locked: &RenderBox, mov: &RenderBox, _prev: &RenderBox) {
    let width = res.max.x - res.min.x;
    
    let locked_center_x = (locked.min.x + locked.max.x) * 0.5;
    let mov_center_x = (mov.min.x + mov.max.x) * 0.5;

    if mov_center_x > locked_center_x {
        res.min.x = locked.max.x;
        res.max.x = res.min.x + width;
    } else {
        res.max.x = locked.min.x;
        res.min.x = res.max.x - width;
    }
}

fn apply_static_y(res: &mut RenderBox, locked: &RenderBox, mov: &RenderBox, _prev: &RenderBox) {
    let height = res.max.y - res.min.y;
    
    let locked_center_y = (locked.min.y + locked.max.y) * 0.5;
    let mov_center_y = (mov.min.y + mov.max.y) * 0.5;

    if mov_center_y > locked_center_y {
        res.min.y = locked.max.y;
        res.max.y = res.min.y + height;
    } else {
        res.max.y = locked.min.y;
        res.min.y = res.max.y - height;
    }
}

fn resolve_dynamic_x(r1: &mut RenderBox, r2: &mut RenderBox, p1: &RenderBox, p2: &RenderBox, overlap: f32) {
    let sep = overlap * 0.5;
    
    let c1 = (p1.min.x + p1.max.x) * 0.5;
    let c2 = (p2.min.x + p2.max.x) * 0.5;

    if c1 < c2 {
        r1.min.x -= sep; r1.max.x -= sep;
        r2.min.x += sep; r2.max.x += sep;
    } else {
        r1.min.x += sep; r1.max.x += sep;
        r2.min.x -= sep; r2.max.x -= sep;
    }
}

fn resolve_dynamic_y(r1: &mut RenderBox, r2: &mut RenderBox, p1: &RenderBox, p2: &RenderBox, overlap: f32) {
    let sep = overlap * 0.5;
    
    let c1 = (p1.min.y + p1.max.y) * 0.5;
    let c2 = (p2.min.y + p2.max.y) * 0.5;

    if c1 < c2 {
        r1.min.y -= sep; r1.max.y -= sep;
        r2.min.y += sep; r2.max.y += sep;
    } else {
        r1.min.y += sep; r1.max.y += sep;
        r2.min.y -= sep; r2.max.y -= sep;
    }
}

impl GridPosition {
    fn get_bbox(&self, position: ChunkPosition<f32>) -> Option<RenderBox> {
        match self {
            Self::Obstacle => Some(RenderBox::new_simple(position.position, PositionMode::Center, glm::vec2(1.0, 1.0))),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum StaticId {
    Fabricator,
    Ghost,
}

impl StaticId {
    fn get_bbox(&self, position: ChunkPosition<f32>) -> RenderBox {
        match self {
            Self::Fabricator => Fabricator::get_bbox(position.position),
            Self::Ghost => Robot::get_render_box(position.position),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct DynamicId {
    value: u32,
}

impl DynamicId {
    const ROBOT_ID: u32 = 0;

    fn new_robot(id: usize) -> Self {
        Self { value: (Self::ROBOT_ID << 24) | (id as u32 & 0xffffff) }
    }

    fn get_bbox(&self, chunk: &Chunk) -> Option<(RenderBox, RenderBox)> {
        match self.value >> 24 {
            DynamicId::ROBOT_ID => {
                let robot = chunk.robots.get(self.value as usize & 0xffffff)?;
                let cur_pos = robot.movement_module.get_position();
                let prev_pos = robot.movement_module.get_prev_position();
                Some((
                    Robot::get_render_box(cur_pos),
                    Robot::get_render_box(prev_pos),
                ))
            },
            _ => None,
        }
    }

    fn on_resolved(&self, chunk: &mut Chunk, bbox: &RenderBox) -> Option<()> {
        match self.value >> 24 {
            DynamicId::ROBOT_ID => {
                let robot = chunk.robots.get_mut(self.value as usize & 0xffffff)?;
                robot.movement_module.set_position((bbox.min + bbox.max) * 0.5);
                Some(())
            },
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
struct ChunkElements {
    dynamic_kd_tree_positions: Vec<ChunkPosition<f32>>,
    dynamic_kd_tree_id: Vec<DynamicId>,

    static_kd_tree_positions: Vec<ChunkPosition<f32>>,
    static_kd_tree_id: Vec<StaticId>,
}

impl ChunkElements {
    fn push_robot(&mut self, position: ChunkPosition<f32>, id: usize) {
        self.dynamic_kd_tree_positions.push(position);
        self.dynamic_kd_tree_id.push(DynamicId::new_robot(id));
    }

    fn push_fabricator(&mut self, position: ChunkPosition<f32>) {
        self.static_kd_tree_positions.push(position);
        self.static_kd_tree_id.push(StaticId::Fabricator);
    }

    fn push_ghost_robot(&mut self, position: ChunkPosition<f32>) {
        self.static_kd_tree_positions.push(position);
        self.static_kd_tree_id.push(StaticId::Ghost);
    }
}

impl Chunk {
    fn get_elements(&self) -> ChunkElements {
        let mut elements = ChunkElements::default();

        for (id, robot) in self.robots.iter_enumerate_unordered() {
            let position = robot.movement_module.get_position();
            elements.push_robot(ChunkPosition::new(position.x, position.y), id);
        }

        for fabricator in self.fabricators.iter() {
            let position = fabricator.position.map(|x| x as f32);
            elements.push_fabricator(ChunkPosition::new(position.x, position.y));

            if let Some(position) = fabricator.get_ghost_position() {
                elements.push_ghost_robot(ChunkPosition::new(position.x, position.y));
            }
        }

        elements
    }

    pub(super) fn handle_collisions(&mut self) {
        let ChunkElements {
            dynamic_kd_tree_id,
            dynamic_kd_tree_positions,
            static_kd_tree_id,
            static_kd_tree_positions,
        } = self.get_elements();

        let mut dyn_dyn_collisions = vec![];
        let mut dyn_search_ids = vec![];
        let mut static_search_ids = vec![];

        let dynamic_kd_tree = KdTree::new(dynamic_kd_tree_positions, dynamic_kd_tree_id);
        let static_kd_tree = KdTree::new(static_kd_tree_positions, static_kd_tree_id);

        for (id, pos) in dynamic_kd_tree.iter() {

            static_search_ids.clear();
            static_kd_tree.find_in_range(pos, 1.5, &mut static_search_ids);
            for (near_id, near_pos) in static_search_ids.iter() {
                let bbox = near_id.get_bbox(*near_pos);
                let Some((cur_bbox, prev_bbox)) = id.get_bbox(self) else { continue };
                let resolved = resolve_static_aabb(&bbox, &cur_bbox, &prev_bbox);
                id.on_resolved(self, &resolved);
            }

            dyn_search_ids.clear();
            dynamic_kd_tree.find_in_range(pos, 1.5, &mut dyn_search_ids);
            for (near_idx, _) in dyn_search_ids.iter() {
                if near_idx.value < id.value {
                    dyn_dyn_collisions.push((id, *near_idx));
                }
            }

            let rounded_pos = pos.map(|x| x.round() as u8);
            let offsets: [glm::U8Vec2; _] = [
                glm::vec2(255,   1), glm::vec2(0,   1), glm::vec2(1,   1),
                glm::vec2(255,   0), glm::vec2(0,   0), glm::vec2(1,   0),
                glm::vec2(255, 255), glm::vec2(0, 255), glm::vec2(1, 255),
            ];

            for offset in offsets {
                let check_pos = ChunkPosition::new(
                    rounded_pos.x.wrapping_add(offset.x),
                    rounded_pos.y.wrapping_add(offset.y),
                );

                if let Some(bbox) = self.get_grid()[check_pos.y as usize][check_pos.x as usize].get_bbox(check_pos.map(|x| x as f32)) {
                    let Some((cur_bbox, prev_bbox)) = id.get_bbox(self) else { continue };
                    let resolved = resolve_static_aabb(&bbox, &cur_bbox, &prev_bbox);
                    id.on_resolved(self, &resolved);
                }
            }
        }

        for (id1, id2) in dyn_dyn_collisions.iter() {
            let Some((bb1, pbb1)) = id1.get_bbox(self) else { continue };
            let Some((bb2, pbb2)) = id2.get_bbox(self) else { continue };

            let (r1, r2) = resolve_dynamic_aabb(&bb1, &pbb1, &bb2, &pbb2);

            id1.on_resolved(self, &r1);
            id2.on_resolved(self, &r2);
        }
    }
}
