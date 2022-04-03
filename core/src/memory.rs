use std::ops::{Index, IndexMut, Range};

#[derive(Debug, Clone)]
pub struct Memory {
    rom: [u8; 0x2000],
    ram: [u8; 0x2000],
}

impl Memory {
    pub fn new(rom: [u8; 0x2000]) -> Self {
        Self {
            rom,
            ram: [0; 0x2000],
        }
    }

    pub fn reset_ram(&mut self) {
        self.ram.fill(0);
    }
}

impl Index<u16> for Memory {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        let rom_len = self.rom.len();
        let index = index as usize;

        if index < rom_len {
            &self.rom[index]
        } else {
            &self.ram[(index - rom_len) % self.ram.len()]
        }
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        let rom_len = self.rom.len();
        let index = index as usize;

        if index < rom_len { panic!("cannot write to ROM"); }

        &mut self.ram[(index - rom_len) % self.ram.len()]
    }
}

impl Index<Range<u16>> for Memory {
    type Output = [u8];

    fn index(&self, range: Range<u16>) -> &Self::Output {
        let start = range.start as usize;
        let end = range.end as usize;
        let rom_len = self.rom.len();

        if start >= rom_len {
            &self.ram[start - rom_len..end - rom_len]
        } else {
            &self.rom[start..end]
        }
    }
}