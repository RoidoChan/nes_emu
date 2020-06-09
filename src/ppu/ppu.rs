use crate::memory::RAM;
use crate::mem_map::*;

const NAME_TABLE_ADDRS : [usize; 4] = [0x2000, 0x2400, 0x2800, 0x2C00]; 
const VRAM_INCRS : [usize; 2] = [1, 32];
const PAT_TABLE_ADDR : [usize; 2] = [0, 0x1000];

#[derive(Default)]
pub struct PPU{
    PPUCTRL : ppuCtrl,
    PPUMASK : ppuMask,
    PPUSTATUS : ppuStatus, // read ONLY
    OAMADDR : u8,
    OAMDATA : u8,
    PPUSCROLL : ppuScroll,
    PPUADDR : ppuAddr,
}

fn get_bit(byte : u8, index: u8) -> u8 {
    (byte & (0x1 << index)) >> index
}

impl PPU {
    
    pub fn run(&mut self, mem : &RAM){
        // run?
        let mut reg = PPU_REGISTERS_START;

        let ppuCtrlVal = mem.read_mem_value(reg as u16);

        self.updatePpuCtrl(ppuCtrlVal);
        
    }

    pub fn updatePpuCtrl(&mut self, byte_val : u8){
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

    pub fn updatePpuMask(&mut self, byte_val : u8){
        todo!();
    }

    pub fn updatePpuStatus(&mut self, byte_val : u8){
        todo!();
    }

    pub fn updatePpuScroll(&mut self, byte_val : u8){
        todo!();
    }

    pub fn updatePpuAddr(&mut self, byte_val : u8){
        todo!();
    }

    pub fn updateOAMAddr(&mut self, byte_val : u8){
        todo!();
    }

    pub fn updateOAMData(&mut self, byte_val : u8){
        todo!();
    }
}

#[derive(Default)]
struct ppuCtrl {
    nametableAddress : usize,
    VRAM_address_increment : usize,
    sprite_pattern_table_addr : usize,
    bg_pattern_table_addr : usize,
    sprite_size : usize,
    gen_nmi : usize
}

#[derive(Default)]
struct ppuMask {
    greyScale : u8,
    show_bg_left : u8,
    show_spr_left : u8,
    show_bg : u8,
    show_spr : u8,
    emphasize_colors : u8,
}

#[derive(Default)]
struct ppuStatus {
    overflow : u8,
    sprite_hit : u8,
    vert_blank : u8
}

#[derive(Default)]
struct ppuScroll{
    address : u16,
    write_byte : u8
}

#[derive(Default)]
struct ppuAddr {
    address : u16,
    write_byte : u8
}
