//! # Chameleon
//! A general game playing (GGP) framework written to be extensible, embedable and efficient.
//!
//! Under the hood, it uses Monte-Carlo Tree Search (MCTS) with various available
//! heuristics to let you chose one best adapted to your game.
//!
//! Games are described using Rust's trait system. This allows you to:
//! - use a game description language for truly general game playing.
//! - implement your own game logic to implement engines for specific games.

pub mod game;
pub mod mcts;
