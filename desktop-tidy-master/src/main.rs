#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use winio::prelude::*;

use crate::errors::Error;
use crate::model::MainModel;

mod errors;
mod model;
mod utils;

const PROGRESS_TIME_MS: u64 = 3000;
const PROGRESS_MAX: f64 = 1000.0;

fn main() -> std::result::Result<(), Error> {
    App::new(env!("CARGO_PKG_NAME"))?.run_until_event::<MainModel>(())?;

    Ok(())
}
