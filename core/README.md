# Zebes - Core

The emulator core for [Zebes](../README.md): CPU, PPU, cartridge/mapper handling, and the system bus that ties them together. This crate has no rendering or windowing code, it's just a plain Rust library used by both the [frontend](../zebes/README.md) and the [debugger](../debugger/README.md).

## Features

- CPU (Ricoh 2A03 / MOS 6502 core):
  - All [addressing modes](https://www.nesdev.org/wiki/CPU_addressing_modes).
  - All [official instructions](https://www.nesdev.org/wiki/Instruction_reference).
  - Cycle-accurate timing, including page-crossing and branch cycle penalties.
  - NMI handling.
- PPU (Ricoh 2C02):
  - Background rendering with proper scrolling (loopy registers, fine + coarse scroll).
  - Sprite rendering with sprite zero hit, per-scanline sprite evaluation, and sprite overflow flag.
  - OAM DMA.
  - PPUDATA read buffering, including the palette RAM bypass.
  - All four nametable mirroring modes.
  - NTSC odd-frame cycle skip.
- Mappers:
  - [NROM](https://www.nesdev.org/wiki/NROM).
  - [MMC1](https://www.nesdev.org/wiki/MMC1).
  - [UxROM](https://www.nesdev.org/wiki/UxROM).
  - [CNROM](https://www.nesdev.org/wiki/CNROM).

> [!NOTE]
> There is no audio emulation yet (APU), and only the official 6502 instruction set is implemented at the moment. Also, more mappers like MMC3 and MMC5 are WIP.

## Usage

Add it as a path dependency from within the workspace:

```toml
[dependencies]
zebes-core = { path = "../core" }
```

Check [`zebes`](../zebes/README.md) for a working example that turns this into a rendered window, and [`zebes-debugger`](../debugger/README.md) for stepping through execution instruction by instruction.

## Testing

```bash
cargo test
```
