// Logging
use chrono::Local;
use env_logger::Builder;
use log::{error, info, LevelFilter};
use std::io::Write;

// Argument parsing
use clap::Parser;

/// SystemVerilog simulation tool. Takes a single file as an output and produces
/// an object file in the same directory by default
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// File input path
    input_path: std::path::PathBuf,

    /// File output path
    output_path: Option<std::path::PathBuf>,

    /// Sets logging level (0 = off ... 4 = trace)
    #[arg(short, long, default_value_t = LevelFilter::Error)]
    log_level: LevelFilter,

    /// Enables verbose file output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

fn main() {
    let args = Cli::parse();

    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
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
        Ok(input) => {
            match sv_sim::parse_sv_file(input) {
                Ok(object) => {
                    info!(
                        "succesfully parsed input file {}",
                        &args.input_path.display()
                    );
                    format!("{object:?}");
                }
                Err(_) => (),
            };
        }
        Err(e) => error!(
            "encountered an error reading {:?}: '{}'",
            args.input_path, e
        ),
    };
}
