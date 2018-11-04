# wriggling-pandas

The project aims at improving my skills in Rust and in game dev just for fun.

- [wriggling-pandas](#wriggling-pandas)
    - [About](#about)
    - [Requirements](#requirements)
    - [Configuration](#configuration)
    - [Usage](#usage)
    - [TODO](#todo)

## About

The actual videogame is intended to be played by the A.I. engine ([fluffy-penguin](https://github.com/dymayday/fluffy-penguin)), an artificial neural network combinedd with a genetic algorithm.

## Requirements

- Rust compiler: See https://www.rust-lang.org or https://doc.rust-lang.org/book/second-edition/ch01-01-installation.html
- [Graphviz](http://www.graphviz.org/): used to export to SVG the rendered artificial neural networks.

## Configuration

The graphic configuration of the game is set up for my gear, so if you encounter some trouble, first you can check if its config match your screen resolution in the file [resources/conf.toml](resources/conf.toml) and change the width and height accordingly

```toml
[window_mode]
width = 1920
height = 1080
borderless = false
fullscreen_type = "Desktop"
```

## Usage

Once cargo is installed on your system, just run:

```bash
cargo run --release
```

## TODO

- [ ] Add a visual indicator when a Panda get shot (a color blinking for example).
- [ ] Add the ability to load previous games from save files.
- [ ] Add the ability to fastforward the game / evolution process.
