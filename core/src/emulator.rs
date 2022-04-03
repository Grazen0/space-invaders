use crate::{concat_u16, Result, Error, CPU, CPUEvent, Button};

#[derive(Debug, Clone)]
pub enum ExecutionStatus {
    Continue(u32),
    Halt,
}

#[derive(Debug, Clone)]
pub struct Emulator {
    cpu: CPU,
    shift_lo: u8,
    shift_hi: u8,
    shift_offset: u8,
    input_1: u8,
    input_2: u8,
}

impl Emulator {
    pub fn new(program: &[u8]) -> Self {
        Self {
            cpu: CPU::new(program),
            shift_lo: 0,
            shift_hi: 0,
            shift_offset: 0,
            input_1: 1,
            input_2: 0,
        }
    }

    pub fn step(&mut self) -> Result<ExecutionStatus> {
        let cycles = self.cpu.step()?;

        if let Some(event) = self.cpu.event() {
            match event {
                CPUEvent::Halt => return Ok(ExecutionStatus::Halt),
                CPUEvent::PortWrite(port, val) => self.write_port(port, val)?,
                CPUEvent::PortRead(port) => {
                    let val = self.read_port(port)?;
                    self.cpu.port_in(val);
                }
            }
        }

        Ok(ExecutionStatus::Continue(cycles))
    }

    pub fn video_ram(&self) -> &[u8] {
        &self.cpu.memory[0x2400..0x4000]
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    pub fn button_press(&mut self, button: Button) {
        let mask = button.mask();
        match button {
            Button::Coin => self.input_1 &= !mask,
            Button::Tilt | Button::P2Shoot | Button::P2Left | Button::P2Right => self.input_2 |= mask,
            _ => self.input_1 |= mask,
        }
    }

    pub fn button_release(&mut self, button: Button) {
        let mask = button.mask();
        match button {
            Button::Coin => self.input_1 |= mask,
            Button::Tilt | Button::P2Shoot | Button::P2Left | Button::P2Right => self.input_2 &= !mask,
            _ => self.input_1 &= !mask,
        }
    }

    pub fn cpu_mut(&mut self) -> &mut CPU {
        &mut self.cpu
    }

    fn write_port(&mut self, port: u8, val: u8) -> Result<()> {
        match port {
            2 => self.shift_offset = val & 0x7,
            3 => {}
            4 => {
                self.shift_lo = self.shift_hi;
                self.shift_hi = val;
            }
            5 => {}
            6 => {}
            _ => return Err(Error::InvalidWritePort { port })
        }

        Ok(())
    }

    fn read_port(&mut self, port: u8) -> Result<u8> {
        Ok(match port {
            1 => self.input_1,
            2 => self.input_2,
            3 => {
                let shift_val = concat_u16!(self.shift_hi, self.shift_lo);
                ((shift_val >> (8 - self.shift_offset)) & 0xFF) as u8
            }
            _ => return Err(Error::InvalidReadPort { port })
        })
    }
}
