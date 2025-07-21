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

### Improvements for version 0.5

After nearly ten years, we finally fixed a bug in the beta-cutoff caching: When caching a cutoff, we have not only to store the actual
cutoff value, but also the alpha and beta parameters with which we called abeta(). And when we use this cached result, we have
to check if current alpha and beta is compatible with the cached values. This fix should finally solve the rare endgame issues,
and might improve playing strength. This correct cutoff caching is more restrictive, so one may ask if it makes much sense at all.

This version also contains the recently applied fixes for the endgame: In endgame, only data from transposition table is used, that matches the
current search depth, but not deeper knowledge. This ensures that the direct shortest path to checkmate is used.

### Temporary endgame improvements for version 0.4

The endgame logic has been improved and simplified. In endgame, a problem is to correctly estimate the number of moves to checkmate, and
to do not run into a trap repeating the same moves forever. When using the transposition table, it might occur that not always the shortest path
to checkmate is selected, leading to move repetitions: The engine sees the checkmate, but does not choose the correct shortest path, which can
lead to infinitive loops. We now use a simplified logic for this, which is simple and quite good, but not perfect. This is more a theoretical
restriction -- in an ordinary game the human player will typically not survive to the endgame state, and in most cases the computer managed the endgame quite well.

### Future Plans

We might develop a Xilem GUI by the end of this year or extend the current `egui` version. Other Rust GUI toolkits like [iced](https://iced.rs/) and [dioxus](https://dioxuslabs.com/) are also potential options.

We have already an alternative Bevy 3D GUI, for which we will replace the current engine with the one of this package soon.
Xilem is now able to use buttons with custom colors and font sizes, so we can start with a Xilem GUI soon.

### How to Run

```sh
git clone https://github.com/stefansalewski/tiny-chess.git
cd tiny-chess
cargo run --release
```


