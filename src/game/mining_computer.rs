
use crate::game::{instruction::Syscall, world::{GenericChunk, GenericWorld, GridPosition}};
use nalgebra_glm as glm;

const MINING_TIME: f32 = 10.0;
const SCAN_TIME: f32 = 1.0;

#[derive(Debug, Clone, Copy)]
pub struct MiningComputer {
    syscall: Syscall,
    stored: u8,
    max_storage: u8,
    remaining_time: f32,
}

impl MiningComputer {
    pub fn new(max_storage: u8) -> Self {
        Self { stored: 0, max_storage, syscall: Syscall::Nop, remaining_time: 0.0, }
    }

    pub fn get_stored(&self) -> u8 {
        self.stored
    }

    pub fn receive_syscall(&mut self, syscall: Syscall) {
        self.syscall = syscall;
        match self.syscall {
            Syscall::Mine => self.remaining_time = MINING_TIME,
            Syscall::Scan => self.remaining_time = SCAN_TIME,
            _ => {},
        }
    }

    pub fn update<C: GenericChunk, T: GenericWorld<C>>(&mut self, position: glm::Vec2, world: &mut T, delta: f32, is_instruction_tick: bool) -> Option<u8> {
        match self.syscall {
            Syscall::Mine => {
                if self.stored == self.max_storage {
                    is_instruction_tick.then_some(0)
                } else if delta >= self.remaining_time {
                    self.remaining_time = 0.0;
                    if world.try_mine(position.map(|x| x.round() as i32)) {
                        self.stored += 1;
                        Some(1)
                    } else {
                        Some(0)
                    }
                } else {
                    self.remaining_time -= delta;
                    None
                }
            },
            Syscall::MaxStorage => is_instruction_tick.then_some(self.max_storage),
            Syscall::Stored => is_instruction_tick.then_some(self.stored),
            Syscall::Scan => {
                if delta >= self.remaining_time {
                    self.remaining_time = 0.0;
                    let position = 
                        world.get_position(position.map(|x| x.round() as i32))
                        .unwrap_or(super::world::GridPosition::None);
                    match position {
                        GridPosition::None => Some(0),
                        GridPosition::Resource(amount, _) => Some(amount as u8),
                    }
                } else {
                    self.remaining_time -= delta;
                    None
                }
            },
            _ => None,
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::world::{GenericChunk, GenericWorld, GridPosition};

    // --- MOCKS PARA TESTE ---

    struct MockChunk;
    impl GenericChunk for MockChunk {
        fn get_position(&self, _: glm::IVec2) -> GridPosition { GridPosition::None }
        fn get_position_mut(&mut self, _: glm::IVec2) -> &mut GridPosition { 
            unreachable!("Mocks simples não precisam de mutabilidade interna") 
        }
    }

    struct MockWorld {
        pub resource_at_pos: GridPosition,
        pub mine_success: bool,
        pub mine_called: bool,
    }

    impl MockWorld {
        fn new() -> Self {
            Self {
                resource_at_pos: GridPosition::None,
                mine_success: true,
                mine_called: false,
            }
        }
    }

    impl GenericWorld<MockChunk> for MockWorld {
        fn get_position(&self, _: glm::IVec2) -> Option<GridPosition> {
            Some(self.resource_at_pos)
        }
        fn try_mine(&mut self, _: glm::IVec2) -> bool {
            self.mine_called = true;
            self.mine_success
        }
        // Métodos obrigatórios pela trait mas não usados pelo MiningComputer atual
        fn get_position_mut(&mut self, _: glm::IVec2) -> Option<&mut GridPosition> { None }
        fn get_chunk(&self, _: glm::IVec2) -> Option<&MockChunk> { None }
        fn get_chunk_mut(&mut self, _: glm::IVec2) -> Option<&mut MockChunk> { None }
    }

    // --- TESTES UNITÁRIOS ---

    #[test]
    fn test_mining_duration_and_completion() {
        let mut mc = MiningComputer::new(10);
        let mut world = MockWorld::new();
        let pos = glm::vec2(0.0, 0.0);

        mc.receive_syscall(Syscall::Mine);

        // Teste: 5 segundos se passaram (metade do tempo)
        // O delta ainda é menor que remaining_time (10.0), deve retornar None
        let res = mc.update(pos, &mut world, 5.0, false);
        assert_eq!(res, None);
        assert!(!world.mine_called);

        // Teste: Mais 5.1 segundos se passaram (total > 10.0)
        // Deve tentar minerar no mundo e retornar Some(1)
        let res = mc.update(pos, &mut world, 5.1, false);
        assert_eq!(res, Some(1));
        assert!(world.mine_called);
        assert_eq!(mc.stored, 1);
    }

    #[test]
    fn test_mining_inventory_full() {
        let mut mc = MiningComputer::new(1); // Capacidade máxima: 1
        let mut world = MockWorld::new();
        mc.stored = 1; // Já está cheio

        mc.receive_syscall(Syscall::Mine);

        // Se estiver cheio, deve retornar 0 no primeiro tick de instrução
        let res = mc.update(glm::vec2(0.0, 0.0), &mut world, 0.0, true);
        assert_eq!(res, Some(0));
        assert_eq!(mc.stored, 1); // Não aumentou
    }

    #[test]
    fn test_scan_functionality() {
        let mut mc = MiningComputer::new(10);
        let mut world = MockWorld::new();
        
        // Simulamos um recurso com 150 unidades no mundo
        world.resource_at_pos = GridPosition::Resource(150, 0);
        
        mc.receive_syscall(Syscall::Scan);

        // Scan leva 1.0 segundo. 0.5s não deve bastar.
        let res = mc.update(glm::vec2(0.0, 0.0), &mut world, 0.5, false);
        assert_eq!(res, None);

        // Completando o tempo
        let res = mc.update(glm::vec2(0.0, 0.0), &mut world, 0.6, false);
        assert_eq!(res, Some(150));
    }

    #[test]
    fn test_stored_and_max_storage_immediate() {
        let mut mc = MiningComputer::new(16);
        let mut world = MockWorld::new();
        mc.stored = 5;

        // MaxStorage
        mc.receive_syscall(Syscall::MaxStorage);
        // Sem tick de instrução retorna None
        assert_eq!(mc.update(glm::vec2(0.0, 0.0), &mut world, 0.0, false), None);
        // Com tick retorna o valor
        assert_eq!(mc.update(glm::vec2(0.0, 0.0), &mut world, 0.0, true), Some(16));

        // Stored
        mc.receive_syscall(Syscall::Stored);
        assert_eq!(mc.update(glm::vec2(0.0, 0.0), &mut world, 0.0, true), Some(5));
    }

    #[test]
    fn test_mining_fail_in_world() {
        let mut mc = MiningComputer::new(10);
        let mut world = MockWorld::new();
        world.mine_success = false; // Simula mundo vazio ou erro ao minerar

        mc.receive_syscall(Syscall::Mine);
        
        // Finaliza o tempo de mineração
        let res = mc.update(glm::vec2(0.0, 0.0), &mut world, 11.0, false);
        
        assert_eq!(res, Some(0)); // Retorna 0 (falha)
        assert_eq!(mc.stored, 0); // Não incrementou inventário
    }
}
