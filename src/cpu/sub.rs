//sub.rs - subtract with carry
use super::addressing::{self, Operation};
use crate::memory::{RAM, *};

pub fn sbc_immediate(
    operand: u8,
    pc_reg: &mut u16,
    accumulator: &mut u8,
    mut status_flags: &mut u8,
    cycles_until_next: &mut u8,
) {
    *accumulator = addressing::immediate(*accumulator, operand, &mut status_flags, Operation::Sub);
    *pc_reg += 2;
    *cycles_until_next = 2;
}

pub fn sbc_zero_page(
    operand: u8,
    pc_reg: &mut u16,
    accumulator: &mut u8,
    mut status_flags: &mut u8,
    memory: &mut RAM,
    cycles_until_next: &mut u8,
) {
    *accumulator = addressing::zero_page(
        *accumulator,
        operand,
        memory,
        &mut status_flags,
        Operation::Sub,
    );
    *pc_reg += 2;
    *cycles_until_next = 3;
}

pub fn sbc_zero_page_x(
    operand: u8,
    x_reg: u8,
    pc_reg: &mut u16,
    accumulator: &mut u8,
    mut status_flags: &mut u8,
    memory: &mut RAM,
    cycles_until_next: &mut u8,
) {
    *accumulator = addressing::zero_page_x(
        *accumulator,
        x_reg,
        operand,
        memory,
        &mut status_flags,
        Operation::Sub,
    );
    *pc_reg += 2;
    *cycles_until_next = 4;
}

pub fn sbc_absolute(
    operand: u16,
    pc_reg: &mut u16,
    accumulator: &mut u8,
    mut status_flags: &mut u8,
    memory: &mut RAM,
    cycles_until_next: &mut u8,
) {
    *accumulator = addressing::absolute(
        *accumulator,
        operand,
        memory,
        &mut status_flags,
        Operation::Sub,
    );
    *pc_reg += 3;
    *cycles_until_next = 4;
}

pub fn sbc_absolute_reg(
    operand: u16,
    reg: u8,
    pc_reg: &mut u16,
    accumulator: &mut u8,
    mut status_flags: &mut u8,
    memory: &mut RAM,
    cycles_until_next: &mut u8,
) {
    *accumulator = addressing::absolute_reg(
        *accumulator,
        reg,
        operand,
        memory,
        &mut status_flags,
        Operation::Sub,
    );
    *pc_reg += 3;
    *cycles_until_next = 4;
}

pub fn sbc_indexed_indirect(
    operand: u8,
    x_val: u8,
    pc_reg: &mut u16,
    accumulator: &mut u8,
    mut status_flags: &mut u8,
    memory: &mut RAM,
    cycles_until_next: &mut u8,
) {
    *accumulator = addressing::indexed_indirect(
        *accumulator,
        x_val,
        operand,
        memory,
        &mut status_flags,
        Operation::Sub,
    );
    *pc_reg += 2;
    *cycles_until_next = 6;
}

pub fn sbc_indirect_indexed(
    operand: u8,
    y_val: u8,
    pc_reg: &mut u16,
    accumulator: &mut u8,
    mut status_flags: &mut u8,
    memory: &mut RAM,
    cycles_until_next: &mut u8,
) {
    *accumulator = addressing::indirect_indexed(
        *accumulator,
        y_val,
        operand,
        memory,
        &mut status_flags,
        Operation::Sub,
    );
    *pc_reg += 2;
    *cycles_until_next = 5;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::flags;
    use crate::memory;

    #[test]
    fn sub_tests() {
        let operand = 12;
        let mut pc_reg = 0;
        let mut accumulator = 12;
        let mut status: u8 = 0;
        let mut test_memory: memory::RAM = memory::RAM::new();

        let mut cycles = 0;

        // init mem
        for i in 0..2048 {
            test_memory.write_mem_value(i, (i * 2) as u8);
        }

        sbc_immediate(2, &mut pc_reg, &mut accumulator, &mut status, &mut cycles);

        assert_eq!(pc_reg, 2);
        assert_eq!(accumulator, 9);
        assert_eq!(status, 0);

        status = 1;
        accumulator = 12;
        sbc_immediate(2, &mut pc_reg, &mut accumulator, &mut status, &mut cycles);

        assert_eq!(pc_reg, 4);
        assert_eq!(accumulator, 10);
    }
}
