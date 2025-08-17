# Tiny Chess

A Rust implementation of the **Salewski chess engine**, now with a lightweight `egui` interface.

![Chess UI](http://ssalewski.de/tmp/egui-chess2.png)

This Rust port improves upon the original Nim version by removing global state, fixing several bugs, and introducing new features.

---

## ✨ Features

* **Simple `egui` interface**
  Adjust time per move, pick human or engine players, and rotate the board.
* **Game modes**
  Play human vs. human or watch the engine play itself.
* **Move list**
  When run from a terminal, the full move list can be printed.
* **Non-blocking UI**
  The engine runs in its own thread, keeping the GUI responsive.

---

## 🏗 Background

The original plan was to build a Rust interface using the new **Xilem** GUI framework. However, at the time Xilem was still very early in development with little documentation or examples, making it impractical.

To test GUI programming in Rust, we created this simpler `egui` version. It also doubles as an example of **threading with `spawn` and channels** in a GUI context.

---

## 📌 Current Status

The engine logic has only received limited testing, but the project already serves as a neat demonstration of:

* `egui` with a custom graphics area
* running background tasks without blocking the UI

As of summer 2025, newer interfaces are available:

* [Xilem version](https://github.com/StefanSalewski/xilem-chess)
* [Bevy 3D version](https://github.com/StefanSalewski/Bevy-3D-Chess)

Still, this `egui` edition remains useful as a compact example for Rust GUI development.

---

## ♟ Engine Improvements

### Version 0.5 — Correct Beta-Cutoff Caching

After nearly ten years, a long-standing bug in **beta-cutoff caching** was fixed.
Previously, the engine cached cutoffs without recording the exact alpha/beta bounds. Now it stores those values and checks for compatibility when reusing cached results.

This correction:

* prevents rare endgame errors
* may slightly improve playing strength
* makes caching stricter, so its overall usefulness is debatable

Additionally, endgame transposition table usage was refined: only results matching the current search depth are trusted, ensuring the **shortest checkmate path** is always selected.

---

### Version 0.4 — Endgame Simplification

Endgame search logic was revised to avoid traps caused by repeated moves. Earlier, the engine sometimes recognized checkmate but didn’t always choose the shortest path, leading to infinite loops.

The new simplified logic:

* prevents endless repetition
* ensures reasonable endgame play
* is not perfect in theory, but in practice works well (humans rarely survive into those deep endgames anyway).

---

## 🚀 How to Run

```sh
git clone https://github.com/stefansalewski/tiny-chess.git
cd tiny-chess
cargo run
```

---

## 🔀 Alternative Interfaces

The same chess engine is available with other UIs:

* **Xilem UI** — [xilem-chess](https://github.com/StefanSalewski/xilem-chess)
* **Bevy 3D UI** — [Bevy-3D-Chess](https://github.com/StefanSalewski/Bevy-3D-Chess)

Older Nim, GTK, and blocking Egui variants are deprecated and will be removed.


