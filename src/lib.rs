//! Polymarket Toolkits — shared library.
//!
//! Multi-venue prediction-market trading engine. The production-ready surface
//! is the copy-trading bot (see [`bot::copy_trading`]). All other strategies
//! expose typed stubs over the same engine and risk layer.

pub mod bot;
pub mod config;
pub mod models;
pub mod service;
pub mod ui;
pub mod utils;
