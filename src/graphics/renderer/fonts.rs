use std::marker::PhantomData;

use msdfgen::{Bitmap, FillRule, FontExt, MsdfGeneratorConfig, Range};
use ttf_parser::Face;
use macros::{GlVertex, program_interface};
use crate::graphics::renderer::{
    Renderer, 
    buffer::{DynamicBuffer, GenericBuffer, StaticBuffer}, 
    drawable::DrawMode, 
    positioning::{OrientedBox, PositionMode, SCALE_FACTOR, vec_to_short}, 
    program::{Program, ShaderType}, 
    texture::Texture, 
    uniform::UniformValue, 
    vertex_array_object::{FieldType, StaticVertexLayout, VertexArrayObject}
};
use nalgebra_glm as glm;
use anyhow::Result;

const GLYPHS_PER_RENDER: usize = 1 << 10;

#[repr(C)]
#[derive(GlVertex)]
#[vertex(divisor = 1)]
pub struct GlyphAttrs {
    pub top_left: glm::I16Vec2,
    pub top_right: glm::I16Vec2,
    pub bottom_right: glm::I16Vec2,
    #[normalized]
    pub color: glm::U8Vec3,
    pub character_idx: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct UvData {
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

#[derive(Clone, Copy, Default)]
struct GlyphInfo {
    bound_min: glm::Vec2,
    bound_max: glm::Vec2,
    advance: f32,
}

#[program_interface(
    vert = "../../../res/shaders/font.vert",
    frag = "../../../res/shaders/font.frag"
)]
struct FontProgram {
    u_texture: i32,
    u_projection: glm::Mat4,
    u_glyph_size: u32,
    u_glyph_margin: u32,
    u_world_scale: f32,
}

pub struct Fonts<'a> {
    font_program: FontProgram<'a>,
    _marker: PhantomData<&'a Renderer>,
}

impl<'a> Fonts<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        let font_program = FontProgram::init(renderer).unwrap();
        font_program.set_u_texture(&0);
        font_program.set_u_glyph_size(&(GLYPH_SIZE as u32));
        font_program.set_u_glyph_margin(&(GLYPYH_MARGIN as u32));
        font_program.set_u_world_scale(&(1.0 / SCALE_FACTOR));

        return Self {
            font_program,
            _marker: PhantomData,
        };
    }

    pub fn new_font(&self, renderer: &'a Renderer, bytes: &[u8]) -> Result<Font<'a>> {
        Font::from_bytes(renderer, bytes)
    }

    pub fn draw_buffer(&self, font: &Font, buffer: &FontVbo, proj_matrix: &glm::Mat4) {
        self.font_program.bind();
        self.font_program.set_u_projection(proj_matrix);
        font.texture.bind_to_unit(0);
        font.uv_buf.bind_as_storage(5);
        buffer.vao.draw_instanced(4, buffer.amount, DrawMode::TriangleStrip);
    }

    pub fn draw(&self, font: &mut Font, proj_matrix: &glm::Mat4, vp_matrix: &glm::Mat4) {
        self.font_program.bind();
        font.texture.bind_to_unit(0);
        font.uv_buf.bind_as_storage(5);
        for (i, (vbo, vao)) in font.buffs.iter_mut().enumerate() {
            if i == 0 {
                self.font_program.set_u_projection(vp_matrix);
            } else {
                self.font_program.set_u_projection(proj_matrix);
            }
            vao.draw_instanced(4, vbo.get_len() as i32, DrawMode::TriangleStrip);
            vbo.lock();
        }
    }
}

pub struct FontVbo<'a> {
    _vbo: StaticBuffer<'a, GlyphAttrs>,
    vao: VertexArrayObject<'a>,
    amount: i32,
}

pub struct Font<'a> {
    texture: Texture<'a>,
    glyphs: [GlyphInfo; WESTERN_CHAR_COUNT],
    ascender: f32,
    descender: f32,
    buffs: [(DynamicBuffer<'a, GlyphAttrs>, VertexArrayObject<'a>); 2],
    uv_buf: StaticBuffer<'a, UvData>,
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

pub const GLYPH_SIZE: usize = 32;
pub const GLYPYH_MARGIN: usize = 0;
pub const SPACE_PER_GLYPH: usize = GLYPH_SIZE + GLYPYH_MARGIN;
pub const PX_RANGE: f32 = 4.0;
pub const UTF_8_START: usize = 0x0020;
pub const UTF_8_END: usize = 0x007F;
pub const EXTRA_START: usize = 0x00A0;
pub const EXTRA_END: usize = 0x0100;
pub const WESTERN_CHAR_COUNT: usize = UTF_8_END - UTF_8_START + EXTRA_END - EXTRA_START;

const ATLAS_PARAMS: (usize, usize, usize) = calculate_atlas_params(WESTERN_CHAR_COUNT);
pub const ATLAS_HEIGHT: usize = ATLAS_PARAMS.0;
pub const ATLAS_WIDTH: usize = ATLAS_PARAMS.1;
pub const ATLAS_COLUMNS: usize = ATLAS_PARAMS.2;

#[derive(Debug, Clone, Copy)]
pub struct TextRenderConfig<'a> {
    pub text: &'a str,
    pub position: glm::Vec2,
    pub line_height: f32,
    pub rotation: f32,
    pub position_mode: PositionMode,
    pub color: glm::U8Vec3,
    pub group: usize,
}

impl<'a> TextRenderConfig<'a> {
    pub fn new(text: &'a str, position: glm::Vec2, line_height: f32, color: glm::U8Vec3) -> Self {
        Self {
            text,
            position,
            line_height,
            position_mode: PositionMode::Center,
            color,
            rotation: 0.0,
            group: 0,
        }
    }

    pub fn set_mode(mut self, mode: PositionMode) -> Self {
        self.position_mode = mode;
        return self;
    }

    pub fn set_group(mut self, group: usize) -> Self {
        self.group = group;
        return self;
    }

    pub fn set_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        return self;
    }
}

impl<'a> Font<'a> {
    fn from_bytes(renderer: &'a Renderer, bytes: &[u8]) -> Result<Self> {
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
                                let pixel = bitmap.pixel(x as u32, (GLYPH_SIZE - y - 1) as u32);
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

                        info.bound_min = glm::vec2(bb.x_min, bb.y_max)
                            .map(|x| x as f32);
                        info.bound_max = glm::vec2(bb.x_max, bb.y_min)
                            .map(|x| x as f32);

                        uvs[idx] = UvData {
                            min: (vstart + (translate + info.bound_min).component_mul(&scale))
                                .component_div(&atlas_size),
                            max: (vstart + (translate + info.bound_max).component_mul(&scale))
                                .component_div(&atlas_size),
                        };
                    }
                }
            }
        }

        let texture = Texture::from_font_raw(renderer, raw_image_data.as_slice(), ATLAS_WIDTH);
        let buffs = std::array::from_fn(|_| {
            let vbo = DynamicBuffer::<GlyphAttrs>::new(renderer, GLYPHS_PER_RENDER);
            let vao = VertexArrayObject::new(renderer, &[&vbo]);
            return (vbo, vao);
        });
        let uv_buf = StaticBuffer::new(renderer, &uvs);

        return Ok(Self {
            texture,
            glyphs,
            ascender: font.ascender() as f32,
            descender: font.descender() as f32,
            buffs,
            uv_buf,
        });
    }

    pub fn reset(&mut self) {
        for (vbo, _) in self.buffs.iter_mut() {
            vbo.wait();
            vbo.reset_index();
        }
    }

    pub fn bind_to_unit(&self, unit: u32) {
        self.texture.bind_to_unit(unit);
    }

    pub fn add_text(&mut self, text: &TextRenderConfig) {
        let glyphs = &self.glyphs;
        let ascender = self.ascender;
        let descender = self.descender;
        let vbo = &mut self.buffs[text.group].0;
        Self::generate_vertex(glyphs, ascender, descender, text,
            |vertex| vbo.write(&vertex));
    }

    pub fn create_text_vbo<'b>(&self, renderer: &'b Renderer, rendered_text: &[TextRenderConfig]) -> FontVbo<'b> {
        let mut glyph_buffer_data = Vec::with_capacity(
            rendered_text.iter().fold(0usize, |prev, cur| prev + cur.text.len())
        );
        
        let glyphs = &self.glyphs;
        let ascender = self.ascender;
        let descender = self.descender;
        let mut on_generation = |vertex: GlyphAttrs| glyph_buffer_data.push(vertex);
        for config in rendered_text {
            Self::generate_vertex(glyphs, ascender, descender, config, &mut on_generation);
        }

        let vbo = StaticBuffer::<GlyphAttrs>::new(renderer, &glyph_buffer_data);
        let vao = VertexArrayObject::new(renderer, &[&vbo]);
        
        FontVbo {
            _vbo: vbo,
            vao,
            amount: glyph_buffer_data.len() as i32
        }
    }

    fn generate_vertex(
        glyphs: &[GlyphInfo; WESTERN_CHAR_COUNT],
        ascender: f32,
        descender: f32,
        config: &TextRenderConfig, 
        mut on_generation: impl FnMut(GlyphAttrs)
    ) {   
        let font_height = ascender - descender;

        let cos = config.rotation.cos();
        let sin = config.rotation.sin();
        let dir_x = glm::vec2(cos, sin);
        let dir_y = glm::vec2(-sin, cos);

        let total_width = config.text.chars().fold(0.0f32, |acc, c|
            map_char_to_index(c).map_or(acc, |idx| acc + glyphs[idx].advance));

        let scale = config.line_height / font_height;
        let full_size = glm::vec2(total_width * scale, font_height * scale);

        let obox = OrientedBox::new_rotated(
            config.position,
            config.position_mode,
            full_size,
            dir_y
        );

        let mut cursor_pos = obox.bot_left - (dir_y * (descender * scale));

        for c in config.text.chars() {
            if let Some(idx) = map_char_to_index(c) {
                let info = &glyphs[idx];
                
                let lx0 = info.bound_min.x * scale;
                let lx1 = info.bound_max.x * scale;
                let ly0 = info.bound_min.y * scale;
                let ly1 = info.bound_max.y * scale;

                let tl = cursor_pos + (dir_x * lx0) + (dir_y * ly1);
                let tr = cursor_pos + (dir_x * lx1) + (dir_y * ly1);
                let br = cursor_pos + (dir_x * lx1) + (dir_y * ly0);

                on_generation(GlyphAttrs {
                    top_left: vec_to_short(&tl),
                    top_right: vec_to_short(&tr),
                    bottom_right: vec_to_short(&br),
                    character_idx: idx as u8,
                    color: config.color,
                });

                cursor_pos += dir_x * (info.advance * scale);
            }
        }
    }

    pub fn finish(&self) {
        for (vbo, _) in self.buffs.iter() {
            vbo.flush();
        }
    }
}
