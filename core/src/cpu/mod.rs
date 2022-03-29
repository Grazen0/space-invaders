#[cfg(test)]
mod test;

use std::mem;
use crate::{Result, Memory, concat_u16};

pub const CARRY_FLAG: u8 = 1 << 0;
pub const PARITY_FLAG: u8 = 1 << 2;
pub const ZERO_FLAG: u8 = 1 << 6;
pub const SIGN_FLAG: u8 = 1 << 7;

#[derive(Debug)]
pub enum ExecutionStatus {
    Continue,
    Halt,
}

#[derive(Debug)]
pub enum InterruptStatus {
    Enabled,
    Disabled,
}

#[derive(Debug)]
pub struct CPU {
    memory: Memory,
    interrupt_status: InterruptStatus,
    pc: u16,
    sp: u16,
    flags: u8,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
}

impl CPU {
    pub fn new(program: &[u8]) -> Self {
        let mut rom = [0; 0x2000];
        for (i, val) in program.iter().enumerate() {
            rom[i] = *val;
        }

        Self {
            memory: Memory::new(rom),
            interrupt_status: InterruptStatus::Enabled,
            pc: 0,
            sp: 0,
            flags: 0,
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
        }
    }

    pub fn reset(&mut self) {
        self.memory.reset_ram();
        self.interrupt_status = InterruptStatus::Disabled;
        self.pc = 0;
        self.sp = 0;
        self.flags = 0;
        self.a = 0;
        self.b = 0;
        self.c = 0;
        self.d = 0;
        self.e = 0;
        self.h = 0;
        self.l = 0;
    }

    pub fn step(&mut self) -> Result<ExecutionStatus> {
        let opcode = self.read_pc();
        println!("PC: {:04X}", self.pc);

        match opcode {
            // Misc/control instructions
            0x00 | 0x10 | 0x20 | 0x30 | 0x08 | 0x18 | 0x28 | 0x38 => {} // NOP
            0x76 => return Ok(ExecutionStatus::Halt),                   // HLT
            0xD3 => {                                                   // OUT   d8
                self.read_pc();
                todo!();
            }
            0xDB => {                                                   // IN    d8
                self.read_pc();
                todo!();
            }

            0xF3 => self.interrupt_status = InterruptStatus::Disabled,  // DI
            0xFB => self.interrupt_status = InterruptStatus::Enabled,   // EI

            // Jumps/calls
            0xC0 => if self.flag(ZERO_FLAG) == 0 { self.ret(); }        // RNZ
            0xD0 => if self.flag(CARRY_FLAG) == 0 { self.ret(); }       // RNC
            0xE0 => if self.flag(PARITY_FLAG) == 0 { self.ret(); }      // RPO
            0xF0 => if self.flag(SIGN_FLAG) == 0 { self.ret(); }        // RP
            0xC2 => if self.flag(ZERO_FLAG) == 0 { self.jmp() }         // JNZ   a16
            0xD2 => if self.flag(CARRY_FLAG) == 0 { self.jmp() }        // JNC   a16
            0xE2 => if self.flag(PARITY_FLAG) == 0 { self.jmp() }       // JPO   a16
            0xF2 => if self.flag(SIGN_FLAG) == 0 { self.jmp() }         // JP    a16
            0xC3 | 0xCB => self.pc = self.read_pc_u16(),                // JMP   a16
            0xC4 => if self.flag(ZERO_FLAG) == 0 { self.call(); }       // CNZ   a16
            0xD4 => if self.flag(CARRY_FLAG) == 0 { self.call(); }      // CNC   a16
            0xE4 => if self.flag(PARITY_FLAG) == 0 { self.call(); }     // CPO   a16
            0xF4 => if self.flag(SIGN_FLAG) == 0 { self.call(); }       // CP    a16
            0xC7 => self.rst(0),                                        // RST   0
            0xCF => self.rst(1),                                        // RST   1
            0xD7 => self.rst(2),                                        // RST   2
            0xDF => self.rst(3),                                        // RST   3
            0xE7 => self.rst(4),                                        // RST   4
            0xEF => self.rst(5),                                        // RST   5
            0xF7 => self.rst(6),                                        // RST   6
            0xFF => self.rst(7),                                        // RST   7
            0xC8 => if self.flag(ZERO_FLAG) != 0 { self.ret(); }        // RZ
            0xD8 => if self.flag(CARRY_FLAG) != 0 { self.ret(); }       // RC
            0xE8 => if self.flag(PARITY_FLAG) != 0 { self.ret(); }      // RPE
            0xF8 => if self.flag(SIGN_FLAG) != 0 { self.ret(); }        // RM
            0xC9 | 0xD9 => self.ret(),                                  // RET   a16
            0xE9 => self.pc = concat_u16!(self.h, self.l),              // PCHL
            0xCA => if self.flag(ZERO_FLAG) != 0 { self.jmp() }         // JZ    a16
            0xDA => if self.flag(CARRY_FLAG) != 0 { self.jmp() }        // JC    a16
            0xEA => if self.flag(PARITY_FLAG) != 0 { self.jmp() }       // JPE   a16
            0xFA => if self.flag(SIGN_FLAG) != 0 { self.jmp() }         // JM    a16
            0xCC => if self.flag(ZERO_FLAG) != 0 { self.call(); }       // CZ    a16
            0xDC => if self.flag(CARRY_FLAG) != 0 { self.call(); }      // CC    a16
            0xEC => if self.flag(PARITY_FLAG) != 0 { self.call(); }     // CPE   a16
            0xFC => if self.flag(SIGN_FLAG) != 0 { self.call(); }       // CM    a16
            0xCD | 0xDD | 0xED | 0xFD => self.call(),                   // CALL  a16

            // 8-bit load/store/move instructions
            0x12 => *self.de_mut() = self.a,                            // STAX  D
            0x02 => *self.bc_mut() = self.a,                            // STAX  B
            0x32 => {                                                   // STA   a16
                let adr = self.read_pc_u16();
                self.memory[adr] = self.a;
            }
            0x06 => self.b = self.read_pc(),                            // MVI   B,d8
            0x0E => self.c = self.read_pc(),                            // MVI   C,d8
            0x16 => self.d = self.read_pc(),                            // MVI   D,d8
            0x1E => self.e = self.read_pc(),                            // MVI   E,d8
            0x26 => self.h = self.read_pc(),                            // MVI   H,d8
            0x2E => self.l = self.read_pc(),                            // MVI   L,d8
            0x36 => *self.m_mut() = self.read_pc(),                     // MVI   M,d8
            0x3E => self.a = self.read_pc(),                            // MVI   A,d8
            0x0A => self.a = self.bc(),                                 // LDAX  B
            0x1A => self.a = self.de(),                                 // LDAX  D
            0x3A => {                                                   // LDA   a16
                let adr = self.read_pc_u16();
                self.a = self.memory[adr];
            }
            0x40 => self.b = self.b,                                    // MOV   B,B
            0x41 => self.b = self.c,                                    // MOV   B,C
            0x42 => self.b = self.d,                                    // MOV   B,D
            0x43 => self.b = self.e,                                    // MOV   B,E
            0x44 => self.b = self.h,                                    // MOV   B,H
            0x45 => self.b = self.l,                                    // MOV   B,L
            0x46 => self.b = self.m(),                                  // MOV   B,M
            0x47 => self.b = self.a,                                    // MOV   B,A
            0x48 => self.c = self.b,                                    // MOV   C,B
            0x49 => self.c = self.c,                                    // MOV   C,C
            0x4A => self.c = self.d,                                    // MOV   C,D
            0x4B => self.c = self.e,                                    // MOV   C,E
            0x4C => self.c = self.h,                                    // MOV   C,H
            0x4D => self.c = self.l,                                    // MOV   C,L
            0x4E => self.c = self.m(),                                  // MOV   C,M
            0x4F => self.c = self.a,                                    // MOV   C,A
            0x50 => self.d = self.b,                                    // MOV   D,B
            0x51 => self.d = self.c,                                    // MOV   D,C
            0x52 => self.d = self.d,                                    // MOV   D,D
            0x53 => self.d = self.e,                                    // MOV   D,E
            0x54 => self.d = self.h,                                    // MOV   D,H
            0x55 => self.d = self.l,                                    // MOV   D,L
            0x56 => self.d = self.m(),                                  // MOV   D,M
            0x57 => self.d = self.a,                                    // MOV   D,A
            0x58 => self.e = self.b,                                    // MOV   E,B
            0x59 => self.e = self.c,                                    // MOV   E,C
            0x5A => self.e = self.d,                                    // MOV   E,D
            0x5B => self.e = self.e,                                    // MOV   E,E
            0x5C => self.e = self.h,                                    // MOV   E,H
            0x5D => self.e = self.l,                                    // MOV   E,L
            0x5E => self.e = self.m(),                                  // MOV   E,M
            0x5F => self.e = self.a,                                    // MOV   E,A
            0x60 => self.h = self.b,                                    // MOV   H,B
            0x61 => self.h = self.c,                                    // MOV   H,C
            0x62 => self.h = self.d,                                    // MOV   H,D
            0x63 => self.h = self.e,                                    // MOV   H,E
            0x64 => self.h = self.h,                                    // MOV   H,H
            0x65 => self.h = self.l,                                    // MOV   H,L
            0x66 => self.h = self.m(),                                  // MOV   H,M
            0x67 => self.h = self.a,                                    // MOV   H,A
            0x68 => self.l = self.b,                                    // MOV   L,B
            0x69 => self.l = self.c,                                    // MOV   L,C
            0x6A => self.l = self.d,                                    // MOV   L,D
            0x6B => self.l = self.e,                                    // MOV   L,E
            0x6C => self.l = self.h,                                    // MOV   L,H
            0x6D => self.l = self.l,                                    // MOV   L,L
            0x6E => self.l = self.m(),                                  // MOV   L,M
            0x6F => self.l = self.a,                                    // MOV   L,A
            0x70 => *self.m_mut() = self.b,                             // MOV   M,B
            0x71 => *self.m_mut() = self.c,                             // MOV   M,C
            0x72 => *self.m_mut() = self.d,                             // MOV   M,D
            0x73 => *self.m_mut() = self.e,                             // MOV   M,E
            0x74 => *self.m_mut() = self.h,                             // MOV   M,H
            0x75 => *self.m_mut() = self.l,                             // MOV   M,L
            0x77 => *self.m_mut() = self.a,                             // MOV   M,A
            0x78 => self.a = self.b,                                    // MOV   A,B
            0x79 => self.a = self.c,                                    // MOV   A,C
            0x7A => self.a = self.d,                                    // MOV   A,D
            0x7B => self.a = self.e,                                    // MOV   A,E
            0x7C => self.a = self.h,                                    // MOV   A,H
            0x7D => self.a = self.l,                                    // MOV   A,L
            0x7E => self.a = self.m(),                                  // MOV   A,M
            0x7F => self.a = self.a,                                    // MOV   A,A

            // 16-bit load/store/move instructions
            0x01 => {                                                   // LXI   B,d16
                self.c = self.read_pc();
                self.b = self.read_pc();
            }
            0x11 => {                                                   // LXI   D,d16
                self.e = self.read_pc();
                self.d = self.read_pc();
            }
            0x21 => {                                                   // LXI   H,d16
                self.l = self.read_pc();
                self.h = self.read_pc();
            }
            0x31 => self.sp = self.read_pc_u16(),                       // LXI   SP,d16
            0x22 => {                                                   // SHLD
                let adr = self.read_pc_u16();
                self.memory[adr] = self.l;
                self.memory[adr + 1] = self.h;
            }
            0x2A => {                                                   // LHLD
                let adr = self.read_pc_u16();
                self.l = self.memory[adr];
                self.h = self.memory[adr + 1];
            }
            0xC1 => {                                                   // POP   B
                self.c = self.stack_pop();
                self.b = self.stack_pop();
            }
            0xD1 => {                                                   // POP   D
                self.e = self.stack_pop();
                self.d = self.stack_pop();
            }
            0xE1 => {                                                   // POP   H
                self.l = self.stack_pop();
                self.h = self.stack_pop();
            }
            0xF1 => {                                                   // POP   PSW
                self.flags = self.stack_pop();
                self.a = self.stack_pop();
            },
            0xE3 => {                                                   // XTHL
                mem::swap(&mut self.h, &mut self.memory[self.sp + 1]);
                mem::swap(&mut self.l, &mut self.memory[self.sp]);
            }
            0xC5 => {                                                   // PUSH  B
                self.stack_push(self.b);
                self.stack_push(self.c);
            }
            0xD5 => {                                                   // PUSH  D
                self.stack_push(self.d);
                self.stack_push(self.e);
            }
            0xE5 => {                                                   // PUSH  H
                self.stack_push(self.h);
                self.stack_push(self.l);
            }
            0xF5 => {                                                   // PUSH  PSW
                self.stack_push(self.a);
                self.stack_push(self.flags);
            },
            0xF9 => {                                                   // SPHL
                todo!();
            }
            0xEB => {                                                   // XCHG
                mem::swap(&mut self.h, &mut self.d);
                mem::swap(&mut self.l, &mut self.e);
            }

            // 8-bit arithmetic/logical instructions
            0x04 => self.b = self.inr(self.b),                          // INR   B
            0x0C => self.c = self.inr(self.c),                          // INR   C
            0x14 => self.d = self.inr(self.d),                          // INR   D
            0x1C => self.e = self.inr(self.e),                          // INR   E
            0x24 => self.h = self.inr(self.h),                          // INR   H
            0x2C => self.l = self.inr(self.l),                          // INR   L
            0x34 => *self.m_mut() = self.inr(self.m()),                 // INR   M
            0x3C => self.a = self.inr(self.a),                          // INR   A
            0x05 => self.b = self.dcr(self.b),                          // DCR   B
            0x0D => self.c = self.dcr(self.c),                          // DCR   C
            0x15 => self.d = self.dcr(self.d),                          // DCR   D
            0x1D => self.e = self.dcr(self.e),                          // DCR   E
            0x25 => self.h = self.dcr(self.h),                          // DCR   H
            0x2D => self.l = self.dcr(self.l),                          // DCR   L
            0x35 => *self.m_mut() = self.dcr(self.m()),                 // DCR   M
            0x3D => self.a = self.dcr(self.a),                          // DCR   A
            0x07 => {                                                   // RLC
                self.set_flag(CARRY_FLAG, self.a & (1 << 7));
                self.a = self.a.rotate_left(1);
            }
            0x0F => {                                                   // RRC
                self.set_flag(CARRY_FLAG, self.a & 1);
                self.a = self.a.rotate_right(1);
            }
            0x17 => {                                                   // RAL
                let carry = self.a & (1 << 7);
                self.a = (self.a << 1) | self.flag(CARRY_FLAG);
                self.set_flag(CARRY_FLAG, carry);
            }
            0x1F => {                                                   // RAR
                let carry = self.a & 1;
                self.a = (self.a >> 1) | (self.flag(CARRY_FLAG) << 7);
                self.set_flag(CARRY_FLAG, carry);
            }
            0x27 => {                                                   // DAA
                if self.a & 0x0F > 0x09 {
                    let (result, overflow) = self.a.overflowing_add(0x06);
                    self.a = result;
                    self.set_flags(result, overflow as u8);
                }

                if self.a & 0xF0 > 0x90 {
                    let (result, overflow) = self.a.overflowing_add(0x60);
                    self.a = result;
                    self.set_flags(result, overflow as u8);
                }
            }
            0x37 => self.set_flag(CARRY_FLAG, 1),                       // STC
            0x2F => self.a = !self.a,                                   // CMA
            0x3F => self.flags ^= CARRY_FLAG,                           // CMC
            0x80 => self.add_a(self.b),                                 // ADD   B
            0x81 => self.add_a(self.c),                                 // ADD   C
            0x82 => self.add_a(self.d),                                 // ADD   D
            0x83 => self.add_a(self.e),                                 // ADD   E
            0x84 => self.add_a(self.h),                                 // ADD   H
            0x85 => self.add_a(self.l),                                 // ADD   L
            0x86 => self.add_a(self.m()),                               // ADD   M
            0x87 => self.add_a(self.a),                                 // ADD   A
            0x88 => self.add_a(self.b + self.flag(CARRY_FLAG)),         // ADC   B
            0x89 => self.add_a(self.c + self.flag(CARRY_FLAG)),         // ADC   C
            0x8A => self.add_a(self.d + self.flag(CARRY_FLAG)),         // ADC   D
            0x8B => self.add_a(self.e + self.flag(CARRY_FLAG)),         // ADC   E
            0x8C => self.add_a(self.h + self.flag(CARRY_FLAG)),         // ADC   H
            0x8D => self.add_a(self.l + self.flag(CARRY_FLAG)),         // ADC   L
            0x8E => self.add_a(self.m() + self.flag(CARRY_FLAG)),       // ADC   M
            0x8F => self.add_a(self.a + self.flag(CARRY_FLAG)),         // ADC   A
            0x90 => self.sub_a(self.b),                                 // SUB   B
            0x91 => self.sub_a(self.c),                                 // SUB   C
            0x92 => self.sub_a(self.d),                                 // SUB   D
            0x93 => self.sub_a(self.e),                                 // SUB   E
            0x94 => self.sub_a(self.h),                                 // SUB   H
            0x95 => self.sub_a(self.l),                                 // SUB   L
            0x96 => self.sub_a(self.m()),                               // SUB   M
            0x97 => self.sub_a(self.a),                                 // SUB   A
            0x98 => self.sub_a(self.b + self.flag(CARRY_FLAG)),         // ADC   B
            0x99 => self.sub_a(self.c + self.flag(CARRY_FLAG)),         // ADC   C
            0x9A => self.sub_a(self.d + self.flag(CARRY_FLAG)),         // ADC   D
            0x9B => self.sub_a(self.e + self.flag(CARRY_FLAG)),         // ADC   E
            0x9C => self.sub_a(self.h + self.flag(CARRY_FLAG)),         // ADC   H
            0x9D => self.sub_a(self.l + self.flag(CARRY_FLAG)),         // ADC   L
            0x9E => self.sub_a(self.m() + self.flag(CARRY_FLAG)),       // ADC   M
            0x9F => self.sub_a(self.a + self.flag(CARRY_FLAG)),         // ADC   A
            0xA0 => self.and_a(self.b),                                 // ANA   B
            0xA1 => self.and_a(self.c),                                 // ANA   C
            0xA2 => self.and_a(self.d),                                 // ANA   D
            0xA3 => self.and_a(self.e),                                 // ANA   E
            0xA4 => self.and_a(self.h),                                 // ANA   H
            0xA5 => self.and_a(self.l),                                 // ANA   L
            0xA6 => self.and_a(self.m()),                               // ANA   M
            0xA7 => self.and_a(self.a),                                 // ANA   A
            0xA8 => self.xor_a(self.b),                                 // XRA   B
            0xA9 => self.xor_a(self.c),                                 // XRA   C
            0xAA => self.xor_a(self.d),                                 // XRA   D
            0xAB => self.xor_a(self.e),                                 // XRA   E
            0xAC => self.xor_a(self.h),                                 // XRA   H
            0xAD => self.xor_a(self.l),                                 // XRA   L
            0xAE => self.xor_a(self.m()),                               // XRA   M
            0xAF => self.xor_a(self.a),                                 // XRA   A
            0xB0 => self.or_a(self.b),                                  // ORA   B
            0xB1 => self.or_a(self.c),                                  // ORA   C
            0xB2 => self.or_a(self.d),                                  // ORA   D
            0xB3 => self.or_a(self.e),                                  // ORA   E
            0xB4 => self.or_a(self.h),                                  // ORA   H
            0xB5 => self.or_a(self.l),                                  // ORA   L
            0xB6 => self.or_a(self.m()),                                // ORA   M
            0xB7 => self.or_a(self.a),                                  // ORA   A
            0xB8 => self.cmp_a(self.b),                                 // CMP   B
            0xB9 => self.cmp_a(self.c),                                 // CMP   C
            0xBA => self.cmp_a(self.d),                                 // CMP   D
            0xBB => self.cmp_a(self.e),                                 // CMP   E
            0xBC => self.cmp_a(self.h),                                 // CMP   H
            0xBD => self.cmp_a(self.l),                                 // CMP   L
            0xBE => self.cmp_a(self.m()),                               // CMP   M
            0xBF => self.cmp_a(self.a),                                 // CMP   A
            0xC6 => {                                                   // ADI   d8
                let d8 = self.read_pc();
                self.add_a(d8);
            }
            0xD6 => {                                                   // SUI   d8
                let d8 = self.read_pc();
                self.sub_a(d8);
            }
            0xE6 => {                                                   // ANI   d8
                let d8 = self.read_pc();
                self.and_a(d8);
            }
            0xF6 => {                                                   // ORI   d8
                let d8 = self.read_pc();
                self.or_a(d8);
            }
            0xCE => {                                                   // ACI   d8
                let d8 = self.read_pc();
                self.add_a(d8 + self.flag(CARRY_FLAG));
            }
            0xDE => {                                                   // SBI   d8
                let d8 = self.read_pc();
                self.sub_a(d8 + self.flag(CARRY_FLAG));
            }
            0xEE => {                                                   // XRI   d8
                let d8 = self.read_pc();
                self.xor_a(d8);
            }
            0xFE => {                                                   // CPI   d8
                let d8 = self.read_pc();
                self.cmp_a(d8);
            }

            // 16-bit arithmetic/logical instructions
            0x03 => Self::inx(&mut self.b, &mut self.c),                // INX   B
            0x13 => Self::inx(&mut self.d, &mut self.e),                // INX   D
            0x23 => Self::inx(&mut self.h, &mut self.l),                // INX   H
            0x33 => self.sp = self.sp.wrapping_add(1),                  // INX   SP
            0x09 => self.dad(self.b, self.c),                           // DAD   B
            0x19 => self.dad(self.d, self.e),                           // DAD   D
            0x29 => self.dad(self.h, self.l),                           // DAD   H
            0x39 => self.dad((self.sp >> 8) as u8, self.sp as u8),      // DAD   SP
            0x0B => Self::dcx(&mut self.b, &mut self.c),                // DCX   B
            0x1B => Self::dcx(&mut self.d, &mut self.e),                // DCX   D
            0x2B => Self::dcx(&mut self.h, &mut self.l),                // DCX   H
            0x3B => self.sp = self.sp.wrapping_sub(1),                  // DCX   SP
        }

        Ok(ExecutionStatus::Continue)
    }

    fn call(&mut self) {
        let adr = self.read_pc_u16();
        self.call_adr(adr);
    }

    fn rst(&mut self, code: u8) {
        self.stack_push_u16(self.pc);
        self.pc = (code << 3) as u16;
    }

    fn call_adr(&mut self, adr: u16) {
        self.stack_push_u16(self.pc);
        self.pc = adr;
    }

    fn ret(&mut self) {
        self.pc = self.stack_pop_u16();
    }

    fn jmp(&mut self) {
        self.pc = self.read_pc_u16();
    }

    fn inr(&mut self, val: u8) -> u8 {
        let result = val.wrapping_add(1);
        self.set_flags(result, self.flag(CARRY_FLAG));
        result
    }

    fn dcr(&mut self, val: u8) -> u8 {
        let result = val.wrapping_sub(1);
        self.set_flags(result, self.flag(CARRY_FLAG));
        result
    }

    fn add_a(&mut self, right: u8) {
        let (result, overflow) = self.a.overflowing_add(right);
        self.set_flags(result, overflow as u8);
        self.a = result;
    }

    fn sub_a(&mut self, val: u8) {
        let (result, underflow) = self.a.overflowing_sub(val);
        self.set_flags(result, underflow as u8);
        self.a = result;
    }

    fn and_a(&mut self, val: u8) {
        self.a &= val;
        self.set_flags(self.a, 0);
    }

    fn xor_a(&mut self, val: u8) {
        self.a ^= val;
        self.set_flags(self.a, 0);
    }

    fn or_a(&mut self, val: u8) {
        self.a |= val;
        self.set_flags(self.a, 0);
    }

    fn cmp_a(&mut self, val: u8) {
        let (result, underflow) = self.a.overflowing_sub(val);
        self.set_flags(result, underflow as u8);
    }

    fn inx(hi: &mut u8, lo: &mut u8) {
        let (result_lo, carry) = lo.overflowing_add(1);
        *lo = result_lo;
        *hi = hi.wrapping_add(carry as u8);
    }

    fn dcx(hi: &mut u8, lo: &mut u8) {
        let (result_lo, carry) = lo.overflowing_sub(1);
        *lo = result_lo;
        *hi = hi.wrapping_add(carry as u8);
    }

    fn dad(&mut self, hi: u8, lo: u8) {
        let val = concat_u16!(hi, lo);
        let hl = concat_u16!(self.h, self.l);

        let (result, carry) = hl.overflowing_add(val);
        self.set_flag(CARRY_FLAG, carry as u8);
        self.h = (result >> 8) as u8;
        self.l = (result & 0xFF) as u8
    }

    fn stack_push(&mut self, val: u8) {
        self.sp -= 1;
        self.memory[self.sp] = val;
    }

    fn stack_push_u16(&mut self, val: u16) {
        self.stack_push((val >> 8) as u8);
        self.stack_push((val & 0xFF) as u8);
    }

    fn stack_pop(&mut self) -> u8 {
        let val = self.memory[self.sp];
        self.sp += 1;
        val
    }

    fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;
        (hi << 8) | lo
    }

    fn set_flags(&mut self, val: u8, carry: u8) {
        self.set_flag(CARRY_FLAG, carry);
        self.set_flag(PARITY_FLAG, crate::even_parity(val) as u8);
        self.set_flag(ZERO_FLAG, (val == 0) as u8);
        self.set_flag(SIGN_FLAG, val & (1 << 7));
    }

    fn read_pc(&mut self) -> u8 {
        let val = self.memory[self.pc];
        self.pc += 1;
        val
    }

    fn read_pc_u16(&mut self) -> u16 {
        let val = concat_u16!(self.memory[self.pc + 1], self.memory[self.pc]);
        self.pc += 2;
        val
    }

    fn flag(&self, flag: u8) -> u8 {
        (self.flags & flag != 0).into()
    }

    fn set_flag(&mut self, flag: u8, value: u8) {
        if value != 0 {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
    }

    fn m_adr(&self) -> u16 {
        concat_u16!(self.h, self.l)
    }

    fn bc(&self) -> u8 {
        let adr = concat_u16!(self.b, self.c);
        self.memory[adr]
    }

    fn bc_mut(&mut self) -> &mut u8 {
        let adr = concat_u16!(self.b, self.c);
        &mut self.memory[adr]
    }

    fn de(&self) -> u8 {
        let adr = concat_u16!(self.d, self.e);
        self.memory[adr]
    }

    fn de_mut(&mut self) -> &mut u8 {
        let adr = concat_u16!(self.d, self.e);
        &mut self.memory[adr]
    }

    fn m(&self) -> u8 {
        let adr = self.m_adr();
        self.memory[adr]
    }

    fn m_mut(&mut self) -> &mut u8 {
        let adr = self.m_adr();
        &mut self.memory[adr]
    }
}
