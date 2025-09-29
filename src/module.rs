use crate::var_types::*;
use log::debug;
use std::fmt;

/// SystemVerilog module representation
///
/// Contains a module I/O header, variable assignments, combinational
/// and sequential logic, as well as any constants
#[derive(Default, Clone)]
pub struct Module {
    /// Module friendly name
    pub name: String,

    /// Module parameters
    pub parameters: Vec<Parameter>,

    /// Module I/O information
    pub io: ModuleIO,

    /// Module "variables" (wire, reg, etc.)
    pub vars: Vec<Var>,

    /// Module logic blocks (assign, always, etc.)
    pub logic: Vec<Logic>,

    /// Module instantiations
    pub instances: Vec<ModuleInstance>,

    /// Functions defined in this module
    pub functions: Vec<Function>,

    /// Tasks defined in this module
    pub tasks: Vec<Task>,
}

impl fmt::Debug for Module {
    fn fmt(&self, _: &mut std::fmt::Formatter) -> fmt::Result {
        debug!("MODULE: {:?}", self.name);
        let _ = format!("{0:?}", self.io);
        for var in self.vars.clone() {
            debug!("VAR: {:?}", var);
        }
        Ok(())
    }
}

/// Parameter definition
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub value: Expression,
    pub param_type: ParameterType,
}

/// Parameter types
#[derive(Debug, Clone)]
pub enum ParameterType {
    Parameter,
    Localparam,
}

/// Module instance
#[derive(Debug, Clone)]
pub struct ModuleInstance {
    pub module_name: String,
    pub instance_name: String,
    pub parameters: Vec<ParameterOverride>,
    pub connections: Vec<Connection>,
}

/// Parameter override in module instantiation
#[derive(Debug, Clone)]
pub struct ParameterOverride {
    pub name: String,
    pub value: Expression,
}

/// Port connection in module instantiation
#[derive(Debug, Clone)]
pub struct Connection {
    pub port_name: String,
    pub signal_name: String,
}

/// Function definition
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub return_type: VarType,
    pub return_width: u64,
    pub inputs: Vec<FunctionInput>,
    pub statements: Vec<Statement>,
}

/// Function input
#[derive(Debug, Clone)]
pub struct FunctionInput {
    pub name: String,
    pub var_type: VarType,
    pub width: u64,
}

/// Task definition
#[derive(Debug, Clone)]
pub struct Task {
    pub name: String,
    pub inputs: Vec<TaskInput>,
    pub outputs: Vec<TaskOutput>,
    pub statements: Vec<Statement>,
}

/// Task input
#[derive(Debug, Clone)]
pub struct TaskInput {
    pub name: String,
    pub var_type: VarType,
    pub width: u64,
}

/// Task output
#[derive(Debug, Clone)]
pub struct TaskOutput {
    pub name: String,
    pub var_type: VarType,
    pub width: u64,
}

/// Logic block types in SystemVerilog
#[derive(Debug, Clone)]
pub enum Logic {
    /// Continuous assignment (assign)
    Assign {
        target: String,
        expression: Expression,
    },
    /// Combinational always block
    AlwaysComb {
        statements: Vec<Statement>,
    },
    /// Sequential always block
    AlwaysFF {
        sensitivity: Vec<SensitivityItem>,
        statements: Vec<Statement>,
    },
    /// Always block (general)
    Always {
        sensitivity: Vec<SensitivityItem>,
        statements: Vec<Statement>,
    },
    /// Generate block
    Generate {
        statements: Vec<GenerateItem>,
    },
}

/// Sensitivity list items for always blocks
#[derive(Debug, Clone)]
pub enum SensitivityItem {
    /// Positive edge trigger
    Posedge(String),
    /// Negative edge trigger
    Negedge(String),
    /// Level sensitive
    Level(String),
}

/// Generate block items
#[derive(Debug, Clone)]
pub enum GenerateItem {
    /// For generate loop
    ForGenerate {
        init: Statement,
        condition: Expression,
        update: Statement,
        body: Vec<GenerateItem>,
    },
    /// If generate
    IfGenerate {
        condition: Expression,
        then_items: Vec<GenerateItem>,
        else_items: Option<Vec<GenerateItem>>,
    },
    /// Case generate
    CaseGenerate {
        expression: Expression,
        cases: Vec<CaseGenerateItem>,
        default: Option<Vec<GenerateItem>>,
    },
    /// Module instantiation in generate
    ModuleInstantiation(ModuleInstance),
    /// Variable declaration in generate
    VariableDeclaration(Var),
    /// Logic block in generate
    LogicBlock(Logic),
}

/// Case generate item
#[derive(Debug, Clone)]
pub struct CaseGenerateItem {
    pub values: Vec<Expression>,
    pub items: Vec<GenerateItem>,
}

/// Statements within logic blocks
#[derive(Debug, Clone)]
pub enum Statement {
    /// If statement
    If {
        condition: Expression,
        then_stmt: Box<Statement>,
        else_stmt: Option<Box<Statement>>,
    },
    /// Case statement
    Case {
        expression: Expression,
        cases: Vec<CaseItem>,
        default: Option<Vec<Statement>>,
    },
    /// For loop
    For {
        init: Box<Statement>,
        condition: Expression,
        update: Box<Statement>,
        body: Box<Statement>,
    },
    /// While loop
    While {
        condition: Expression,
        body: Box<Statement>,
    },
    /// Begin-end block
    Block {
        statements: Vec<Statement>,
    },
    /// Assignment statement
    Assignment {
        target: String,
        expression: Expression,
        blocking: bool, // true for =, false for <=
    },
    /// Function call
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
    },
    /// Task call
    TaskCall {
        name: String,
        arguments: Vec<Expression>,
    },
    /// Return statement
    Return {
        value: Option<Expression>,
    },
}

/// Case statement item
#[derive(Debug, Clone)]
pub struct CaseItem {
    pub values: Vec<Expression>,
    pub statements: Vec<Statement>,
}

/// Expressions in SystemVerilog
#[derive(Debug, Clone)]
pub enum Expression {
    /// Identifier (variable name)
    Identifier(String),
    /// Integer literal
    Integer(u64),
    /// Binary literal
    Binary { width: u64, value: u64 },
    /// Binary operation
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    /// Unary operation
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
    },
    /// Ternary conditional
    Ternary {
        condition: Box<Expression>,
        true_expr: Box<Expression>,
        false_expr: Box<Expression>,
    },
}

/// Binary operators
#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    LogicalAnd,
    LogicalOr,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
}

/// Unary operators
#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Not,
    BitwiseNot,
    ReductionAnd,
    ReductionOr,
    ReductionXor,
}

/// Module I/O information
///
/// Stores all inputs, outputs, and inouts for a given module
#[derive(Default, Clone)]
pub struct ModuleIO {
    /// Module name
    pub name: String,

    /// Module inputs
    pub inputs: Vec<Input>,

    /// Module outputs
    pub outputs: Vec<Output>,

    // Module combination input/outputs
    pub inouts: Vec<Inout>,
}

impl fmt::Debug for ModuleIO {
    fn fmt(&self, _: &mut std::fmt::Formatter) -> fmt::Result {
        debug!("MODULE I/O: {:?}", self.name);
        for input in self.inputs.clone() {
            debug!("IO: {:?}", input);
        }
        for output in self.outputs.clone() {
            debug!("IO: {:?}", output);
        }
        for inout in self.inouts.clone() {
            debug!("IO: {:?}", inout);
        }
        Ok(())
    }
}

// All parsing functions moved to parser.rs - this file now contains only type definitions
