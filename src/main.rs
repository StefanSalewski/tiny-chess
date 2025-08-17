// Plain egui frontend for the tiny Salewski chess engine
// v 0.5 -- 17-AUG-2025
// (C) 2015 - 2032 Dr. Stefan Salewski
// All rights reserved.

// Threaded UI: engine runs on a worker thread; UI uses channels to receive the move.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)]

use eframe::egui;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

mod engine;

// ────────────────────────────────────────────────────────────────────────────────
// Domain enums (avoid magic numbers)
// ────────────────────────────────────────────────────────────────────────────────

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum PlayerKind {
    Human,
    Engine,
}

// unused
impl PlayerKind {
    fn as_u8(self) -> u8 {
        match self {
            PlayerKind::Human => 0,
            PlayerKind::Engine => 1,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum TurnState {
    // A “stable” terminal state (no more moves).
    GameOver,

    // Decide whose turn it is (reads move counter & player settings).
    DecideTurn,

    // Human: waiting for source square.
    AwaitSource,

    // Human: waiting for destination square (after showing tags).
    AwaitDestination,

    // Engine: spawn worker and wait for result.
    EngineThinking,
}

// ────────────────────────────────────────────────────────────────────────────────
// UI constants and helpers
// ────────────────────────────────────────────────────────────────────────────────

const FIGURES: [&str; 13] = [
    "♚", "♛", "♜", "♝", "♞", "♟", "", "♙", "♘", "♗", "♖", "♕",
    "♔",
    //"♚", "♛", "♜", "♝", "♞", "♟\u{FE0E}", "", "♙", "♘", "♗", "♖", "♕", "♔", // the variant for black pawn avoiding emoji seems not to work in EGUI
];

fn idx(row: usize, col: usize) -> usize {
    col + row * 8
}

fn rotated_coords(rotated: bool, row: usize, col: usize) -> (usize, usize) {
    if rotated {
        (7 - row, 7 - col)
    } else {
        (row, col)
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// App
// ────────────────────────────────────────────────────────────────────────────────

struct MyApp {
    // Game state shared with worker thread
    game: Arc<Mutex<engine::Game>>,

    // UI state
    window_title: String,
    rotated: bool,
    seconds_per_move: f32,

    // For square highlights: 0=none, 1=possible move, 2=last move; -1=selected
    tags: engine::Board,

    // Who plays white/black
    players: [PlayerKind; 2],
    engine_plays_white: bool,
    engine_plays_black: bool,

    // Turn state machine
    state: TurnState,

    // Selection
    selected_from: i32,

    // Bookkeeping
    board: engine::Board,
    start_new_game: bool,

    // Engine worker
    rx: Option<mpsc::Receiver<engine::Move>>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            game: Arc::new(Mutex::new(engine::new_game())),
            window_title: "Tiny chess".to_owned(),
            rotated: true,
            seconds_per_move: 1.5,
            tags: [0; 64],
            players: [PlayerKind::Human, PlayerKind::Engine],
            engine_plays_white: false,
            engine_plays_black: true,
            state: TurnState::DecideTurn,
            selected_from: -1,
            board: [0; 64],
            start_new_game: true,
            rx: None,
        }
    }
}

fn main() -> eframe::Result<()> {
    // env_logger::init(); // enable if you want logging via RUST_LOG
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1050.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Tiny chess",
        options,
        Box::new(|cc| {
            // Enable image loading (e.g. png, jpg) for egui
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<MyApp>::default())
        }),
    )
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Tweak UI scale to keep the board crisp on most displays.
        ctx.set_pixels_per_point(1.5);

        // ── Apply deferred "new game" and sync seconds per move to engine
        if let Ok(ref mut game) = self.game.try_lock() {
            if self.start_new_game {
                engine::reset_game(game);
                self.start_new_game = false;
                self.state = TurnState::DecideTurn;
                self.tags = [0; 64];
            }
            self.board = engine::get_board(game);
            game.secs_per_move = self.seconds_per_move;
        }

        // Will capture a click, if any, during the paint pass.
        let mut clicked_col: Option<usize> = None;
        let mut clicked_row: Option<usize> = None;

        // ───────────────────────────────────────────
        // Left side panel
        // ───────────────────────────────────────────
        egui::SidePanel::left("side_panel")
            //.min_width(200.0)
            .show(ctx, |ui| {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Title(self.window_title.clone()));

                ui.heading(&self.window_title);
                //ui.add(egui::Slider::new(&mut self.seconds_per_move, 0.1..=5.0).text("Sec/move"));

                ui.label("Seconds per move");
                ui.add(egui::Slider::new(&mut self.seconds_per_move, 0.1..=5.0).text(" "));

                if ui.button("Rotate").clicked() {
                    self.rotated = !self.rotated;
                    self.tags.reverse(); // mirror highlights with the board
                }

                if ui.button("Print movelist").clicked() {
                    if let Ok(game) = self.game.lock() {
                        engine::print_move_list(&game);
                    }
                }

                if ui.button("New Game").clicked() {
                    self.start_new_game = true;
                }

                if ui
                    .checkbox(&mut self.engine_plays_white, "Engine plays white")
                    .changed()
                {
                    self.players[0] = if self.engine_plays_white {
                        PlayerKind::Engine
                    } else {
                        PlayerKind::Human
                    };
                    self.state = TurnState::DecideTurn;
                }

                if ui
                    .checkbox(&mut self.engine_plays_black, "Engine plays black")
                    .changed()
                {
                    self.players[1] = if self.engine_plays_black {
                        PlayerKind::Engine
                    } else {
                        PlayerKind::Human
                    };
                    self.state = TurnState::DecideTurn;
                }

                //ui.image(egui::include_image!("ferris.png"));
            });

        // ───────────────────────────────────────────
        // Central panel (board)
        // ───────────────────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.state == TurnState::EngineThinking {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Title(
                    " ... one moment please, reply is:".to_owned(),
                ));
            }

            let available_size = ui.available_size();
            let central_rect = ui.min_rect();
            let center = central_rect.center();

            // Square geometry
            let board_size = available_size.min_elem();
            let square = board_size / 8.0;
            let board_top_left = egui::pos2(center.x - 4.0 * square, center.y - 4.0 * square);

            let mut tiles = Vec::with_capacity(64);

            // Create 64 interactive rects
            for row in 0..8 {
                for col in 0..8 {
                    let p = idx(row, col);
                    let highlight = self.tags[p];
                    let shade = match highlight {
                        2 => 25,
                        1 => 50,
                        -1 => 0, // selected uses normal color (piece will stand out)
                        _ => 0,
                    } as u8;

                    let base = if (row + col) % 2 == 0 {
                        egui::Color32::from_rgb(255, 255, 255 - shade)
                    } else {
                        egui::Color32::from_rgb(205, 205, 205 - shade)
                    };

                    let top_left = egui::pos2(
                        board_top_left.x + (col as f32) * square,
                        board_top_left.y + (row as f32) * square,
                    );
                    let rect = egui::Rect::from_min_size(top_left, egui::vec2(square, square));
                    let response = ui.allocate_rect(rect, egui::Sense::click());

                    // Convert to logical board coords (0..=7)
                    let (b_row, b_col) = rotated_coords(self.rotated, row, col);
                    tiles.push((response, rect, base, b_row, b_col));
                }
            }

            // Paint
            let painter = ui.painter();
            for (response, rect, color, b_row, b_col) in tiles {
                if response.clicked() {
                    clicked_col = Some(b_col);
                    clicked_row = Some(b_row);
                }
                painter.rect_filled(rect, 0.0, color);

                // Draw piece using engine board
                let piece_index = (self.board[idx(b_row, b_col)] + 6) as usize;
                let glyph = FIGURES[piece_index];
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    glyph,
                    egui::FontId::proportional(square * 0.9),
                    egui::Color32::BLACK,
                );
            }

            // While engine is thinking we want to keep repainting so the UI stays responsive
            if self.state == TurnState::EngineThinking {
                ui.ctx().request_repaint();
            }
        });

        // ───────────────────────────────────────────
        // State machine
        // ───────────────────────────────────────────

        match self.state {
            TurnState::GameOver => {
                // nothing to do
            }

            TurnState::DecideTurn => {
                // Decide whose turn from game.move_counter and player settings:
                let next = (self.game.lock().unwrap().move_counter as usize) % 2;
                let actor = self.players[next];
                self.state = match actor {
                    PlayerKind::Human => TurnState::AwaitSource,
                    PlayerKind::Engine => TurnState::EngineThinking,
                };

                // If engine to move, kick off worker right away.
                if self.state == TurnState::EngineThinking {
                    let (tx, rx) = mpsc::channel();
                    self.rx = Some(rx);
                    let game = self.game.clone();
                    thread::spawn(move || {
                        let mv = engine::reply(&mut game.lock().unwrap());
                        // Ignore send errors (UI may have been closed).
                        let _ = tx.send(mv);
                    });
                }
            }

            TurnState::AwaitSource => {
                if let (Some(c), Some(r)) = (clicked_col, clicked_row) {
                    let from = (c + r * 8) as i32;
                    self.selected_from = from;
                    self.tags = [0; 64];

                    // Ask engine for legal targets and mark them
                    for m in engine::tag(&mut self.game.lock().unwrap(), from as i64) {
                        self.tags[m.di as usize] = 1;
                    }
                    self.tags[from as usize] = -1;

                    if self.rotated {
                        self.tags.reverse();
                    }
                    self.state = TurnState::AwaitDestination;
                }
            }

            TurnState::AwaitDestination => {
                if let (Some(c), Some(r)) = (clicked_col, clicked_row) {
                    let to = (c + r * 8) as i32;
                    let from = self.selected_from;

                    let valid = from != to
                        && engine::move_is_valid2(
                            &mut self.game.lock().unwrap(),
                            from as i64,
                            to as i64,
                        );

                    if !valid {
                        self.window_title = "invalid move, ignored.".to_owned();
                        self.tags = [0; 64];
                        self.state = TurnState::DecideTurn;
                        return;
                    }

                    let flag = engine::do_move(
                        &mut self.game.lock().unwrap(),
                        from as i8,
                        to as i8,
                        false,
                    );

                    self.tags = [0; 64];
                    self.tags[from as usize] = 2;
                    self.tags[to as usize] = 2;
                    if self.rotated {
                        self.tags.reverse();
                    }

                    //self.window_title = engine::move_to_str(&self.game.lock().unwrap(), from as i8, to as i8, flag);
                    self.window_title =
                        engine::move_to_str(&self.game.lock().unwrap(), from as i8, to as i8, flag)
                            .trim_start()
                            .to_string();

                    self.state = TurnState::DecideTurn;
                }
            }

            TurnState::EngineThinking => {
                // Poll for the worker result (non-blocking)
                if let Some(rx) = &self.rx {
                    if let Ok(m) = rx.try_recv() {
                        // Clear the receiver so we never read twice
                        self.rx = None;

                        if m.state == engine::STATE_CHECKMATE {
                            self.window_title = " Checkmate, game terminated!".to_owned();
                            self.state = TurnState::GameOver;
                            return;
                        }

                        self.tags = [0; 64];
                        self.tags[m.src as usize] = 2;
                        self.tags[m.dst as usize] = 2;
                        if self.rotated {
                            self.tags.reverse();
                        }

                        let flag = engine::do_move(
                            &mut self.game.lock().unwrap(),
                            m.src as i8,
                            m.dst as i8,
                            false,
                        );

                        let mut title = engine::move_to_str(
                            &self.game.lock().unwrap(),
                            m.src as i8,
                            m.dst as i8,
                            flag,
                        )
                        .trim_start()
                        .to_string();
                        title.push_str(&format!(" (scr: {})", m.score));

                        // Checkmate reporting heuristics
                        if m.checkmate_in == 2 && m.score == engine::KING_VALUE as i64 {
                            self.window_title = " Checkmate, game terminated!".to_owned();
                            self.state = TurnState::GameOver;
                            return;
                        } else if m.score > engine::KING_VALUE_DIV_2 as i64
                            || m.score < -engine::KING_VALUE_DIV_2 as i64
                        {
                            // Convert ply distance to “in N”
                            let cm_in = if m.score > 0 {
                                m.checkmate_in / 2 - 1
                            } else {
                                m.checkmate_in / 2 + 1
                            };
                            title.push_str(&format!(" Checkmate in {}", cm_in));
                        }

                        self.window_title = title;
                        self.state = TurnState::DecideTurn;
                    } else {
                        // Still thinking — request another repaint for smooth UI
                        ctx.request_repaint();
                    }
                } else {
                    // Shouldn't happen: Thinking state must hold a receiver
                    self.state = TurnState::DecideTurn;
                }
            }
        }
    }
}
// 467 lines
