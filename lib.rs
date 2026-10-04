//! # Velotype
//!
//! A high-performance, kinetic mechanical typing tutor and benchmark application built with Rust and `egui`.
//!
//! ## Architecture Overview
//!
//! Velotype is organized into clean, decoupled domain modules:
//! - [`app`]: Core GUI application state machine (`VelotypeApp`), main loop, screen router, and config auto-saving.
//! - [`audio`]: Audio synthesis and low-latency sound effects for mechanical switch clicks and errors.
//! - [`auth`]: Local user authentication, bcrypt password hashing, and active profile management.
//! - [`data`]: Configuration persistence, path resolution (`%APPDATA%/typingforge`), and structured JSON schema (`setting.json`).
//! - [`db`]: SQLite embedded database connection pool, migrations, FTS5 full-text search, and analytical queries.
//! - [`fx`]: Visual aesthetics and physics systems: spring-interpolated caret animations, particle bursts, and screen shake.
//! - [`game`]: High-level typing game models (text generation, session stats).
//! - [`typing`]: Core typing engine (`GameEngine`), live telemetry calculations (WPM, accuracy, streak, velocity), and text layout.
//! - [`ui`]: Immediate-mode user interface components, settings panels, results graph, fuzzy command palette, and themes.
//! - [`utils`]: Text sanitization, comprehensive Unicode symbol normalization, and public-domain literature generators.

pub mod app;
pub mod audio;
pub mod auth;
pub mod data;
pub mod db;
pub mod fx;
pub mod game;
pub mod typing;
pub mod ui;
pub mod utils;

pub use app::{AppScreen, VelotypeApp};

