use crate::logic::{SimulationEngine, SignalValue};
use crate::module::Module;
use crate::LexingError;
use log::{info, debug};

/// Testbench for running SystemVerilog simulations
pub struct Testbench {
    /// Simulation engine
    pub engine: SimulationEngine,
    
    /// Module under test
    pub module: Module,
    
    /// Test vectors
    pub test_vectors: Vec<TestVector>,
}

/// A test vector containing input values and expected outputs
#[derive(Debug, Clone)]
pub struct TestVector {
    /// Time when this vector should be applied
    pub time: u64,
    
    /// Input signal values
    pub inputs: std::collections::HashMap<String, SignalValue>,
    
    /// Expected output values (for verification)
    pub expected_outputs: std::collections::HashMap<String, SignalValue>,
}

impl Testbench {
    /// Create a new testbench for a module
    pub fn new(module: Module) -> Self {
        let mut engine = SimulationEngine::new();
        engine.initialize_signals(&module);
        
        Self {
            engine,
            module,
            test_vectors: Vec::new(),
        }
    }
    
    /// Add a test vector
    pub fn add_test_vector(&mut self, vector: TestVector) {
        self.test_vectors.push(vector);
    }
    
    /// Run the testbench
    pub fn run(&mut self) -> Result<TestResults, LexingError> {
        let mut results = TestResults::new();
        
        info!("Starting testbench for module: {}", self.module.name);
        
        // Sort test vectors by time
        self.test_vectors.sort_by_key(|v| v.time);
        
        for vector in &self.test_vectors {
            // Apply inputs
            for (signal_name, value) in &vector.inputs {
                self.engine.set_signal(signal_name, value.clone());
                debug!("Set {} = {:?}", signal_name, value);
            }
            
            // Execute combinational logic
            for logic in &self.module.logic {
                self.engine.execute_logic(logic)?;
            }
            
            // Check outputs
            for (signal_name, expected) in &vector.expected_outputs {
                if let Some(actual) = self.engine.get_signal(signal_name) {
                    let passed = actual.value == expected.value;
                    results.add_result(TestResult {
                        time: vector.time,
                        signal: signal_name.clone(),
                        expected: expected.clone(),
                        actual: actual.clone(),
                        passed,
                    });
                    
                    if passed {
                        debug!("✓ {} = {} (expected {})", signal_name, actual.value, expected.value);
                    } else {
                        debug!("✗ {} = {} (expected {})", signal_name, actual.value, expected.value);
                    }
                } else {
                    results.add_result(TestResult {
                        time: vector.time,
                        signal: signal_name.clone(),
                        expected: expected.clone(),
                        actual: SignalValue::default(),
                        passed: false,
                    });
                }
            }
            
            // Advance time
            self.engine.run_until(vector.time);
        }
        
        info!("Testbench completed. Passed: {}/{}", results.passed_count(), results.total_count());
        
        Ok(results)
    }
    
    /// Create a simple test vector
    pub fn create_test_vector(
        time: u64,
        inputs: Vec<(&str, u64)>,
        expected_outputs: Vec<(&str, u64)>,
    ) -> TestVector {
        let mut input_map = std::collections::HashMap::new();
        let mut output_map = std::collections::HashMap::new();
        
        for (name, value) in inputs {
            input_map.insert(name.to_string(), SignalValue {
                width: 32,
                value,
                ..Default::default()
            });
        }
        
        for (name, value) in expected_outputs {
            output_map.insert(name.to_string(), SignalValue {
                width: 32,
                value,
                ..Default::default()
            });
        }
        
        TestVector {
            time,
            inputs: input_map,
            expected_outputs: output_map,
        }
    }
}

/// Results from running a testbench
#[derive(Debug)]
pub struct TestResults {
    /// Individual test results
    pub results: Vec<TestResult>,
}

impl TestResults {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }
    
    pub fn add_result(&mut self, result: TestResult) {
        self.results.push(result);
    }
    
    pub fn passed_count(&self) -> usize {
        self.results.iter().filter(|r| r.passed).count()
    }
    
    pub fn total_count(&self) -> usize {
        self.results.len()
    }
    
    pub fn all_passed(&self) -> bool {
        self.results.iter().all(|r| r.passed)
    }
}

/// Result of a single test
#[derive(Debug)]
pub struct TestResult {
    /// Time when test was performed
    pub time: u64,
    
    /// Signal being tested
    pub signal: String,
    
    /// Expected value
    pub expected: SignalValue,
    
    /// Actual value
    pub actual: SignalValue,
    
    /// Whether the test passed
    pub passed: bool,
}

/// Waveform dumper for debugging
pub struct WaveformDumper {
    /// File to write to
    pub filename: String,
    
    /// Signals to dump
    pub signals: Vec<String>,
    
    /// Time points and values
    pub data: Vec<(u64, std::collections::HashMap<String, SignalValue>)>,
}

impl WaveformDumper {
    pub fn new(filename: String) -> Self {
        Self {
            filename,
            signals: Vec::new(),
            data: Vec::new(),
        }
    }
    
    pub fn add_signal(&mut self, signal: String) {
        self.signals.push(signal);
    }
    
    pub fn record_time_point(&mut self, time: u64, engine: &SimulationEngine) {
        let mut values = std::collections::HashMap::new();
        
        for signal in &self.signals {
            if let Some(value) = engine.get_signal(signal) {
                values.insert(signal.clone(), value.clone());
            }
        }
        
        self.data.push((time, values));
    }
    
    pub fn dump_vcd(&self) -> Result<(), std::io::Error> {
        use std::fs::File;
        use std::io::Write;
        
        let mut file = File::create(&self.filename)?;
        
        // VCD header
        writeln!(file, "$version sv_sim $end")?;
        writeln!(file, "$timescale 1ps $end")?;
        
        // Variable declarations
        writeln!(file, "$scope module testbench $end")?;
        for (i, signal) in self.signals.iter().enumerate() {
            writeln!(file, "$var wire 32 {} {} $end", (b'!' + i as u8) as char, signal)?;
        }
        writeln!(file, "$upscope $end")?;
        writeln!(file, "$enddefinitions $end")?;
        
        // Initial values
        writeln!(file, "$dumpvars")?;
        if let Some((_, values)) = self.data.first() {
            for (i, signal) in self.signals.iter().enumerate() {
                if let Some(value) = values.get(signal) {
                    writeln!(file, "b{:032b} {}", value.value, (b'!' + i as u8) as char)?;
                }
            }
        }
        writeln!(file, "$end")?;
        
        // Time points
        for (time, values) in &self.data {
            writeln!(file, "#{}", time)?;
            for (i, signal) in self.signals.iter().enumerate() {
                if let Some(value) = values.get(signal) {
                    writeln!(file, "b{:032b} {}", value.value, (b'!' + i as u8) as char)?;
                }
            }
        }
        
        Ok(())
    }
}
