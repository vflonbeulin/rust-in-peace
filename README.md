![Demo](demo.gif)

# 🎮 Rust in Peace

> A 3D First-Person Shooter built with Rust & Bevy.

![Status](https://img.shields.io/badge/status-WIP%20%2F%20Prototype-orange)
![Rust](https://img.shields.io/badge/Rust-1.93.0-orange?logo=rust)
![Bevy](https://img.shields.io/badge/Bevy-0.18.0-blue?logo=bevy)
![License](https://img.shields.io/badge/license-MIT-green)

---

## About

**Rust in Peace** is a 3D FPS game prototype developed in [Rust](https://www.rust-lang.org/) using the [Bevy](https://bevyengine.org/) game engine. I wanted to learn Rust and understand 3D concepts.

How to play (qwerty and azerty 🇫🇷 players) ?

```bash
w/z : forward
s : backward
a/q : left
d : right
Shoot with left click
Escape : exit
```

*Note : Tested only on linux*

---

## ✅ Current Features

- **Player movement** — First-person controls and navigation
- **Shooting system** — Basic weapon mechanics and projectiles
- **Enemies / AI** — Basic enemy behaviour and interactions
- **Levels / Maps** — Initial game environments
- **Superpowers** — Superpowers like SuperJump, SuperSpeed... 
- **Trophies** — Gold, silver and bronze

--- 

## 🛠️ Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- Cargo (included with Rust)

---

## 🚀 Getting Started

```bash
# Clone the repository
git clone https://github.com/vflonbeulin/rust-in-peace.git
cd rust-in-peace

# Run the game
cargo run
```

For a release build (better performance):

```bash
cargo run --release
```

First time with bevy ? Plz read this about compilation times : 
https://bevy.org/learn/quick-start/getting-started/setup/

---

## 📁 Project Structure

```
rust-in-peace/
├── src/
│   ├── main.rs          # Entry point
│   ├── player/          # Player movement & controls
│   ├── enemy/         # Enemy AI & behaviour
│   ├── [...]
│
├── assets/              # Textures, models, sounds
├── Cargo.toml
└── README.md
```

---

## 🤝 Contributing

Contributions are welcome! Here's how to get involved:

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/my-feature`)
3. **Commit** your changes (`git commit -m 'Add my feature'`)
4. **Push** to the branch (`git push origin feature/my-feature`)
5. **Open a Pull Request**

Please make sure your code compiles (`cargo build`) and is reasonably clean before submitting a PR.

---

## Credits

This project uses the following assets

- 3d models from Stark Crafts
- Diablo 2 library sounds
- Font Alagar
- and with my own hands using Blender & Gimp

> Feel free to open an issue if you notice a missing or incorrect credit.

---

## License

This project is licensed under the MIT License.

---

<p align="center">Made with 🦀 Rust & ❤️ by <strong>vflonbeulin</strong></p>
