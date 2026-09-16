# 🎃 Skinny - Native PumpkinMC World Chunk Pre-Generator

[![Build Status](https://github.com/Someone-with-a-brain/skinny-pumpkin/actions/workflows/build.yml/badge.svg)](https://github.com/Someone-with-a-brain/skinny-pumpkin/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Target: WASM](https://img.shields.io/badge/Target-wasm32--wasip2-blue.svg)](https://pumpkinmc.org)

**Skinny** is a high-performance, asynchronous Minecraft world chunk pre-generator built natively for **PumpkinMC** (inspired by *Chunky* for Paper/Spigot). 

It pre-generates world chunks in concentric square or circular spiral patterns to eliminate world exploration server lag and chunk generation stutters.

---

## ✨ Features

- 🌀 **Concentric Spiral Generation**: Pre-generates chunks outward from a center point in square or circle shapes.
- ⚡ **Asynchronous Execution**: Leverages PumpkinMC's native WASM API for background chunk loading without blocking server tick rates.
- 📊 **Real-Time Telemetry**: Live Progress percentage, Chunks per Second (CPS) rate monitor, and Estimated Time Remaining (ETA).
- 🛑 **Task Control**: Pause, resume, start, or cancel pre-generation tasks dynamically via server commands.
- 🌐 **WorldGrid Ready**: Emits region readiness signals to integrate seamlessly with multi-server network setups.

---

## 💻 Command Reference

| Command | Description | Example |
| :--- | :--- | :--- |
| `/skinny center <x> <z>` | Sets center coordinates for pre-generation | `/skinny center 0 0` |
| `/skinny radius <blocks>` | Sets pre-generation radius in blocks | `/skinny radius 5000` |
| `/skinny shape <square\|circle>` | Sets boundary shape (`square` or `circle`) | `/skinny shape circle` |
| `/skinny start` | Starts pre-generation task | `/skinny start` |
| `/skinny pause` | Temporarily pauses active pre-generation task | `/skinny pause` |
| `/skinny continue` | Resumes paused pre-generation task | `/skinny continue` |
| `/skinny cancel` | Aborts current pre-generation task | `/skinny cancel` |
| `/skinny progress` | Displays current status, CPS, percentage, and ETA | `/skinny progress` |

---

## 🛠️ Building for Production (GitHub Ready)

### Prerequisites
Install Rust and the WebAssembly WASI target:
```bash
rustup target add wasm32-wasip2
```

### Build Command
In the `skinny-pumpkin` directory, run:
```bash
cargo build --release
```

The compiled native plugin file will be produced at:
```text
target/wasm32-wasip2/release/skinny_pumpkin.wasm
```

---

## 🚀 Installation & Deployment

1. Copy `skinny_pumpkin.wasm` to your PumpkinMC server's `plugins/` directory.
2. Start or reload PumpkinMC.
3. Run `/skinny radius 5000` followed by `/skinny start` to pre-generate your world!

---

## 👤 Author & AI Disclosure

- **Author**: `Someone_with_a_brain`
- **Repository**: [https://github.com/Someone-with-a-brain/skinny-pumpkin](https://github.com/Someone-with-a-brain/skinny-pumpkin)
- **AI Attribution**: This plugin was designed and developed by **Someone_with_a_brain** with AI pair-programming assistance provided by **Google DeepMind Antigravity AI**.
- **License**: MIT License

