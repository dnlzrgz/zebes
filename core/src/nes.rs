use std::{cell::RefCell, rc::Rc};

use crate::{
    cartridge::Cartridge,
    cpu::Cpu,
    cpu_bus::CpuBus,
    ppu::{Ppu, ppu_bus::PpuBus},
};

/// NES main system.
pub struct Nes {
    cpu: Cpu,
    bus: CpuBus,
}

impl Default for Nes {
    fn default() -> Self {
        Self {
            cpu: Cpu::new(),
            bus: CpuBus::new(),
        }
    }
}

impl Nes {
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a cartridge parsed from a raw iNES byte buffer.
    pub fn load(&mut self, data: &[u8]) -> Result<(), String> {
        let cartridge = Cartridge::try_from_ines(data)?;
        let cartridge = Rc::new(RefCell::new(cartridge));

        let ppu_bus = PpuBus::new().with_cartridge(cartridge.clone());
        let ppu = Ppu::new().with_bus(ppu_bus);

        self.bus = CpuBus::new().with_cartridge(cartridge).with_ppu(ppu);
        Ok(())
    }

    /// Simulates pressing the reset button.
    pub fn reset(&mut self) {
        self.cpu.reset(&self.bus);
    }

    /// Advances the system by one cycle.
    pub fn clock(&mut self) {
        self.cpu.step(&mut self.bus);
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn cpu_mut(&mut self) -> &mut Cpu {
        &mut self.cpu
    }

    pub fn bus(&self) -> &CpuBus {
        &self.bus
    }

    pub fn bus_mut(&mut self) -> &mut CpuBus {
        &mut self.bus
    }
}
