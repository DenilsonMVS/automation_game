use std::collections::HashMap;
use nalgebra_glm as glm;
use crate::{game::logic::{GameState, fabricator::Fabricator, robot::Robot, world::{CHUNK_SIZE, GridPosition}}, graphics::renderer::positioning::{PositionMode, RenderBox}, kd_tree::KdTree};

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

fn apply_static_x(res: &mut RenderBox, locked: &RenderBox, mov: &RenderBox, prev: &RenderBox) {
    let width = res.max.x - res.min.x;
    if mov.min.x > prev.min.x {
        res.max.x = locked.min.x;
        res.min.x = res.max.x - width;
    } else {
        res.min.x = locked.max.x;
        res.max.x = res.min.x + width;
    }
}

fn apply_static_y(res: &mut RenderBox, locked: &RenderBox, mov: &RenderBox, prev: &RenderBox) {
    let height = res.max.y - res.min.y;
    if mov.min.y > prev.min.y {
        res.max.y = locked.min.y;
        res.min.y = res.max.y - height;
    } else {
        res.min.y = locked.max.y;
        res.max.y = res.min.y + height;
    }
}

fn resolve_dynamic_x(r1: &mut RenderBox, r2: &mut RenderBox, p1: &RenderBox, p2: &RenderBox, overlap: f32) {
    let sep = overlap * 0.5;
    if p1.min.x < p2.min.x {
        r1.min.x -= sep; r1.max.x -= sep;
        r2.min.x += sep; r2.max.x += sep;
    } else {
        r1.min.x += sep; r1.max.x += sep;
        r2.min.x -= sep; r2.max.x -= sep;
    }
}

fn resolve_dynamic_y(r1: &mut RenderBox, r2: &mut RenderBox, p1: &RenderBox, p2: &RenderBox, overlap: f32) {
    let sep = overlap * 0.5;
    if p1.min.y < p2.min.y {
        r1.min.y -= sep; r1.max.y -= sep;
        r2.min.y += sep; r2.max.y += sep;
    } else {
        r1.min.y += sep; r1.max.y += sep;
        r2.min.y -= sep; r2.max.y -= sep;
    }
}

impl GridPosition {
    fn get_bbox(&self, position: glm::Vec2) -> Option<RenderBox> {
        match self {
            Self::Obstacle => Some(RenderBox::new_simple(position, PositionMode::Center, glm::vec2(1.0, 1.0))),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
struct ChunkElements {
    dynamic_global_ids: Vec<u32>,
    dynamic_updated_position: Vec<glm::Vec2>,
    dynamic_prev_position: Vec<glm::Vec2>,

    dynamic_kd_tree_positions: Vec<glm::Vec2>,
    dynamic_kd_tree_update_idx: Vec<u32>,

    static_kd_tree_positions: Vec<glm::Vec2>,
    static_kd_tree_id: Vec<StaticId>,
}

#[derive(Debug, Clone)]
enum StaticId {
    Fabricator,
    Ghost,
}

impl StaticId {
    fn get_bbox(&self, position: glm::Vec2) -> RenderBox {
        match self {
            Self::Fabricator => Fabricator::get_bbox(position),
            Self::Ghost => Robot::get_render_box(position),
        }
    }
}

fn get_bbox_from_dynamic_id(id: u32, position: glm::Vec2) -> RenderBox {
    match id >> 24 {
        _ => Robot::get_render_box(position),
    }
}

impl ChunkElements {
    const ROBOT_ID: u32 = 0;

    fn pack_robot(id: usize) -> u32 {
        (Self::ROBOT_ID << 24) | (id as u32 & 0xffffff)
    }

    fn push_robot(&mut self, position: glm::Vec2, prev_position: glm::Vec2, id: usize) {
        self.dynamic_global_ids.push(Self::pack_robot(id));
        self.dynamic_updated_position.push(position);
        self.dynamic_prev_position.push(prev_position);

        let idx = self.dynamic_kd_tree_positions.len() as u32;
        self.dynamic_kd_tree_positions.push(position);
        self.dynamic_kd_tree_update_idx.push(idx);
    }

    fn push_fabricator(&mut self, position: glm::Vec2) {
        self.static_kd_tree_positions.push(position);
        self.static_kd_tree_id.push(StaticId::Fabricator);
    }

    fn push_ghost_robot(&mut self, position: glm::Vec2) {
        self.static_kd_tree_positions.push(position);
        self.static_kd_tree_id.push(StaticId::Ghost);
    }
}

impl GameState {
    fn separate_chunks(&self) -> HashMap<glm::IVec2, ChunkElements> {
        let mut chunk_elements: HashMap<glm::IVec2, ChunkElements> = HashMap::new();

        for (id, robot) in self.robots.iter_enumerate_unordered() {
            let position = robot.movement_module.get_position();
            let prev_position = robot.movement_module.get_prev_position();
            let chunk_id = position.map(|x| (x.floor() as i32).div_euclid(CHUNK_SIZE as i32));
            
            chunk_elements.entry(chunk_id).or_default().push_robot(position, prev_position, id);
        }

        for fabricator in self.fabricators.iter() {
            let position = fabricator.position;
            let chunk_id = position.map(|x| x.div_euclid(CHUNK_SIZE as i32));

            let entry = chunk_elements.entry(chunk_id).or_default();
            entry.push_fabricator(position.map(|x| x as f32));

            if let Some(pos) = fabricator.get_ghost_position() {
                entry.push_ghost_robot(pos);
            }
        }

        for (chunk_id, _) in self.world.chunks.iter() {
            chunk_elements.entry(*chunk_id).or_default();
        }

        chunk_elements
    }

    pub(super) fn handle_collisions(&mut self) {
        let chunk_elements = self.separate_chunks();

        let mut dyn_dyn_collisions = vec![];
        let mut dyn_search_ids = vec![];
        let mut static_search_ids = vec![];
        for (chunk_id, els) in chunk_elements {
            let Some(chunk) = self.world.get_chunks().get(&chunk_id) else { continue };
            let ChunkElements {
                dynamic_global_ids,
                mut dynamic_updated_position,
                dynamic_prev_position,

                dynamic_kd_tree_positions,
                dynamic_kd_tree_update_idx,

                static_kd_tree_positions,
                static_kd_tree_id,
            } = els;

            let dynamic_kd_tree = KdTree::new(dynamic_kd_tree_positions, dynamic_kd_tree_update_idx);
            let static_kd_tree = KdTree::new(static_kd_tree_positions, static_kd_tree_id);

            dyn_dyn_collisions.clear();
            for (i, (id, (position, prev_position))) in
                dynamic_global_ids
                .iter()
                .zip(dynamic_updated_position.iter_mut()
                .zip(dynamic_prev_position.iter()))
                .enumerate()
            {
                static_search_ids.clear();
                static_kd_tree.find_in_range(*position, 1.1, &mut static_search_ids);
                for (near_id, near_pos) in static_search_ids.iter() {
                    let bbox = near_id.get_bbox(*near_pos);
                    let (cur_bbox, prev_bbox) = (
                        get_bbox_from_dynamic_id(*id, *position),
                        get_bbox_from_dynamic_id(*id, *prev_position),
                    );
                    let resolved = resolve_static_aabb(&bbox, &cur_bbox, &prev_bbox);
                    *position = (resolved.max + resolved.min) * 0.5;
                }

                dyn_search_ids.clear();
                dynamic_kd_tree.find_in_range(*position, 1.1, &mut dyn_search_ids);
                for (near_idx, _) in dyn_search_ids.iter() {
                    if (i as u32) < *near_idx {
                        dyn_dyn_collisions.push((i as u32, *near_idx));
                    }
                }

                let rounded_pos = position.map(|x| x.round() as i32);
                let chunk_pos = rounded_pos.map(|x| (x.rem_euclid(CHUNK_SIZE as i32)) as u8);
                
                let offsets: [glm::U8Vec2; _] = [
                    glm::vec2(255,   1), glm::vec2(0,   1), glm::vec2(1,   1),
                    glm::vec2(255,   0), glm::vec2(0,   0), glm::vec2(1,   0),
                    glm::vec2(255, 255), glm::vec2(0, 255), glm::vec2(1, 255),
                ];

                for offset in offsets {
                    let check_pos = glm::vec2(
                        chunk_pos.x.wrapping_add(offset.x),
                        chunk_pos.y.wrapping_add(offset.y),
                    );
                    let quad_pos = (chunk_id * (CHUNK_SIZE as i32) + check_pos.map(|x| x as i32))
                        .map(|x| x as f32);

                    if let Some(bbox) = chunk.get_grid()[check_pos.y as usize][check_pos.x as usize].get_bbox(quad_pos) {
                        let (cur_bbox, prev_bbox) = (
                            get_bbox_from_dynamic_id(*id, *position),
                            get_bbox_from_dynamic_id(*id, *prev_position),
                        );
                        let resolved = resolve_static_aabb(&bbox, &cur_bbox, &prev_bbox);
                        *position = (resolved.max + resolved.min) * 0.5;
                    }
                }
            }

            for (idx1, idx2) in dyn_dyn_collisions.iter() {
                let (p1, pp1) = (dynamic_updated_position[*idx1 as usize], dynamic_prev_position[*idx1 as usize]);
                let (p2, pp2) = (dynamic_updated_position[*idx2 as usize], dynamic_prev_position[*idx2 as usize]);
                let (bb1, pbb1) = (Robot::get_render_box(p1), Robot::get_render_box(pp1));
                let (bb2, pbb2) = (Robot::get_render_box(p2), Robot::get_render_box(pp2));

                let (r1, r2) = resolve_dynamic_aabb(&bb1, &pbb1, &bb2, &pbb2);

                dynamic_updated_position[*idx1 as usize] = (r1.min + r1.max) * 0.5;
                dynamic_updated_position[*idx2 as usize] = (r2.min + r2.max) * 0.5;
            }

            for (id, pos) in dynamic_global_ids.iter().zip(dynamic_updated_position) {
                match id >> 24 {
                    ChunkElements::ROBOT_ID => self.robots[(id & 0xffffff) as usize].movement_module.set_position(pos),
                    _ => unreachable!(),
                }
            }
        }
    }
}
