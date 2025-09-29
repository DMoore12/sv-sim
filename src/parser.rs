use crate::var_types::*;
use crate::module::*;
use crate::{LexingError, Token};
use log::{error, trace};
use logos::{Lexer, Logos};

/// Enhanced parser for SystemVerilog that handles identifiers with underscores properly
pub struct SVParser<'source> {
    lexer: Lexer<'source, Token>,
    current_token: Option<Result<Token, LexingError>>,
    current_slice: String,
}

impl<'source> SVParser<'source> {
    pub fn new(input: &'source str) -> Self {
        let mut lexer = Token::lexer(input);
        let current_token = lexer.next();
        let current_slice = lexer.slice().to_string();

        Self {
            lexer,
            current_token,
            current_slice,
        }
    }

    /// Advance to the next token
    pub fn advance(&mut self) {
        self.current_token = self.lexer.next();
        self.current_slice = self.lexer.slice().to_string();
    }

    /// Peek at the current token without consuming it
    pub fn current(&self) -> Option<&Result<Token, LexingError>> {
        self.current_token.as_ref()
    }

    /// Get the current slice
    pub fn slice(&self) -> &str {
        &self.current_slice
    }

    /// Skip whitespace and newlines
    pub fn skip_whitespace(&mut self) {
        while let Some(Ok(token)) = self.current() {
            match token {
                Token::WhiteSpace | Token::Newline => self.advance(),
                _ => break,
            }
        }
    }

    /// Parse an identifier that may contain underscores
    fn parse_identifier(&mut self) -> Result<String, LexingError> {
        let mut identifier = String::new();

        // First token should be a Word
        if let Some(Ok(Token::Word)) = self.current() {
            identifier = self.slice().to_string();
            self.advance();

            // Continue parsing underscores and words
            loop {
                match self.current() {
                    Some(Ok(Token::Underscore)) => {
                        identifier.push('_');
                        self.advance();
                    }
                    Some(Ok(Token::Word)) => {
                        identifier.push_str(self.slice());
                        self.advance();
                    }
                    _ => break,
                }
            }
        } else {
            return Err(LexingError::UnexpectedToken);
        }

        Ok(identifier)
    }

    /// Parse a module
    pub fn parse_module(&mut self) -> Result<Module, LexingError> {
        trace!("parsing module with enhanced parser");

        // Expect 'module' keyword
        if !matches!(self.current(), Some(Ok(Token::Module))) {
            return Err(LexingError::UnexpectedToken);
        }
        self.advance();
        self.skip_whitespace();

        // Parse module name
        let name = self.parse_identifier()?;
        self.skip_whitespace();

        // Parse module I/O
        let io = self.parse_module_io(name.clone())?;

        // Parse module body
        let mut parameters = Vec::new();
        let mut vars = Vec::new();
        let mut logic = Vec::new();

        loop {
            self.skip_whitespace();

            match self.current() {
                Some(Ok(Token::Parameter)) => {
                    self.advance();
                    parameters.push(self.parse_parameter(ParameterType::Parameter)?);
                }
                Some(Ok(Token::Localparam)) => {
                    self.advance();
                    parameters.push(self.parse_parameter(ParameterType::Localparam)?);
                }
                Some(Ok(Token::Wire)) => {
                    self.advance();
                    vars.push(self.parse_variable(VarType::Wire)?);
                }
                Some(Ok(Token::Reg)) => {
                    self.advance();
                    vars.push(self.parse_variable(VarType::Reg)?);
                }
                Some(Ok(Token::Assign)) => {
                    self.advance();
                    logic.push(self.parse_assign()?);
                }
                Some(Ok(Token::Always)) => {
                    self.advance();
                    logic.push(self.parse_always_block()?);
                }
                Some(Ok(Token::Case)) => {
                    self.advance();
                    // For now, skip case statements outside always blocks
                    trace!("Skipping standalone case statement");
                    self.skip_to_endcase();
                }
                Some(Ok(Token::EndModule)) => {
                    self.advance();
                    break;
                }
                Some(Ok(Token::Comment)) => {
                    self.skip_comment();
                }
                Some(Ok(_)) => {
                    // Skip unknown tokens for now
                    trace!("Skipping token: {:?}", self.current());
                    self.advance();
                }
                Some(Err(e)) => {
                    error!("Lexer error: {:?}", e);
                    return Err(e.clone());
                }
                None => break,
            }
        }

        Ok(Module {
            name,
            parameters,
            io,
            vars,
            logic,
            instances: Vec::new(), // TODO: Parse module instances
            functions: Vec::new(), // TODO: Parse functions
            tasks: Vec::new(), // TODO: Parse tasks
        })
    }

    /// Parse module I/O
    fn parse_module_io(&mut self, name: String) -> Result<ModuleIO, LexingError> {
        // Check for parameter list first
        if matches!(self.current(), Some(Ok(Token::Pound))) {
            self.advance(); // consume '#'
            self.skip_whitespace();

            // Skip parameter list for now - just consume until ')'
            if matches!(self.current(), Some(Ok(Token::OpenParen))) {
                self.advance();
                let mut paren_count = 1;
                while paren_count > 0 && self.current().is_some() {
                    match self.current() {
                        Some(Ok(Token::OpenParen)) => {
                            paren_count += 1;
                            self.advance();
                        }
                        Some(Ok(Token::CloseParen)) => {
                            paren_count -= 1;
                            self.advance();
                        }
                        _ => {
                            self.advance();
                        }
                    }
                }
            }
            self.skip_whitespace();
        }

        // Expect '(' for port list
        if !matches!(self.current(), Some(Ok(Token::OpenParen))) {
            return Err(LexingError::UnexpectedToken);
        }
        self.advance();

        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut inouts = Vec::new();

        loop {
            self.skip_whitespace();

            match self.current() {
                Some(Ok(Token::Input)) => {
                    self.advance();
                    inputs.push(self.parse_input()?);
                }
                Some(Ok(Token::Output)) => {
                    self.advance();
                    outputs.push(self.parse_output()?);
                }
                Some(Ok(Token::Inout)) => {
                    self.advance();
                    inouts.push(self.parse_inout()?);
                }
                Some(Ok(Token::CloseParen)) => {
                    self.advance();
                    break;
                }
                Some(Ok(Token::Comma)) => {
                    self.advance();
                }
                Some(Ok(Token::Comment)) => {
                    self.skip_comment();
                }
                Some(Ok(_)) => {
                    error!("Unexpected token in module I/O: {:?}", self.current());
                    return Err(LexingError::UnexpectedToken);
                }
                Some(Err(e)) => return Err(e.clone()),
                None => return Err(LexingError::UnexpectedToken),
            }
        }

        // Expect ';'
        self.skip_whitespace();
        if matches!(self.current(), Some(Ok(Token::Semicolon))) {
            self.advance();
        }

        Ok(ModuleIO {
            name,
            inputs,
            outputs,
            inouts,
        })
    }

    /// Parse an input declaration
    fn parse_input(&mut self) -> Result<Input, LexingError> {
        self.skip_whitespace();

        // Parse optional wire/reg type
        let var_type = match self.current() {
            Some(Ok(Token::Wire)) => {
                self.advance();
                VarType::Wire
            }
            Some(Ok(Token::Reg)) => {
                self.advance();
                VarType::Reg
            }
            _ => VarType::Wire, // Default to wire
        };

        self.skip_whitespace();

        // Parse optional width
        let width = if matches!(self.current(), Some(Ok(Token::OpenBracket))) {
            self.parse_width()?
        } else {
            1
        };

        self.skip_whitespace();

        // Parse name
        let name = self.parse_identifier()?;

        Ok(Input {
            name: name.clone(),
            var: Var {
                name,
                width,
                var_type,
                state: false,
                hi_z: false,
            },
        })
    }

    /// Parse an output declaration
    fn parse_output(&mut self) -> Result<Output, LexingError> {
        self.skip_whitespace();

        // Parse optional wire/reg type
        let var_type = match self.current() {
            Some(Ok(Token::Wire)) => {
                self.advance();
                VarType::Wire
            }
            Some(Ok(Token::Reg)) => {
                self.advance();
                VarType::Reg
            }
            _ => VarType::Wire, // Default to wire
        };

        self.skip_whitespace();

        // Parse optional width
        let width = if matches!(self.current(), Some(Ok(Token::OpenBracket))) {
            self.parse_width()?
        } else {
            1
        };

        self.skip_whitespace();

        // Parse name
        let name = self.parse_identifier()?;

        Ok(Output {
            name: name.clone(),
            var: Var {
                name,
                width,
                var_type,
                state: false,
                hi_z: false,
            },
        })
    }

    /// Parse an inout declaration
    fn parse_inout(&mut self) -> Result<Inout, LexingError> {
        self.skip_whitespace();

        // Parse optional wire/reg type
        let var_type = match self.current() {
            Some(Ok(Token::Wire)) => {
                self.advance();
                VarType::Wire
            }
            Some(Ok(Token::Reg)) => {
                self.advance();
                VarType::Reg
            }
            _ => VarType::Wire, // Default to wire
        };

        self.skip_whitespace();

        // Parse optional width
        let width = if matches!(self.current(), Some(Ok(Token::OpenBracket))) {
            self.parse_width()?
        } else {
            1
        };

        self.skip_whitespace();

        // Parse name
        let name = self.parse_identifier()?;

        Ok(Inout {
            name: name.clone(),
            var: Var {
                name,
                width,
                var_type,
                state: false,
                hi_z: false,
            },
        })
    }

    /// Parse a variable declaration
    fn parse_variable(&mut self, var_type: VarType) -> Result<Var, LexingError> {
        self.skip_whitespace();

        // Parse optional width
        let width = if matches!(self.current(), Some(Ok(Token::OpenBracket))) {
            self.parse_width()?
        } else {
            1
        };

        self.skip_whitespace();

        // Parse name
        let name = self.parse_identifier()?;

        // Expect ';'
        self.skip_whitespace();
        if matches!(self.current(), Some(Ok(Token::Semicolon))) {
            self.advance();
        }

        Ok(Var {
            name,
            width,
            var_type,
            state: false,
            hi_z: false,
        })
    }

    /// Parse width specification [high:low]
    fn parse_width(&mut self) -> Result<u64, LexingError> {
        // Expect '['
        if !matches!(self.current(), Some(Ok(Token::OpenBracket))) {
            return Err(LexingError::UnexpectedToken);
        }
        self.advance();
        self.skip_whitespace();

        // Parse high value
        let high = if let Some(Ok(Token::Integer(val))) = self.current() {
            let val = *val;
            self.advance();
            val
        } else {
            return Err(LexingError::UnexpectedToken);
        };

        self.skip_whitespace();

        // Expect ':'
        if matches!(self.current(), Some(Ok(Token::Colon))) {
            self.advance();
            self.skip_whitespace();

            // Parse low value
            let low = if let Some(Ok(Token::Integer(val))) = self.current() {
                let val = *val;
                self.advance();
                val
            } else {
                return Err(LexingError::UnexpectedToken);
            };

            self.skip_whitespace();

            // Expect ']'
            if !matches!(self.current(), Some(Ok(Token::CloseBracket))) {
                return Err(LexingError::UnexpectedToken);
            }
            self.advance();

            if high < low {
                return Err(LexingError::NegativeBitWidth);
            }

            Ok(high - low + 1)
        } else {
            // Single bit specification [n]
            self.skip_whitespace();

            // Expect ']'
            if !matches!(self.current(), Some(Ok(Token::CloseBracket))) {
                return Err(LexingError::UnexpectedToken);
            }
            self.advance();

            Ok(high + 1) // [n] means n+1 bits
        }
    }

    /// Parse an assign statement
    fn parse_assign(&mut self) -> Result<Logic, LexingError> {
        self.skip_whitespace();

        // Parse target
        let target = self.parse_identifier()?;

        self.skip_whitespace();

        // Expect '='
        if !matches!(self.current(), Some(Ok(Token::Equals))) {
            return Err(LexingError::UnexpectedToken);
        }
        self.advance();

        self.skip_whitespace();

        // Parse expression (simplified for now)
        let expression = self.parse_simple_expression()?;

        // Expect ';'
        self.skip_whitespace();
        if matches!(self.current(), Some(Ok(Token::Semicolon))) {
            self.advance();
        }

        Ok(Logic::Assign { target, expression })
    }

    /// Parse a more comprehensive expression with operators
    fn parse_simple_expression(&mut self) -> Result<Expression, LexingError> {
        self.skip_whitespace();

        // Parse primary expression
        let mut left = self.parse_primary_expression()?;

        // Check for binary operators
        self.skip_whitespace();
        while let Some(Ok(token)) = self.current() {
            let op = match token {
                Token::Add => BinaryOperator::Add,
                Token::Subtract => BinaryOperator::Subtract,
                Token::Multiply => BinaryOperator::Multiply,
                Token::Divide => BinaryOperator::Divide,
                Token::BEQ => BinaryOperator::Equal,
                Token::BLT => BinaryOperator::LessThan,
                Token::BLTE => BinaryOperator::LessEqual,
                Token::BGT => BinaryOperator::GreaterThan,
                Token::BGTE => BinaryOperator::GreaterEqual,
                _ => break,
            };

            self.advance();
            self.skip_whitespace();

            let right = self.parse_primary_expression()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };

            self.skip_whitespace();
        }

        Ok(left)
    }

    /// Parse a primary expression (identifier, number, parenthesized expression)
    fn parse_primary_expression(&mut self) -> Result<Expression, LexingError> {
        self.skip_whitespace();

        match self.current() {
            Some(Ok(Token::Word)) => {
                let identifier = self.parse_identifier()?;

                // Check for array indexing [index]
                self.skip_whitespace();
                if matches!(self.current(), Some(Ok(Token::OpenBracket))) {
                    self.advance(); // consume '['
                    self.skip_whitespace();

                    let index = self.parse_simple_expression()?;

                    self.skip_whitespace();
                    if matches!(self.current(), Some(Ok(Token::CloseBracket))) {
                        self.advance(); // consume ']'
                    }

                    // For now, just return the identifier (array indexing not fully implemented)
                    Ok(Expression::Identifier(identifier))
                } else {
                    Ok(Expression::Identifier(identifier))
                }
            }
            Some(Ok(Token::Integer(val))) => {
                let val = *val;
                self.advance();
                Ok(Expression::Integer(val))
            }
            Some(Ok(Token::BinaryValue)) => {
                // Parse binary literals like 4'b1010
                let slice = self.slice().to_string();
                self.advance();

                // Simple parsing - extract width and value
                if let Some(quote_pos) = slice.find('\'') {
                    if let Ok(width) = slice[..quote_pos].parse::<u64>() {
                        let value_part = &slice[quote_pos + 2..]; // Skip 'b
                        if let Ok(value) = u64::from_str_radix(value_part, 2) {
                            return Ok(Expression::Binary { width, value });
                        }
                    }
                }

                // Fallback
                Ok(Expression::Binary { width: 1, value: 0 })
            }
            Some(Ok(Token::HexValue)) => {
                // Parse hex literals like 4'h0F
                let slice = self.slice().to_string();
                self.advance();

                if let Some(quote_pos) = slice.find('\'') {
                    if let Ok(width) = slice[..quote_pos].parse::<u64>() {
                        let value_part = &slice[quote_pos + 2..]; // Skip 'h
                        if let Ok(value) = u64::from_str_radix(value_part, 16) {
                            return Ok(Expression::Binary { width, value });
                        }
                    }
                }

                // Fallback
                Ok(Expression::Binary { width: 4, value: 0 })
            }
            Some(Ok(Token::OpenParen)) => {
                self.advance(); // consume '('
                let expr = self.parse_simple_expression()?;
                self.skip_whitespace();

                if matches!(self.current(), Some(Ok(Token::CloseParen))) {
                    self.advance(); // consume ')'
                }

                Ok(expr)
            }
            Some(Ok(Token::EMark)) => {
                self.advance(); // consume '!'
                let operand = self.parse_primary_expression()?;
                Ok(Expression::UnaryOp {
                    op: UnaryOperator::Not,
                    operand: Box::new(operand),
                })
            }
            Some(Ok(Token::Tilde)) => {
                self.advance(); // consume '~'
                let operand = self.parse_primary_expression()?;
                Ok(Expression::UnaryOp {
                    op: UnaryOperator::BitwiseNot,
                    operand: Box::new(operand),
                })
            }
            _ => {
                error!("Unsupported primary expression: {:?}", self.current());
                Err(LexingError::UnexpectedToken)
            }
        }
    }

    /// Parse an always block
    fn parse_always_block(&mut self) -> Result<Logic, LexingError> {
        self.skip_whitespace();

        // Check for sensitivity list
        let mut sensitivity = Vec::new();

        if matches!(self.current(), Some(Ok(Token::At))) {
            self.advance(); // consume '@'
            self.skip_whitespace();

            if matches!(self.current(), Some(Ok(Token::OpenParen))) {
                self.advance(); // consume '('

                // Parse sensitivity list
                loop {
                    self.skip_whitespace();

                    match self.current() {
                        Some(Ok(Token::Multiply)) => {
                            // Handle @(*) - sensitive to all signals
                            self.advance();
                            sensitivity.push(SensitivityItem::Level("*".to_string()));
                        }
                        Some(Ok(Token::Posedge)) => {
                            self.advance();
                            self.skip_whitespace();
                            let signal = self.parse_identifier()?;
                            sensitivity.push(SensitivityItem::Posedge(signal));
                        }
                        Some(Ok(Token::Negedge)) => {
                            self.advance();
                            self.skip_whitespace();
                            let signal = self.parse_identifier()?;
                            sensitivity.push(SensitivityItem::Negedge(signal));
                        }
                        Some(Ok(Token::Word)) => {
                            let signal = self.parse_identifier()?;
                            sensitivity.push(SensitivityItem::Level(signal));
                        }
                        Some(Ok(Token::CloseParen)) => {
                            self.advance();
                            break;
                        }
                        Some(Ok(Token::Comma)) => {
                            self.advance();
                        }
                        _ => break,
                    }
                }
            }
        }

        self.skip_whitespace();

        // Expect 'begin'
        if !matches!(self.current(), Some(Ok(Token::Begin))) {
            return Err(LexingError::UnexpectedToken);
        }
        self.advance();

        // Parse statements
        let statements = self.parse_statement_block()?;

        // Determine the type of always block
        if sensitivity.is_empty() {
            Ok(Logic::AlwaysComb { statements })
        } else {
            // Check if it's a sequential block (has clock edges)
            let has_clock_edge = sensitivity.iter().any(|item| {
                matches!(item, SensitivityItem::Posedge(_) | SensitivityItem::Negedge(_))
            });

            if has_clock_edge {
                Ok(Logic::AlwaysFF { sensitivity, statements })
            } else {
                Ok(Logic::Always { sensitivity, statements })
            }
        }
    }

    /// Parse a block of statements until 'end'
    fn parse_statement_block(&mut self) -> Result<Vec<Statement>, LexingError> {
        let mut statements = Vec::new();

        loop {
            self.skip_whitespace();

            match self.current() {
                Some(Ok(Token::End)) => {
                    self.advance();
                    break;
                }
                Some(Ok(Token::If)) => {
                    self.advance();
                    statements.push(self.parse_if_statement()?);
                }
                Some(Ok(Token::Case)) => {
                    trace!("Parsing case statement");
                    self.advance();
                    statements.push(self.parse_case_statement()?);
                }
                Some(Ok(Token::For)) => {
                    self.advance();
                    statements.push(self.parse_for_statement()?);
                }
                Some(Ok(Token::While)) => {
                    self.advance();
                    statements.push(self.parse_while_statement()?);
                }
                Some(Ok(Token::Begin)) => {
                    self.advance();
                    let block_statements = self.parse_statement_block()?;
                    statements.push(Statement::Block { statements: block_statements });
                }
                Some(Ok(Token::Word)) => {
                    let target = self.parse_identifier()?;
                    statements.push(self.parse_assignment_statement(target)?);
                }
                Some(Ok(Token::Comment)) => {
                    self.skip_comment();
                }
                Some(Ok(_)) => {
                    // Skip unknown tokens
                    trace!("Skipping unknown token in statement block: {:?}", self.current());
                    self.advance();
                }
                Some(Err(e)) => return Err(e.clone()),
                None => break,
            }
        }

        Ok(statements)
    }

    /// Parse an if statement
    fn parse_if_statement(&mut self) -> Result<Statement, LexingError> {
        self.skip_whitespace();

        // Expect '('
        if !matches!(self.current(), Some(Ok(Token::OpenParen))) {
            return Err(LexingError::UnexpectedToken);
        }
        self.advance();

        // Parse condition
        let condition = self.parse_expression_until_close_paren()?;

        // Parse then statement
        let then_stmt = Box::new(self.parse_single_statement()?);

        // Check for else
        self.skip_whitespace();
        let else_stmt = if matches!(self.current(), Some(Ok(Token::Else))) {
            self.advance();
            Some(Box::new(self.parse_single_statement()?))
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_stmt,
            else_stmt,
        })
    }

    /// Parse a single statement
    fn parse_single_statement(&mut self) -> Result<Statement, LexingError> {
        self.skip_whitespace();

        match self.current() {
            Some(Ok(Token::Begin)) => {
                self.advance();
                let statements = self.parse_statement_block()?;
                Ok(Statement::Block { statements })
            }
            Some(Ok(Token::If)) => {
                self.advance();
                self.parse_if_statement()
            }
            Some(Ok(Token::Word)) => {
                let target = self.parse_identifier()?;
                self.parse_assignment_statement(target)
            }
            _ => {
                error!("Expected statement, got: {:?}", self.current());
                Err(LexingError::UnexpectedToken)
            }
        }
    }

    /// Parse an assignment statement (blocking or non-blocking)
    fn parse_assignment_statement(&mut self, target: String) -> Result<Statement, LexingError> {
        self.skip_whitespace();

        let blocking = match self.current() {
            Some(Ok(Token::Equals)) => {
                self.advance();
                true
            }
            Some(Ok(Token::BLTE)) => { // <=
                self.advance();
                false
            }
            _ => {
                error!("Expected assignment operator, got: {:?}", self.current());
                return Err(LexingError::UnexpectedToken);
            }
        };

        self.skip_whitespace();
        let expression = self.parse_simple_expression()?;

        // Expect ';'
        self.skip_whitespace();
        if matches!(self.current(), Some(Ok(Token::Semicolon))) {
            self.advance();
        }

        Ok(Statement::Assignment {
            target,
            expression,
            blocking,
        })
    }

    /// Parse expression until close parenthesis (simplified)
    fn parse_expression_until_close_paren(&mut self) -> Result<Expression, LexingError> {
        // For now, just parse a simple expression and consume until ')'
        let expr = self.parse_simple_expression()?;

        // Skip to closing parenthesis
        let mut paren_count = 0;
        while let Some(Ok(token)) = self.current() {
            match token {
                Token::OpenParen => {
                    paren_count += 1;
                    self.advance();
                }
                Token::CloseParen => {
                    if paren_count == 0 {
                        self.advance(); // consume the closing paren
                        break;
                    } else {
                        paren_count -= 1;
                        self.advance();
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        Ok(expr)
    }

    /// Parse a parameter declaration
    fn parse_parameter(&mut self, param_type: ParameterType) -> Result<Parameter, LexingError> {
        self.skip_whitespace();

        // Parse parameter name
        let name = self.parse_identifier()?;

        self.skip_whitespace();

        // Expect '='
        if !matches!(self.current(), Some(Ok(Token::Equals))) {
            return Err(LexingError::UnexpectedToken);
        }
        self.advance();

        self.skip_whitespace();

        // Parse parameter value
        let value = self.parse_simple_expression()?;

        // Expect ';'
        self.skip_whitespace();
        if matches!(self.current(), Some(Ok(Token::Semicolon))) {
            self.advance();
        }

        Ok(Parameter {
            name,
            value,
            param_type,
        })
    }

    /// Parse a case statement
    fn parse_case_statement(&mut self) -> Result<Statement, LexingError> {
        trace!("Starting case statement parsing");
        self.skip_whitespace();

        // Expect '('
        if !matches!(self.current(), Some(Ok(Token::OpenParen))) {
            error!("Expected '(' after case, got: {:?}", self.current());
            return Err(LexingError::UnexpectedToken);
        }
        self.advance();

        // Parse case expression
        let expression = self.parse_expression_until_close_paren()?;

        self.skip_whitespace();

        let mut cases = Vec::new();
        let mut default = None;

        // Parse case items
        loop {
            self.skip_whitespace();

            match self.current() {
                Some(Ok(Token::Endcase)) => {
                    self.advance();
                    break;
                }
                Some(Ok(Token::Default)) => {
                    self.advance();
                    self.skip_whitespace();

                    // Expect ':'
                    if matches!(self.current(), Some(Ok(Token::Colon))) {
                        self.advance();
                    }

                    // Parse default statements
                    let mut default_stmts = Vec::new();
                    while !matches!(self.current(), Some(Ok(Token::Endcase))) && self.current().is_some() {
                        if let Ok(stmt) = self.parse_single_statement() {
                            default_stmts.push(stmt);
                        } else {
                            break;
                        }
                        self.skip_whitespace();
                    }
                    default = Some(default_stmts);
                }
                Some(Ok(_)) => {
                    // Parse case value
                    let case_value = self.parse_simple_expression()?;
                    self.skip_whitespace();

                    // Expect ':'
                    if matches!(self.current(), Some(Ok(Token::Colon))) {
                        self.advance();
                    }

                    // Parse case statements
                    let mut case_stmts = Vec::new();
                    while !matches!(self.current(), Some(Ok(Token::Endcase))) &&
                          !matches!(self.current(), Some(Ok(Token::Default))) &&
                          self.current().is_some() {

                        // Check if this looks like another case value
                        if let Some(Ok(Token::Integer(_))) = self.current() {
                            break;
                        }
                        if let Some(Ok(Token::BinaryValue)) = self.current() {
                            break;
                        }

                        if let Ok(stmt) = self.parse_single_statement() {
                            case_stmts.push(stmt);
                        } else {
                            break;
                        }
                        self.skip_whitespace();
                    }

                    cases.push(CaseItem {
                        values: vec![case_value],
                        statements: case_stmts,
                    });
                }
                Some(Err(e)) => return Err(e.clone()),
                None => break,
            }
        }

        Ok(Statement::Case {
            expression,
            cases,
            default,
        })
    }

    /// Parse a for statement
    fn parse_for_statement(&mut self) -> Result<Statement, LexingError> {
        self.skip_whitespace();

        // Expect '('
        if !matches!(self.current(), Some(Ok(Token::OpenParen))) {
            return Err(LexingError::UnexpectedToken);
        }
        self.advance();

        // Parse initialization
        self.skip_whitespace();
        let init_target = self.parse_identifier()?;
        let init = Box::new(self.parse_assignment_statement(init_target)?);

        // Parse condition
        self.skip_whitespace();
        let condition = self.parse_simple_expression()?;

        // Expect ';'
        self.skip_whitespace();
        if matches!(self.current(), Some(Ok(Token::Semicolon))) {
            self.advance();
        }

        // Parse update
        self.skip_whitespace();
        let update_target = self.parse_identifier()?;
        let update = Box::new(self.parse_assignment_statement(update_target)?);

        // Expect ')'
        self.skip_whitespace();
        if matches!(self.current(), Some(Ok(Token::CloseParen))) {
            self.advance();
        }

        // Parse body
        let body = Box::new(self.parse_single_statement()?);

        Ok(Statement::For {
            init,
            condition,
            update,
            body,
        })
    }

    /// Parse a while statement
    fn parse_while_statement(&mut self) -> Result<Statement, LexingError> {
        self.skip_whitespace();

        // Expect '('
        if !matches!(self.current(), Some(Ok(Token::OpenParen))) {
            return Err(LexingError::UnexpectedToken);
        }
        self.advance();

        // Parse condition
        let condition = self.parse_expression_until_close_paren()?;

        // Parse body
        let body = Box::new(self.parse_single_statement()?);

        Ok(Statement::While {
            condition,
            body,
        })
    }

    /// Skip to endcase
    fn skip_to_endcase(&mut self) {
        while let Some(Ok(token)) = self.current() {
            if matches!(token, Token::Endcase) {
                self.advance();
                break;
            }
            self.advance();
        }
    }

    /// Skip a comment
    pub fn skip_comment(&mut self) {
        if matches!(self.current(), Some(Ok(Token::Comment))) {
            self.advance();
            // Skip until newline
            while let Some(Ok(token)) = self.current() {
                if matches!(token, Token::Newline) {
                    self.advance();
                    break;
                }
                self.advance();
            }
        }
    }
}
