pub mod main_loop;

mod bases;
mod common;
mod config;
mod constants;
#[cfg(feature = "dev")]
mod dev;
mod discoveries;
mod followers;
mod funds;
mod new_game;
mod regions;
mod rng;
mod save_load;
mod state;
mod suspicion;
mod tasks;
mod text;
mod time;
mod ui;
