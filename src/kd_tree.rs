
use nalgebra_glm as glm;

#[derive(Debug)]
pub struct KdTree<Id> {
    positions: Vec<glm::Vec2>,
    ids: Vec<Id>,
}

#[derive(Debug)]
struct Slice<'a, Id: Clone> {
    positions: &'a mut [glm::Vec2],
    ids: &'a mut [Id],
}

impl<'a, Id: Clone> Slice<'a, Id> {
    fn get_x(&self, i: usize) -> f32 {
        self.positions[i].x
    }

    fn get_y(&self, i: usize) -> f32 {
        self.positions[i].y
    }

    fn len(&self) -> usize {
        self.positions.len()
    }

    fn swap(&mut self, i: usize, j: usize) {
        self.positions.swap(i, j);
        self.ids.swap(i, j);
    }

    fn split(self, mid: usize) -> (Slice<'a, Id>, Slice<'a, Id>) {
        let (pos_left, pos_mid_and_right) = self.positions.split_at_mut(mid);
        let pos_right = &mut pos_mid_and_right[1..];

        let (ids_left, ids_mid_and_right) = self.ids.split_at_mut(mid);
        let ids_right = &mut ids_mid_and_right[1..];

        (
            Slice {
                positions: pos_left,
                ids: ids_left,
            },
            Slice {
                positions: pos_right,
                ids: ids_right,
            },
        )
    }

    fn sub_slice(self, l: usize, r: usize) -> Slice<'a, Id> {
        Slice { positions: &mut self.positions[l..r], ids: &mut self.ids[l..r] }
    }

    pub fn reborrow(&mut self) -> Slice<'_, Id> {
        Slice {
            positions: &mut *self.positions,
            ids: &mut *self.ids,
        }
    }
}

impl<Id: Clone> KdTree<Id> {
    pub fn new(mut positions: Vec<glm::Vec2>, mut ids: Vec<Id>) -> Self {
        let slice = Slice {
            positions: &mut positions,
            ids: &mut ids,
        };
        Self::build_recursive_x(slice);
        Self { positions, ids }
    }

    pub fn find_in_range(&self, query: glm::Vec2, max_dist: f32, output: &mut Vec<(Id, glm::Vec2)>) {
        if self.positions.is_empty() {
            return;
        }
        self.search_recursive_x(0, self.positions.len(), query, max_dist, output);
    }

    fn search_recursive_x(
        &self,
        start: usize,
        end: usize,
        query: glm::Vec2,
        max_dist: f32,
        output: &mut Vec<(Id, glm::Vec2)>
    ) {
        if start >= end { return; }

        let mid = start + (end - start) / 2;
        let current_pos = self.positions[mid];

        let dx = (query.x - current_pos.x).abs();
        let dy = (query.y - current_pos.y).abs();
        if dx.max(dy) <= max_dist {
            output.push((self.ids[mid].clone(), self.positions[mid]));
        }

        let axis_dist = query.x - current_pos.x;

        if axis_dist <= 0.0 {
            self.search_recursive_y(start, mid, query, max_dist, output);
            if axis_dist.abs() <= max_dist {
                self.search_recursive_y(mid + 1, end, query, max_dist, output);
            }
        } else {
            self.search_recursive_y(mid + 1, end, query, max_dist, output);
            if axis_dist.abs() <= max_dist {
                self.search_recursive_y(start, mid, query, max_dist, output);
            }
        }
    }

    fn search_recursive_y(
        &self,
        start: usize,
        end: usize,
        query: glm::Vec2,
        max_dist: f32,
        output: &mut Vec<(Id, glm::Vec2)>,
    ) {
        if start >= end { return; }

        let mid = start + (end - start) / 2;
        let current_pos = self.positions[mid];
        
        let dx = (query.x - current_pos.x).abs();
        let dy = (query.y - current_pos.y).abs();
        if dx.max(dy) <= max_dist {
            output.push((self.ids[mid].clone(), self.positions[mid]));
        }

        let axis_dist = query.y - current_pos.y;

        if axis_dist <= 0.0 {
            self.search_recursive_x(start, mid, query, max_dist, output);
            if axis_dist.abs() <= max_dist {
                self.search_recursive_x(mid + 1, end, query, max_dist, output);
            }
        } else {
            self.search_recursive_x(mid + 1, end, query, max_dist, output);
            if axis_dist.abs() <= max_dist {
                self.search_recursive_x(start, mid, query, max_dist, output);
            }
        }
    }

    fn build_recursive_x(slice: Slice<'_, Id>) {
        let len = slice.len();
        if len <= 1 {
            return;
        }

        let mid = len / 2;
        let (left, right) =  Self::nth_unstable_x(slice, mid);
        Self::build_recursive_y(left);
        Self::build_recursive_y(right);
    }

    fn build_recursive_y(slice: Slice<'_, Id>) {
        let len = slice.len();
        if len <= 1 {
            return;
        }

        let mid = len / 2;
        let (left, right) =  Self::nth_unstable_y(slice, mid);
        Self::build_recursive_x(left);
        Self::build_recursive_x(right);
    }

    fn sort5_x<'a>(mut slice: Slice<'a, Id>) {
        macro_rules! s { ($i:expr, $j:expr) => {
            if slice.get_x($i) > slice.get_x($j) { slice.swap($i, $j); }
        }}

        match slice.len() {
            2 => {
                s!(0, 1);
            }
            3 => {
                s!(0, 1); s!(1, 2); s!(0, 1);
            }
            4 => {
                s!(0, 1); s!(2, 3);
                s!(0, 2); s!(1, 3);
                s!(1, 2);
            }
            5 => {
                s!(0, 1); s!(3, 4); 
                s!(2, 4); s!(2, 3); 
                s!(0, 3); s!(0, 2); 
                s!(1, 4); s!(1, 3); 
                s!(1, 2);
            }
            _ => {},
        }
    }

    fn sort5_y<'a>(mut slice: Slice<'a, Id>) {
        macro_rules! s { ($i:expr, $j:expr) => {
            if slice.get_y($i) > slice.get_y($j) { slice.swap($i, $j); }
        }}

        match slice.len() {
            2 => {
                s!(0, 1);
            }
            3 => {
                s!(0, 1); s!(1, 2); s!(0, 1);
            }
            4 => {
                s!(0, 1); s!(2, 3);
                s!(0, 2); s!(1, 3);
                s!(1, 2);
            }
            5 => {
                s!(0, 1); s!(3, 4); 
                s!(2, 4); s!(2, 3); 
                s!(0, 3); s!(0, 2); 
                s!(1, 4); s!(1, 3); 
                s!(1, 2);
            }
            _ => {},
        }
    }

    fn pivot_x(mut slice: Slice<'_, Id>) {
        let mut n = slice.len();
        
        while n > 1 {
            let mut j = 0;
            let mut i = 0;
            
            while i < n {
                let current_chunk_len = (i + 5).min(n) - i;
                
                {
                    let part = slice.reborrow().sub_slice(i, i + current_chunk_len);
                    Self::sort5_x(part);
                }

                let relative_median = current_chunk_len / 2;
                slice.swap(i + relative_median, j);

                i += 5;
                j += 1;
            }
            
            n = j;
        }
    }

    fn pivot_y(mut slice: Slice<'_, Id>) {
        let mut n = slice.len();
        
        while n > 1 {
            let mut j = 0;
            let mut i = 0;
            
            while i < n {
                let current_chunk_len = (i + 5).min(n) - i;
                {
                    let part = slice.reborrow().sub_slice(i, i + current_chunk_len);
                    Self::sort5_y(part);
                }

                let relative_median = current_chunk_len / 2;
                slice.swap(i + relative_median, j);

                i += 5;
                j += 1;
            }
            n = j;
        }
    }

    fn nth_unstable_x<'a>(mut slice: Slice<'a, Id>, position: usize) -> (Slice<'a, Id>, Slice<'a, Id>) {
        let mut offset = 0;
        let mut current = slice.reborrow();

        loop {
            Self::pivot_x(current.reborrow());

            let len = current.len();
            let mut j = len;
            let mut pivot_pos = 0;

            while pivot_pos + 1 < j {
                if current.get_x(pivot_pos + 1) < current.get_x(pivot_pos) {
                    current.swap(pivot_pos, pivot_pos + 1);
                    pivot_pos += 1;
                } else {
                    current.swap(pivot_pos + 1, j - 1);
                    j -= 1;
                }
            }

            let abs_pivot_pos = offset + pivot_pos;

            if abs_pivot_pos == position {
                break;
            } else if position < abs_pivot_pos {
                current = current.sub_slice(0, pivot_pos);
            } else {
                offset += pivot_pos + 1;
                let new_len = current.len();
                current = current.sub_slice(pivot_pos + 1, new_len);
            }
        }

        slice.split(position)
    }

    fn nth_unstable_y<'a>(mut slice: Slice<'a, Id>, position: usize) -> (Slice<'a, Id>, Slice<'a, Id>) {
        let mut offset = 0;
        let mut current = slice.reborrow();

        loop {
            Self::pivot_y(current.reborrow());

            let len = current.len();
            let mut j = len;
            let mut pivot_pos = 0;

            while pivot_pos + 1 < j {
                if current.get_y(pivot_pos + 1) < current.get_y(pivot_pos) {
                    current.swap(pivot_pos, pivot_pos + 1);
                    pivot_pos += 1;
                } else {
                    current.swap(pivot_pos + 1, j - 1);
                    j -= 1;
                }
            }

            let abs_pivot_pos = offset + pivot_pos;

            if abs_pivot_pos == position {
                break;
            } else if position < abs_pivot_pos {
                current = current.sub_slice(0, pivot_pos);
            } else {
                offset += pivot_pos + 1;
                let new_len = current.len();
                current = current.sub_slice(pivot_pos + 1, new_len);
            }
        }

        slice.split(position)
    }
}
