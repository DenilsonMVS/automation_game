
use nalgebra_glm::{self as glm};

use crate::{game::{compute_context::ComputeContext, computer::{Computer, MEMORY_SIZE, X}, display_computer::DisplayComputer, instruction::{Instruction, Syscall}, mining_computer::MiningComputer, movement_computer::MovementComputer, world::{GenericChunk, GenericWorld}}, graphics::{camera::Camera, renderer::{fonts::Font, positioning::PositionMode, simple_shape_renderer::SimpleShapeRenderer}}};


const ROBOT_SIZE: f32 = 0.8;
const WEEL_DIAMETER: f32 = 0.26;
const WEEL_DISTANCE: f32 = 0.37;
const ROBOT_DEPTH: u16 = 5000;
const WEELS_DEPTH: u16 = ROBOT_DEPTH + 1;
const WEEL_COLOR: [u8; 3] = [45, 40, 40];
const EPS: f32 = 0.0001;

const MAX_ACCELLERATION: f32 = 1.0;
const DRAG: f32 = 1.0;

#[derive(Debug)]
pub struct Robot {
    movement_computer: MovementComputer,
    mining_computer: MiningComputer,
    display_computer: DisplayComputer,
    computer: Computer,
    color: glm::U8Vec3,
    context: ComputeContext,
}

impl Robot {
    pub fn new(position: glm::Vec2, instructions: &[Instruction; MEMORY_SIZE]) -> Self {
        Self {
            computer: Computer::default().with_instructions(instructions),
            movement_computer: MovementComputer::new(position),
            mining_computer: MiningComputer::new(16),
            color: glm::vec3(120, 200, 190),
            context: ComputeContext::Computer,
            display_computer: DisplayComputer::new(),
        }
    }

    pub fn set_instruction(mut self, idx: usize, instruction: Instruction) -> Self {
        self.computer.set_instruction(idx, instruction);
        return self;
    }

    // pub fn draw_vec(&self, simple_shape_renderer: &mut SimpleShapeRenderer, vec: glm::Vec2, color: glm::U8Vec4) {
    //     let size = glm::length(&vec);
    //     simple_shape_renderer.send_rotated_rectangle(
    //         self.position,
    //         glm::vec2(0.5, 0.04) * size,
    //         PositionMode::CenterLeft,
    //         glm::vec2(-vec.y, vec.x),
    //         color, 2000, 0);
        
    //     simple_shape_renderer.send_symmetrical_triangle(
    //         self.position + vec * 0.6,
    //         self.position + vec * 0.5,
    //         size * 0.1,
    //         color, 2000, 0);
    // }

    // pub fn draw_compute_info(&self, font: &mut Font, simple_shape_renderer: &mut SimpleShapeRenderer) {
    //     self.instructions.send_draw(
    //         glm::vec2(1.0, 0.01),
    //         glm::vec2(0.98, 0.78), 
    //         PositionMode::BottomCenter,
    //         font, simple_shape_renderer,
    //         Some(self.memory.read_memory(PROGRAM_COUNTER_IDX) as usize),
    //         false,
    //     );

    //     self.memory.send_draw(
    //         glm::vec2(1.0, -0.05),
    //         glm::vec2(0.9, 0.7),
    //         PositionMode::TopCenter,
    //         font, simple_shape_renderer,
    //         4,
    //     );

    //     let code_background = glm::vec4(10, 10, 20, 255);
    //     simple_shape_renderer.send_rectangle(
    //         glm::vec2(1.0, 0.0),
    //         glm::vec2(1.0, 1.6),
    //         PositionMode::Center,
    //         code_background,
    //         50001, 1,
    //     );
    // }


    pub fn draw(&self, simple_shape_renderer: &mut SimpleShapeRenderer, font: &mut Font, camera: &Camera) {
        // self.draw_vec(simple_shape_renderer, self.acc, glm::make_vec4(&COLOR_DATA[0]));
        // self.draw_vec(simple_shape_renderer, self.drag, glm::make_vec4(&COLOR_DATA[1]));

        let weel_offsets = [
            glm::vec2(-1.0, -1.0),
            glm::vec2( 1.0, -1.0),
            glm::vec2( 1.0,  1.0),
            glm::vec2(-1.0,  1.0),
        ];

        for weel_offset in weel_offsets {
            simple_shape_renderer.send_ellipse(
                self.movement_computer.get_position()  + weel_offset * WEEL_DISTANCE,
                glm::vec2(WEEL_DIAMETER, WEEL_DIAMETER),
                PositionMode::Center,
                glm::vec4(WEEL_COLOR[0], WEEL_COLOR[1], WEEL_COLOR[2], 255),
                WEELS_DEPTH, 0
            );
        }

        simple_shape_renderer.send_rectangle(
            self.movement_computer.get_position(),
            glm::vec2(ROBOT_SIZE, ROBOT_SIZE),
            PositionMode::Center,
            glm::vec4(self.color.x, self.color.y, self.color.z, 255),
            ROBOT_DEPTH, 0
        );

        self.display_computer.send(font, self.movement_computer.get_position(), camera);
    }

    pub fn update<C: GenericChunk, W: GenericWorld<C>>(&mut self, delta: f32, is_instruction_tick: bool, world : &mut W) {
        match self.context {
            ComputeContext::Computer => {
                if is_instruction_tick {
                    let comp = self.computer.run_computer();
                    if let Some(sys) = comp {
                        match sys {
                            Syscall::MoveBottom | Syscall::MoveLeft | Syscall::MoveRight | Syscall::MoveTop => {
                                self.context = ComputeContext::MovementComputer;
                                self.movement_computer.receive_syscall(sys);
                            },
                            Syscall::Mine | Syscall::Stored | Syscall::MaxStorage | Syscall::Scan => {
                                self.context = ComputeContext::MiningComputer;
                                self.mining_computer.receive_syscall(sys);
                            },
                            Syscall::Display => {
                                self.context = ComputeContext::DisplayComputer;
                                self.display_computer.receive_syscall(
                                    if self.computer.read_reg(X[14]) != 0 {
                                        Some(self.computer.read_reg(X[15]))
                                    } else {
                                        None
                                    }
                                );
                            },
                            _ => {}
                        }
                    }
                }
            },
            ComputeContext::MovementComputer => {
                if let Some(ret) = self.movement_computer.update() {
                    self.context = ComputeContext::Computer;
                    self.computer.write_reg(X[15], ret);
                }
            },
            ComputeContext::MiningComputer => {
                if let Some(ret) = self.mining_computer.update(
                    self.movement_computer.get_position(),
                    world, delta, is_instruction_tick,
                ) {
                    self.context = ComputeContext::Computer;
                    self.computer.write_reg(X[15], ret);
                }
            },
            ComputeContext::DisplayComputer => {
                if let Some(ret) = self.display_computer.update(is_instruction_tick) {
                    self.context = ComputeContext::Computer;
                    self.computer.write_reg(X[15], ret);
                }
            }
        };

        self.movement_logic(delta, self.movement_computer.calculate_accelleration());
    }

    fn movement_logic(&mut self, delta: f32, acc: glm::Vec2) {
        let sq = |x| x * x;

        let acc_mag = glm::length(&acc);
        let acc_adjust = acc_mag.min(MAX_ACCELLERATION);
        let acc = acc / (acc_mag + EPS) * acc_adjust;

        let position = self.movement_computer.get_position();
        let speed = self.movement_computer.get_speed();

        let speed_mag = glm::length(&speed);
        let norm_speed = speed / (speed_mag + EPS);
        let drag_vector = -norm_speed * sq(speed_mag) * DRAG;

        let all_acc = acc + drag_vector;

        let next_position = position + speed * delta + all_acc * sq(delta) * 0.5;
        let next_speed = speed + all_acc * delta;

        self.movement_computer.update_sensor(next_position, next_speed);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::game::{instruction::{Instruction, Syscall}, world::GridPosition};

    pub struct MockChunk {
        pub local_grid: HashMap<glm::IVec2, GridPosition>,
    }

    impl MockChunk {
        pub fn new() -> Self {
            Self {
                local_grid: HashMap::new(),
            }
        }
    }

    impl GenericChunk for MockChunk {
        fn get_position(&self, global_position: glm::IVec2) -> GridPosition {
            // Retorna o que estiver no mapa ou None por padrão
            *self.local_grid.get(&global_position).unwrap_or(&GridPosition::None)
        }

        fn get_position_mut(&mut self, global_position: glm::IVec2) -> &mut GridPosition {
            // or_insert garante que temos uma referência mutável válida para retornar
            self.local_grid.entry(global_position).or_insert(GridPosition::None)
        }
    }

    struct MockWorld {
        pub chunks: HashMap<glm::IVec2, MockChunk>,
        pub mining_log: Vec<glm::IVec2>,
    }

    impl MockWorld {
        fn new() -> Self {
            Self { 
                chunks: HashMap::new(), 
                mining_log: Vec::new() 
            }
        }

        // Helper para facilitar o setup do teste
        fn add_resource(&mut self, pos: glm::IVec2, amount: u16) {
            let chunk_id = pos.map(|x| x.div_euclid(256)); // Usando o CHUNK_SIZE fixo de 256
            let chunk = self.chunks.entry(chunk_id).or_insert(MockChunk::new());
            chunk.local_grid.insert(pos, GridPosition::Resource(amount, 0));
        }
    }

    impl GenericWorld<MockChunk> for MockWorld {
        fn get_position(&self, pos: glm::IVec2) -> Option<GridPosition> {
            let chunk_id = pos.map(|x| x.div_euclid(256));
            self.chunks.get(&chunk_id).map(|c| c.get_position(pos))
        }

        fn try_mine(&mut self, pos: glm::IVec2) -> bool {
            let chunk_id = pos.map(|x| x.div_euclid(256));
            if let Some(chunk) = self.chunks.get_mut(&chunk_id) {
                let grid_pos = chunk.get_position_mut(pos);
                if let GridPosition::Resource(qty, _) = grid_pos {
                    if *qty > 0 {
                        *qty -= 1;
                        self.mining_log.push(pos);
                        if *qty == 0 { *grid_pos = GridPosition::None; }
                        return true;
                    }
                }
            }
            false
        }

        fn get_chunk(&self, pos: glm::IVec2) -> Option<&MockChunk> {
            let chunk_id = pos.map(|x| x.div_euclid(256));
            self.chunks.get(&chunk_id)
        }

        fn get_chunk_mut(&mut self, pos: glm::IVec2) -> Option<&mut MockChunk> {
            let chunk_id = pos.map(|x| x.div_euclid(256));
            self.chunks.get_mut(&chunk_id)
        }

        fn get_position_mut(&mut self, pos: glm::IVec2) -> Option<&mut GridPosition> {
            let chunk_id = pos.map(|x| x.div_euclid(256));
            self.chunks.get_mut(&chunk_id).map(|c| c.get_position_mut(pos))
        }
    }

    #[test]
    fn test_robot_context_switching_movement() {
        // Programa: Mover para a Direita
        let instructions = [
            Instruction::new_sys(Syscall::MoveRight),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
        ];
        
        let mut robot = Robot::new(glm::vec2(0.0, 0.0), &instructions);
        let mut world = MockWorld::new();

        // 1. No primeiro tick de instrução, o robô deve ler MoveRight e mudar o contexto
        robot.update(0.016, true, &mut world);
        assert_eq!(robot.context, ComputeContext::MovementComputer);

        // 2. Em updates subsequentes (sem tick de instrução), ele deve permanecer no MovementComputer
        robot.update(0.016, false, &mut world);
        assert_eq!(robot.context, ComputeContext::MovementComputer);

        // 3. Simulamos que o MovementComputer atingiu o alvo
        // Para isso, forçamos a posição do sensor para o alvo
        robot.movement_computer.update_sensor(glm::vec2(1.0, 0.0), glm::vec2(0.0, 0.0));
        
        // 4. No próximo update, ele deve detectar a conclusão, voltar para o Computer e escrever 1 em X15
        robot.update(0.016, false, &mut world);
        assert_eq!(robot.context, ComputeContext::Computer);
        assert_eq!(robot.computer.read_reg(X[15]), 1);
    }

    #[test]
    fn test_simple_mine_integration() {
        let mut world = MockWorld::new();
        let pos = glm::vec2(0, 0);
        world.add_resource(pos, 5); // Recurso disponível

        // Programa: Minerar e depois ficar em NOP
        let instructions = [
            Instruction::new_sys(Syscall::Mine),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
        ];

        let mut robot = Robot::new(glm::vec2(0.0, 0.0), &instructions);

        // --- PASSO 1: O Disparo ---
        // Executa a instrução no PC 0. O robô deve entrar em modo de mineração.
        robot.update(0.0, true, &mut world);
        assert_eq!(robot.context, ComputeContext::MiningComputer);

        // --- PASSO 2: A Espera ---
        // Passam-se 5 segundos. O robô ainda deve estar minerando (None no X15).
        robot.update(5.0, false, &mut world);
        assert_eq!(robot.context, ComputeContext::MiningComputer);
        assert_eq!(robot.computer.read_reg(X[15]), 0); // Ainda não escreveu o sucesso

        // --- PASSO 3: A Conclusão ---
        // Passam-se mais 5.1 segundos (Total > 10.0). 
        // O robô deve terminar, voltar para o Computer e registrar Sucesso (1).
        robot.update(5.1, false, &mut world);
        
        assert_eq!(robot.context, ComputeContext::Computer);
        assert_eq!(robot.computer.read_reg(X[15]), 1); // Sucesso!
        assert_eq!(robot.mining_computer.get_stored(), 1);   // Inventário subiu
        
        // O PC deve ter avançado para 1 após a syscall terminar
        assert_eq!(robot.computer.read_reg(X[1]), 1);
    }

    #[test]
    fn test_border_scout_integration() {
        let mut world = MockWorld::new();
        
        // Colocamos um recurso com valor 123 no primeiro bloco do Chunk (1, 0)
        // Coordenada global 256 (CHUNK_SIZE)
        world.add_resource(glm::vec2(256, 0), 123);

        // Programa:
        // PC 0: Mover para a Direita (para entrar no novo chunk)
        // PC 1: Escanear o chão
        // PC 2: Mostrar no Display (precisa de x14 != 0 para ativar)
        let mut instructions = [Instruction::default(); 16];
        instructions[0] = Instruction::new_sys(Syscall::MoveRight);
        instructions[1] = Instruction::new_sys(Syscall::Scan);
        instructions[2] = Instruction::new_sys(Syscall::Display);

        // Robô começa no último bloco do Chunk (0, 0), na posição 255
        let mut robot = Robot::new(glm::vec2(255.0, 0.0), &instructions);
        
        // Ativamos o flag de display no registrador x14 para a syscall Display funcionar
        robot.computer.write_reg(X[14], 1);

        // --- PASSO 1: MOVIMENTO ---
        robot.update(0.0, true, &mut world); // PC 0: MoveRight
        assert_eq!(robot.context, ComputeContext::MovementComputer);
        
        // Simulamos a chegada física no bloco 256
        robot.movement_computer.update_sensor(glm::vec2(256.0, 0.0), glm::vec2(0.0, 0.0));
        robot.update(0.1, false, &mut world); // Robot detecta fim do movimento
        assert_eq!(robot.context, ComputeContext::Computer);
        assert_eq!(robot.computer.read_reg(X[1]), 1); // PC avançou para 1

        // --- PASSO 2: SCAN ---
        robot.update(0.0, true, &mut world); // PC 1: Scan
        assert_eq!(robot.context, ComputeContext::MiningComputer);
        
        // Passamos o tempo de scan (1.0s)
        robot.update(1.1, false, &mut world); 
        assert_eq!(robot.context, ComputeContext::Computer);
        
        // VALIDAÇÃO: O robô leu o valor 123 do Chunk vizinho?
        assert_eq!(robot.computer.read_reg(X[15]), 123);

        // --- PASSO 3: DISPLAY ---
        robot.update(0.0, true, &mut world); // PC 2: Display
        assert_eq!(robot.context, ComputeContext::DisplayComputer);
        
        // O DisplayComputer precisa de um tick de instrução para "travar" o valor
        robot.update(0.0, true, &mut world);
        assert_eq!(robot.context, ComputeContext::Computer);
        
        println!("Teste de Escaneador de Fronteira concluído com sucesso!");
    }

    #[test]
    fn test_complex_mining_logic_with_display() {
        let mut world = MockWorld::new();
        let pos_start = glm::vec2(0, 0);
        
        // Colocamos 2 unidades de recurso no (0,0)
        world.add_resource(pos_start, 2); 

        let instructions = [
            /* 0 */ Instruction::new_lui(X[14], 1),             // Ativa flag de Display
            /* 1 */ Instruction::new_sys(Syscall::Display),    // Mostra 0 inicial
            /* 2 */ Instruction::new_sys(Syscall::Scan),       // x15 = 2
            /* 3 */ Instruction::new_addi(X[15], X[3], 0),      // x3 = x15 (2)
            /* 4 */ Instruction::new_beq(X[3], X[0], 6),       // Se vazio, pula p/ PC 11
            /* 5 */ Instruction::new_sys(Syscall::Mine),       // Minera -> x15 = 1
            /* 6 */ Instruction::new_sub(X[3], X[15], X[3]),   // x3 = x3 - x15 (2 - 1 = 1)
            /* 7 */ Instruction::new_add(X[2], X[15], X[2]),   // x2 = x2 + x15 (Total Minerado: 1)
            /* 8 */ Instruction::new_addi(X[2], X[15], 0),     // x15 = x2 (Para o display)
            /* 9 */ Instruction::new_sys(Syscall::Display),    // Mostra Total Minerado
            /* 10*/ Instruction::new_bne(X[3], X[0], 9),       // Se x3 != 0, pula 9: (10+1+9)=20 -> PC 4
            /* 11*/ Instruction::new_sys(Syscall::MoveRight),  // Se vazio, move
            /* 12*/ Instruction::new_lui(X[1], 1),             // PC = 1 (LUI coloca 1 no PC, vira 2 no final)
            /* 13*/ Instruction::new_sys(Syscall::Nop),
            /* 14*/ Instruction::new_sys(Syscall::Nop),
            /* 15*/ Instruction::new_sys(Syscall::Nop),
        ];

        let mut robot = Robot::new(glm::vec2(0.0, 0.0), &instructions);

        // --- EXECUÇÃO PASSO A PASSO ---

        // PC 0 e 1: Setup Display
        robot.update(0.0, true, &mut world); // LUI
        robot.update(0.0, true, &mut world); // Display Syscall
        robot.update(0.0, true, &mut world); // Fecha Display (DisplayComputer::update(true))

        // PC 2: Scan (1.0s)
        robot.update(1.0, true, &mut world);
        robot.update(1.0, false, &mut world); 
        assert_eq!(robot.computer.read_reg(X[15]), 2); // Detectou 2 recursos

        // PC 3 e 4: Backup e Check
        robot.update(0.0, true, &mut world); // x3 = 2
        robot.update(0.0, true, &mut world); // BEQ (não pula)

        // PC 5: Mine (10.0s)
        robot.update(10.0, true, &mut world);
        robot.update(10.0, false, &mut world);
        assert_eq!(world.mining_log.len(), 1);

        // PC 6, 7, 8: Atualiza contadores
        for _ in 0..3 { robot.update(0.0, true, &mut world); }
        assert_eq!(robot.computer.read_reg(X[3]), 1); // Restante no chão: 1
        assert_eq!(robot.computer.read_reg(X[2]), 1); // Total no inv: 1

        // PC 9: Display (Mostra o 1 minerado)
        robot.update(0.0, true, &mut world); // Syscall
        robot.update(0.0, true, &mut world); // Fecha Display

        // PC 10: BNE (x3 é 1, então pula 9) -> Volta para o PC 4 (Check de recurso)
        robot.update(0.0, true, &mut world);
        assert_eq!(robot.computer.read_reg(X[1]) & 0xF, 4);

        // --- SEGUNDA RODADA DE MINERAÇÃO ---
        // Repete PC 4, 5, 6, 7, 8, 9...
        robot.update(0.0, true, &mut world); // PC 4
        robot.update(10.0, true, &mut world); robot.update(10.0, false, &mut world); // PC 5: Mine 2
        for _ in 0..4 { robot.update(0.0, true, &mut world); } // PC 6, 7, 8, 9 (Display)
        robot.update(0.0, true, &mut world); // Fecha Display do PC 9
        
        assert_eq!(world.mining_log.len(), 2);
        assert_eq!(robot.computer.read_reg(X[3]), 0); // x3 agora é 0

        // PC 10: BNE (x3 é 0, NÃO pula)
        robot.update(0.0, true, &mut world);
        assert_eq!(robot.computer.read_reg(X[1]) & 0xF, 11); // Segue para o próximo: PC 11

        // PC 11: MoveRight
        robot.update(0.0, true, &mut world);
        assert_eq!(robot.context, ComputeContext::MovementComputer);
        
        println!("Integração completa: Robô minerou tudo, atualizou display e moveu-se.");
    }

    #[test]
    fn test_robot_integration_stepped_delta() {
        let mut world = MockWorld::new();
        let pos_a = glm::vec2(0, 0);
        world.add_resource(pos_a, 2); 

        let instructions = [
            /* 0 */ Instruction::new_lui(X[14], 1),
            /* 1 */ Instruction::new_sys(Syscall::Display),
            /* 2 */ Instruction::new_sys(Syscall::Scan),
            /* 3 */ Instruction::new_addi(X[15], X[3], 0),
            /* 4 */ Instruction::new_beq(X[3], X[0], 6),
            /* 5 */ Instruction::new_sys(Syscall::Mine),
            /* 6 */ Instruction::new_sub(X[3], X[15], X[3]),
            /* 7 */ Instruction::new_add(X[2], X[15], X[2]),
            /* 8 */ Instruction::new_addi(X[2], X[15], 0),
            /* 9 */ Instruction::new_sys(Syscall::Display),
            /* 10*/ Instruction::new_bne(X[3], X[0], 9),
            /* 11*/ Instruction::new_sys(Syscall::MoveRight),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
            Instruction::new_sys(Syscall::Nop),
        ];

        let mut robot = Robot::new(glm::vec2(0.0, 0.0), &instructions);
        let dt = 0.1;

        let step_until_computer = |robot: &mut Robot, world: &mut MockWorld| {
            let mut elapsed = 0.0;
            while robot.context != ComputeContext::Computer {
                robot.update(dt, false, world);
                elapsed += dt;
                if elapsed > 15.0 { panic!("Timeout: Robô preso em {:?}", robot.context); }
            }
            elapsed
        };

        // --- EXECUÇÃO ---

        // 1. Setup
        robot.update(dt, true, &mut world); // LUI
        robot.update(dt, true, &mut world); // Display Syscall
        robot.update(dt, true, &mut world); // Fecha Display

        // 2. Scan (PC 2) 
        robot.update(dt, true, &mut world); // O delta aqui foi gasto na CPU
        assert_eq!(robot.context, ComputeContext::MiningComputer);
        
        let time_taken = step_until_computer(&mut robot, &mut world);
        
        // Agora esperamos 1.0s cheio, com margem para erro de f32
        assert!((time_taken - 1.0).abs() < 0.001, "Scan levou {}s, esperado 1.0s", time_taken);
        assert_eq!(robot.computer.read_reg(X[15]), 2);

        // 3. Logica (PC 3, 4)
        robot.update(dt, true, &mut world); 
        robot.update(dt, true, &mut world); 

        // 4. Mineração (PC 5)
        robot.update(dt, true, &mut world); // Delta gasto na CPU
        assert_eq!(robot.context, ComputeContext::MiningComputer);
        
        let time_taken = step_until_computer(&mut robot, &mut world);
        
        // Esperamos 10.0s cheio
        assert!((time_taken - 10.0).abs() < 0.001, "Mine levou {}s, esperado 10.0s", time_taken);
        assert_eq!(world.mining_log.len(), 1);

        // 5. Verificações Finais
        for _ in 0..5 { robot.update(dt, true, &mut world); } 
        robot.update(dt, true, &mut world); 
        
        assert_eq!(robot.computer.read_reg(X[2]), 1);
        assert_eq!(robot.computer.read_reg(X[1]) & 0xF, 4);
    }
}
