# Tiny Chess

Rust implementation of the salewski-chess engine, now featuring a straightforward `egui` user interface.

![Chess UI](http://ssalewski.de/tmp/egui-chess.png)

This Rust version of the chess engine has several enhancements and bug fixes compared to the original Nim version, and it eliminates the use of global variables.

### Features

- **User Interface**: The new plain `egui` interface allows you to set time per move, select players, and rotate the board.
- **Game Modes**: Supports human vs. human gameplay and engine auto-play.
- **Move List**: When launched from the terminal, the program can print the move list.
- **Non-blocking UI**: The chess engine runs in a background thread to prevent blocking the GUI.

### Background

We initially planned to wait for the new Rust Xilem GUI for an improved interface. However, Xilem is still in a very early stage with limited documentation and examples, which makes it challenging to use.

To test the difficulty of creating GUI programs in Rust, we developed a simple `egui` version. This serves as an example of threading using `spawn` and channels simultaneously.

### AI Assistance

Parts of the user interface were created with the help of AI tools. GPT-4 was used to design the initial board layout and the protocol for the engine's background thread.

### Current Status

The chess engine's functionality has undergone minimal testing so far. Nevertheless, it serves as a compact example of using `egui` with a custom graphic area and background task execution.

### Future Plans

We might develop a Xilem GUI by the end of this year or extend the current `egui` version. Other Rust GUI toolkits like [iced](https://iced.rs/) and [dioxus](https://dioxuslabs.com/) are also potential options.

### How to Run

```sh
git clone https://github.com/stefansalewski/tiny-chess.git
cd tiny-chess
cargo run --release
```

[Text content and layout was optimized by GPT-4]

