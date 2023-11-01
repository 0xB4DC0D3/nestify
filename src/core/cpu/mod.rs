use std::cell::RefCell;
use std::rc::Rc;

use super::bus::Bus;
use super::memorymap::MemoryMap;
use super::registers::Register;
use super::registers::cpu::status::{CpuStatusRegister, CpuStatusRegisterFlags};
use super::memory::Memory;

pub enum AddressingMode {
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndexedIndirect,
    IndirectIndexed,
}

pub struct Cpu {
    register_a: u8,
    register_x: u8,
    register_y: u8,
    status: CpuStatusRegister,
    stack_pointer: u8,
    program_counter: u16,
    bus: Rc<RefCell<Bus>>,
}

impl Cpu {
    pub fn new(bus: &Rc<RefCell<Bus>>) -> Self {
        Self {
            register_a: 0x00,
            register_x: 0x00,
            register_y: 0x00,
            status: CpuStatusRegister::new(),
            stack_pointer: 0xFD,
            program_counter: 0x8000,
            bus: bus.clone(),
        }
    }

    fn push_stack(&mut self, value: u8) {
        self.write(0x0100 + self.stack_pointer as u16, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn push_stack_u16(&mut self, value: u16) {
        let [lo, hi] = value.to_le_bytes();

        self.push_stack(hi);
        self.push_stack(lo);
    }

    fn pop_stack(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.read(0x0100 + self.stack_pointer as u16)
    }

    fn pop_stack_u16(&mut self) -> u16 {
        let lo = self.pop_stack();
        let hi = self.pop_stack();

        u16::from_le_bytes([lo, hi])
    }

    pub fn reset(&mut self) {
        self.register_a = 0x00;
        self.register_x = 0x00;
        self.register_y = 0x00;
        self.status = CpuStatusRegister::new();
        self.stack_pointer = 0xFD;
        self.program_counter = self.read_u16(0xFFFC);
        // TODO: add here +7 cycles
    }

    fn is_page_cross(&self, page1: u16, page2: u16) -> bool {
        (page1 & 0xFF00) != (page2 & 0xFF00)
    }

    pub fn get_memory_data(&self, addressing_mode: &AddressingMode) -> Option<(u16, bool)> {
        match addressing_mode {
            AddressingMode::Implicit => {
                None
            },
            AddressingMode::Accumulator => {
                None
            },
            AddressingMode::Immediate => {
                Some((self.program_counter, false))
            },
            AddressingMode::ZeroPage => {
                Some((self.read(self.program_counter) as u16, false))
            },
            AddressingMode::ZeroPageX => {
                let pointer = self
                    .read(self.program_counter);

                let memory_pointer = pointer 
                    .wrapping_add(self.register_x);

                let is_page_cross = self.is_page_cross(pointer as u16, memory_pointer as u16);

                Some((memory_pointer as u16, is_page_cross))
            },
            AddressingMode::ZeroPageY => {
                let pointer = self
                    .read(self.program_counter);

                let memory_pointer = pointer
                    .wrapping_add(self.register_y);

                let is_page_cross = self.is_page_cross(pointer as u16, memory_pointer as u16);

                Some((memory_pointer as u16, is_page_cross))
            },
            AddressingMode::Relative => {
                Some((self.read(self.program_counter) as u16, false))
            },
            AddressingMode::Absolute => {
                Some((self.read_u16(self.program_counter), false))
            },
            AddressingMode::AbsoluteX => {
                let pointer = self
                    .read_u16(self.program_counter);

                let memory_pointer = pointer
                    .wrapping_add(self.register_x as u16);

                let is_page_cross = self.is_page_cross(pointer, memory_pointer);

                Some((memory_pointer, is_page_cross))
            },
            AddressingMode::AbsoluteY => {
                let pointer = self
                    .read_u16(self.program_counter);

                let memory_pointer = pointer
                    .wrapping_add(self.register_y as u16);

                let is_page_cross = self.is_page_cross(pointer, memory_pointer);

                Some((memory_pointer, is_page_cross))
            },
            AddressingMode::Indirect => {
                let pointer = self.read_u16(self.program_counter);
                let memory_pointer = self.read_u16(pointer);

                Some((memory_pointer, false))
            },
            AddressingMode::IndexedIndirect => {
                let pointer = self
                    .read(self.program_counter)
                    .wrapping_add(self.register_x) as u16;

                let memory_pointer = self.read_u16(pointer);

                Some((memory_pointer, false))
            },
            AddressingMode::IndirectIndexed => {
                let pointer = self
                    .read(self.program_counter) as u16;

                let deref_pointer = self
                    .read_u16(pointer);

                let memory_pointer = deref_pointer
                    .wrapping_add(self.register_y as u16);

                let is_page_cross = self.is_page_cross(deref_pointer, memory_pointer as u16);

                Some((memory_pointer, is_page_cross))
            },
        }
    }

    fn execute_adc(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for ADC instruction!");

        let a = self.register_a as u16;
        let m = self.read(memory_pointer) as u16;
        let c = if self.status.get_flag(CpuStatusRegisterFlags::Carry) { 1u16 } else { 0u16 };
        let result = a.wrapping_add(m).wrapping_add(c);
        let overflow = (a ^ result) & !(a ^ m) & 0x80 == 0x80;

        self.status.set_flag(CpuStatusRegisterFlags::Carry, result > 255);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result as u8 & 0x80 == 0x80);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, result as u8 == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Overflow, overflow);
        self.register_a = result as u8;

        if additional_cycle {
            // TODO: add additional cycle
        }
    }

    fn execute_and(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for AND instruction!");

        let result = self.register_a & self.read(memory_pointer);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_a = result;

        if additional_cycle {
            // TODO: add additional cycle
        }
    }

    fn execute_asl(&mut self, addressing_mode: &AddressingMode) {
        let memory_data = self.get_memory_data(addressing_mode);
        let value = if let Some((memory_pointer, _)) = memory_data {
            self.read(memory_pointer)
        } else {
            self.register_a
        };

        let result = value << 2;

        self.status.set_flag(CpuStatusRegisterFlags::Carry, value & 0x80 == 0x80);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);

        if let Some((memory_pointer, _)) = memory_data {
            self.write(memory_pointer, result);
        } else {
            self.register_a = result;
        }

        // TODO: add ticks
    }

    fn branch(&mut self, flag_active: bool) {
        if flag_active {
            // TODO: add +1 cycle

            let (memory_pointer, _) = self.get_memory_data(&AddressingMode::Relative)
                .expect("Invalid Addressing mode for branch instructions!");

            let offset = memory_pointer as i16;
            let next_pc = self.program_counter.wrapping_add(1);
            let jump_pc = (next_pc as i16).wrapping_add(offset) as u16;

            if self.is_page_cross(next_pc, jump_pc) {
                // TODO: add +1 cycle
            }

            self.program_counter = jump_pc;
        } else {
            self.program_counter = self.program_counter.wrapping_add(1);
        }
    }

    fn execute_bcc(&mut self) {
        self.branch(!self.status.get_flag(CpuStatusRegisterFlags::Carry));
    }

    fn execute_bcs(&mut self) {
        self.branch(self.status.get_flag(CpuStatusRegisterFlags::Carry));
    }

    fn execute_beq(&mut self) {
        self.branch(self.status.get_flag(CpuStatusRegisterFlags::Zero));
    }

    fn execute_bit(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for BIT instruction!");

        let memory_value = self.read(memory_pointer);
        let result = self.register_a & memory_value;

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Overflow, memory_value & 0x40 == 0x40);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, memory_value & 0x80 == 0x80);

        // TODO: add ticks
    }

    fn execute_bmi(&mut self) {
        self.branch(self.status.get_flag(CpuStatusRegisterFlags::Negative));
    }

    fn execute_bne(&mut self) {
        self.branch(!self.status.get_flag(CpuStatusRegisterFlags::Zero));
    }

    fn execute_bpl(&mut self) {
        self.branch(!self.status.get_flag(CpuStatusRegisterFlags::Negative));
    }

    fn execute_brk(&mut self) {
        // Do nothing
    }

    fn execute_bvc(&mut self) {
        self.branch(!self.status.get_flag(CpuStatusRegisterFlags::Overflow));
    }

    fn execute_bvs(&mut self) {
        self.branch(self.status.get_flag(CpuStatusRegisterFlags::Overflow));
    }

    fn execute_clc(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::Carry, false);

        // TODO: add ticks
    }

    fn execute_cld(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::DecimalMode, false);

        // TODO: add ticks
    }

    fn execute_cli(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::InterruptDisable, false);

        // TODO: add ticks
    }

    fn execute_clv(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::Overflow, false);

        // TODO: add ticks
    }

    fn compare(&mut self, addressing_mode: &AddressingMode, register_value: u8) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for CMP/CPX/CPY instructions!");

        let memory_value = self.read(memory_pointer);
        let result = register_value.wrapping_sub(memory_value);

        self.status.set_flag(CpuStatusRegisterFlags::Carry, register_value >= memory_value);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);

        if additional_cycle {
            // TODO: handle additional cycle
        }

        // TODO: handle cycles
    }

    fn execute_cmp(&mut self, addressing_mode: &AddressingMode) {
        self.compare(addressing_mode, self.register_a);

        // TODO: handle cycles
    }

    fn execute_cpx(&mut self, addressing_mode: &AddressingMode) {
        self.compare(addressing_mode, self.register_x);

        // TODO: handle cycles
    }

    fn execute_cpy(&mut self, addressing_mode: &AddressingMode) {
        self.compare(addressing_mode, self.register_y);

        // TODO: handle cycles
    }

    fn execute_dec(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for DEC instruction!");

        let memory_value = self.read(memory_pointer);
        let result = memory_value.wrapping_sub(1);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.write(memory_pointer, result);

        // TODO: handle cycles
    }

    fn execute_dex(&mut self) {
        let result = self.register_x.wrapping_sub(1);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_x = result;
    }

    fn execute_dey(&mut self) {
        let result = self.register_y.wrapping_sub(1);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_y = result;
    }

    fn execute_eor(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for EOR instruction!");

        let result = self.register_a ^ self.read(memory_pointer);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_a = result;

        if additional_cycle {
            // TODO: handle additional cycle
        }

        // TODO: handle cycles
    }

    fn execute_inc(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for INC instruction!");

        let memory_value = self.read(memory_pointer);
        let result = memory_value.wrapping_add(1);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.write(memory_pointer, result);

        // TODO: handle cycles
    }

    fn execute_inx(&mut self) {
        let result = self.register_x.wrapping_add(1);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_x = result;
    }

    fn execute_iny(&mut self) {
        let result = self.register_y.wrapping_add(1);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_y = result;
    }

    fn execute_jmp(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for JMP instruction!");

        self.program_counter = memory_pointer;
    }

    fn execute_jsr(&mut self) {
        let (memory_pointer, _) = self.get_memory_data(&AddressingMode::Absolute).unwrap();

        self.push_stack_u16(self.program_counter.wrapping_add(2));
        self.program_counter = memory_pointer;
    }

    fn execute_lda(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for LDA instruction!");

        let memory_value = self.read(memory_pointer);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, memory_value == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, memory_value & 0x80 == 0x80);
        self.register_a = memory_value;

        if additional_cycle {
            // TODO: handle add. cycle
        }
    }

    fn execute_ldx(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for LDA instruction!");

        let memory_value = self.read(memory_pointer);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, memory_value == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, memory_value & 0x80 == 0x80);
        self.register_x = memory_value;

        if additional_cycle {
            // TODO: handle add. cycle
        }
    }

    fn execute_ldy(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for LDA instruction!");

        let memory_value = self.read(memory_pointer);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, memory_value == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, memory_value & 0x80 == 0x80);
        self.register_y = memory_value;

        if additional_cycle {
            // TODO: handle add. cycle
        }
    }

    fn execute_lsr(&mut self, addressing_mode: &AddressingMode) {
        let memory_data = self.get_memory_data(addressing_mode);
        let value = if let Some((memory_pointer, _)) = memory_data {
            self.read(memory_pointer)
        } else {
            self.register_a
        };

        let result = value >> 1;

        self.status.set_flag(CpuStatusRegisterFlags::Carry, value & 0x1 == 0x1);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);

        if let Some((memory_pointer, _)) = memory_data {
            self.write(memory_pointer, result);
        } else {
            self.register_a = result;
        }
    }

    fn execute_nop(&self) {
        // Do nothing
    }

    fn execute_ora(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for ORA instruction!");
        
        let result = self.register_a | self.read(memory_pointer);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_a = result;

        if additional_cycle {
            // TODO: handle add. cycle
        }
    }

    fn execute_pha(&mut self) {
        self.push_stack(self.register_a);
    }

    fn execute_php(&mut self) {
        self.push_stack(self.status.get());
    }

    fn execute_pla(&mut self) {
        let value_from_stack = self.pop_stack();

        self.status.set_flag(CpuStatusRegisterFlags::Zero, value_from_stack == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, value_from_stack & 0x80 == 0x80);
        self.register_a = value_from_stack;
    }

    fn execute_plp(&mut self) {
        let status = self.pop_stack();

        self.status.set(status);
    }

    fn execute_rol(&mut self, addressing_mode: &AddressingMode) {
        let memory_data = self.get_memory_data(addressing_mode);
        let value = if let Some((memory_pointer, _)) = memory_data {
            self.read(memory_pointer)
        } else {
            self.register_a
        };

        let result = value.rotate_left(1);

        self.status.set_flag(CpuStatusRegisterFlags::Carry, value & 0x80 == 0x80);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);

        if let Some((memory_pointer, _)) = memory_data {
            self.write(memory_pointer, result);
        } else {
            self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
            self.register_a = result;
        }
    }

    fn execute_ror(&mut self, addressing_mode: &AddressingMode) {
        let memory_data = self.get_memory_data(addressing_mode);
        let value = if let Some((memory_pointer, _)) = memory_data {
            self.read(memory_pointer)
        } else {
            self.register_a
        };

        let result = value.rotate_right(1);

        self.status.set_flag(CpuStatusRegisterFlags::Carry, value & 0x1 == 0x1);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);

        if let Some((memory_pointer, _)) = memory_data {
            self.write(memory_pointer, result);
        } else {
            self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
            self.register_a = result;
        }
    }

    fn execute_rti(&mut self) {
        let status = self.pop_stack();
        let program_counter = self.pop_stack_u16();

        self.status.set(status);
        self.program_counter = program_counter;
    }

    fn execute_rts(&mut self) {
        self.program_counter = self.pop_stack_u16();
    }

    fn execute_sbc(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for ADC instruction!");

        let a = self.register_a as u16;
        let m = self.read(memory_pointer) as u16 ^ 0xFF;
        let c = if self.status.get_flag(CpuStatusRegisterFlags::Carry) { 1u16 } else { 0u16 };
        let result = a.wrapping_add(m).wrapping_add(c);
        let overflow = (a ^ result) & !(a ^ m) & 0x80 == 0x80;

        self.status.set_flag(CpuStatusRegisterFlags::Carry, result > 255);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result as u8 & 0x80 == 0x80);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, result as u8 == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Overflow, overflow);
        self.register_a = result as u8;

        if additional_cycle {
            // TODO: add additional cycle
        }
    }

    fn execute_sec(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::Carry, true);
    }

    fn execute_sed(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::DecimalMode, true);
    }

    fn execute_sei(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::InterruptDisable, true);
    }

    fn execute_sta(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for STA instruction!");

        self.write(memory_pointer, self.register_a);
    }

    fn execute_stx(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for STA instruction!");

        self.write(memory_pointer, self.register_x);
    }

    fn execute_sty(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for STA instruction!");

        self.write(memory_pointer, self.register_y);
    }

    fn execute_tax(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::Zero, self.register_a == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, self.register_a & 0x80 == 0x80);
        self.register_x = self.register_a;
    }

    fn execute_tay(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::Zero, self.register_a == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, self.register_a & 0x80 == 0x80);
        self.register_y = self.register_a;
    }

    fn execute_tsx(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::Zero, self.stack_pointer == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, self.stack_pointer & 0x80 == 0x80);
        self.register_x = self.stack_pointer;
    }

    fn execute_txa(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::Zero, self.register_x == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, self.register_x & 0x80 == 0x80);
        self.register_a = self.register_x;
    }

    fn execute_txs(&mut self) {
        self.stack_pointer = self.register_x;
    }

    fn execute_tya(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::Zero, self.register_y == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, self.register_y & 0x80 == 0x80);
        self.register_a = self.register_y;
    }
}

impl Memory for Cpu {
    fn read(&self, address: u16) -> u8 {
        self.bus
            .borrow_mut()
            .get_cpu_memory_map()
            .read(address)
    }

    fn write(&mut self, address: u16, data: u8) {
        self.bus
            .borrow_mut()
            .get_cpu_memory_map()
            .write(address, data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adc_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.register_a = 127;
        cpu.write(0x0000, 0x69);
        cpu.write(0x0001, 0x7F);
        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, false);

        cpu.execute_adc(&AddressingMode::Immediate);
        assert_eq!(cpu.register_a, 254, "Register A should be 254!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "CPU Status: Carry should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "CPU Status: Zero should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "CPU Status: Overflow flag should be set!");

        cpu.register_a = 128;
        cpu.write(0x0000, 0x69);
        cpu.write(0x0001, 0x80);
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, true);

        cpu.execute_adc(&AddressingMode::Immediate);
        assert_eq!(cpu.register_a, 1, "Register A should be 1!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "CPU Status: Carry should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "CPU Status: Zero should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "CPU Status: Overflow flag should be set!");
    }

    #[test]
    fn test_and_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.register_a = 127;
        cpu.write(0x0000, 0x29);
        cpu.write(0x0001, 0x7E);
        cpu.program_counter = 0x0001;

        cpu.execute_and(&AddressingMode::Immediate);
        assert_eq!(cpu.register_a, 0x7E, "Register A should be 126!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "CPU Status: Zero should be unset!");
    }

    #[test]
    fn test_asl_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.register_a = 0x80;
        cpu.execute_asl(&AddressingMode::Accumulator);

        assert_eq!(cpu.register_a, 0x00, "Register A should be 0!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "CPU Status: Zero should be set!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "CPU Status: Carry should be set!");
    }

    #[test]
    fn test_bcc_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, false);
        cpu.execute_bcc();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BCC with inactive Carry flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, true);
        cpu.execute_bcc();
        assert_eq!(cpu.program_counter, 0x0002, "CPU PC should be 0x0002 after BCC with active Carry flag!");
    }

    #[test]
    fn test_bcs_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, false);
        cpu.execute_bcs();
        assert_eq!(cpu.program_counter, 0x0002, "CPU PC should be 0x0002 after BCS with inactive Carry flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, true);
        cpu.execute_bcs();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BCS with active Carry flag!");
    }

    #[test]
    fn test_beq_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Zero, false);
        cpu.execute_beq();
        assert_eq!(cpu.program_counter, 0x0002, "CPU PC should be 0x0002 after BEQ with inactive Zero flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Zero, true);
        cpu.execute_beq();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BEQ with active Zero flag!");
    }

    #[test]
    fn test_bit_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);

        cpu.register_a = 0xFF;
        cpu.program_counter = 0x0001;
        cpu.write(0x0000, 0xC0);

        // 0x0001 - lobyte of 0x0000, 0x0002 - hibyte of 0x0000
        cpu.write(0x0001, 0x00);
        cpu.write(0x0002, 0x00);
        cpu.execute_bit(&AddressingMode::Absolute);
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative should be set!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "CPU Status: Overflow should be set!");

        cpu.write(0x0000, 0x80);
        cpu.execute_bit(&AddressingMode::Absolute);
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "CPU Status: Overflow should be unset!");

        cpu.write(0x0000, 0x40);
        cpu.execute_bit(&AddressingMode::Absolute);
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative should be set!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "CPU Status: Overflow should be unset!");

        cpu.write(0x0000, 0x3F);
        cpu.execute_bit(&AddressingMode::Absolute);
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "CPU Status: Overflow should be unset!");
    }

    #[test]
    fn test_bmi_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Negative, false);
        cpu.execute_bmi();
        assert_eq!(cpu.program_counter, 0x0002, "CPU PC should be 0x0002 after BMI with inactive Negative flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Negative, true);
        cpu.execute_bmi();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BMI with active negative flag!");
    }

    #[test]
    fn test_bne_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Zero, false);
        cpu.execute_bne();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BNE with inactive Zero flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Zero, true);
        cpu.execute_bne();
        assert_eq!(cpu.program_counter, 0x0002, "CPU PC should be 0x0002 after BNE with active Zero flag!");
    }

    #[test]
    fn test_bpl_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Negative, false);
        cpu.execute_bpl();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BPL with inactive Negative flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Negative, true);
        cpu.execute_bpl();
        assert_eq!(cpu.program_counter, 0x0002, "CPU PC should be 0x0002 after BPL with active Negative flag!");
    }

    #[test]
    fn test_bvc_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Overflow, false);
        cpu.execute_bvc();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BVC with inactive Overflow flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Overflow, true);
        cpu.execute_bvc();
        assert_eq!(cpu.program_counter, 0x0002, "CPU PC should be 0x0002 after BVC with active Overflow flag!");
    }

    #[test]
    fn test_bvs_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Overflow, true);
        cpu.execute_bvs();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BVC with active Overflow flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Overflow, false);
        cpu.execute_bvs();
        assert_eq!(cpu.program_counter, 0x0002, "CPU PC should be 0x0002 after BVC with inactive Overflow flag!");
    }

    #[test] 
    fn test_clc_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, true);
        cpu.execute_clc();

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be unset!");
    }

    #[test]
    fn test_cld_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.status.set_flag(CpuStatusRegisterFlags::DecimalMode, true);
        cpu.execute_cld();

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::DecimalMode), "Decimal mode flag should be unset!");
    }

    #[test]
    fn test_cli_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.status.set_flag(CpuStatusRegisterFlags::InterruptDisable, true);
        cpu.execute_cli();

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::InterruptDisable), "Interrupt disable flag should be unset!");
    }

    #[test]
    fn test_clv_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.status.set_flag(CpuStatusRegisterFlags::Overflow, true);
        cpu.execute_clv();

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "Overflow flag should be unset!");
    }

    #[test]
    fn test_cmp_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.register_a = 100;
        cpu.write(0x0000, 99);
        cpu.program_counter = 0x0000;
        cpu.execute_cmp(&AddressingMode::Immediate);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");

        cpu.register_a = 98;
        cpu.program_counter = 0x0000;
        cpu.execute_cmp(&AddressingMode::Immediate);

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");

        cpu.register_a = 99;
        cpu.program_counter = 0x0000;
        cpu.execute_cmp(&AddressingMode::Immediate);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");
    }

    #[test]
    fn test_cpx_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.register_x = 100;
        cpu.write(0x0000, 99);
        cpu.program_counter = 0x0000;
        cpu.execute_cpx(&AddressingMode::Immediate);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");

        cpu.register_x = 98;
        cpu.program_counter = 0x0000;
        cpu.execute_cpx(&AddressingMode::Immediate);

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");

        cpu.register_x = 99;
        cpu.program_counter = 0x0000;
        cpu.execute_cpx(&AddressingMode::Immediate);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");
    }

    #[test]
    fn test_cpy_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.register_y = 100;
        cpu.write(0x0000, 99);
        cpu.program_counter = 0x0000;
        cpu.execute_cpy(&AddressingMode::Immediate);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");

        cpu.register_y = 98;
        cpu.program_counter = 0x0000;
        cpu.execute_cpy(&AddressingMode::Immediate);

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");

        cpu.register_y = 99;
        cpu.program_counter = 0x0000;
        cpu.execute_cpy(&AddressingMode::Immediate);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");
    }

    #[test]
    fn test_dec_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.write(0x0001, 129);
        cpu.write(0x0002, 0x1);
        cpu.program_counter = 0x0002;
        cpu.execute_dec(&AddressingMode::ZeroPageX);

        let memory_value = cpu.read(0x0001);
        assert_eq!(memory_value, 128, "Memory value at 0x0001 should be 128!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
    }

    #[test]
    fn test_dex_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.register_x = 128;
        cpu.execute_dex();

        assert_eq!(cpu.register_x, 127, "Register X should be 127!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
    }

    #[test]
    fn test_dey_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.register_y = 128;
        cpu.execute_dey();

        assert_eq!(cpu.register_y, 127, "Register Y should be 127!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
    }

    #[test]
    fn test_eor_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.register_a = 12;
        cpu.write(0x0000, 37);
        cpu.program_counter = 0x0000;
        cpu.execute_eor(&AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 41, "Register A should contain result of 12 ^ 37!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
    }

    #[test]
    fn test_inc_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.write(0x0001, 129);
        cpu.write(0x0002, 0x1);
        cpu.program_counter = 0x0002;
        cpu.execute_inc(&AddressingMode::ZeroPageX);

        let memory_value = cpu.read(0x0001);
        assert_eq!(memory_value, 130, "Memory value at 0x0001 should be 130!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
    }

    #[test]
    fn test_inx_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.register_x = 128;
        cpu.execute_inx();

        assert_eq!(cpu.register_x, 129, "Register X should be 127!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
    }

    #[test]
    fn test_iny_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_y = 128;
        cpu.execute_iny();

        assert_eq!(cpu.register_y, 129, "Register Y should be 127!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
    }

    #[test]
    fn test_jmp_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0xFF);
        cpu.write(0x0001, 0xAA);
        cpu.execute_jmp(&AddressingMode::Absolute);

        assert_eq!(cpu.program_counter, 0xAAFF, "Program counter should be 0xAAFF!");
    }

    #[test]
    fn test_jsr_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        let stack_pointer_buf = cpu.stack_pointer;
        cpu.program_counter = 0xDEAD;
        cpu.write(0xDEAD, 0xFF);
        cpu.write(0xDEAE, 0xAA);
        cpu.execute_jsr();

        assert_eq!(cpu.program_counter, 0xAAFF, "Program counter should be 0xAAFF");
        assert_eq!(cpu.stack_pointer, stack_pointer_buf.wrapping_sub(2), "Invalid stack pointer!");

        let lo = cpu.read(0x0100 + cpu.stack_pointer.wrapping_add(1) as u16);
        let hi = cpu.read(0x0100 + cpu.stack_pointer.wrapping_add(2) as u16);
        assert_eq!(lo, 0xAF, "Invalid lobyte of PC in Stack!");
        assert_eq!(hi, 0xDE, "Invalid hibyte of PC in Stack!");
    }

    #[test]
    fn test_lda_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0xAA);
        cpu.execute_lda(&AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0xAA, "Register A should be 0xAA!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.write(0x0000, 0x00);
        cpu.execute_lda(&AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0x00, "Register A should be 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_ldx_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0xAA);
        cpu.execute_ldx(&AddressingMode::Immediate);

        assert_eq!(cpu.register_x, 0xAA, "Register A should be 0xAA!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.write(0x0000, 0x00);
        cpu.execute_ldx(&AddressingMode::Immediate);

        assert_eq!(cpu.register_x, 0x00, "Register A should be 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_ldy_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0xAA);
        cpu.execute_ldy(&AddressingMode::Immediate);

        assert_eq!(cpu.register_y, 0xAA, "Register A should be 0xAA!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.write(0x0000, 0x00);
        cpu.execute_ldy(&AddressingMode::Immediate);

        assert_eq!(cpu.register_y, 0x00, "Register A should be 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_lsr_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_a = 0x81;
        cpu.execute_lsr(&AddressingMode::Accumulator);

        assert_eq!(cpu.register_a, 0x40, "Register A should be 0x40!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "CPU Status: Zero should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "CPU Status: Carry should be set!");
    }

    #[test]
    fn test_ora_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_a = 0x81;
        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0xFF);
        cpu.execute_ora(&AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0xFF, "Register A should be 0xFF!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
    }

    #[test]
    fn test_pha_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        let stack_pointer_buf = cpu.stack_pointer;

        cpu.execute_pha();
        assert_eq!(cpu.stack_pointer, stack_pointer_buf.wrapping_sub(1), "Invalid Stack pointer!");

        let register_from_stack = cpu.read(0x0100 + stack_pointer_buf as u16);
        assert_eq!(cpu.register_a, register_from_stack, "Invalid value of register inside Stack!");
    }

    #[test]
    fn test_php_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        let stack_pointer_buf = cpu.stack_pointer;
        
        cpu.execute_php();
        assert_eq!(cpu.stack_pointer, stack_pointer_buf.wrapping_sub(1), "Invalid Stack pointer!");

        let status_from_stack = cpu.read(0x0100 + stack_pointer_buf as u16);
        assert_eq!(cpu.status.get(), status_from_stack, "Invalid value of register inside Stack!");
    }

    #[test]
    fn test_pla_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.write(0x150, 0xFF);
        cpu.stack_pointer = 0x4F;
        cpu.execute_pla();

        assert_eq!(cpu.register_a, 0xFF, "Register A should have 0xFF!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");

        cpu.write(0x150, 0x00);
        cpu.stack_pointer = 0x4F;
        cpu.execute_pla();

        assert_eq!(cpu.register_a, 0x00, "Register A should have 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be set!");
    }

    #[test]
    fn test_plp_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.write(0x150, 0xFF);
        cpu.stack_pointer = 0x4F;
        cpu.execute_plp();

        assert_eq!(cpu.status.get(), 0xFF, "Status should have 0xFF!");
    }

    #[test]
    fn test_rol_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.status.set_flag(CpuStatusRegisterFlags::Zero, true);
        cpu.write(0x0000, 0xAA);
        cpu.execute_rol(&AddressingMode::ZeroPage);

        let zeropage_value = cpu.read(0x0000);
        assert_eq!(zeropage_value, 0xAAu8.rotate_left(1), "Invalid value in ZeroPage!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unchanged!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");

        cpu.register_a = 0xAA;
        cpu.execute_rol(&AddressingMode::Accumulator);

        assert_eq!(cpu.register_a, 0xAAu8.rotate_left(1), "Invalid value in Register A!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
    }

    #[test]
    fn test_ror_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.status.set_flag(CpuStatusRegisterFlags::Zero, true);
        cpu.write(0x0000, 0xAA);
        cpu.execute_ror(&AddressingMode::ZeroPage);

        let zeropage_value = cpu.read(0x0000);
        assert_eq!(zeropage_value, 0xAAu8.rotate_left(1), "Invalid value in ZeroPage!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unchanged!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");

        cpu.register_a = 0xAA;
        cpu.execute_ror(&AddressingMode::Accumulator);

        assert_eq!(cpu.register_a, 0xAAu8.rotate_left(1), "Invalid value in Register A!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
    }

    #[test]
    fn test_rti_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.write(0x0152, 0xFF);
        cpu.write(0x0151, 0xAB);
        cpu.write(0x0150, 0b1010_1010);
        cpu.stack_pointer = 0x4F;
        cpu.execute_rti();

        assert_eq!(cpu.program_counter, 0xFFAB, "Program counter should have 0xAAFF!");
        assert_eq!(cpu.status.get(), 0b1010_1010, "Status should have 0b10101010!");
    }

    #[test]
    fn test_rts_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.write(0x0151, 0xFF);
        cpu.write(0x0150, 0xAA);
        cpu.stack_pointer = 0x4F;
        cpu.execute_rts();

        assert_eq!(cpu.program_counter, 0xFFAA, "Program counter should have 0xAAFF!");
    }

    #[test]
    fn test_sbc_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_a = 0x01;
        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0x80);
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, false);
        cpu.execute_sbc(&AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0x80, "Register A should have 0x80!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "Overflow flag should be set!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");

        cpu.register_a = 0x80;
        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0x01);
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, false);
        cpu.execute_sbc(&AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0x7E, "Register A should have 0x80!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "Overflow flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
    }

    #[test]
    fn test_sec_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, false);
        cpu.execute_sec();

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
    }

    #[test]
    fn test_sed_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.status.set_flag(CpuStatusRegisterFlags::DecimalMode, false);
        cpu.execute_sed();

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::DecimalMode), "Decimal mode flag should be set!");
    }

    #[test]
    fn test_sei_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.status.set_flag(CpuStatusRegisterFlags::InterruptDisable, false);
        cpu.execute_sei();

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::InterruptDisable), "Interrupt disable flag should be set!");
    }

    #[test]
    fn test_sta_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_a = 0xFF;
        cpu.write(0x0001, 0x00);
        cpu.program_counter = 0x0001;
        cpu.execute_sta(&AddressingMode::ZeroPage);

        let result = cpu.read(0x0000);
        assert_eq!(result, 0xFF, "Zero page value should have 0xFF!");
    }

    #[test]
    fn test_stx_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_x = 0xFF;
        cpu.write(0x0001, 0x00);
        cpu.program_counter = 0x0001;
        cpu.execute_stx(&AddressingMode::ZeroPage);

        let result = cpu.read(0x0000);
        assert_eq!(result, 0xFF, "Zero page value should have 0xFF!");
    }

    #[test]
    fn test_sty_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_y = 0xFF;
        cpu.write(0x0001, 0x00);
        cpu.program_counter = 0x0001;
        cpu.execute_sty(&AddressingMode::ZeroPage);

        let result = cpu.read(0x0000);
        assert_eq!(result, 0xFF, "Zero page value should have 0xFF!");
    }

    #[test]
    fn test_tax_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_a = 0xFF;
        cpu.execute_tax();

        assert_eq!(cpu.register_x, 0xFF, "Register X should have 0xFF!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.register_a = 0x00;
        cpu.execute_tax();
        assert_eq!(cpu.register_x, 0x00, "Register X should have 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_tay_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_a = 0xFF;
        cpu.execute_tay();

        assert_eq!(cpu.register_y, 0xFF, "Register Y should have 0xFF!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.register_a = 0x00;
        cpu.execute_tay();
        assert_eq!(cpu.register_y, 0x00, "Register Y should have 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_tsx_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.stack_pointer = 0xAB;
        cpu.execute_tsx();

        assert_eq!(cpu.register_x, 0xAB, "Register X should have 0xAB!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.stack_pointer = 0x00;
        cpu.execute_tsx();
        assert_eq!(cpu.register_x, 0x00, "Register X should have 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_txa_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_x = 0xAB;
        cpu.execute_txa();

        assert_eq!(cpu.register_a, 0xAB, "Register A should have 0xAB!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.register_x = 0x00;
        cpu.execute_txa();
        assert_eq!(cpu.register_a, 0x00, "Register A should have 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_txs_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_x = 0xAB;
        cpu.execute_txs();

        assert_eq!(cpu.stack_pointer, 0xAB, "Stack pointer should have 0xAB!");
    }

    #[test]
    fn test_tya_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_y = 0xAB;
        cpu.execute_tya();

        assert_eq!(cpu.register_a, 0xAB, "Register A should have 0xAB!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.register_y = 0x00;
        cpu.execute_tya();
        assert_eq!(cpu.register_a, 0x00, "Register A should have 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }
}
