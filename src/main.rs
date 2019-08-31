mod cache;
mod chess;
mod core;
mod eval;
mod protocols;
mod result;
mod time;
mod utils;
mod version;

use crate::core::Zorro;
use crate::protocols::{Protocol, Uci};

fn main() {
    Uci::init(Zorro::default())
}