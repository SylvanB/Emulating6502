use bitflags::bitflags;

type Byte = u8;
type Word = u16;

bitflags! {
    #[derive(Default)]
    struct ProcessorStatus: u8 {
        const CARRY = 1;
        const ZERO = 2;
        const IRQ_DISABLE = 4;
        const DECIMAL_MODE = 8;
        const BREAK_COMMAND = 16;
        const OVERFLOW = 64;
        const NEGATIVE = 128;
    }
}

struct CPU {
    pub sp: Byte,
    pub pc: Word,

    pub a: Byte,
    pub x: Byte,
    pub y: Byte,

    pub status: ProcessorStatus,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            sp: 0xFF,
            pc: 0xFF00, // This might have to be changed back to 0xFFFC
            a: 0,
            x: 0,
            y: 0,
            status: Default::default(),
        }
    }

    fn fetch(&mut self, cycles: &mut u32, memory: &Memory) -> Word {
        let data = memory.get(self.pc.into());
        println!("Fetched data: {:X}", data);
        self.pc += 1;
        *cycles -= 1;
        data
    }

    fn fetch_word(&mut self, cycles: &mut u32, memory: &Memory) -> Word {
        // 6502 is little endian, LSB come first

        let byte1 = self.fetch(cycles, memory);
        let byte2 = self.fetch(cycles, memory);

        let full_addr: Word = (byte1 as u16) | ((byte2 as u16) << 8);
        *cycles -= 1;
        full_addr
    }

    fn read(&self, addr: &Word, cycles: &mut u32, memory: &Memory) -> Word {
        println!("[READ] cycles: {} addr: {}", *cycles, addr);
        *cycles -= 1;
        let data = memory.get((*addr).into());
        println!("Get data: {:X}", data);
        data
    }

    fn set(&self, addr: &Word, value: Word, cycles: &mut u32, memory: &mut Memory) {
        memory.data[*addr as usize] = value;
        *cycles -= 1;
    }

    fn set_lda_flags(&mut self) {
        if self.a == 0x0 {
            println!("SETTING ZERO STATUS");
            self.status = self.status | ProcessorStatus::ZERO;
        }

        if (self.a & (0x1 << 7)) > 0 {
            println!("SETTING NEGATIVE STATUS");
            self.status = self.status | ProcessorStatus::NEGATIVE;
        }
    }

    pub fn execute(&mut self, cycles: &mut u32, memory: &mut Memory) {
        while *cycles > 0 {
            let instruction = self.fetch(cycles, &memory);

            match OpCode::from(instruction) {
                OpCode::LdaIm => {
                    let value = self.fetch(cycles, &memory);
                    self.a = value as u8;

                    self.set_lda_flags();
                }
                OpCode::LdaZp => {
                    let zp_addr = self.fetch(cycles, &memory);
                    self.a = self.read(&zp_addr, cycles, &memory) as u8;
                    self.set_lda_flags();
                }
                OpCode::LdaZpX => {
                    let zp_addr = self.fetch(cycles, &memory);
                    let zpx_addr = zp_addr + self.x as u16;
                    *cycles -= 1;
                    self.a = self.read(&zpx_addr, cycles, &memory) as u8;
                    self.set_lda_flags();
                }
                OpCode::Jsr => {
                    self.sp -= 1;
                    self.set(
                        &(0x0100 as u16 | self.sp as u16),
                        self.pc - 1,
                        cycles,
                        memory,
                    );
                    self.pc = self.fetch_word(cycles, &memory);
                    *cycles -= 1;
                }
                _ => {}
            }
        }
    }
}

const MAX_MEM: usize = 1024 * 64;

struct Memory {
    pub data: Vec<Word>,
}

impl Memory {
    pub fn initialise() -> Self {
        Memory {
            data: vec![0; MAX_MEM],
        }
    }

    pub fn get(&self, index: usize) -> Word {
        *self.data.get(index).unwrap()
    }
}

enum OpCode {
    // LDA Instructions
    LdaIm = 0xA9,
    LdaZp = 0xA5,
    LdaZpX = 0xB5,

    Jsr = 0x20,

    Unknown,
}

// TODO: Fix this mess, surely there is a cleaner way..
impl From<Word> for OpCode {
    fn from(value: Word) -> Self {
        match value {
            0xA9 => Self::LdaIm,
            0xA5 => Self::LdaZp,
            0xB5 => Self::LdaZpX,
            0x20 => Self::Jsr,
            _ => Self::Unknown,
        }
    }
}

impl From<OpCode> for Word {
    fn from(value: OpCode) -> Self {
        match value {
            OpCode::LdaIm => 0xA9,
            OpCode::LdaZp => 0xA5,
            OpCode::LdaZpX => 0xB5,
            OpCode::Jsr => 0x20,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lda_im_sets_acc() {
        let mut cpu = CPU::new();
        let mut mem = Memory::initialise();

        let pc: usize = cpu.pc.into();
        mem.data[pc] = OpCode::LdaIm.into();
        mem.data[pc + 1] = 0x2A;

        cpu.execute(&mut 2, &mut mem);

        assert_eq!(42, cpu.a);
    }

    #[test]
    fn lda_zp_sets_acc() {
        let mut cpu = CPU::new();
        let mut mem = Memory::initialise();

        let pc: usize = cpu.pc.into();
        mem.data[pc] = OpCode::LdaZp.into();
        mem.data[pc + 1] = 0x2A;
        mem.data[0x002A] = 0x45;

        cpu.execute(&mut 3, &mut mem);

        assert_eq!(0x45, cpu.a);
    }

    #[test]
    fn lda_zpx_sets_acc() {
        let mut cpu = CPU::new();
        let mut mem = Memory::initialise();

        cpu.x = 1;

        let pc: usize = cpu.pc.into();
        mem.data[pc] = OpCode::LdaZpX.into();
        mem.data[pc + 1] = 0x2A;
        mem.data[0x002B] = 0x45;

        cpu.execute(&mut 4, &mut mem);

        assert_eq!(0x45, cpu.a);
    }

    #[test]
    fn jsr_sets_pc() {
        let mut cpu = CPU::new();
        let mut mem = Memory::initialise();

        cpu.x = 1;

        let pc: usize = cpu.pc.into();
        mem.data[pc] = OpCode::Jsr.into();
        mem.data[pc + 1] = 0x34;
        mem.data[pc + 2] = 0x12;

        cpu.execute(&mut 6, &mut mem);

        assert_eq!(0x1234, cpu.pc);
    }

    #[test]
    fn lda_sets_zero_flag() {
        let mut cpu = CPU::new();
        let mut mem = Memory::initialise();

        let pc: usize = cpu.pc.into();

        mem.data[pc] = OpCode::LdaZp.into();
        mem.data[pc + 1] = 0x42;
        mem.data[0x42] = 0x0;

        cpu.execute(&mut 3, &mut mem);

        println!("CPU STATUS: {:?} ", cpu.status);

        assert_eq!((cpu.status & ProcessorStatus::ZERO), ProcessorStatus::ZERO);
    }

    #[test]
    fn lda_sets_neg_flag_on_signed_byte() {
        let mut cpu = CPU::new();
        let mut mem = Memory::initialise();

        let pc: usize = cpu.pc.into();

        mem.data[pc] = OpCode::LdaZp.into();
        mem.data[pc + 1] = 0x42;
        mem.data[0x42] = 0b10000000;

        cpu.execute(&mut 3, &mut mem);

        assert_eq!(
            (cpu.status & ProcessorStatus::NEGATIVE),
            ProcessorStatus::NEGATIVE
        );
    }

    #[test]
    fn lda_doesnt_set_neg_flag_on_non_signed_byte() {
        let mut cpu = CPU::new();
        let mut mem = Memory::initialise();

        let pc: usize = cpu.pc.into();

        mem.data[pc] = OpCode::LdaZp.into();
        mem.data[pc + 1] = 0x42;
        mem.data[0x42] = 0b01111111;

        cpu.execute(&mut 3, &mut mem);

        assert_ne!(
            cpu.status & ProcessorStatus::NEGATIVE,
            ProcessorStatus::NEGATIVE
        );
    }
}

fn main() {
    println!("6502 Emulator");

    let mut cpu = CPU::new();
    let mut memory = Memory::initialise();

    cpu.execute(&mut 2, &mut memory);
}
