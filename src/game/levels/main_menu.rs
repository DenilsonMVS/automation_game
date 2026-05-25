use std::{array, collections::{HashMap, HashSet}};

use crate::{MainContext, game::{logic::{GameState, computer::{PC, X0, X2, X3, X4, X15}, fabricator::{Fabricator, InputOutputDirection}, instruction::{Instruction, Syscall}, robot::Robot, target::Targets, world::Chunk}, render::{RenderState, UI_BASE_DEPTH, robot_info::{get_button_bgcolor, get_button_color}}, scene::Scene}, graphics::{camera::Camera, renderer::{Renderer, positioning::{PositionMode, RenderBox}}}, open_simplex::OpenSimplex, stable_index_vector::StableIndexVector, ui::{HasUi, events::ActionType, rendered_button::{Content, RenderedButton}, ui_element::UiElement}};
use nalgebra_glm as glm;

fn create_main_menu_back_view() -> GameState {
    let noise = OpenSimplex::new(0);
    let grid = Chunk::proc_generated(glm::vec2(0, 0), &noise);

    /*
    x3 = MaxStorage;
    do {
        do {
            move_right();
            x2++;
            x15 = scan();
        } while (x15 == 0);

        do {
            mine();
            x4++;
        } while (x4 != x3);

        do {
            move_left();
            x2--;
        } while (x2 != 0);

        do {
            release();
            x4--;
        } while (x4 != 0);
    } while (1);
    */

    let mining_code = [
        Instruction::new_sys(Syscall::MaxStorage),
        Instruction::new_addi(X3, X15, 0),

        Instruction::new_sys(Syscall::MoveRight),
        Instruction::new_addi(X2, X2, 1),

        Instruction::new_sys(Syscall::Scan),
        Instruction::new_beq(X15, X0, 12),
 
        Instruction::new_sys(Syscall::Mine),
        Instruction::new_addi(X4, X4, 1),
        Instruction::new_bne(X4, X3, 11),

        Instruction::new_sys(Syscall::MoveLeft),
        Instruction::new_subi(X2, X2, 1),
        Instruction::new_bne(X2, X0, 13),

        Instruction::new_sys(Syscall::Release),
        Instruction::new_subi(X4, X4, 1),
        Instruction::new_bne(X4, X0, 13),

        Instruction::new_beq(X0, X0, 2),
    ];

    let run_code = std::array::from_fn(|i| {
        match i {
            0 => Instruction::new_sys(Syscall::MoveLeft),
            1 => Instruction::new_lui(PC, 15),
            _ => Instruction::default(),
        }
    });

    let mut robots = StableIndexVector::with_capacity(15);
    for i in 0..21 {
        robots.push(Robot::new(
            glm::vec2(100.0, 90.0 + i as f32),
            mining_code,
        ));
    }

    let fabricators = (0..21)
        .map(|i|
            Fabricator::new(
                glm::vec2(99, 90 + i),
                InputOutputDirection::RightLeft,
            ).with_code(run_code))
        .collect();

    let chunk = Chunk::new(grid, fabricators, robots, Targets::new(HashSet::new()));

    let mut chunks = HashMap::new();
    chunks.insert(glm::vec2(0, 0), chunk);
    GameState::new(chunks)
}

pub struct MainMenu<'a> {
    pub logic: GameState,
    pub render: RenderState<'a>,
    pub buttons: [RenderedButton; 4],
}

impl<'a> MainMenu<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        let logic = create_main_menu_back_view();
        let render = RenderState::new(renderer, &logic);
        let contents = ["1", "2", "3", "4"];
        return Self {
            logic, render,
            buttons: array::from_fn(|i| RenderedButton::new(get_button_bgcolor()).with_content(Content::Str(contents[i]), get_button_color())),
        };
    }

    fn get_button_bbox(idx: usize) -> RenderBox {
        let offset = glm::vec2(0.5, -0.5);
        let i = (idx / 2) as f32;
        let j = (idx % 2) as f32;
        let offset = glm::vec2(offset.x * j, offset.y * i);
        RenderBox::new_simple(
            glm::vec2(-0.25, 0.25) + offset,
            PositionMode::Center,
            glm::vec2(0.45, 0.45),
        )
    }
}

fn get_camera() -> Camera {
    Camera::new(glm::vec2(99.0, 100.0), 0.1)
}

impl<'a> Scene for MainMenu<'a> {
    fn game_logic(&mut self, context: &mut MainContext) {
        let mut camera = get_camera();
        std::mem::swap(&mut context.ui.camera, &mut camera);

        self.logic.update(context);
        self.render.update(&mut self.logic, context, false);

        for (i, button) in self.buttons.iter_mut().enumerate() {

            if matches!(context.ui.events.mouse_actions, ActionType::AreaPress(id, _) if id == button.id) {
                match i {
                    0 => *context.next_scene = Some(super::SceneOptionId::First),
                    _ => todo!(),
                }
            }

            button.disabled = i as u8 > context.save.get_last_finished_level();
            button.update(
                &Self::get_button_bbox(i),
                UI_BASE_DEPTH,
                context.get_ui(),
            );
        }

        std::mem::swap(&mut context.ui.camera, &mut camera);
    }

    fn send(&mut self, context: &mut crate::MainContext) {
        let mut camera = get_camera();
        std::mem::swap(&mut context.ui.camera, &mut camera);

        self.render.send(&self.logic, context);
        for button in self.buttons.iter_mut() {
            button.send(&mut context.get_button_context());
        }

        std::mem::swap(&mut context.ui.camera, &mut camera);
    }

    fn generate_output(&mut self, context: &mut crate::MainContext) {
        let camera = get_camera();
        // let camera = context.ui.camera;
        self.render.draw(&camera.get_vp_matrix(&context.ui.get_projection_matrix()));
    }
}
