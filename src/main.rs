#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;

use anyhow::Result;
use ccconsensus::engine::PowEngine;
use fern::{
  colors::{Color, ColoredLevelConfig},
  Dispatch, FormatCallback,
};
use log::{LevelFilter, Record};
use sawtooth_sdk::consensus::zmq_driver::ZmqDriver;
use std::{fmt::Arguments, io::stdout};

const DEFAULT_ENDPOINT: &str = "tcp://localhost:5050";

const TIME_FMT: &str = "%Y-%m-%d %H:%M:%S.%3f";

fn fmt_log(out: FormatCallback, message: &Arguments, record: &Record) {
  let module: &str = record
    .module_path_static()
    .or_else(|| record.module_path())
    .unwrap_or("???");
  let colors = ColoredLevelConfig::new()
    .info(Color::Green)
    .debug(Color::Blue)
    .trace(Color::BrightMagenta);

  out.finish(format_args!(
    "[{} {:<5} {}] {}",
    chrono::Utc::now().format(TIME_FMT),
    colors.color(record.level()),
    module,
    message
  ))
}

fn setup_logs(verbose_count: u64) -> Result<()> {
  let level = match verbose_count {
    0 => LevelFilter::Warn,
    1 => LevelFilter::Info,
    2 => LevelFilter::Debug,
    _ => LevelFilter::Trace,
  };

  Dispatch::new()
    .level(level)
    .level_for("sawtooth_sdk::consensus::zmq_driver", LevelFilter::Error)
    .level_for("sawtooth_sdk::messaging::zmq_stream", LevelFilter::Error)
    .format(fmt_log)
    .chain(stdout())
    .apply()?;

  Ok(())
}

fn main() -> Result<()> {
  let matches = clap_app!(consensus_engine =>
    (version: crate_version!())
    (author: crate_authors!())
    (about: crate_description!())
    (@arg endpoint: -E --endpoint +takes_value "connection endpoint for validator")
    (@arg verbose: -v --verbose +multiple "increase output verbosity")
  )
  .get_matches();

  let endpoint: &str = matches.value_of("endpoint").unwrap_or(DEFAULT_ENDPOINT);
  setup_logs(matches.occurrences_of("verbose"))?;

  info!("PoW engine ({})", env!("CARGO_PKG_VERSION"));
  info!("PoW engine connecting to {} ...", endpoint);

  let engine: PowEngine = PowEngine::new();
  let (driver, _stop) = ZmqDriver::new();

  driver.start(endpoint, engine)?;

  info!("PoW engine exiting ...");

  Ok(())
}
