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
                        "successfully parsed input file {}",
                        &args.input_path.display()
                    );

                    // Run simulation if modules are present
                    if !object.mods.is_empty() {
                        info!("Running simulation...");

                        for module in &object.mods {
                            info!("Simulating module: {}", module.name);

                            // Create a simple testbench
                            let mut testbench = sv_sim::Testbench::new(module.clone());

                            // Add some basic test vectors based on module I/O
                            if !module.io.inputs.is_empty() {
                                // Create test vectors with different input combinations
                                for i in 0..4 {
                                    let mut inputs = Vec::new();
                                    let mut outputs = Vec::new();

                                    // Set inputs to different values
                                    for (idx, input) in module.io.inputs.iter().enumerate() {
                                        inputs.push((input.name.as_str(), (i + idx) as u64));
                                    }

                                    // For now, just expect outputs to be 0
                                    for output in &module.io.outputs {
                                        outputs.push((output.name.as_str(), 0u64));
                                    }

                                    let vector = sv_sim::Testbench::create_test_vector(
                                        (i * 10) as u64, // Time in ps
                                        inputs,
                                        outputs,
                                    );

                                    testbench.add_test_vector(vector);
                                }

                                // Run the testbench
                                match testbench.run() {
                                    Ok(results) => {
                                        if results.all_passed() {
                                            info!("✓ All tests passed!");
                                        } else {
                                            info!("✗ Some tests failed. Passed: {}/{}",
                                                results.passed_count(), results.total_count());
                                        }
                                    }
                                    Err(e) => error!("Simulation error: {:?}", e),
                                }
                            } else {
                                info!("Module has no inputs, skipping simulation");
                            }
                        }
                    } else {
                        info!("No modules found in file");
                    }
                }
                Err(e) => error!("Parse error: {:?}", e),
            };
        }
        Err(e) => error!(
            "encountered an error reading {:?}: '{}'",
            args.input_path, e
        ),
    };
}
