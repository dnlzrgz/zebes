# Zebes - Emulator

![Screenshot of Zebes running Metroid on KDE plasma](../.github/assets/metroid.png)

The playable frontend for [Zebes](../README.md), built with [macroquad](https://macroquad.rs).

## Features

- Cross-platform windowed rendering via `macroquad`.
- Nearest-neighbor texture scaling, stretched to fill the window.
- Keyboard controller input (Single player only, for now).
- Toggleable FPS counter.

## Usage

```bash
cargo run --release -p zebes -- <rom.nes>
```

### Controls

| Key     | Action             |
| ------- | ------------------ |
| `Z`     | A                  |
| `X`     | B                  |
| `Space` | Select             |
| `Enter` | Start              |
| `↑`     | Up                 |
| `↓`     | Down               |
| `←`     | Left               |
| `→`     | Right              |
| `F`     | Toggle FPS counter |
| `R`     | Reset              |

> [!Note]
> In the future I am planning to add support for controllers as well as to make them customizable.
