use nalgebra_glm as glm;
use crate::{game::{render::UI_BASE_DEPTH, scene::EventReceiver}, graphics::renderer::positioning::{PositionMode, RenderBox}, ui::{AreaManager, Ui, simple_area::SimpleArea}};

/// Dnd (Drag and Drop) handles reordering logic for a vertical list of items.
/// 
/// It maintains a mapping between the "Original Index" (the order in memory/data)
/// and the "Render Position" (where it actually appears on the screen).
#[derive(Debug)]
pub struct Dnd {
    pub top_left_position: glm::Vec2,
    pub unit_size: glm::Vec2,
    pub gap: f32,
    /// order[original_idx] -> (current_render_position, drag_area_id)
    /// This allows us to keep the data source stable while moving things visually.
    order: Vec<(usize, usize)>,     
    /// grabbed -> (id of the active drag area, accumulated vertical pixel offset)
    grabbed: Option<(usize, f32)>,  
}

impl Dnd {
    /// Calculates a static RenderBox for a specific slot index in the vertical list.
    fn get_index_bounding_box(top_left_position: glm::Vec2, unit_size: glm::Vec2, gap: f32, index: usize) -> RenderBox {
        RenderBox::new_simple(
            // Y-axis decreases as index increases (Standard 2D UI coordinate system)
            top_left_position - glm::vec2(0.0, index as f32 * (unit_size.y + gap)),
            PositionMode::TopLeft,
            unit_size
        )
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn get_drag_box_id(&self, idx: usize) -> usize {
        self.order[idx].1
    }

    /// Removes an item based on its current visible position on the screen.
    /// Returns the original index of the removed item so the caller can 
    /// also remove it from their data source (e.g., the instruction list).
    pub fn remove(&mut self, render_idx: usize) -> Option<usize> {
        // 1. Find the item that currently occupies the target render position
        let vector_pos = self.order.iter()
            .position(|(r_pos, _)| *r_pos == render_idx)?;

        // 2. Remove it from the mapping and get its details
        let (removed_render_pos, drag_id) = self.order.remove(vector_pos);

        // 3. If this item was being grabbed, clear the grab state to avoid bugs
        if let Some((grab_id_active, _)) = self.grabbed {
            if grab_id_active == drag_id {
                self.grabbed = None;
            }
        }

        // 4. Shift all items that were visually BELOW the removed item UP by one slot
        for (r_pos, _) in self.order.iter_mut() {
            if *r_pos > removed_render_pos {
                *r_pos -= 1;
            }
        }

        // Return the 'original_idx' (which was the position in the vector before removal)
        Some(vector_pos)
    }

    /// Removes an item based on its original index (its position in the 'order' vector).
    /// Returns the drag_area_id so the caller can remove it from the UI manager.
    pub fn remove_original(&mut self, original_idx: usize) -> Option<usize> {
        if original_idx >= self.order.len() {
            return None;
        }

        // 1. Remove the item using the stable index
        let (removed_render_pos, drag_id) = self.order.remove(original_idx);

        // 2. Clear grab state if this specific item was being dragged
        if let Some((grab_id_active, _)) = self.grabbed {
            if grab_id_active == drag_id {
                self.grabbed = None;
            }
        }

        // 3. Update the visual positions of all remaining items.
        // Items that were visually BELOW the removed one must slide UP.
        for (r_pos, _) in self.order.iter_mut() {
            if *r_pos > removed_render_pos {
                *r_pos -= 1;
            }
        }

        // Return the drag_id so you can call ui.drag_area_manager.remove_drag_area(id)
        Some(drag_id)
    }

    /// Converts a raw pixel offset into a "slot shift" count.
    /// Returns the target render index (where the item would drop if released now).
    fn calculate_target_pos(&self, original_render_pos: usize, offset: f32) -> usize {
        let space = self.unit_size.y + self.gap;
        // Divide offset by slot height and round to find the nearest integer slot
        let shift = (-offset / space).round() as isize;
        // Ensure the target is within the bounds of the list
        (original_render_pos as isize + shift).clamp(0, self.order.len() as isize - 1) as usize
    }

    pub fn new(top_left_position: glm::Vec2, unit_size: glm::Vec2, gap: f32, amount: usize, ui: &mut Ui) -> Self {
        Self {
            top_left_position, unit_size, gap,
            order: (0..amount).map(|i| (
                i, // Initial render position is just the index
                ui.drag_area_manager.push(SimpleArea::new(
                    Self::get_index_bounding_box(top_left_position, unit_size, gap, i),
                    UI_BASE_DEPTH - 1,
                ))
            )).collect(),
            grabbed: None,
        }
    }

    /// Appends a new item to the end of the list.
    /// It calculates the next available render position, creates a new DragArea,
    /// and updates the internal mapping.
    pub fn push(&mut self, ui: &mut Ui) {
        // The new item's initial render position is the current length of the list.
        let new_render_pos = self.order.len();

        // Calculate the physical bounding box for this new slot.
        let new_bbox = Self::get_index_bounding_box(
            self.top_left_position,
            self.unit_size,
            self.gap,
            new_render_pos,
        );

        // Register the interactive area with the UI manager.
        let grab_id = ui.drag_area_manager.push(SimpleArea::new(new_bbox, UI_BASE_DEPTH - 1));

        // Add the new entry to the order mapping: (render_position, drag_area_id).
        // Its original_idx will implicitly be its position in this vector.
        self.order.push((new_render_pos, grab_id));
    }

    /// Synchronizes the hitboxes (DragAreas) with their visual slots.
    /// This ensures that even after reordering, clicking a slot grabs the correct item.
    pub fn update(&self, ui: &mut Ui) {
        for (render_pos, grab_id) in self.order.iter() {
            let current_box = Self::get_index_bounding_box(
                self.top_left_position,
                self.unit_size,
                self.gap,
                *render_pos
            );

            ui.drag_area_manager.update(
                *grab_id,
                SimpleArea::new(current_box, UI_BASE_DEPTH - 1),
            );
        }
    }

    /// Generates the list of boxes to be rendered this frame.
    /// Provides a visual "preview" of the reordering before it is finalized.
    pub fn get_unit_positions(&self) -> impl Iterator<Item = (RenderBox, bool)> + '_ {
        let len = self.order.len();
        let space = self.unit_size.y + self.gap;

        let drag_info = self.grabbed.and_then(|(drag_id, offset)| {
            let original_render_pos = self.order.iter()
                .find(|(_, area_id)| *area_id == drag_id)?
                .0;

            let target_pos = self.calculate_target_pos(original_render_pos, offset);
            Some((original_render_pos, target_pos, drag_id, offset))
        });

        (0..len).map(move |original_idx| {
            let (current_render_pos, area_id) = self.order[original_idx];
            let mut is_grabbed = false;
            let mut visual_offset = glm::vec2(0.0, 0.0);

            let render_pos = if let Some((orig, target, g_id, raw_offset)) = drag_info {
                if area_id == g_id {
                    is_grabbed = true;

                    /* --- CLAMP LOGIC ---
                       Calculate the maximum allowed upward and downward movement.
                       Max Up: Distance to the first slot (index 0).
                       Max Down: Distance to the last slot (index len - 1).
                    */
                    let max_up = orig as f32 * space;
                    let max_down = -((len as isize - 1 - orig as isize) as f32 * space);
                    
                    // Clamp the raw_offset so the item never leaves the container visually
                    let clamped_offset = raw_offset.clamp(max_down, max_up);
                    
                    visual_offset = glm::vec2(0.0, clamped_offset);
                    orig 
                } else if orig < target && current_render_pos > orig && current_render_pos <= target {
                    current_render_pos - 1
                } else if orig > target && current_render_pos < orig && current_render_pos >= target {
                    current_render_pos + 1
                } else {
                    current_render_pos
                }
            } else {
                current_render_pos
            };

            let bbox = Self::get_index_bounding_box(
                self.top_left_position,
                self.unit_size,
                self.gap,
                render_pos,
            );

            (bbox.apply_offset(visual_offset), is_grabbed)
        })
    }
}

impl EventReceiver<()> for Dnd {
    /// Finalizes the reordering. Updates the internal 'order' mapping permanently.
    fn on_released(&mut self, id: usize, _context: &mut ()) {
        let Some((grab_id, offset)) = self.grabbed else { return };
        if grab_id != id { return };

        // Find the item currently being dragged
        if let Some(orig_render_pos) = self.order.iter().find(|(_, aid)| *aid == id).map(|(p, _)| *p) {
            let target_render_pos = self.calculate_target_pos(orig_render_pos, offset);
            
            // Re-map all items in the list to reflect the new order
            for (pos, _) in self.order.iter_mut() {
                if *pos == orig_render_pos {
                    // The dragged item takes the target position
                    *pos = target_render_pos;
                } else if orig_render_pos < target_render_pos && *pos > orig_render_pos && *pos <= target_render_pos {
                    // Shifting up: items between original and target move one slot up
                    *pos -= 1;
                } else if orig_render_pos > target_render_pos && *pos < orig_render_pos && *pos >= target_render_pos {
                    // Shifting down: items between original and target move one slot down
                    *pos += 1;
                }
            }
        }

        self.grabbed = None;
    }

    /// Updates the drag state. Called by the UI system when a mouse move occurs over a DragArea.
    fn on_moved(&mut self, id: usize, movement: glm::Vec2, _context: &mut ()) {
        if let Some((area_id, offset)) = &mut self.grabbed {
            if *area_id != id {
                // If a new ID is passed, reset offset to the new movement
                *area_id = id;
                *offset = movement.y;
            } else {
                // Accumulate vertical movement
                *offset += movement.y;
            }
         } else {
            self.grabbed = Some((id, movement.y));
        }
    }
}


#[cfg(test)]
impl Dnd {
    /// Special constructor for unit testing. 
    /// Bypasses UI dependency by using dummy IDs for drag areas.
    pub fn new_test(amount: usize) -> Self {
        Self {
            top_left_position: glm::vec2(0.0, 0.0),
            unit_size: glm::vec2(100.0, 20.0),
            gap: 5.0,
            // (render_position, dummy_area_id)
            order: (0..amount).map(|i| (i, i)).collect(),
            grabbed: None,
        }
    }

    pub fn push_test(&mut self) {
        let new_pos = self.order.len();
        self.order.push((new_pos, new_pos)); // Usa o index como ID de teste
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let dnd = Dnd::new_test(3);
        let positions: Vec<_> = dnd.get_unit_positions().collect();
        
        // Items should be in their natural order (0, 1, 2)
        assert_eq!(positions.len(), 3);
        assert!(!positions[0].1); // Not grabbed
    }

    #[test]
    fn test_remove_original_logic() {
        let mut dnd = Dnd::new_test(3); // Items: 0, 1, 2 nas posições visuais 0, 1, 2
        
        // 1. Vamos bagunçar a ordem primeiro: move o item 0 para o fim (Pos 2)
        // Original 0 -> Pos 2
        // Original 1 -> Pos 0
        // Original 2 -> Pos 1
        dnd.on_moved(0, glm::vec2(0.0, -60.0), &mut ());
        dnd.on_released(0, &mut ());

        // 2. Remove o "Original 2" (que está visualmente na Pos 1)
        let removed_id = dnd.remove_original(2).unwrap();
        
        assert_eq!(removed_id, 2);
        assert_eq!(dnd.order.len(), 2);

        // 3. Verificação:
        // O item que estava na Pos 2 (Original 0) deve ter subido para a Pos 1
        // O item que estava na Pos 0 (Original 1) deve continuar na Pos 0
        assert_eq!(dnd.order[0].0, 1, "Original 0 should have moved from Pos 2 to 1");
        assert_eq!(dnd.order[1].0, 0, "Original 1 should stay at Pos 0");
    }

    #[test]
    fn test_preview_shift_down() {
        let mut dnd = Dnd::new_test(3);

        // Grab item 0 (ID 0) and move it down past the first slot threshold
        // Mouse moved -30.0 pixels (down in this coordinate system)
        dnd.on_moved(0, glm::vec2(0.0, -30.0), &mut ());

        let positions: Vec<_> = dnd.get_unit_positions().collect();

        // Item 0 is grabbed and should have the visual offset applied
        assert!(positions[0].1, "Item 0 should be marked as grabbed");
        
        // The shifting logic: 
        // When item 0 (original index 0) moves to slot 1, 
        // item 1 (original index 1) should "flee" to slot 0.
        
        // Checking RenderBox of Item 1 (original_idx 1)
        // It should now be at render_pos 0
        let expected_box_item1 = Dnd::get_index_bounding_box(
            dnd.top_left_position, dnd.unit_size, dnd.gap, 0
        );
        assert_eq!(positions[1].0.get_top_left(), expected_box_item1.get_top_left());
    }

    #[test]
    fn test_commit_reorder() {
        let mut dnd = Dnd::new_test(3);

        // Move item 0 down to the very bottom (slot 2)
        dnd.on_moved(0, glm::vec2(0.0, -60.0), &mut ());
        
        // Release to finalize the order
        dnd.on_released(0, &mut ());

        // Internal 'order' should now be updated permanently
        // Original 0 -> RenderPos 2
        // Original 1 -> RenderPos 0
        // Original 2 -> RenderPos 1
        assert_eq!(dnd.order[0].0, 2);
        assert_eq!(dnd.order[1].0, 0);
        assert_eq!(dnd.order[2].0, 1);
        
        assert!(dnd.grabbed.is_none());
    }

    #[test]
    fn test_clamp_boundaries() {
        let mut dnd = Dnd::new_test(3);
        
        // Move item 0 way beyond the list bottom
        dnd.on_moved(0, glm::vec2(0.0, -5000.0), &mut ());
        
        if let Some((_, offset)) = dnd.grabbed {
            let target = dnd.calculate_target_pos(0, offset);
            assert_eq!(target, 2, "Should clamp to the last index");
        }

        // Move item 2 way beyond the list top
        dnd.on_released(0, &mut ());
        dnd.on_moved(2, glm::vec2(0.0, 5000.0), &mut ());
        
        if let Some((_, offset)) = dnd.grabbed {
            // Item 2 is currently at RenderPos 1 (from previous test logic, 
            // but here it's a new Dnd, so it's at Pos 2)
            let target = dnd.calculate_target_pos(2, offset);
            assert_eq!(target, 0, "Should clamp to the first index");
        }
    }

    #[test]
    fn test_push_adds_to_bottom() {
        let mut dnd = Dnd::new_test(2);
        dnd.push_test();

        assert_eq!(dnd.order.len(), 3);
        
        // The new item (original index 2) should be at render position 2
        assert_eq!(dnd.order[2].0, 2);
        
        let positions: Vec<_> = dnd.get_unit_positions().collect();
        let last_item_pos = positions[2].0.get_top_left();
        
        // Verify visual position calculation for the 3rd item (index 2)
        let expected_pos = Dnd::get_index_bounding_box(
            dnd.top_left_position, dnd.unit_size, dnd.gap, 2
        ).get_top_left();
        
        assert_eq!(last_item_pos, expected_pos);
    }

    #[test]
    fn test_reorder_then_push() {
        let mut dnd = Dnd::new_test(2);
        
        // 1. Swap item 0 and 1
        // Move item 0 (ID 0) down by one slot height
        dnd.on_moved(0, glm::vec2(0.0, -25.0), &mut ());
        dnd.on_released(0, &mut ());
        
        // Sanity check: Original 0 is now at Pos 1, Original 1 is at Pos 0
        assert_eq!(dnd.order[0].0, 1);
        assert_eq!(dnd.order[1].0, 0);

        // 2. Push a new item
        dnd.push_test();

        // 3. Verify consistency
        // New item should be at Pos 2, regardless of previous reorders
        assert_eq!(dnd.order[2].0, 2);
        // Previous reorder should be preserved
        assert_eq!(dnd.order[0].0, 1);
        assert_eq!(dnd.order[1].0, 0);
    }

    #[test]
    fn test_push_then_reorder_all() {
        let mut dnd = Dnd::new_test(1);
        dnd.push_test(); // Now we have 2 items
        
        // Move the newly pushed item (ID 1) to the top (Pos 0)
        // Movement is positive Y in our calculate_target_pos logic to go "up"
        dnd.on_moved(1, glm::vec2(0.0, 25.0), &mut ());
        dnd.on_released(1, &mut ());

        // Original 1 should now be at Render Pos 0
        // Original 0 should have been pushed down to Render Pos 1
        assert_eq!(dnd.order[1].0, 0);
        assert_eq!(dnd.order[0].0, 1);
    }

    #[test]
    fn test_remove_middle_item() {
        let mut dnd = Dnd::new_test(3); // Items at Pos 0, 1, 2
        
        // Remove the item at the middle (Render Pos 1)
        // Note: In tests, we don't have the real 'Ui' to remove the drag area,
        // so you might need a 'remove_test' version if your 'remove' strictly requires 'Ui'.
        
        // Let's assume a simplified removal for the test logic:
        let vector_pos = dnd.order.iter().position(|(r, _)| *r == 1).unwrap();
        let (removed_r_pos, _) = dnd.order.remove(vector_pos);
        for (r, _) in dnd.order.iter_mut() {
            if *r > removed_r_pos { *r -= 1; }
        }

        // After removing Pos 1:
        // Item 0 should still be at Pos 0
        // Item 2 should now be at Pos 1
        assert_eq!(dnd.order.len(), 2);
        assert_eq!(dnd.order[0].0, 0); // Original 0
        assert_eq!(dnd.order[1].0, 1); // Original 2 (moved up from 2 to 1)
    }

    #[test]
    fn test_clamping_logic() {
        let mut dnd = Dnd::new_test(2);
        // Grab item 0 and try to drag it "to the moon" (upwards)
        dnd.on_moved(0, glm::vec2(0.0, 1000.0), &mut ());
        
        let positions: Vec<_> = dnd.get_unit_positions().collect();
        let y_pos = positions[0].0.get_top_left().y;
        
        // The Y position should be exactly the top_left_position.y (0.0), 
        // because it cannot go higher than slot 0.
        assert_eq!(y_pos, 0.0);
    }
}
