use crate::game::logic::instruction::Instruction;


const ZERO_IDX: usize = 0;
pub const PROGRAM_COUNTER_IDX: usize = 1;

pub trait GenericMemory<T: Copy, const SIZE: usize> {
    fn read<ADDR: Into<usize>>(&self, address: ADDR) -> T;
    fn write<ADDR: Into<usize>>(&mut self, address: ADDR, value: T);
}

#[derive(Debug)]
pub struct RegisterMemory<T: Copy, const SIZE: usize> {
    memory: [T; SIZE],
}

impl<T: Copy + Default, const SIZE: usize> RegisterMemory<T, SIZE> {
    pub fn new(memory: [T; SIZE]) -> Self {
        Self { memory }
    }

    pub fn inst1<ADDR, F>(&mut self, input_addr: ADDR, out_addr: ADDR, func: F)
    where
        ADDR: Into<usize> + Copy,
        F: Fn(T) -> T, 
    {
        let val = self.read(input_addr);
        let result = func(val);
        self.write(out_addr, result);
    }

    pub fn inst2<ADDR, F>(&mut self, input_addr1: ADDR, input_addr2: ADDR, out_addr: ADDR, func: F)
    where
        ADDR: Into<usize> + Copy,
        F: Fn(T, T) -> T, 
    {
        let val1 = self.read(input_addr1);
        let val2 = self.read(input_addr2);
        let result = func(val1, val2);
        self.write(out_addr, result);
    }

    pub fn inst3<ADDR, F>(&mut self, input_addr1: ADDR, input_addr2: ADDR, input_addr3: ADDR, out_addr: ADDR, func: F)
    where
        ADDR: Into<usize> + Copy,
        F: Fn(T, T, T) -> T, 
    {
        let val1 = self.read(input_addr1);
        let val2 = self.read(input_addr2);
        let val3 = self.read(input_addr3);
        let result = func(val1, val2, val3);
        self.write(out_addr, result);
    }
}

impl<T: Copy + Default, const SIZE: usize> GenericMemory<T, SIZE> for RegisterMemory<T, SIZE> {
    fn read<ADDR: Into<usize>>(&self, address: ADDR) -> T {
        let idx = address.into() % SIZE;
        if idx == ZERO_IDX {
            return T::default();
        } else {
            return self.memory[idx];
        }
    }

    fn write<ADDR: Into<usize>>(&mut self, address: ADDR, value: T) {
        self.memory[address.into() % SIZE] = value;
    }
}

#[derive(Debug)]
pub struct InstructionMemory<const SIZE: usize> {
    memory: [Instruction; SIZE],
}

impl<const SIZE: usize> InstructionMemory<SIZE> {
    pub fn new(memory: [Instruction; SIZE]) -> Self {
        Self { memory }
    }
}

impl<const SIZE: usize> GenericMemory<Instruction, SIZE> for InstructionMemory<SIZE> {
    fn read<ADDR: Into<usize>>(&self, address: ADDR) -> Instruction {
        let idx = address.into() % SIZE;
        return self.memory[idx];
    }
    fn write<ADDR: Into<usize>>(&mut self, address: ADDR, value: Instruction) {
        self.memory[address.into() % SIZE] = value;
    }
}
