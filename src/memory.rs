// memory access - uses values in mem_map to check what address being passed actually is before
// returning value
use crate::mem_map::*;

const RAM_SIZE: usize = 2 * 1024;
const VRAM_SIZE: usize = 2 * 1024;
const ROM_BLOCK_SIZE: usize = 16 * 1024;
const CHR_BLOCK_SIZE: usize = 8 * 1024;

pub struct RAM {
    ram: [u8; RAM_SIZE],
    rom: Box<[u8]>,
    ppu_ram: [u8; VRAM_SIZE],
    chr_ram: Box<[u8]>,
    ppu_regs: [u8; 8],
    ppu_reg_write: [u8; 8],
    ppu_reg_read: [u8; 8],
    OAM: [u8; 256],
    universal_bg_color: u8,
    pallette_colors: [u8; 32],
    num_prg_blocks : usize,
    num_chr_blocks : usize,
    mapper : u8,
    mirror: u8,
}

impl RAM {
    pub fn new(num_prg_blocks : usize, num_chr_blocks : usize, mapper : u8, mirror : u8) -> RAM {
        RAM {
            ram: [0; RAM_SIZE],
            rom: vec![0; ROM_BLOCK_SIZE * num_prg_blocks].into_boxed_slice(),
            ppu_ram: [0; VRAM_SIZE],
            chr_ram: vec![0; CHR_BLOCK_SIZE * num_chr_blocks].into_boxed_slice(),
            ppu_regs: [0; 8],
            ppu_reg_write: [0; 8],
            ppu_reg_read: [0; 8],
            OAM: [0; 256],
            universal_bg_color: 0,
            pallette_colors: [0; 32],
            num_prg_blocks : num_prg_blocks,
            num_chr_blocks : num_chr_blocks,
            mapper : mapper,
            mirror : mirror,
        }
    }

    pub fn clear_read_write_regs(&mut self) {
        self.ppu_reg_write = [0, 0, 0, 0, 0, 0, 0, 0];
        self.ppu_reg_read = [0, 0, 0, 0, 0, 0, 0, 0];
    }

    fn chr_debug(&self, byte1 : u8, byte2 : u8){
        for i in 0..8 {
            print!("{}", ((byte1 >> (7 - i)) & 1) + ((byte2 >> (7 - i)) & 1));
        }
        println!("");
    }

    fn block(&self){

        let mut idx = 0x1000;

        for i in 0..8 {
            self.chr_debug(self.chr_ram[idx], self.chr_ram[idx + 8]);
            self.chr_debug(self.chr_ram[idx + 1], self.chr_ram[idx + 9]);
            self.chr_debug(self.chr_ram[idx + 2], self.chr_ram[idx + 10]);
            self.chr_debug(self.chr_ram[idx + 3], self.chr_ram[idx + 11]);
            self.chr_debug(self.chr_ram[idx + 4], self.chr_ram[idx + 12]);
            self.chr_debug(self.chr_ram[idx + 5], self.chr_ram[idx + 13]);
            self.chr_debug(self.chr_ram[idx + 6], self.chr_ram[idx + 14]);
            self.chr_debug(self.chr_ram[idx + 7], self.chr_ram[idx + 15]);
            idx += 16;
            println!("");
        }

    }

    pub fn load_rom(&mut self, rom_data: Box<[u8]>) {

        let prg_len = self.num_prg_blocks * ROM_BLOCK_SIZE;
        for i in 0..prg_len {
            self.rom[i] = rom_data[i + 16];
        }

        let chr_len = self.num_chr_blocks * CHR_BLOCK_SIZE;
        for i in 0..chr_len {
            self.chr_ram[i] = rom_data[prg_len + i + 16];
        }
        // self.block();
        // panic!();
    }

    pub fn read_mem_value(&mut self, addr: u16) -> u8 {
        self.check_address_read(addr as usize)
    }

    pub fn read_mem_address(&mut self, addr: u16) -> u16 {
        let byte_one = self.check_address_read(addr as usize);
        let byte_two = self.check_address_read((addr + 1) as usize);
        ((byte_two as u16) << 8) | (byte_one as u16)
    }

    pub fn write_mem_value(&mut self, addr: u16, value: u8) {
        self.check_address_write(addr as usize, value);
    }

    pub fn write_mem_address(&mut self, addr: u16, new_addr: u16) {
        let byte_one = (new_addr) as u8;
        let byte_two = (new_addr >> 8) as u8;
        self.check_address_write(addr as usize, byte_one);
        self.check_address_write((addr + 1) as usize, byte_two);
    }

    pub fn push_address_on_stack(&mut self, stack_ptr: &mut u8, push_address: u16) {
        if *stack_ptr == 254 {
            panic!("stack overflow")
        }

        let addr = STACK_START + *stack_ptr as usize;
        self.ram[addr - 1] = push_address as u8;
        self.ram[addr] = (push_address >> 8) as u8;
        //println!("push addr on stack {:#x}, {:#x}, {:#x}", push_address, self.ram[addr], self.ram[addr - 1] );
        *stack_ptr -= 2;
    }

    pub fn push_value_on_stack(&mut self, stack_ptr: &mut u8, push_value: u8) {
        if *stack_ptr == 255 {
            panic!("stack overflow")
        }

        let addr = STACK_START + *stack_ptr as usize;
        self.ram[addr] = push_value;
        *stack_ptr -= 1;
    }

    pub fn pop_address_off_stack(&mut self, stack_ptr: &mut u8) -> u16 {
        if *stack_ptr == 0 {
            panic!("stack underflow")
        }

        *stack_ptr += 2;
        let addr = STACK_START + *stack_ptr as usize;
        let pop_addr = (self.ram[addr] as u16) << 8 | self.ram[addr - 1] as u16;

        pop_addr
    }

    pub fn pop_value_off_stack(&mut self, stack_ptr: &mut u8) -> u8 {
        *stack_ptr += 1;
        let value = self.ram[STACK_START + *stack_ptr as usize];
        value
    }

    // maps addresses to other addresses
    fn check_address_write(&mut self, address: usize, value: u8) {
        match address {
            INTERNAL_RAM_START..=INTERNAL_RAM_MIRROR_THREE_END => {
                let lookup = address & 0x7FF;
                self.ram[lookup] = value;
            }
            MIRROR_ONE_ROM_START..=MIRROR_ONE_ROM_END => {
                let base = address - 0x8000;
                self.rom[base] = value;
            }
            MIRROR_TWO_ROM_START..=MIRROR_TWO_ROM_END => {
                let base =  match self.mapper {
                    0x0 => {
                        address - 0xC000
                    },
                    _ => {
                        address
                    },
                };

                self.rom[base] = value;
            }
            PPU_REGISTERS_START..=PPU_REGISTERS_MIRRORS_END => {
                let base = address - 0x2000;
                let indx = base % 8;
                self.ppu_reg_write[indx] = 1;
                self.ppu_regs[indx] = value;
            }
            OAM_DMA => {
                // writing a byte to this causes a 256 byte page to be copied to the OAM mem
                // TODO cycles on cpu?
                let page_addr = ((value as u16) << 8) as usize;
                for i in 0..256 {
                    self.OAM[i] = self.ram[page_addr + i];
                }
            }

            _ => {
                panic!("{:#x}", address);
            }
        }
    }

    fn check_address_read(&mut self, address: usize) -> u8 {
        match address {
            INTERNAL_RAM_START..=INTERNAL_RAM_MIRROR_THREE_END => {
                let lookup = address & 0x7FF;
                self.ram[lookup]
            }
            MIRROR_ONE_ROM_START..=MIRROR_ONE_ROM_END => {
                let base = address - 0x8000;
                self.rom[base]
            }
            MIRROR_TWO_ROM_START..=MIRROR_TWO_ROM_END => {
                let base =  match self.mapper {
                    0x0 => {
                        address - 0xC000
                    },
                    _ => {
                        address
                    },
                };

                self.rom[base]
            }
            PPU_REGISTERS_START..=PPU_REGISTERS_MIRRORS_END => {
                let base = address - 0x2000;
                let indx = base % 8;
                self.ppu_reg_read[indx] = 1;
                self.ppu_regs[indx]
            }
            _ => {
                panic!("{:#x}", address);
            }
        }
    }

    pub fn write_ppu_data_no_incr(&mut self, value: u8) {
        self.ppu_regs[PPUDATA - 0x2000] = value
    }

    pub fn read_ppu_data_no_incr(&mut self) -> u8 {
        self.ppu_regs[PPUDATA - 0x2000]
    }

    pub fn write_vram_value(&mut self, address: usize, value: u8) {
        self.check_vram_write(address, value)
    }

    pub fn read_vram_value(&mut self, address: usize) -> u8 {
        let val = self.check_vram_address_read(address);
        val
    }

    // maps vram addresses to other addresses
    fn check_vram_address_read(&self, address: usize) -> u8 {
        //println!("read addr {:#x}", address);
        match address {
            PATTERN_TABLE_ZERO_START..=PATTERN_TABLE_ZERO_END => self.chr_ram[address],
            PATTERN_TABLE_ONE_START..=PATTERN_TABLE_ONE_END => self.chr_ram[address],
            NAME_TABLE_ZERO_START..=NAME_TABLE_ZERO_END => self.ppu_ram[address - NAME_TABLE_ZERO_START],
            NAME_TABLE_ONE_START..=NAME_TABLE_ONE_END => {  
                                                            if self.mirror == 0 {
                                                                self.ppu_ram[address - NAME_TABLE_ONE_START]
                                                            } else {
                                                                self.ppu_ram[address - NAME_TABLE_ZERO_START]
                                                            }
                                                         },
            NAME_TABLE_TWO_START..=NAME_TABLE_TWO_END => {
                                                                if self.mirror == 0 {
                                                                    self.ppu_ram[address - NAME_TABLE_ONE_START]
                                                                } else {
                                                                    self.ppu_ram[address - NAME_TABLE_TWO_START]
                                                                }
                                                         },
            NAME_TABLE_THREE_START..=NAME_TABLE_THREE_END => { 
                                                                self.ppu_ram[address - NAME_TABLE_TWO_START]
                                                             },
            NAME_TABLE_ZERO_MIRROR_START..=NAME_TABLE_ZERO_MIRROR_END => self.ppu_ram[address - NAME_TABLE_ZERO_MIRROR_START],
            NAME_TABLE_ONE_MIRROR_START..=NAME_TABLE_ONE_MIRROR_END => {
                                                             if self.mirror == 0 {
                                                                self.ppu_ram[address - NAME_TABLE_ONE_MIRROR_START]
                                                             } else {
                                                                self.ppu_ram[address - NAME_TABLE_ZERO_MIRROR_START]
                                                             }
                                                         },
            NAME_TABLE_TWO_MIRROR_START..=NAME_TABLE_TWO_MIRROR_END => {
                                                           if self.mirror == 0 {
                                                              self.ppu_ram[address - NAME_TABLE_ONE_MIRROR_START]
                                                           } else {
                                                                self.ppu_ram[address - NAME_TABLE_TWO_MIRROR_START]
                                                           }
                                                          },
            NAME_TABLE_THREE_MIRROR_START..=NAME_TABLE_THREE_MIRROR_END => { 
                                                               self.ppu_ram[address - NAME_TABLE_TWO_MIRROR_START]
                                                             }
            PALLETE_RAM_INDICES_START..=PALLETE_RAM_INDICES_END => {
                let base = address - PALLETE_RAM_INDICES_START;
                self.pallette_colors[base]
            },
            PALLETE_RAM_MIRRORS_START..=PALLETE_RAM_MIRRORS_END => {
                let base = (address - PALLETE_RAM_MIRRORS_START) % 0x20;
                self.pallette_colors[base]
            },
            _ => {
                panic!("{:#x}", address);
            }
        }
    }

    fn check_vram_write(&mut self, address: usize, value: u8) {
        println!("write addr {:#x} val {:#x}", address, value);
        match address {
            NAME_TABLE_ZERO_START..=NAME_TABLE_ZERO_END => self.ppu_ram[address - NAME_TABLE_ZERO_START] = value,
            NAME_TABLE_ONE_START..=NAME_TABLE_ONE_END => self.ppu_ram[address - NAME_TABLE_ZERO_START] = value,
            NAME_TABLE_TWO_START..=NAME_TABLE_TWO_END => self.ppu_ram[address - NAME_TABLE_ZERO_START] = value,
            NAME_TABLE_THREE_START..=NAME_TABLE_THREE_END => self.ppu_ram[address - NAME_TABLE_ZERO_START] = value,
            PALLETE_RAM_INDICES_START..=PALLETE_RAM_INDICES_END => {
                let base = address - PALLETE_RAM_INDICES_START;
                self.pallette_colors[base] = value;
            },
            PALLETE_RAM_MIRRORS_START..=PALLETE_RAM_MIRRORS_END => {
                let base = (address - PALLETE_RAM_MIRRORS_START) % 0x20;
                self.pallette_colors[base] = value;
            },
            _ => {
                panic!("{:#x}", address);
            }
        }
    }

    pub fn was_read(&self, idx: usize) -> bool {
        self.ppu_reg_read[idx] == 1
    }

    pub fn was_written(&self, idx: usize) -> bool {
        self.ppu_reg_write[idx] == 1
    }
}

pub fn swap_bytes(in_val: u16) -> u16 {
    let out_val = (in_val << 8) | (in_val >> 8);
    out_val
}

#[cfg(test)]
mod tests {
    #[test]
    fn mem_tests() {
        use super::*;

        // let mut test_memory  : RAM = RAM::new();
        // let mut stack_ptr = 0;

        // // init mem
        // for i in 0..2048 {
        //     test_memory.write_mem_value(i, i as u8);
        // }

        // let byteVal = 0x1FF1;

        // let newBytes = swap_bytes(byteVal);
        // assert_eq!(newBytes, 0xF11F);

        // let value = test_memory.read_mem_value(18);
        // assert_eq!(value, 18);

        // test_memory.write_mem_value(0x10, 128);
        // let value = test_memory.read_mem_value(0x10);
        // assert_eq!(value, 128);

        // let address = test_memory.read_mem_address(0x4);
        // assert_eq!(address, 0x504);

        // test_memory.write_mem_address(0x20, 0x3FFF);
        // let new_address =  test_memory.read_mem_address(0x20);
        // assert_eq!(new_address, 0x3FFF);

        // test_memory.push_address_on_stack(&mut stack_ptr, 0x286);
        // assert_eq!(stack_ptr, 2);

        // let stack_addr = test_memory.pop_address_off_stack(&mut stack_ptr);
        // println!("{:#x}", stack_addr);
        // assert_eq!(stack_ptr, 0);
        // assert_eq!(stack_addr, 0x286);

        // test_memory.push_value_on_stack(&mut stack_ptr, 0x86);
        // assert_eq!(stack_ptr, 1);

        // let stack_val = test_memory.pop_value_off_stack(&mut stack_ptr);
        // assert_eq!(stack_val, 0x86);
    }
}
