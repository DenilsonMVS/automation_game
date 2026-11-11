
#[derive(Debug, Clone, Copy)]
pub struct DisplayModule {
    value: Option<u8>,
}

impl DisplayModule {
    pub fn new() -> Self {
        Self { value: None }
    }

    pub fn receive_syscall(&mut self, value: Option<u8>) {
        self.value = value;
    }

    pub fn update(&self) -> Option<u8> {
        Some(1)
    }

    pub fn get_value(&self)  -> Option<u8> {
        self.value
    }
}
