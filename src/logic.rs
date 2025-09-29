use crate::LexingError;
use crate::module::{Logic, Statement, Expression};
use log::{error, trace};

/// Simulation engine for executing SystemVerilog designs
pub struct SimulationEngine {
    /// Current simulation time in picoseconds
    pub current_time: u64,

    /// Signal values storage
    pub signals: std::collections::HashMap<String, SignalValue>,

    /// Event queue for scheduled updates
    pub event_queue: std::collections::BinaryHeap<SimulationEvent>,
}

/// Represents a signal value in the simulation
#[derive(Debug, Clone)]
pub struct SignalValue {
    /// Signal width in bits
    pub width: u64,

    /// Current value (for multi-bit signals, stored as u64)
    pub value: u64,

    /// High-impedance state
    pub hi_z: bool,

    /// Unknown state
    pub unknown: bool,
}

impl Default for SignalValue {
    fn default() -> Self {
        Self {
            width: 1,
            value: 0,
            hi_z: false,
            unknown: false,
        }
    }
}

/// Simulation event for the event queue
#[derive(Debug, Clone)]
pub struct SimulationEvent {
    /// Time when event should occur
    pub time: u64,

    /// Signal to update
    pub signal: String,

    /// New value for the signal
    pub value: SignalValue,
}

impl PartialEq for SimulationEvent {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl Eq for SimulationEvent {}

impl PartialOrd for SimulationEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SimulationEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse ordering for min-heap behavior
        other.time.cmp(&self.time)
    }
}

impl SimulationEngine {
    /// Create a new simulation engine
    pub fn new() -> Self {
        Self {
            current_time: 0,
            signals: std::collections::HashMap::new(),
            event_queue: std::collections::BinaryHeap::new(),
        }
    }

    /// Initialize signals from module
    pub fn initialize_signals(&mut self, module: &crate::module::Module) {
        // Initialize inputs
        for input in &module.io.inputs {
            self.signals.insert(
                input.name.clone(),
                SignalValue {
                    width: input.var.width,
                    ..Default::default()
                }
            );
        }

        // Initialize outputs
        for output in &module.io.outputs {
            self.signals.insert(
                output.name.clone(),
                SignalValue {
                    width: output.var.width,
                    ..Default::default()
                }
            );
        }

        // Initialize internal variables
        for var in &module.vars {
            self.signals.insert(
                var.name.clone(),
                SignalValue {
                    width: var.width,
                    ..Default::default()
                }
            );
        }
    }

    /// Set a signal value
    pub fn set_signal(&mut self, name: &str, value: SignalValue) {
        self.signals.insert(name.to_string(), value);
    }

    /// Get a signal value
    pub fn get_signal(&self, name: &str) -> Option<&SignalValue> {
        self.signals.get(name)
    }

    /// Schedule an event
    pub fn schedule_event(&mut self, time: u64, signal: String, value: SignalValue) {
        self.event_queue.push(SimulationEvent {
            time,
            signal,
            value,
        });
    }

    /// Process events until a specific time
    pub fn run_until(&mut self, end_time: u64) {
        while let Some(event) = self.event_queue.peek() {
            if event.time > end_time {
                break;
            }

            let event = self.event_queue.pop().unwrap();
            self.current_time = event.time;
            self.signals.insert(event.signal, event.value);
        }

        self.current_time = end_time;
    }

    /// Evaluate an expression
    pub fn evaluate_expression(&self, expr: &Expression) -> Result<SignalValue, LexingError> {
        match expr {
            Expression::Identifier(name) => {
                self.get_signal(name)
                    .cloned()
                    .ok_or(LexingError::UnexpectedToken)
            }
            Expression::Integer(val) => Ok(SignalValue {
                width: 32, // Default width for integers
                value: *val,
                ..Default::default()
            }),
            Expression::Binary { width, value } => Ok(SignalValue {
                width: *width,
                value: *value,
                ..Default::default()
            }),
            Expression::BinaryOp { left, op, right } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;

                let result_value = match op {
                    crate::module::BinaryOperator::Add => left_val.value + right_val.value,
                    crate::module::BinaryOperator::Subtract => left_val.value - right_val.value,
                    crate::module::BinaryOperator::Equal => if left_val.value == right_val.value { 1 } else { 0 },
                    crate::module::BinaryOperator::NotEqual => if left_val.value != right_val.value { 1 } else { 0 },
                    crate::module::BinaryOperator::LessThan => if left_val.value < right_val.value { 1 } else { 0 },
                    crate::module::BinaryOperator::GreaterThan => if left_val.value > right_val.value { 1 } else { 0 },
                    crate::module::BinaryOperator::BitwiseAnd => left_val.value & right_val.value,
                    crate::module::BinaryOperator::BitwiseOr => left_val.value | right_val.value,
                    crate::module::BinaryOperator::BitwiseXor => left_val.value ^ right_val.value,
                    _ => {
                        error!("unsupported binary operator: {:?}", op);
                        return Err(LexingError::UnexpectedToken);
                    }
                };

                Ok(SignalValue {
                    width: std::cmp::max(left_val.width, right_val.width),
                    value: result_value,
                    ..Default::default()
                })
            }
            Expression::UnaryOp { op, operand } => {
                let operand_val = self.evaluate_expression(operand)?;

                let result_value = match op {
                    crate::module::UnaryOperator::Not => if operand_val.value == 0 { 1 } else { 0 },
                    crate::module::UnaryOperator::BitwiseNot => !operand_val.value,
                    crate::module::UnaryOperator::ReductionAnd => {
                        // Reduction AND: all bits must be 1
                        let mask = (1u64 << operand_val.width) - 1;
                        if (operand_val.value & mask) == mask { 1 } else { 0 }
                    }
                    crate::module::UnaryOperator::ReductionOr => {
                        // Reduction OR: any bit must be 1
                        if operand_val.value != 0 { 1 } else { 0 }
                    }
                    crate::module::UnaryOperator::ReductionXor => {
                        // Reduction XOR: odd number of 1s
                        if operand_val.value.count_ones() % 2 == 1 { 1 } else { 0 }
                    }
                };

                Ok(SignalValue {
                    width: if matches!(op, crate::module::UnaryOperator::BitwiseNot) {
                        operand_val.width
                    } else {
                        1
                    },
                    value: result_value,
                    ..Default::default()
                })
            }
            _ => {
                error!("unsupported expression type: {:?}", expr);
                Err(LexingError::UnexpectedToken)
            }
        }
    }

    /// Execute a logic block
    pub fn execute_logic(&mut self, logic: &Logic) -> Result<(), LexingError> {
        match logic {
            Logic::Assign { target, expression } => {
                let value = self.evaluate_expression(expression)?;
                self.set_signal(target, value);
                Ok(())
            }
            Logic::AlwaysComb { statements } => {
                for statement in statements {
                    self.execute_statement(statement)?;
                }
                Ok(())
            }
            Logic::AlwaysFF { sensitivity: _, statements } => {
                // For now, execute sequential logic like combinational
                // In a full implementation, this would be triggered by clock edges
                for statement in statements {
                    self.execute_statement(statement)?;
                }
                Ok(())
            }
            Logic::Always { sensitivity: _, statements } => {
                // Execute general always blocks
                for statement in statements {
                    self.execute_statement(statement)?;
                }
                Ok(())
            }
            Logic::Generate { statements: _ } => {
                // For now, skip generate blocks
                // In a full implementation, these would be elaborated at compile time
                trace!("Skipping generate block execution");
                Ok(())
            }
        }
    }

    /// Execute a statement
    pub fn execute_statement(&mut self, statement: &Statement) -> Result<(), LexingError> {
        match statement {
            Statement::Assignment { target, expression, blocking: _ } => {
                let value = self.evaluate_expression(expression)?;
                self.set_signal(target, value);
                Ok(())
            }
            Statement::If { condition, then_stmt, else_stmt } => {
                let cond_value = self.evaluate_expression(condition)?;
                if cond_value.value != 0 {
                    self.execute_statement(then_stmt)?;
                } else if let Some(else_stmt) = else_stmt {
                    self.execute_statement(else_stmt)?;
                }
                Ok(())
            }
            Statement::Block { statements } => {
                for stmt in statements {
                    self.execute_statement(stmt)?;
                }
                Ok(())
            }
            Statement::Case { expression, cases, default } => {
                let expr_value = self.evaluate_expression(expression)?;

                // Try to match against each case
                for case in cases {
                    for case_value in &case.values {
                        let case_val = self.evaluate_expression(case_value)?;
                        if expr_value.value == case_val.value {
                            for stmt in &case.statements {
                                self.execute_statement(stmt)?;
                            }
                            return Ok(());
                        }
                    }
                }

                // Execute default case if no match
                if let Some(default_stmts) = default {
                    for stmt in default_stmts {
                        self.execute_statement(stmt)?;
                    }
                }
                Ok(())
            }
            Statement::For { init, condition, update, body } => {
                // Execute initialization
                self.execute_statement(init)?;

                // Loop while condition is true
                loop {
                    let cond_value = self.evaluate_expression(condition)?;
                    if cond_value.value == 0 {
                        break;
                    }

                    // Execute body
                    self.execute_statement(body)?;

                    // Execute update
                    self.execute_statement(update)?;
                }
                Ok(())
            }
            Statement::While { condition, body } => {
                loop {
                    let cond_value = self.evaluate_expression(condition)?;
                    if cond_value.value == 0 {
                        break;
                    }

                    self.execute_statement(body)?;
                }
                Ok(())
            }
            Statement::FunctionCall { name: _, arguments: _ } => {
                // For now, skip function calls
                trace!("Skipping function call execution");
                Ok(())
            }
            Statement::TaskCall { name: _, arguments: _ } => {
                // For now, skip task calls
                trace!("Skipping task call execution");
                Ok(())
            }
            Statement::Return { value: _ } => {
                // For now, skip return statements
                trace!("Skipping return statement execution");
                Ok(())
            }
        }
    }
}

impl Default for SimulationEngine {
    fn default() -> Self {
        Self::new()
    }
}
