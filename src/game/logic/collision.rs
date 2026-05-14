use std::collections::HashMap;
use nalgebra_glm as glm;
use crate::{game::logic::{GameState, fabricator::Fabricator, robot::Robot, world::{CHUNK_SIZE, GridPosition}}, graphics::renderer::positioning::RenderBox, kd_tree::KdTree};

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

#[derive(Debug, Default)]
struct RobotPositions {
    ids: Vec<u32>,
    pos: Vec<glm::Vec2>,
    prev: Vec<glm::Vec2>,
}

#[derive(Debug, Default)]
struct ChunkElements {
    robot_positions: RobotPositions,
    positions: Vec<glm::Vec2>,
    ids: Vec<u32>,
}

impl ChunkElements {
    const OBSTACLE_ID: u32 = 0;
    const ROBOT_ID: u32 = 1;
    const FABRICATOR_ID: u32 = 2;
    const GHOST_ROBOT_ID: u32 = 3;

    fn pack_obstacle(id: usize) -> u32 {
        (Self::OBSTACLE_ID << 24) | (id as u32 & 0xffffff)
    }

    fn pack_robot(id: usize) -> u32 {
        (Self::ROBOT_ID << 24) | (id as u32 & 0xffffff)
    }

    fn pack_fabricator(id: usize) -> u32 {
        (Self::FABRICATOR_ID << 24) | (id as u32 & 0xffffff)
    }

    fn pack_ghost_robot(id: usize) -> u32 {
        (Self::GHOST_ROBOT_ID << 24) | (id as u32 & 0xffffff)
    }

    fn push(&mut self, position: glm::Vec2, id: u32) {
        self.positions.push(position);
        self.ids.push(id);
    }

    fn push_robot(&mut self, position: glm::Vec2, prev_position: glm::Vec2, id: usize) {
        let kdtree_id = self.robot_positions.ids.len();
        self.robot_positions.ids.push(id as u32);
        self.robot_positions.pos.push(position);
        self.robot_positions.prev.push(prev_position);
        self.push(position, Self::pack_robot(kdtree_id));
    }

    fn push_fabricator(&mut self, position: glm::Vec2, id: usize) {
        self.push(position, Self::pack_fabricator(id));
    }

    fn push_ghost_robot(&mut self, position: glm::Vec2, id: usize) {
        self.push(position, Self::pack_ghost_robot(id));
    }

    fn push_obstacle(&mut self, position: glm::Vec2, id: usize) {
        self.push(position, Self::pack_obstacle(id));
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
            entry.push_fabricator(position.map(|x| x as f32), 0);

            if let Some(pos) = fabricator.get_ghost_position() {
                entry.push_ghost_robot(pos, 0);
            }
        }

        for (chunk_id, chunk) in self.world.chunks.iter() {
            let chunk_pos = chunk_id.map(|x| (x * CHUNK_SIZE as i32) as f32);
            let entry = chunk_elements.entry(*chunk_id).or_default();
            for (i, line) in chunk.get_grid().iter().enumerate() {
                for (j, element) in line.iter().enumerate() {
                    if *element == GridPosition::Obstacle {
                        entry.push_obstacle(chunk_pos + glm::vec2(j as f32, i as f32), 0);
                    }
                }
            }
        }

        chunk_elements
    }

    pub(super) fn handle_collisions(&mut self) {
        let chunk_elements = self.separate_chunks();

        let mut robot_robot_collisions = vec![];
        let mut search_ids = vec![];
        for (_, els) in chunk_elements {
            let ChunkElements { positions, ids, mut robot_positions } = els;
            let kd_tree = KdTree::new(positions, ids);

            robot_robot_collisions.clear();
            for (i, (position, prev_position)) in robot_positions.pos.iter_mut().zip(robot_positions.prev.iter_mut()).enumerate() {
                let packed = ChunkElements::pack_robot(i);
            
                search_ids.clear();
                kd_tree.find_in_range(*position, 2.1, &mut search_ids);
            
                for (found_id, found_pos) in search_ids.iter() {
                    if packed == *found_id {
                        continue;
                    }

                    match found_id >> 24 {
                        ChunkElements::ROBOT_ID => {
                            if (found_id & 0xffffff) > (packed & 0xffffff) {
                                robot_robot_collisions.push((found_id & 0xffffff, packed & 0xffffff));
                            }
                        },
                        ChunkElements::FABRICATOR_ID | ChunkElements::OBSTACLE_ID => {
                            let fabricator_bbox = Fabricator::get_bbox(*found_pos);

                            let (cur_bbox, prev_bbox) = (
                                Robot::get_render_box(*position),
                                Robot::get_render_box(*prev_position),
                            );
                            let resolved = resolve_static_aabb(&fabricator_bbox, &cur_bbox, &prev_bbox);
                            *position = (resolved.max + resolved.min) * 0.5;
                        },
                        ChunkElements::GHOST_ROBOT_ID => {
                            let idx = found_id & 0xffffff;
                            let ghost_position = self.fabricators[idx as usize].get_possible_ghost_position();
                            let ghost_bbox = Robot::get_render_box(ghost_position);

                            let (cur_bbox, prev_bbox) = (
                                Robot::get_render_box(*position),
                                Robot::get_render_box(*prev_position),
                            );
                            let resolved = resolve_static_aabb(&ghost_bbox, &cur_bbox, &prev_bbox);
                            *position = (resolved.max + resolved.min) * 0.5;
                        },
                        _ => {},
                    }
                }
            }

            for (idx1, idx2) in robot_robot_collisions.iter() {
                let (p1, pp1) = (robot_positions.pos[*idx1 as usize], robot_positions.prev[*idx1 as usize]);
                let (p2, pp2) = (robot_positions.pos[*idx2 as usize], robot_positions.prev[*idx2 as usize]);
                let (bb1, pbb1) = (Robot::get_render_box(p1), Robot::get_render_box(pp1));
                let (bb2, pbb2) = (Robot::get_render_box(p2), Robot::get_render_box(pp2));

                let (r1, r2) = resolve_dynamic_aabb(&bb1, &pbb1, &bb2, &pbb2);

                robot_positions.pos[*idx1 as usize] = (r1.min + r1.max) * 0.5;
                robot_positions.pos[*idx2 as usize] = (r2.min + r2.max) * 0.5;
            }

            for (id, pos) in robot_positions.ids.iter().zip(robot_positions.pos.iter()) {
                self.robots[*id as usize].movement_module.set_position(*pos);
            }
        }
    }
}
