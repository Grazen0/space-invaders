# Space Invaders

A pretty cool arcade Space Invaders emulator written in Rust, and powered by [SDL2](https://www.libsdl.org/).

## Try it out!

If you use [Nix](https://nixos.org), you can try out the program with the following command:

```bash
nix run github:Grazen0/space-invaders
```

## Building

You will need the following dependencies:

- `rustc` and `cargo`
- SDL2

The project may be built simply by using `cargo build`, or execute it directly with `cargo run`.

## Usage

The game uses the following mappings:

|     Key     |       Mapping       |
| :---------: | :-----------------: |
|      C      |     Insert coin     |
|      T      |        Tilt         |
|    Enter    |   Player 1 start    |
| Left arrow  | Player 1 move left  |
| Right arrow | Player 1 move right |
|    Up/Z     |   Player 1 shoot    |
|      X      |   Player 2 start    |
|      A      | Player 2 move left  |
|      D      | Player 2 move right |
|   Space/W   |   Player 2 shoot    |

The following additional binds are also present:

|  Key   |     Action      |
| :----: | :-------------: |
| Ctrl+Q |    Quit game    |
| Ctrl+S | Save game state |
| Ctrl+D | Load game state |
| Ctrl+R |   Reset game    |
