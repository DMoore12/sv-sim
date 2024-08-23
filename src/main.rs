// Logging
use chrono::Local;
use std::io::Write;
use env_logger::Builder;
use log::{info, warn, error, debug, trace, LevelFilter};

// Argument parsing
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// File input path
    input_path:  std::path::PathBuf,

    /// File output path
    output_path: std::path::PathBuf,

    /// Sets logging level (0 = off ... 4 = trace)
    #[arg(short, long, default_value_t = LevelFilter::Info)]
    log_level: LevelFilter,

    /// Enables verbose file output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

fn main() {
    let args = Cli::parse();

    Builder::new()
        .format(|buf, record| {
                writeln!(buf,
                    "{} [{}] - {}",
                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                    record.level(),
                    record.args()
            )
        })
        .filter(None, args.log_level)
        .init();

    let ret = sv_sim::read_sv_file(&args.input_path);

    match ret {
        Ok(input) => { sv_sim::parse_sv_file(input).expect("failed to parse input"); },
        Err(e) => error!("encountered an error reading {:?}: '{}'", args.input_path, e),
    };
}
