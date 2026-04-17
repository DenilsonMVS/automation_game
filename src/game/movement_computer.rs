
use nalgebra_glm as glm;

use crate::game::instruction::Syscall;

const POSITION_TOLERANCE: f32 = 0.02;
const SPEED_TOLERANCE: f32 = 0.01;

#[derive(Debug, Clone, Copy)]
pub struct MovementComputer {
    position: glm::Vec2,
    speed: glm::Vec2,
    target: glm::Vec2,
}

impl MovementComputer {
    pub fn new(position: glm::Vec2) -> Self {
        Self {
            position,
            speed: glm::vec2(0.0, 0.0),
            target: position,
        }
    }

    pub fn update_sensor(&mut self, position: glm::Vec2, speed: glm::Vec2) {
        self.position = position;
        self.speed = speed;
    }

    pub fn update(&mut self) -> Option<u8> {
        self.target_achieved().then_some(1)
    }

    fn target_achieved(&self) -> bool {
        let sq = |x: f32| x * x;

        let diff = self.target - self.position;
        let distance2 = glm::length2(&diff);
        let speed_val2 = glm::length2(&self.speed);

        return distance2 <= sq(POSITION_TOLERANCE) && speed_val2 <= sq(SPEED_TOLERANCE);
    }

    pub fn calculate_accelleration(&self) -> glm::Vec2 {
        const PROPORTIONAL: f32 = 10.0;
        const DERIVATIVE: f32 = -6.0;

        let distance = self.target - self.position;
        let d = self.speed * DERIVATIVE;
        let p = distance * PROPORTIONAL;
        return p + d;
    }

    pub fn get_position(&self) -> glm::Vec2 {
        self.position
    }

    pub fn get_speed(&self) -> glm::Vec2 {
        self.speed
    }

    pub fn receive_syscall(&mut self, sys: Syscall) {
        self.target = self.position.map(|x| x.round()) + match sys {
            Syscall::MoveBottom => glm::vec2(0.0, -1.0),
            Syscall::MoveLeft => glm::vec2(-1.0, 0.0),
            Syscall::MoveRight => glm::vec2(1.0, 0.0),
            Syscall::MoveTop => glm::vec2(0.0, 1.0),
            _ => glm::vec2(0.0, 0.0),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::instruction::Syscall;

    #[test]
    fn test_initialization() {
        let pos = glm::vec2(5.0, 5.0);
        let mut mc = MovementComputer::new(pos);
        assert_eq!(mc.get_position(), pos);
        assert_eq!(mc.get_speed(), glm::vec2(0.0, 0.0));
        // O alvo inicial deve ser a posição de criação
        assert_eq!(mc.update(), Some(1)); 
    }

    #[test]
    fn test_target_calculation_snapping() {
        // Caso 1: Robô está em (0.1, 0.1). 
        // round(0.1) = 0.0. MoveRight (+1, 0) -> Alvo (1, 0)
        let mut mc = MovementComputer::new(glm::vec2(0.1, 0.1));
        mc.receive_syscall(Syscall::MoveRight);
        assert_eq!(mc.target, glm::vec2(1.0, 0.0));

        // Caso 2: Robô está em (0.9, 0.1). 
        // round(0.9) = 1.0. MoveRight (+1, 0) -> Alvo (2, 0)
        // Isso garante que ele não tente voltar para o (1,0) se já passou da metade para o próximo
        mc.update_sensor(glm::vec2(0.9, 0.1), glm::vec2(0.0, 0.0));
        mc.receive_syscall(Syscall::MoveRight);
        assert_eq!(mc.target, glm::vec2(2.0, 0.0));

        // Caso 3: Coordenadas negativas (importante para o sistema de Chunks)
        // round(-0.1) = 0.0. round(-0.9) = -1.0.
        // MoveTop (+0, +1) -> Alvo (0.0, 0.0)
        mc.update_sensor(glm::vec2(-0.1, -0.9), glm::vec2(0.0, 0.0));
        mc.receive_syscall(Syscall::MoveTop);
        assert_eq!(mc.target, glm::vec2(0.0, 0.0));
    }

    #[test]
    fn test_pd_controller_direction() {
        let mut mc = MovementComputer::new(glm::vec2(0.0, 0.0));
        mc.receive_syscall(Syscall::MoveRight); // Alvo (1.0, 0.0)
        
        // Sem velocidade, a aceleração deve ser positiva no eixo X (Proporcional)
        let acc = mc.calculate_accelleration();
        assert!(acc.x > 0.0);
        assert_eq!(acc.y, 0.0);

        // Se o robô estiver passando do alvo com muita velocidade,
        // o termo derivativo (D) deve causar aceleração negativa para frear.
        mc.update_sensor(glm::vec2(0.99, 0.0), glm::vec2(10.0, 0.0));
        let acc_braking = mc.calculate_accelleration();
        assert!(acc_braking.x < 0.0, "O robô deve tentar frear para não passar do alvo");
    }

    #[test]
    fn test_target_achieved_tolerances() {
        let mut mc = MovementComputer::new(glm::vec2(0.0, 0.0));
        mc.receive_syscall(Syscall::MoveRight); // Alvo (1, 0)
        
        // 1. Fora da tolerância de distância (distância > 0.02)
        mc.update_sensor(glm::vec2(0.97, 0.0), glm::vec2(0.0, 0.0));
        assert_eq!(mc.update(), None);

        // 2. Dentro da distância, mas velocidade alta demais (speed > 0.01)
        mc.update_sensor(glm::vec2(0.99, 0.0), glm::vec2(0.05, 0.0));
        assert_eq!(mc.update(), None);

        // 3. Dentro de ambas as tolerâncias
        mc.update_sensor(glm::vec2(0.99, 0.0), glm::vec2(0.005, 0.0));
        assert_eq!(mc.update(), Some(1));
    }
}
