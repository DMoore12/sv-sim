# SystemVerilog Simulation

_A simple SystemVerilog simulation tool written in rust_

## Project Scope

- Provide a simple SystemVerilog parser
- Provide simple analysis tools
- Allow design verification for simple projects

## Repository Contents

- [src](https://github.com/DMoore12/sv-sim/tree/main/src): source files
- [sv](https://github.com/DMoore12/sv-sim/tree/main/sv): example SystemVerilog files for testing

## Installation

**sv-sim** uses `cargo` for package management. If you wish to generate documentation with styling, `generate_docs.sh` is provided. In order to apply styling, git submodules must be initialized.

```shell
# Clone repo
git clone https://github.com/DMoore12/sv-sim.git

# Initialize submodules
cd ./sv-sim
git submodule init

# Run test file
cargo run -- ./sv/cu_top.sv none

# Generate documentation
sudo chmod +x generate_docs.sh
./generate_docs.sh
```

## Usage

**sv-sim** uses `clap` for argument parsing. Use `cargo run -- --help` or `sv-sim[EXE] --help` to view input arguments and parameters

### Arguments

- `log_level`
    - Log level for output. Defaults to `error`
- `verbose`
    - Gives additional build information in output
