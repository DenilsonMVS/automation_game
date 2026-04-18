
#[derive(Debug, Clone, Copy)]
pub struct DisplayModule {
    received_value: Option<u8>,
    value: Option<u8>,
}

impl DisplayModule {
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

    pub fn get_value(&self)  -> Option<u8> {
        self.value
    }
}
