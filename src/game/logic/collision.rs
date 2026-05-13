use crate::{game::logic::{GameState, robot::Robot}, graphics::renderer::positioning::RenderBox, kd_tree::KdTree};

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

impl Robot {
    fn get_collision_bboxes(&self) -> (RenderBox, RenderBox) {
        let pos = self.movement_module.get_position();
        let prev = self.movement_module.get_prev_position();
        (Self::get_render_box(pos), Self::get_render_box(prev))
    }

    fn update_collision(&mut self, bbox: &RenderBox) {
        let position = (bbox.min + bbox.max) * 0.5;
        self.movement_module.set_position(position);
    }
}

impl GameState {
    // const OBSTACLE_ID: u32 = 0;
    const ROBOT_ID: u32 = 1;
    const FABRICATOR_ID: u32 = 2;
    const GHOST_ROBOT_ID: u32 = 3;

    // fn pack_obstacle(id: usize) -> u32 {
    //     (Self::OBSTACLE_ID << 24) | (id as u32 & 0xffffff)
    // }

    fn pack_robot(id: usize) -> u32 {
        (Self::ROBOT_ID << 24) | (id as u32 & 0xffffff)
    }

    fn pack_fabricator(id: usize) -> u32 {
        (Self::FABRICATOR_ID << 24) | (id as u32 & 0xffffff)
    }

    fn pack_ghost_robot(id: usize) -> u32 {
        (Self::GHOST_ROBOT_ID << 24) | (id as u32 & 0xffffff)
    }

    pub(super) fn handle_collisions(&mut self) {
        let mut positions = vec![];
        let mut ids = vec![];

        for (id, robot) in self.robots.iter_enumerate_unordered() {
            positions.push(robot.movement_module.get_position());
            ids.push(Self::pack_robot(id));
        }

        for (id, fabricator) in self.fabricators.iter().enumerate() {
            positions.push(fabricator.position.map(|x| x as f32));
            ids.push(Self::pack_fabricator(id));
            if let Some(pos) = fabricator.get_ghost_position() {
                positions.push(pos);
                ids.push(Self::pack_ghost_robot(id));
            }
        }

        let kd_tree = KdTree::new(positions, ids);
        let mut robot_robot_collisions = vec![];
        let mut ids = vec![];
        for (id, robot) in self.robots.iter_enumerate_unordered_mut() {
            let current_packed = Self::pack_robot(id);
            ids.clear();
            kd_tree.find_in_range(robot.movement_module.get_position(), 2.1, &mut ids);
            for found_id in ids.iter() {
                if current_packed == *found_id {
                    continue;
                }

                match found_id >> 24 {
                    Self::ROBOT_ID => {
                        if (found_id & 0xffffff) > (current_packed & 0xffffff) {
                            robot_robot_collisions.push((found_id & 0xffffff, current_packed & 0xffffff));
                        }
                    },
                    Self::FABRICATOR_ID => {
                        let idx = found_id & 0xffffff;
                        let fabricator_bbox = self.fabricators[idx as usize].get_bbox();

                        let (cur_bbox, prev_bbox) = robot.get_collision_bboxes();
                        let resolved = resolve_static_aabb(&fabricator_bbox, &cur_bbox, &prev_bbox);
                        robot.update_collision(&resolved);
                    },
                    Self::GHOST_ROBOT_ID => {
                        let idx = found_id & 0xffffff;
                        let ghost_position = self.fabricators[idx as usize].get_possible_ghost_position();
                        let ghost_bbox = Robot::get_render_box(ghost_position);

                        let (cur_bbox, prev_bbox) = robot.get_collision_bboxes();
                        let resolved = resolve_static_aabb(&ghost_bbox, &cur_bbox, &prev_bbox);
                        robot.update_collision(&resolved);
                    },
                    _ => {},
                }
            }
        }

        for (id1, id2) in robot_robot_collisions {
            let (bb1, pbb1) = self.robots[id1 as usize].get_collision_bboxes();
            let (bb2, pbb2) = self.robots[id2 as usize].get_collision_bboxes();

            let (r1, r2) = resolve_dynamic_aabb(&bb1, &pbb1, &bb2, &pbb2);

            self.robots[id1 as usize].update_collision(&r1);
            self.robots[id2 as usize].update_collision(&r2);
        }
    }
}