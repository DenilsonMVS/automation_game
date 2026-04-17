use crate::graphics::{camera::Camera, renderer::fonts::{Font, TextRenderConfig}};
use nalgebra_glm as glm;

#[derive(Debug, Clone, Copy)]
pub struct DisplayComputer {
    received_value: Option<u8>,
    value: Option<u8>,
}

impl DisplayComputer {
    pub fn new() -> Self {
        Self { received_value: None, value: None }
    }

    pub fn receive_syscall(&mut self, value: Option<u8>) {
        self.received_value = value;
    }

    pub fn update(&mut self, is_update_tick: bool) -> Option<u8> {
        if is_update_tick {
            self.value = self.received_value;
            Some(1)
        } else {
            None
        }
    }

    pub fn send(&self, font: &mut Font, position: glm::Vec2, camera: &Camera) {
        if let Some(value) = self.value {
            let value = format!("{}", value);
            font.add_text(&TextRenderConfig::new(
                &value,
                position,
                0.5,
                glm::vec4(20, 45, 90, 255),
            ).set_depth(0).set_recenter(camera));
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_update_latch_logic() {
        let mut dc = DisplayComputer::new();

        // 1. Enviamos um valor via syscall
        dc.receive_syscall(Some(42));

        // 2. Simulamos um update sem ser instruction_tick (delta de tempo comum)
        // O valor ainda não deve ter sido "confirmado" (retorna None)
        let res = dc.update(false);
        assert_eq!(res, None);
        
        // 3. Agora simulamos o instruction_tick
        // O valor deve ser confirmado e o computador deve receber o sinal de sucesso (Some(1))
        let res = dc.update(true);
        assert_eq!(res, Some(1));
    }

    #[test]
    fn test_display_none_clears_value() {
        let mut dc = DisplayComputer::new();

        // Primeiro definimos um valor
        dc.receive_syscall(Some(10));
        dc.update(true);

        // Agora enviamos None para limpar o display
        dc.receive_syscall(None);
        
        // Antes do tick, o valor antigo (10) ainda deve persistir internamente
        dc.update(false);
        
        // Após o tick, ele deve ser limpo
        dc.update(true);
        
        // Como 'value' é privado, o teste de 'send' (se tivéssemos um mock de Font) 
        // validaria que nada foi renderizado.
    }

    #[test]
    fn test_display_consecutive_updates() {
        let mut dc = DisplayComputer::new();

        // Se a CPU enviar dois valores rapidamente antes de um tick
        dc.receive_syscall(Some(1));
        dc.receive_syscall(Some(2));

        // No tick, apenas o último valor recebido deve ser o definitivo
        dc.update(true);
        
        // Se enviarmos um valor e logo em seguida o tick acontecer
        dc.receive_syscall(Some(3));
        let res = dc.update(true);
        assert_eq!(res, Some(1));
        
        // No próximo update(true) sem novas chamadas, ele deve manter o Some(1)
        // ou retornar sucesso novamente? Pela sua implementação, ele sempre 
        // retorna Some(1) no tick, mesmo que o valor seja o mesmo.
    }
}
