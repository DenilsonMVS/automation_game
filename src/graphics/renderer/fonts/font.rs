use crate::graphics::renderer::{buffer::{Buffer, block_buffer::BlockBuffer, static_buffer::StaticBuffer}, fonts::{GlyphAttrs, TextAlign, TextRenderConfig}, positioning::OrientedBox, texture::Texture};
use msdfgen::{Bitmap, FillRule, FontExt, MsdfGeneratorConfig, Range};
use nalgebra_glm as glm;
use anyhow::{Ok, Result};
use ttf_parser::Face;

pub const GLYPH_SIZE: usize = 32;
pub const GLYPYH_MARGIN: usize = 0;
pub const SPACE_PER_GLYPH: usize = GLYPH_SIZE + GLYPYH_MARGIN;
pub const PX_RANGE: f32 = 4.0;
pub const UTF_8_START: usize = 0x0020;
pub const UTF_8_END: usize = 0x007F;
pub const EXTRA_START: usize = 0x00A0;
pub const EXTRA_END: usize = 0x0100;
pub const WESTERN_CHAR_COUNT: usize = UTF_8_END - UTF_8_START + EXTRA_END - EXTRA_START;

fn get_western_iterator() -> impl Iterator<Item = char> {
    (UTF_8_START as u32..UTF_8_END as u32)
        .chain(EXTRA_START as u32..EXTRA_END as u32)
        .filter_map(std::char::from_u32)
}

const fn calculate_atlas_params(char_amount: usize) -> (usize, usize, usize) {
    let mut size = 1usize;
    while size * size < char_amount {
        size *= 2;
    }
    let columns = size;
    let rows = (char_amount + columns - 1) / columns;
    
    (rows * SPACE_PER_GLYPH, columns * SPACE_PER_GLYPH, columns)
}

const fn map_char_to_index(c: char) -> Option<usize> {
    let code = c as usize;
    if code >= UTF_8_START && code < UTF_8_END {
        Some(code - UTF_8_START)
    } else if code >= EXTRA_START {
        Some((code - EXTRA_START + UTF_8_END - UTF_8_START) as usize)
    } else {
        None
    }
}

const ATLAS_PARAMS: (usize, usize, usize) = calculate_atlas_params(WESTERN_CHAR_COUNT);
pub const ATLAS_HEIGHT: usize = ATLAS_PARAMS.0;
pub const ATLAS_WIDTH: usize = ATLAS_PARAMS.1;
pub const ATLAS_COLUMNS: usize = ATLAS_PARAMS.2;

#[derive(Clone, Copy, Default)]
struct GlyphInfo {
    bound_min: glm::Vec2,
    bound_max: glm::Vec2,
    advance: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct UvData {
    min: glm::Vec2,
    max: glm::Vec2,
}

impl Default for UvData {
    fn default() -> Self {
        Self {
            min: glm::vec2(0.0, 0.0),
            max: glm::vec2(0.0, 0.0),
        }
    }
}



pub struct Font {
    texture: Texture,
    glyphs: [GlyphInfo; WESTERN_CHAR_COUNT],
    ascender: f32,
    descender: f32,
    uv_buffer: StaticBuffer<UvData>,
    layers: Vec<Option<u32>>,
}

impl Font {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let font = Face::parse(bytes, 0)?;

        let mut raw_image_data = [glm::vec3(0, 0, 0); ATLAS_HEIGHT * ATLAS_WIDTH];
        let mut glyphs = [GlyphInfo::default(); WESTERN_CHAR_COUNT];
        let mut uvs = [UvData::default(); WESTERN_CHAR_COUNT];
        let atlas_size = glm::vec2(ATLAS_WIDTH as f32, ATLAS_HEIGHT as f32);

        for (idx, c) in get_western_iterator().enumerate() {
            let info = &mut glyphs[idx];
            let idx = map_char_to_index(c).unwrap();
            let start_y = (idx / ATLAS_COLUMNS) * SPACE_PER_GLYPH;
            let start_x = (idx % ATLAS_COLUMNS) * SPACE_PER_GLYPH;
            let vstart = glm::vec2(start_x, start_y).map(|x| x as f32);

            if let Some(glyph_id) = font.glyph_index(c) {
                info.advance = font.glyph_hor_advance(glyph_id).unwrap_or(0) as f32;

                if let Some(mut shape) = font.glyph_shape(glyph_id) {
                    if let Some(framing) = shape.get_bound().autoframe(
                        GLYPH_SIZE as u32,
                        GLYPH_SIZE as u32,
                        Range::Px(PX_RANGE as f64),
                        None
                    ) {
                        let mut bitmap = Bitmap::new(GLYPH_SIZE as u32, GLYPH_SIZE as u32);
                        let config = MsdfGeneratorConfig::default();

                        shape.edge_coloring_simple(3.0, 0);
                        shape.generate_msdf(&mut bitmap, &framing, &config);
                        shape.correct_sign(&mut bitmap, &framing, FillRule::default());
                        shape.correct_msdf_error(&mut bitmap, &framing, &config);

                        for y in 0..GLYPH_SIZE {
                            for x in 0..GLYPH_SIZE {
                                let pixel = bitmap.pixel(x as u32, y as u32);
                                let r = (pixel.r * 255.0).clamp(0.0, 255.0) as u8;
                                let g = (pixel.g * 255.0).clamp(0.0, 255.0) as u8;
                                let b = (pixel.b * 255.0).clamp(0.0, 255.0) as u8;

                                raw_image_data[(start_y + y) * ATLAS_WIDTH + start_x + x] = 
                                    glm::vec3(r, g, b);
                            }
                        }

                        let scale = framing.projection.scale;
                        let translate = framing.projection.translate;

                        let scale = glm::vec2(scale.x, scale.y)
                            .map(|x| x as f32);
                        let translate = glm::vec2(translate.x, translate.y)
                            .map(|x| x as f32);

                        let bb = font.glyph_bounding_box(glyph_id).unwrap();

                        info.bound_min = glm::vec2(bb.x_min, bb.y_min)
                            .map(|x| x as f32);
                        info.bound_max = glm::vec2(bb.x_max, bb.y_max)
                            .map(|x| x as f32);

                        uvs[idx] = UvData {
                            min: (vstart + (translate + info.bound_min)
                                .component_mul(&scale))
                                .component_div(&atlas_size),
                            max: (vstart + (translate + info.bound_max)
                                .component_mul(&scale))
                                .component_div(&atlas_size),
                        };
                    }
                }
            }
        }

        let texture = Texture::from_font_raw(raw_image_data.as_slice(), ATLAS_WIDTH);
        let uv_buffer = StaticBuffer::new(&uvs);

        Ok(Self {
            texture,
            glyphs,
            ascender: font.ascender() as f32,
            descender: font.descender() as f32,
            uv_buffer,
            layers: vec![],
        })
    }

    pub fn reset(&mut self) {
        self.layers.clear();
    }

    pub fn get_monospace_width(&self, line_height: f32) -> f32 {
        let font_design_height = self.ascender - self.descender;
        
        if font_design_height <= 0.0 {
            return 0.0;
        }

        let scale = line_height / font_design_height;
        
        let space_idx = map_char_to_index(' ').unwrap_or(0);
        let advance_design = self.glyphs[space_idx].advance;

        advance_design * scale
    }

    pub(crate) fn bind_to_unit(&self, unit: u32) {
        self.texture.bind_to_unit(unit);
    }

    pub(crate) fn get_cursor(&mut self, layer_idx: usize) -> Option<u32> {
        self.layers.get(layer_idx).and_then(|v| *v)
    }

    pub(crate) fn bind_uv_storage(&self) {
        self.uv_buffer.bind_as_storage(5);
    }

    fn create_layer_if_doesnt_exist(&mut self, layer_idx: usize) {
        while self.layers.len() <= layer_idx {
            self.layers.push(None);
        }
    }

    pub(super) fn add_text(
        &mut self,
        text: &TextRenderConfig,
        buffer: &mut BlockBuffer<GlyphAttrs>,
        layer_idx: usize,
    ) {
        self.create_layer_if_doesnt_exist(layer_idx);

        let glyphs = &self.glyphs;
        let ascender = self.ascender;
        let descender = self.descender;
        let cursor = *self.layers[layer_idx].get_or_insert_with(|| buffer.push_cursor());

        Self::generate_vertex(
            glyphs, ascender, descender, text,
            |vertex| {
                buffer.push(cursor, &vertex);
            },
        );
    }

    fn select_blocks_for_line(
        glyphs: &[GlyphInfo; WESTERN_CHAR_COUNT],
        blocks: &[(u32, u32, f32, bool)],
        max_width: f32,
    ) -> Vec<u32> {
        let space_advance = glyphs[map_char_to_index(' ').unwrap()].advance;

        let mut line_starts = Vec::new();
        let mut current_line_width = 0.0;
        
        if !blocks.is_empty() {
            line_starts.push(0);
        }

        for (idx, &(_, _, block_width, force_newline)) in blocks.iter().enumerate() {
            let is_first_in_line = line_starts.last() == Some(&(idx as u32));
            let added_width = if is_first_in_line { block_width } else { block_width + space_advance };

            if !is_first_in_line && current_line_width + added_width > max_width {
                line_starts.push(idx as u32);
                current_line_width = block_width;
            } else {
                current_line_width += added_width;
            }

            if force_newline && idx + 1 < blocks.len() {
                line_starts.push((idx + 1) as u32);
                current_line_width = 0.0; 
            }
        }
        
        line_starts
    }

    fn separate_in_blocks(glyphs: &[GlyphInfo; WESTERN_CHAR_COUNT], config: &TextRenderConfig) -> Vec<(u32, u32, f32, bool)> {
        let mut blocks = Vec::new();
        let mut block_start = (0, 0);
        let mut current_advance = 0.0;

        for (i, (text, _)) in config.text.iter().enumerate() {
            if current_advance == 0.0 {
                block_start = (i as u32, 0);
            }

            for (j, c) in text.chars().enumerate() {
                if c == '\n' {
                    blocks.push((block_start.0, block_start.1, current_advance, true));
                    block_start = (i as u32, (j + 1) as u32);
                    current_advance = 0.0;
                } else if c == ' ' {
                    blocks.push((block_start.0, block_start.1, current_advance, false));
                    block_start = (i as u32, (j + 1) as u32);
                    current_advance = 0.0;
                } else {
                    let Some(idx) = map_char_to_index(c) else { continue };
                    current_advance += glyphs[idx].advance;
                }
            }
        }

        if !config.text.is_empty() && (current_advance > 0.0 || blocks.is_empty()) {
            blocks.push((block_start.0, block_start.1, current_advance, false));
        }

        blocks
    }

    fn generate_vertex(
        glyphs: &[GlyphInfo; WESTERN_CHAR_COUNT],
        ascender: f32,
        descender: f32,
        config: &TextRenderConfig, 
        mut on_generation: impl FnMut(GlyphAttrs)
    ) {   
        let font_height = ascender - descender;
        let scale = config.line_height / font_height;

        let dir_x = glm::normalize(&config.forward);
        let dir_y = glm::vec2(-dir_x.y, dir_x.x);

        let blocks = Self::separate_in_blocks(glyphs, config);
        let line_start_indices = Self::select_blocks_for_line(
            glyphs, 
            &blocks, 
            config.max_line_width / scale,
        );

        let total_height = line_start_indices.len() as f32 * config.line_height;
        let full_size = glm::vec2(config.max_line_width, total_height);

        let obox = OrientedBox::new_rotated(
            config.position,
            config.position_mode,
            full_size,
            dir_x
        );

        let space_advance = map_char_to_index(' ')
            .map_or(0.0, |idx| glyphs[idx].advance * scale);

        for (line_idx, &start_block_idx) in line_start_indices.iter().enumerate() {
            let end_block_idx = line_start_indices
                .get(line_idx + 1)
                .cloned()
                .unwrap_or(blocks.len() as u32);

            let num_blocks = end_block_idx - start_block_idx;
            
            let ends_with_force = blocks[(end_block_idx - 1) as usize].3;
            let is_last_line = line_idx == line_start_indices.len() - 1;
            
            let mut current_line_width = 0.0;
            for b_idx in start_block_idx..end_block_idx {
                current_line_width += blocks[b_idx as usize].2 * scale;
            }
            if num_blocks > 1 {
                current_line_width += (num_blocks - 1) as f32 * space_advance;
            }

            let mut extra_space_per_gap = 0.0;
            let x_offset = match config.align {
                TextAlign::Left => 0.0,
                TextAlign::Center => (full_size.x - current_line_width) * 0.5,
                TextAlign::Right => full_size.x - current_line_width,
                TextAlign::Justify => {
                    if !is_last_line && !ends_with_force && num_blocks > 1 {
                        extra_space_per_gap = (full_size.x - current_line_width) / (num_blocks - 1) as f32;
                    }
                    0.0
                },
            };

            let line_offset = (line_idx as f32) * config.line_height;
            let mut cursor_pos = obox.top_left 
                - (dir_y * (line_offset + ascender * scale))
                + (dir_x * x_offset);

            for b_idx in start_block_idx..end_block_idx {
                let (str_idx, char_idx, _, _) = blocks[b_idx as usize]; // Ignoramos a flag aqui
                let (text_segment, color) = &config.text[str_idx as usize];
                
                for c in text_segment.chars().skip(char_idx as usize) {
                    if c == '\n' || c == ' ' { break; }

                    let Some(idx) = map_char_to_index(c) else { continue };
                    let info = &glyphs[idx];
                    
                    let lx0 = info.bound_min.x * scale;
                    let lx1 = info.bound_max.x * scale;

                    let ly1 = info.bound_max.y * scale;
                    let top_left = cursor_pos + (dir_x * lx0) + (dir_y * ly1);
                    let top_right = cursor_pos + (dir_x * lx1) + (dir_y * ly1);

                    on_generation(GlyphAttrs {
                        top_left, top_right,
                        character_idx: idx as u8,
                        color: *color,
                        depth: config.depth,
                    });

                    cursor_pos += dir_x * (info.advance * scale);
                }

                if b_idx + 1 < end_block_idx {
                    cursor_pos += dir_x * (space_advance + extra_space_per_gap);
                }
            }
        }
    }
}