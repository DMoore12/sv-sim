use sv_sim::*;
use log::info;

fn main() {
    // Initialize logging
    env_logger::init();
    
    info!("SystemVerilog Simulator Demo");
    
    // Create a simple module programmatically
    let mut module = Module::default();
    module.name = "demo_module".to_string();
    
    // Add inputs
    module.io.inputs.push(Input {
        name: "a".to_string(),
        var: Var {
            name: "a".to_string(),
            width: 1,
            var_type: VarType::Wire,
            state: false,
            hi_z: false,
        },
    });
    
    module.io.inputs.push(Input {
        name: "b".to_string(),
        var: Var {
            name: "b".to_string(),
            width: 1,
            var_type: VarType::Wire,
            state: false,
            hi_z: false,
        },
    });
    
    // Add output
    module.io.outputs.push(Output {
        name: "c".to_string(),
        var: Var {
            name: "c".to_string(),
            width: 1,
            var_type: VarType::Wire,
            state: false,
            hi_z: false,
        },
    });
    
    // Add logic: assign c = a & b (AND gate)
    module.logic.push(Logic::Assign {
        target: "c".to_string(),
        expression: Expression::BinaryOp {
            left: Box::new(Expression::Identifier("a".to_string())),
            op: BinaryOperator::BitwiseAnd,
            right: Box::new(Expression::Identifier("b".to_string())),
        },
    });
    
    // Create testbench
    let mut testbench = Testbench::new(module);
    
    // Add test vectors for AND gate truth table
    testbench.add_test_vector(Testbench::create_test_vector(
        0, 
        vec![("a", 0), ("b", 0)], 
        vec![("c", 0)]
    ));
    
    testbench.add_test_vector(Testbench::create_test_vector(
        10, 
        vec![("a", 0), ("b", 1)], 
        vec![("c", 0)]
    ));
    
    testbench.add_test_vector(Testbench::create_test_vector(
        20, 
        vec![("a", 1), ("b", 0)], 
        vec![("c", 0)]
    ));
    
    testbench.add_test_vector(Testbench::create_test_vector(
        30, 
        vec![("a", 1), ("b", 1)], 
        vec![("c", 1)]
    ));
    
    // Run simulation
    match testbench.run() {
        Ok(results) => {
            println!("\n=== Simulation Results ===");
            println!("Total tests: {}", results.total_count());
            println!("Passed: {}", results.passed_count());
            println!("Failed: {}", results.total_count() - results.passed_count());
            
            if results.all_passed() {
                println!("✓ All tests passed!");
            } else {
                println!("✗ Some tests failed");
                for result in &results.results {
                    if !result.passed {
                        println!("  Failed: {} at time {} - expected {}, got {}", 
                            result.signal, result.time, result.expected.value, result.actual.value);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Simulation failed: {:?}", e);
        }
    }
    
    // Demonstrate waveform dumping
    let mut dumper = WaveformDumper::new("demo.vcd".to_string());
    dumper.add_signal("a".to_string());
    dumper.add_signal("b".to_string());
    dumper.add_signal("c".to_string());
    
    // Create a new simulation for waveform capture
    let mut engine = SimulationEngine::new();
    engine.initialize_signals(&testbench.module);
    
    // Simulate and record waveforms
    for time in (0..40).step_by(10) {
        let a_val = if time >= 20 { 1 } else { 0 };
        let b_val = if time >= 10 && time < 30 { 1 } else { 0 };
        
        engine.set_signal("a", SignalValue { width: 1, value: a_val, ..Default::default() });
        engine.set_signal("b", SignalValue { width: 1, value: b_val, ..Default::default() });
        
        // Execute logic
        for logic in &testbench.module.logic {
            let _ = engine.execute_logic(logic);
        }
        
        dumper.record_time_point(time, &engine);
    }
    
    // Dump VCD file
    match dumper.dump_vcd() {
        Ok(()) => println!("Waveform dumped to demo.vcd"),
        Err(e) => eprintln!("Failed to dump waveform: {}", e),
    }
    
    println!("\nDemo completed!");
}
