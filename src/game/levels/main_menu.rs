use crate::{game::logic::{GameState, computer::{PC, X0, X2, X3, X4, X15}, fabricator::Fabricator, instruction::{Instruction, Syscall}, robot::Robot, world::{Chunk, World}}, open_simplex::OpenSimplex, stable_index_vector::StableIndexVector};
use nalgebra_glm as glm;
use crate::game::logic::fabricator::Direction;

pub fn create_main_menu_back_view() -> GameState {
    let noise = OpenSimplex::new(0);
    let chunk = Chunk::new_proc_generated(glm::vec2(0, 0), &noise);

    let mut world = World::new();
    world.set_chunk(glm::vec2(0, 0), chunk);

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
                Direction::Right,
                Direction::Left,
            ).with_code(run_code))
        .collect();

    GameState::new(world, fabricators, robots)
}
