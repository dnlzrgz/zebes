use crate::{
    bus::Bus,
    cpu::{Cpu, addressing::Operand},
};

impl Cpu {
    /// Transfer A to X
    /// X = A
    ///
    /// TAX copies the accumulator value to the X register.
    pub fn tax(&mut self, _: Operand, _: &mut Bus) -> u8 {
        self.x = self.a;
        self.update_zn(self.x);
        0
    }

    /// Transfer A to Y
    /// Y = A
    ///
    /// TAY copies the accumulator value to the Y register.
    pub fn tay(&mut self, _: Operand, _: &mut Bus) -> u8 {
        self.y = self.a;
        self.update_zn(self.y);
        0
    }

    /// Transfer X to A
    /// A = X
    ///
    /// TXA copies the X register value to the accumulator.
    pub fn txa(&mut self, _: Operand, _: &mut Bus) -> u8 {
        self.a = self.x;
        self.update_zn(self.a);
        0
    }

    /// Transfer Y to A
    /// A = Y
    ///
    /// TYA copies the Y register value to the accumulator.
    pub fn tya(&mut self, _: Operand, _: &mut Bus) -> u8 {
        self.a = self.y;
        self.update_zn(self.a);
        0
    }
}
