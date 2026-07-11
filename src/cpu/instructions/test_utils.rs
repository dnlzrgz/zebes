#[cfg(test)]
use crate::{
    bus::Bus,
    cpu::{Cpu, addressing::Operand},
};

#[cfg(test)]
pub fn operand_at(address: u16) -> Operand {
    Operand::Address {
        address,
        page_crossed: false,
    }
}

#[cfg(test)]
pub fn assert_branch_not_taken(branch_fn: fn(&mut Cpu, Operand, &mut Bus) -> u8, cpu: &mut Cpu) {
    let mut bus = Bus::new();
    cpu.pc = 0x1000;
    let operand = operand_at(0x2000);
    let extra_cycles = branch_fn(cpu, operand, &mut bus);
    assert_eq!(cpu.pc, 0x1000);
    assert_eq!(extra_cycles, 0);
}

#[cfg(test)]
pub fn assert_branch_taken(
    branch_fn: fn(&mut Cpu, Operand, &mut Bus) -> u8,
    cpu: &mut Cpu,
    target: u16,
    expected_extra_cycles: u8,
) {
    let mut bus = Bus::new();
    let operand = operand_at(target);
    let extra_cycles = branch_fn(cpu, operand, &mut bus);
    assert_eq!(cpu.pc, target);
    assert_eq!(extra_cycles, expected_extra_cycles);
}
