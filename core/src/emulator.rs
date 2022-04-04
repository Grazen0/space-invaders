use std::mem;
use crate::{concat_u16, Result, Error, CPU, CPUEvent, Button};

macro_rules! check_sound_events {
    ( $last_port:expr, $val:expr, $ev:expr, $(($msk:expr,$snd:expr)),* ) => {
        $(
            if $val & $msk != 0 && $last_port & $msk == 0 {
                $ev = Some(Event::PlaySound($snd));
            } else if $val & $msk == 0 && $last_port & $msk != 0 {
                $ev = Some(Event::StopSound($snd))
            }
        )*
    };
}

#[derive(Debug, Clone)]
pub enum ExecutionStatus {
    Continue(u32),
    Halt,
}

#[derive(Debug, Clone)]
pub enum Event {
    PlaySound(Sound),
    StopSound(Sound),
    Debug(u8),
}

#[derive(Debug, Clone)]
pub enum Sound {
    UFO,
    Shoot,
    PlayerDie,
    InvaderDie,
    Bomp1,
    Bomp2,
    Bomp3,
    Bomp4,
    UFOExplode,
}

#[derive(Debug, Clone)]
pub struct Emulator {
    cpu: CPU,
    shift_lo: u8,
    shift_hi: u8,
    shift_offset: u8,
    input_1: u8,
    input_2: u8,
    last_port_3: u8,
    last_port_5: u8,
    event: Option<Event>,
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
            last_port_3: 0,
            last_port_5: 0,
            event: None,
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

    pub fn event(&mut self) -> Option<Event> {
        mem::replace(&mut self.event, None)
    }

    fn write_port(&mut self, port: u8, val: u8) -> Result<()> {
        match port {
            2 => self.shift_offset = val & 0x7,
            3 => {
                if val != self.last_port_3 {
                    check_sound_events!(self.last_port_3, val, self.event,
                        (0x01, Sound::UFO),
                        (0x02, Sound::Shoot),
                        (0x04, Sound::PlayerDie),
                        (0x08, Sound::InvaderDie)
                    );
                    self.last_port_3 = val;
                }
            }
            4 => {
                self.shift_lo = self.shift_hi;
                self.shift_hi = val;
            }
            5 => {
                if val != self.last_port_5 {
                    check_sound_events!(self.last_port_5, val, self.event,
                        (0x01, Sound::Bomp1),
                        (0x02, Sound::Bomp2),
                        (0x04, Sound::Bomp3),
                        (0x08, Sound::Bomp4),
                        (0x10, Sound::UFOExplode)
                    );
                    self.last_port_5 = val;
                }
            }
            6 => self.event = Some(Event::Debug(val)),
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