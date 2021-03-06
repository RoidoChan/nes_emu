use crate::mem_map::*;
use crate::memory::RAM;

use std::io::{stdin, stdout, Read, Write};

const NAME_TABLE_ADDRS: [usize; 4] = [0x2000, 0x2400, 0x2800, 0x2C00];
const VRAM_INCRS: [usize; 2] = [1, 32];
const PAT_TABLE_ADDR: [usize; 2] = [0, 0x1000];

const WIDTH: u32 = 256;
const HEIGHT: u32 = 240;

#[derive(Default)]
pub struct PPU {
    PPUCTRL: ppuCtrl,
    PPUMASK: ppuMask,
    PPUSTATUS: ppuStatus, // read ONLY
    OAMADDR: u8,
    OAMDATA: u8,
    PPUSCROLL: ppuScroll,
    PPUADDR: ppuAddr,

    // memory that will represent "composited" image, and will be used for output by system
    output : output_image,
    current_offset : u16,
    current_cycle  : u16,
    cycles_until_action : u8,

    current_scan_line : u16,

    current_vram_address : u16,
    temp_vram_address : u16,
    fine_x_scroll : u8,
    first_second_write_toggle : u8,

    // shift register
    tileOne : u8,
    tileTwo : u8,
    pal_attrib_one : u8,

    current_x : u16,

    name_table_addr : usize,
    v_blank : bool,
}

fn get_bit(byte: u8, index: u8) -> u8 {
    (byte & (0x1 << index)) >> index
}

impl PPU {


    pub fn run(&mut self, mem: &mut RAM) {
        // run?
        let mut reg = PPU_REGISTERS_START;

        if mem.was_written(0) {
            let ppuCtrlVal = mem.read_mem_value(reg as u16);
            self.updatePpuCtrl(ppuCtrlVal);
        }

        reg += 1;

        if mem.was_written(1) {
            let ppuMaskVal = mem.read_mem_value(reg as u16);
            self.updatePpuMask(ppuMaskVal);
        }

        reg += 1;

        // check for ppu status read
        if mem.was_read(2) {
            self.readPpuStatus();
        }

        reg += 1;

        if mem.was_written(3) {
            let OAMaddr = mem.read_mem_value(reg as u16);
            self.OAMADDR = OAMaddr;
        }

        reg += 1;

        if mem.was_written(4) {
            let OAMdata = mem.read_mem_value(reg as u16);
            // TODO
        }

        reg += 1;

        if mem.was_written(5) {
            let scroll = mem.read_mem_value(reg as u16);
            self.updatePpuScroll(scroll);
        }

        reg += 1;

        if mem.was_written(6) {
            let addr = mem.read_mem_value(reg as u16);
            self.updatePpuAddr(addr);
        }

        reg += 1;

        if mem.was_written(7) {
            let value = mem.read_ppu_data_no_incr();
            // write value to mem address stored in ppu addr
            let addr = self.PPUADDR.address as usize;
            mem.write_vram_value(addr, value);
            self.PPUADDR.address += self.PPUCTRL.VRAM_address_increment;
        }

        if mem.was_read(7) {
            let addr = self.PPUADDR.address as u16;
            let value = mem.read_mem_value(addr);
            mem.write_ppu_data_no_incr(value);
            self.PPUADDR.address += self.PPUCTRL.VRAM_address_increment;
            // TODO potential problem with internal read buffer?
        }

        mem.clear_read_write_regs();
            self.do_scan_work(mem);
        }

    fn do_scan_work(&mut self, mem: &mut RAM){
        // 3 ppu cycles per normal cpu cycle
        //let mut attrib_table_addr = ;
        //let 0 be -1 or pre render scanline

        match self.current_scan_line {
            0 => {
                //self.v_blank = false;
                self.name_table_addr = self.PPUCTRL.nametableAddress;
                self.current_x = 0;
            },
            1..=241 => {
                // different stages of "work"
                match self.current_cycle {
                    0 => {
                        self.current_x = 0;
                    },
                    1..=256 => {
                        let pixel = ( self.current_cycle - 1 ) % 8;
                        if pixel == 0 {
                            // get the values needed!
                            let index = mem.read_vram_value(self.name_table_addr) as u16;

                            // debug - can we print the chr?
                            // for every scan line there are 32 ( 8 * 32 = 256) tiles / 256 pixels
                            // each tile has 16 bytes (8 lines of two byte each - one for each bitplane)
                            // each scan line goes through 2 of these bytes per tile

                            // each 8 pixel increment moves onto next pair of bytes 16 bytes on
                            // each scan line is a byte apart from the one above

                            // so offset is scan line * 64 bytes (32 bytes )  plus current tile num 

                            let current_tile_x = self.current_x / 8;
                            let current_tile_y = (self.current_scan_line - 1) / 8;
                            let current_row = (self.current_scan_line - 1) % 8;

                            let offset = index + current_row + self.PPUCTRL.bg_pattern_table_addr as u16; //current_tile_x * 16 + current_tile_y * 512 + current_row;

                            //let offset = // + self.PPUCTRL.bg_pattern_table_addr as u16;
                            self.tileOne = mem.read_vram_value(offset as usize);
                            self.tileTwo = mem.read_vram_value(offset as usize + 8);
                            self.name_table_addr += 1;
                        }

                        let pixVal = (((self.tileOne >> (7 - pixel)) & 1) * 2) +  ((self.tileTwo >> (7 - pixel)) & 1);

                        let current_pix = ( self.current_scan_line - 1 ) * WIDTH as u16 + self.current_x as u16;
                        if(pixVal != 0){
                            // let's try own simple color scheme first?
                            match pixVal {
                                1 => {
                                    self.output.mem[current_pix as usize] = (255, 0, 0); 
                                },
                                2 => {
                                    self.output.mem[current_pix as usize] = (0, 255, 0);
                                }
                                3 => {
                                    self.output.mem[current_pix as usize] = (0, 0, 255);
                                }
                                _ => {}

                            }
                        }else{
                            self.output.mem[current_pix as usize] = (0, 0, 0); 
                        }
                       self.current_x += 1;
                    },
                    257..=320 => {
                        // TODO - for sprites
                    }
                    321..=336 => {
                        // TODO for sprites on next scanline
                    },
                    337..=340 => {
                        // dummy fetches
                    }
                    _ => {
                        //panic!();
                    }
                }
            },
            241 => {
               // post scanline
            },
            242..=261 => {
                // vert blank
                self.v_blank = true;
                panic!();
            }
            _=> {
            }
        }

        
        self.current_cycle = (self.current_cycle + 1) % 340;
        if self.current_cycle == 0 {
            self.current_scan_line = (self.current_scan_line + 1) % 241;
        }

    }

    pub fn updatePpuCtrl(&mut self, byte_val: u8) {
        println!("{:#b}", byte_val);
        let name_table_idx = byte_val & 0x03;
        self.PPUCTRL.nametableAddress = NAME_TABLE_ADDRS[name_table_idx as usize];
        let vram_incr = get_bit(byte_val, 2);
        self.PPUCTRL.VRAM_address_increment = VRAM_INCRS[vram_incr as usize];

        let spr_pattern = get_bit(byte_val, 3);
        self.PPUCTRL.sprite_pattern_table_addr = PAT_TABLE_ADDR[spr_pattern as usize];

        let bg_pattern = get_bit(byte_val, 4);
        self.PPUCTRL.bg_pattern_table_addr = PAT_TABLE_ADDR[bg_pattern as usize];

        self.PPUCTRL.sprite_size = get_bit(byte_val, 5) as usize;
        //self.PPUCTRL. = get_bit(byte_val, 6) as usize; MASTER SLAVE? TODO!
        self.PPUCTRL.gen_nmi = get_bit(byte_val, 7) as usize;
    }

    pub fn updatePpuMask(&mut self, byte_val: u8) {
        self.PPUMASK.greyScale = get_bit(byte_val, 0);
        self.PPUMASK.show_bg_left = get_bit(byte_val, 1);
        self.PPUMASK.show_spr_left = get_bit(byte_val, 2);
        self.PPUMASK.show_bg = get_bit(byte_val, 3);
        self.PPUMASK.show_spr = get_bit(byte_val, 4);
        self.PPUMASK.emphasize_red = get_bit(byte_val, 5);
        self.PPUMASK.emphasize_green = get_bit(byte_val, 6);
        self.PPUMASK.emphasize_blue = get_bit(byte_val, 7);
    }

    pub fn readPpuStatus(&mut self) {
        self.PPUSCROLL.write_byte = 0;
        self.PPUADDR.write_byte = 0;
    }

    pub fn updatePpuScroll(&mut self, byte_val: u8) {
        // two writes to this write the address that will be read when checking the nametable to render
        // starting in top left of screen
        if self.PPUSCROLL.write_byte == 0 {
            self.PPUSCROLL.horiz_offset = byte_val;
        } else {
            self.PPUSCROLL.vert_offset = byte_val;
        }
    }

    pub fn updatePpuAddr(&mut self, byte_val: u8) {
        // two writes to this write the address any writes to updatePpuAddr will write to
        // upper byte / big endian first
        if self.PPUADDR.write_byte == 0 {
            self.PPUADDR.address = 0;
            self.PPUADDR.address = ((byte_val as u16) << 8) as usize;
            self.PPUADDR.write_byte += 1;
        } else {
            self.PPUADDR.address |= ((byte_val as u16) as usize);
        }
    }

    pub fn updateOAMAddr(&mut self, byte_val: u8) {
        todo!();
    }

    pub fn updateOAMData(&mut self, byte_val: u8) {
        todo!();
    }

    pub fn draw_bg(&mut self) {
        todo!();
    }

    pub fn draw_fg(&mut self) {
        todo!();
    }

    pub fn get_output_image(&self) -> &[(u8,u8,u8)] {
        &self.output.mem
    }

    pub fn can_scan_out(&self) -> bool {
        self.v_blank
    }
}

struct ppuCtrl {
    nametableAddress: usize,
    VRAM_address_increment: usize,
    sprite_pattern_table_addr: usize,
    bg_pattern_table_addr: usize,
    sprite_size: usize,
    gen_nmi: usize,
}

impl Default for ppuCtrl {
    fn default() -> Self {
        ppuCtrl {
            nametableAddress: NAME_TABLE_ADDRS[0],
            VRAM_address_increment: VRAM_INCRS[0],
            sprite_pattern_table_addr: PAT_TABLE_ADDR[0],
            bg_pattern_table_addr: PAT_TABLE_ADDR[0],
            sprite_size: 0,
            gen_nmi: 0,
        }
    }
}

#[derive(Default)]
struct ppuMask {
    greyScale: u8,
    show_bg_left: u8,
    show_spr_left: u8,
    show_bg: u8,
    show_spr: u8,
    emphasize_red: u8,
    emphasize_green: u8,
    emphasize_blue: u8,
}

#[derive(Default)]
struct ppuStatus {
    overflow: u8,
    sprite_hit: u8,
    vert_blank_started: u8,
}

#[derive(Default)]
struct ppuScroll {
    horiz_offset: u8,
    vert_offset: u8,
    write_byte: u8,
}

#[derive(Default)]
struct ppuAddr {
    address: usize,
    write_byte: u8,
}

pub struct output_image {
    pub mem : [(u8, u8, u8); (WIDTH * HEIGHT) as usize]
}

impl Default for output_image {
    fn default() -> Self {
        output_image{
            mem : [(0, 0, 0); (WIDTH * HEIGHT) as usize]
        }
    }
}

#[cfg(test)]
pub mod Test {

    use super::*;
    use crate::memory::RAM;

    #[test]
    pub fn ppu_tests() {
        let mut test_memory: RAM = RAM::new();
        let mut ppu: PPU = PPU::default();

        // let's test ppu address writes...
        test_memory.write_mem_value(PPUADDR as u16, 0x01);
        ppu.run(&mut test_memory);
        test_memory.write_mem_value(PPUADDR as u16, 0x02);
        ppu.run(&mut test_memory);

        test_memory.write_mem_value(PPUDATA as u16, 255);
        ppu.run(&mut test_memory);
        let test_val = test_memory.read_vram_value(0x0102);
        ppu.run(&mut test_memory);
        assert_eq!(test_val, 255);

        test_memory.write_mem_value(PPUDATA as u16, 255);
        ppu.run(&mut test_memory);
        let test_val = test_memory.read_vram_value(0x0103);
        ppu.run(&mut test_memory);
        assert_eq!(test_val, 255);

        // test the y increment mode
        let status = 0b00000100;
        test_memory.write_mem_value(PPUCTRL as u16, status);
        ppu.run(&mut test_memory);

        test_memory.write_mem_value(PPUDATA as u16, 255);
        ppu.run(&mut test_memory);
        let test_val = test_memory.read_vram_value(0x0104);
        ppu.run(&mut test_memory);
        assert_eq!(test_val, 255);
        test_memory.write_mem_value(PPUDATA as u16, 255);
        ppu.run(&mut test_memory);
        let test_val = test_memory.read_vram_value(0x0124);

        assert_eq!(test_val, 255);
    }
}
