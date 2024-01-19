#![recursion_limit = "1024"]

#[cfg(debug_assertions)]
pub const LOG_LEVEL: log::Level = log::Level::Debug;

#[cfg(not(debug_assertions))]
pub const LOG_LEVEL: log::Level = log::Level::Info;

use yew::prelude::*;
pub fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::new(LOG_LEVEL));
    yew::Renderer::<calendar_puzzle_web::web::App>::new().render();
}