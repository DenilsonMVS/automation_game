use std::{num::NonZeroU32, sync::mpsc} ;

use glutin::{context::PossiblyCurrentContext, surface::{GlSurface, Surface, WindowSurface}};
use winit::{event::{ElementState, MouseScrollDelta, WindowEvent}, event_loop::{ActiveEventLoop, EventLoop}, keyboard::{Key, KeyCode, NamedKey, PhysicalKey}, window::Window};
use nalgebra_glm as glm;
use crate::{
    game::{logic::{JoinState, top_level::{computer::{X0, X3, X4, X14, X15}, instruction::{Instruction, Syscall}}}, render::{RenderMessage, RenderState}}, graphics::{
        App, ApplicationState, renderer::{
            BlendFactor, Capability, ClearField, DepthFunc, Fence, Renderer, fonts::font::Font,
        },
    }, ui::Ui,
};

pub mod graphics;
pub mod ui;
pub mod game;
pub mod open_simplex;
pub mod snapshot_manager;

pub struct State {
    window: Window,
    renderer: Renderer,
    fence: Option<Fence>,
    gl_context: PossiblyCurrentContext,
    gl_surface: Surface<WindowSurface>,
    ui: Ui,

    mono: Font,

    simulation: JoinState,
    render: RenderState,
    render_rx: mpsc::Receiver<RenderMessage>,

    w: bool,
    a: bool,
    s: bool,
    d: bool,
}

impl ApplicationState for State {
    fn new(
        window: Window,
        gl_context: PossiblyCurrentContext,
        gl_surface: Surface<WindowSurface>,
        renderer: Renderer,
    ) -> Self {
        renderer.enable(Capability::Blend);
        renderer.depth_func(DepthFunc::Less);
        renderer.blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);

        renderer.clear_color(&glm::vec4(0.8, 0.8, 0.8, 1.0));
        
        let (render_tx, render_rx) = mpsc::channel();

        let ui = Ui::new(Self::get_window_size());

        let simulation = JoinState::new(render_tx);
        let instructions = [
            // [0] Consulta limite e salva em X3
            Instruction::new_sys(Syscall::MaxStorage),
            Instruction::new_addi(X3, X15, 0),

            // [2] Configura canal e exibe 0 inicial (X4 começa em 0)
            Instruction::new_addi(X14, X0, 1),
            Instruction::new_addi(X15, X4, 0),
            Instruction::new_sys(Syscall::Display),

            // --- CICLO DO BLOCO ATUAL ---
            // [5] Se cheio (X4 == X3), pula para andar direto à direita no índice [13]
            // Destino 13: 13 - (5 + 1) = 7
            Instruction::new_beq(X4, X3, 7),

            // [6] Escaneia o bloco
            Instruction::new_sys(Syscall::Scan),

            // [7] Se acabou o minério (X15 == 0), pula para andar à direita no índice [13]
            // Destino 13: 13 - (7 + 1) = 5
            Instruction::new_beq(X15, X0, 5),

            // --- MINERAÇÃO NO MESMO BLOCO ---
            // [8] Minera 1 unidade
            Instruction::new_sys(Syscall::Mine),
            // [9] X4 += 1
            Instruction::new_addi(X4, X4, 1),
            // [10] X15 = X4
            Instruction::new_addi(X15, X4, 0),
            // [11] Atualiza display
            Instruction::new_sys(Syscall::Display),

            // [12] Volta para reverificar o MESMO bloco no índice [5]
            // Destino 5: 5 - (12 + 1) = -8 = 16 - 8 = 8
            Instruction::new_beq(X0, X0, 8),

            // --- AVANÇO À DIREITA (COMPARTILHADO) ---
            // [13] Anda para a direita
            Instruction::new_sys(Syscall::MoveRight),

            // [14] Se ainda NÃO estiver cheio (X4 != X3), volta para avaliar novo bloco no índice [5]
            // Destino 5: 5 - (14 + 1) = -10 = 16 - 10 = 6
            Instruction::new_bne(X4, X3, 6),

            // [15] Se CHEIO (caiu por fallthrough do bne), volta para MoveRight no índice [13]
            // Destino 13: 13 - (15 + 1) = 13 - 16 = -3 = 13
            Instruction::new_beq(X0, X0, 13),
        ];

        simulation.push_robot(instructions, glm::vec2(0.0, 0.0));
        simulation.push_robot(instructions, glm::vec2(0.0, 1.0));
        simulation.push_robot(instructions, glm::vec2(0.0, 2.0));
        simulation.push_robot(instructions, glm::vec2(0.0, 3.0));
        simulation.push_robot(instructions, glm::vec2(0.0, 4.0));
        simulation.push_robot(instructions, glm::vec2(0.0, 5.0));
        simulation.push_robot(instructions, glm::vec2(0.0, 6.0));
        simulation.push_robot(instructions, glm::vec2(0.0, 7.0));
        simulation.push_robot(instructions, glm::vec2(0.0, 8.0));
        simulation.push_robot(instructions, glm::vec2(0.0, 9.0));
        simulation.push_robot(instructions, glm::vec2(0.0, 10.0));
        simulation.push_robot(instructions, glm::vec2(0.0, 11.0));
        simulation.push_robot(instructions, glm::vec2(0.0, 12.0));
        simulation.push_robot(instructions, glm::vec2(0.0, 13.0));
        simulation.push_robot(instructions, glm::vec2(0.0, 14.0));
        simulation.push_robot(instructions, glm::vec2(0.0, 15.0));

        let render = RenderState::new(simulation.get_logic_tx());

        Self {
            window,
            renderer,
            gl_context,
            gl_surface,
            fence: None,
            mono: Font::from_bytes(include_bytes!("../res/fonts/ubuntu_mono.ttf")).unwrap(),
            ui,
            w: false, a: false, s: false, d: false,
            simulation,
            render,
            render_rx,
        }
    }

    fn event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        match &event {
            WindowEvent::CloseRequested => {
                self.simulation.stop();
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::Resized(new_size) => {
                self.ui.update_window_size(glm::vec2(
                    new_size.width as f32,
                    new_size.height as f32,
                ));
                if let (Some(w), Some(h)) = (NonZeroU32::new(new_size.width), NonZeroU32::new(new_size.height)) {
                    self.gl_surface.resize(&self.gl_context, w, h);
                }
                unsafe { gl::Viewport(0, 0, new_size.width as i32, new_size.height as i32); }
            },
            WindowEvent::MouseInput { state, .. } => self.ui.update_mouse_state(*state),
            WindowEvent::CursorMoved { position, .. } => {
                let pos_f32 = glm::vec2(position.x as f32, position.y as f32);
                self.ui.update_mouse_position(pos_f32);
            },
            WindowEvent::KeyboardInput { event, .. } => {
                if event.repeat { return; }

                let new_value = event.state == ElementState::Pressed;
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => self.w = new_value,
                    PhysicalKey::Code(KeyCode::KeyA) => self.a = new_value,
                    PhysicalKey::Code(KeyCode::KeyS) => self.s = new_value,
                    PhysicalKey::Code(KeyCode::KeyD) => self.d = new_value,
                    _ => {},
                }

                match (event.state, &event.logical_key) {
                    (ElementState::Released, Key::Named(NamedKey::F11)) => {
                        if self.window.fullscreen().is_some() {
                            self.window.set_fullscreen(None);
                        } else if let Some(monitor) = self.window.current_monitor() {
                            self.window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(Some(monitor))));
                        }
                    },
                    (ElementState::Released, Key::Named(NamedKey::Escape)) =>
                        event_loop.exit(),
                    _ => {},
                }
            },
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    MouseScrollDelta::LineDelta(_, y) =>
                        self.ui.camera.zoom *= (y * 0.05).exp(),
                    _ => {},
                }
            },
            _ => (),
        }
    }

    fn get_window_title() -> &'static str {
        "Automation game"
    }

    fn get_window_size() -> glm::Vec2 {
        glm::vec2(800.0, 600.0)
    }
}

const fn bool_to_f32(v: bool) -> f32 {
    if v { 1.0 } else { 0.0 }
}

impl State {
    fn redraw(&mut self) {
        if let Some(fence) = &self.fence {
            fence.wait();
        }

        self.renderer.simple_shape_renderer.reset();
        self.renderer.font_renderer.reset();
        self.mono.reset();

        {
            let delta = self.ui.get_delta();
            let camera = &mut self.ui.camera;
            let camera_move_vec = (
                bool_to_f32(self.w) * glm::vec2( 0.0,  1.0) + 
                bool_to_f32(self.a) * glm::vec2(-1.0,  0.0) + 
                bool_to_f32(self.s) * glm::vec2( 0.0, -1.0) + 
                bool_to_f32(self.d) * glm::vec2( 1.0,  0.0)) / camera.zoom * delta;

            camera.position += camera_move_vec;
        }

        let ndc_matrix = self.ui.get_projection_matrix();
        let world_matrix = self.ui.camera.get_vp_matrix(&ndc_matrix);

        let physics_state = self.simulation.physics.acquire();
        let logic_state = self.simulation.logic.acquire();
        self.render.update(
            &mut self.ui,
            &*physics_state,
            &*logic_state,
            &mut self.renderer,
            &mut self.mono,
            &self.render_rx,
        );

        self.renderer.clear(&[ClearField::Color, ClearField::Depth]);

        self.render.draw(&world_matrix, &mut self.renderer);
        
        self.renderer.enable(Capability::DepthTest);
        self.renderer.simple_shape_renderer.draw(0, &world_matrix);

        self.renderer.disable(Capability::DepthTest);
        self.renderer.font_renderer.draw(&world_matrix, &mut self.mono, 0);

        self.renderer.simple_shape_renderer.draw(1, &ndc_matrix);    
        self.renderer.font_renderer.draw(&ndc_matrix, &mut self.mono, 1);    

        self.fence = self.renderer.new_fence();
        self.gl_surface.swap_buffers(&self.gl_context).unwrap();
        self.window.request_redraw();

        self.ui.on_frame_end(&self.window);
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::<State>::new().unwrap();
    event_loop.run_app(&mut app).unwrap();
}