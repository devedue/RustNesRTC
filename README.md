# Rust-NES-RTC

NES emulator in rust with multiplayer capabilities using WebRTC. Created using a rough translation of [olcNES by OneLoneCoder](https://github.com/OneLoneCoder/olcNES).

## Features
- [ ] GUI (Iced)
    - [X] Browse ROM
    - [X] Connection over LAN (Port 50000 for server and 60000 for client)
    - [X] Start/Stop emulation
    - [ ] Scaling
- [ ] CPU
    - [x] Official Opcodes
    - [ ] Unofficial Opcodes
- [x] PPU
- [ ] Mapper
    - [X] Mapper000
    - [ ] Others
- [ ] APU (OpenAL)
    - [x] Pulse Wave1
    - [x] Pulse Wave2
    - [ ] Tri Wave
    - [x] Noise
    - [ ] DMC
- [ ] Multiplayer
    - [x] Streaming render
    - [x] Second player input over network
    - [ ] Audio Streaming