# SystemVerilog Simulation

![sv-sim logo](https://github.com/DMoore12/sv-sim/blob/main/sv-sim-logo.png?raw=true)

_A simple SystemVerilog simulation tool written in rust_

## Project Scopt

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
```

## Usage

**sv-sim** uses `clap` for argument parsing. Use `cargo run -- --help` or `sv-sim[EXE] --help` to view input arguments and parameters

### Arguments

- `log_level`
    - Log level for output. Defaults to `error`
- `verbose`
    - Gives additional build information in output
