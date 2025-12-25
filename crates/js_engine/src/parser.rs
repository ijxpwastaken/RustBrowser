//! JavaScript Parser
//! 
//! Parses tokens into an Abstract Syntax Tree.

use crate::{Token, TokenType, JsError, JsResult};
use crate::ast::*;

/// JavaScript Parser
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    /// Parse the program
    pub fn parse(&mut self) -> JsResult<Program> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        
        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> JsResult<Statement> {
        match self.peek_type() {
            TokenType::Var | TokenType::Let | TokenType::Const => self.parse_var_declaration(),
            TokenType::Function => self.parse_function_declaration(),
            TokenType::If => self.parse_if_statement(),
            TokenType::While => self.parse_while_statement(),
            TokenType::For => self.parse_for_statement(),
            TokenType::Return => self.parse_return_statement(),
            TokenType::Try => self.parse_try_statement(),
            TokenType::Throw => self.parse_throw_statement(),
            TokenType::Switch => self.parse_switch_statement(),
            TokenType::Class => self.parse_class_declaration(),
            TokenType::Import => self.skip_import_statement(),
            TokenType::Export => self.skip_export_statement(),
            TokenType::Do => self.parse_do_while_statement(),
            TokenType::Break => {
                self.advance();
                self.consume_semicolon();
                Ok(Statement::Break)
            }
            TokenType::Continue => {
                self.advance();
                self.consume_semicolon();
                Ok(Statement::Continue)
            }
            TokenType::LeftBrace => self.parse_block(),
            TokenType::Semicolon => {
                self.advance();
                Ok(Statement::Empty)
            }
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_try_statement(&mut self) -> JsResult<Statement> {
        self.advance(); // consume 'try'
        let try_block = Box::new(self.parse_block()?);
        
        let catch_block = if self.match_token(TokenType::Catch) {
            // Optional catch parameter
            let param = if self.match_token(TokenType::LeftParen) {
                let name = self.expect_identifier()?;
                self.expect(TokenType::RightParen)?;
                Some(name)
            } else {
                None
            };
            let block = Box::new(self.parse_block()?);
            Some((param, block))
        } else {
            None
        };
        
        let finally_block = if self.match_token(TokenType::Finally) {
            Some(Box::new(self.parse_block()?))
        } else {
            None
        };
        
        Ok(Statement::Try { try_block, catch_block, finally_block })
    }

    fn parse_throw_statement(&mut self) -> JsResult<Statement> {
        self.advance(); // consume 'throw'
        let expr = self.parse_expression()?;
        self.consume_semicolon();
        Ok(Statement::Throw(expr))
    }

    fn parse_switch_statement(&mut self) -> JsResult<Statement> {
        self.advance(); // consume 'switch'
        self.expect(TokenType::LeftParen)?;
        let discriminant = self.parse_expression()?;
        self.expect(TokenType::RightParen)?;
        self.expect(TokenType::LeftBrace)?;
        
        let mut cases = Vec::new();
        let mut default_case = None;
        
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            if self.match_token(TokenType::Case) {
                let test = self.parse_expression()?;
                self.expect(TokenType::Colon)?;
                let mut consequent = Vec::new();
                while !self.check(TokenType::Case) && !self.check(TokenType::Default) && !self.check(TokenType::RightBrace) {
                    consequent.push(self.parse_statement()?);
                }
                cases.push((Some(test), consequent));
            } else if self.match_token(TokenType::Default) {
                self.expect(TokenType::Colon)?;
                let mut consequent = Vec::new();
                while !self.check(TokenType::Case) && !self.check(TokenType::RightBrace) {
                    consequent.push(self.parse_statement()?);
                }
                default_case = Some(consequent);
            } else {
                break;
            }
        }
        
        self.expect(TokenType::RightBrace)?;
        Ok(Statement::Switch { discriminant, cases, default_case })
    }

    fn parse_class_declaration(&mut self) -> JsResult<Statement> {
        self.advance(); // consume 'class'
        let name = self.expect_identifier()?;
        
        let extends = if self.match_token(TokenType::Extends) {
            Some(self.expect_identifier()?)
        } else {
            None
        };
        
        self.expect(TokenType::LeftBrace)?;
        
        // Skip class body for now - just consume until closing brace
        let mut depth = 1;
        while depth > 0 && !self.is_at_end() {
            match self.advance_type() {
                TokenType::LeftBrace => depth += 1,
                TokenType::RightBrace => depth -= 1,
                _ => {}
            }
        }
        
        Ok(Statement::Class { name, extends, body: vec![] })
    }

    fn parse_do_while_statement(&mut self) -> JsResult<Statement> {
        self.advance(); // consume 'do'
        let body = Box::new(self.parse_statement()?);
        self.expect(TokenType::While)?;
        self.expect(TokenType::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(TokenType::RightParen)?;
        self.consume_semicolon();
        Ok(Statement::DoWhile { body, condition })
    }

    fn skip_import_statement(&mut self) -> JsResult<Statement> {
        // Skip import statements - just consume until semicolon or newline
        self.advance(); // consume 'import'
        while !self.check(TokenType::Semicolon) && !self.is_at_end() {
            self.advance();
        }
        self.consume_semicolon();
        Ok(Statement::Empty)
    }

    fn skip_export_statement(&mut self) -> JsResult<Statement> {
        self.advance(); // consume 'export'
        // Handle 'export default', 'export const', etc.
        if self.check(TokenType::Function) || self.check(TokenType::Class) {
            return self.parse_statement();
        }
        if self.match_token(TokenType::Const) || self.match_token(TokenType::Let) || self.match_token(TokenType::Var) {
            self.current -= 1; // Go back to parse the declaration
            return self.parse_var_declaration();
        }
        // Skip other export forms
        while !self.check(TokenType::Semicolon) && !self.is_at_end() {
            self.advance();
        }
        self.consume_semicolon();
        Ok(Statement::Empty)
    }

    fn parse_var_declaration(&mut self) -> JsResult<Statement> {
        let kind = match self.advance_type() {
            TokenType::Var => VarKind::Var,
            TokenType::Let => VarKind::Let,
            TokenType::Const => VarKind::Const,
            _ => return Err(JsError::SyntaxError("Expected var, let, or const".into())),
        };

        let name = self.expect_identifier()?;

        let initializer = if self.match_token(TokenType::Assign) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        self.consume_semicolon();

        Ok(Statement::VariableDeclaration { kind, name, initializer })
    }

    fn parse_function_declaration(&mut self) -> JsResult<Statement> {
        self.advance(); // consume 'function'
        let name = self.expect_identifier()?;
        
        self.expect(TokenType::LeftParen)?;
        let params = self.parse_parameters()?;
        self.expect(TokenType::RightParen)?;
        
        self.expect(TokenType::LeftBrace)?;
        let body = self.parse_block_statements()?;
        self.expect(TokenType::RightBrace)?;
        
        Ok(Statement::FunctionDeclaration { name, params, body })
    }

    fn parse_parameters(&mut self) -> JsResult<Vec<String>> {
        let mut params = Vec::new();
        
        if !self.check(TokenType::RightParen) {
            loop {
                params.push(self.expect_identifier()?);
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        
        Ok(params)
    }

    fn parse_if_statement(&mut self) -> JsResult<Statement> {
        self.advance(); // consume 'if'
        self.expect(TokenType::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(TokenType::RightParen)?;
        
        let then_branch = Box::new(self.parse_statement()?);
        
        let else_branch = if self.match_token(TokenType::Else) {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        
        Ok(Statement::If { condition, then_branch, else_branch })
    }

    fn parse_while_statement(&mut self) -> JsResult<Statement> {
        self.advance(); // consume 'while'
        self.expect(TokenType::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(TokenType::RightParen)?;
        
        let body = Box::new(self.parse_statement()?);
        
        Ok(Statement::While { condition, body })
    }

    fn parse_for_statement(&mut self) -> JsResult<Statement> {
        self.advance(); // consume 'for'
        self.expect(TokenType::LeftParen)?;
        
        let init = if self.match_token(TokenType::Semicolon) {
            None
        } else if self.check(TokenType::Var) || self.check(TokenType::Let) {
            Some(Box::new(self.parse_var_declaration()?))
        } else {
            let expr = self.parse_expression()?;
            self.expect(TokenType::Semicolon)?;
            Some(Box::new(Statement::Expression(expr)))
        };
        
        let condition = if self.check(TokenType::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect(TokenType::Semicolon)?;
        
        let update = if self.check(TokenType::RightParen) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect(TokenType::RightParen)?;
        
        let body = Box::new(self.parse_statement()?);
        
        Ok(Statement::For { init, condition, update, body })
    }

    fn parse_return_statement(&mut self) -> JsResult<Statement> {
        self.advance(); // consume 'return'
        
        let value = if self.check(TokenType::Semicolon) || self.check(TokenType::RightBrace) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        
        self.consume_semicolon();
        
        Ok(Statement::Return(value))
    }

    fn parse_block(&mut self) -> JsResult<Statement> {
        self.expect(TokenType::LeftBrace)?;
        let statements = self.parse_block_statements()?;
        self.expect(TokenType::RightBrace)?;
        
        Ok(Statement::Block(statements))
    }

    fn parse_block_statements(&mut self) -> JsResult<Vec<Statement>> {
        let mut statements = Vec::new();
        
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        
        Ok(statements)
    }

    fn parse_expression_statement(&mut self) -> JsResult<Statement> {
        let expr = self.parse_expression()?;
        self.consume_semicolon();
        Ok(Statement::Expression(expr))
    }

    fn parse_expression(&mut self) -> JsResult<Expression> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> JsResult<Expression> {
        let expr = self.parse_ternary()?;
        
        if self.match_token(TokenType::Assign) {
            let value = self.parse_assignment()?;
            return Ok(Expression::Assignment {
                target: Box::new(expr),
                value: Box::new(value),
            });
        }
        
        Ok(expr)
    }

    fn parse_ternary(&mut self) -> JsResult<Expression> {
        let mut expr = self.parse_or()?;
        
        if self.match_token(TokenType::Question) {
            let then_expr = self.parse_expression()?;
            self.expect(TokenType::Colon)?;
            let else_expr = self.parse_ternary()?;
            
            expr = Expression::Ternary {
                condition: Box::new(expr),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
            };
        }
        
        Ok(expr)
    }

    fn parse_or(&mut self) -> JsResult<Expression> {
        let mut left = self.parse_and()?;
        
        while self.match_token(TokenType::Or) {
            let right = self.parse_and()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator: BinaryOp::Or,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_and(&mut self) -> JsResult<Expression> {
        let mut left = self.parse_equality()?;
        
        while self.match_token(TokenType::And) {
            let right = self.parse_equality()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator: BinaryOp::And,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_equality(&mut self) -> JsResult<Expression> {
        let mut left = self.parse_comparison()?;
        
        loop {
            let op = match self.peek_type() {
                TokenType::Equal => BinaryOp::Equal,
                TokenType::NotEqual => BinaryOp::NotEqual,
                TokenType::StrictEqual => BinaryOp::StrictEqual,
                TokenType::StrictNotEqual => BinaryOp::StrictNotEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_comparison(&mut self) -> JsResult<Expression> {
        let mut left = self.parse_additive()?;
        
        loop {
            let op = match self.peek_type() {
                TokenType::Less => BinaryOp::Less,
                TokenType::Greater => BinaryOp::Greater,
                TokenType::LessEqual => BinaryOp::LessEqual,
                TokenType::GreaterEqual => BinaryOp::GreaterEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_additive(&mut self) -> JsResult<Expression> {
        let mut left = self.parse_multiplicative()?;
        
        loop {
            let op = match self.peek_type() {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Subtract,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> JsResult<Expression> {
        let mut left = self.parse_unary()?;
        
        loop {
            let op = match self.peek_type() {
                TokenType::Star => BinaryOp::Multiply,
                TokenType::Slash => BinaryOp::Divide,
                TokenType::Percent => BinaryOp::Modulo,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_unary(&mut self) -> JsResult<Expression> {
        let op = match self.peek_type() {
            TokenType::Not => UnaryOp::Not,
            TokenType::Minus => UnaryOp::Negate,
            TokenType::Typeof => {
                self.advance();
                let operand = self.parse_unary()?;
                return Ok(Expression::Typeof(Box::new(operand)));
            }
            TokenType::Delete => {
                self.advance();
                let operand = self.parse_unary()?;
                return Ok(Expression::Delete(Box::new(operand)));
            }
            TokenType::Void => {
                self.advance();
                let operand = self.parse_unary()?;
                return Ok(Expression::Void(Box::new(operand)));
            }
            TokenType::PlusPlus => {
                self.advance();
                let operand = self.parse_unary()?;
                return Ok(Expression::Update { operator: "++".to_string(), prefix: true, operand: Box::new(operand) });
            }
            TokenType::MinusMinus => {
                self.advance();
                let operand = self.parse_unary()?;
                return Ok(Expression::Update { operator: "--".to_string(), prefix: true, operand: Box::new(operand) });
            }
            TokenType::BitNot => {
                self.advance();
                let operand = self.parse_unary()?;
                return Ok(Expression::Unary { operator: UnaryOp::BitNot, operand: Box::new(operand) });
            }
            _ => return self.parse_postfix(),
        };
        
        self.advance();
        let operand = self.parse_unary()?;
        
        Ok(Expression::Unary {
            operator: op,
            operand: Box::new(operand),
        })
    }

    fn parse_postfix(&mut self) -> JsResult<Expression> {
        let mut expr = self.parse_call()?;
        
        // Postfix operators
        if self.match_token(TokenType::PlusPlus) {
            expr = Expression::Update { operator: "++".to_string(), prefix: false, operand: Box::new(expr) };
        } else if self.match_token(TokenType::MinusMinus) {
            expr = Expression::Update { operator: "--".to_string(), prefix: false, operand: Box::new(expr) };
        }
        
        Ok(expr)
    }

    fn parse_call(&mut self) -> JsResult<Expression> {
        let mut expr = self.parse_primary()?;
        
        loop {
            if self.match_token(TokenType::LeftParen) {
                let args = self.parse_arguments()?;
                self.expect(TokenType::RightParen)?;
                expr = Expression::Call {
                    callee: Box::new(expr),
                    arguments: args,
                };
            } else if self.match_token(TokenType::Dot) {
                let name = self.expect_identifier()?;
                expr = Expression::Member {
                    object: Box::new(expr),
                    property: Box::new(Expression::String(name)),
                    computed: false,
                };
            } else if self.match_token(TokenType::LeftBracket) {
                let prop = self.parse_expression()?;
                self.expect(TokenType::RightBracket)?;
                expr = Expression::Member {
                    object: Box::new(expr),
                    property: Box::new(prop),
                    computed: true,
                };
            } else {
                break;
            }
        }
        
        Ok(expr)
    }

    fn parse_arguments(&mut self) -> JsResult<Vec<Expression>> {
        let mut args = Vec::new();
        
        if !self.check(TokenType::RightParen) {
            loop {
                args.push(self.parse_expression()?);
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        
        Ok(args)
    }

    fn parse_primary(&mut self) -> JsResult<Expression> {
        let token = self.peek().clone();
        
        match &token.token_type {
            TokenType::Number(n) => {
                self.advance();
                Ok(Expression::Number(*n))
            }
            TokenType::String(s) => {
                self.advance();
                Ok(Expression::String(s.clone()))
            }
            TokenType::Boolean(b) => {
                self.advance();
                Ok(Expression::Boolean(*b))
            }
            TokenType::Null => {
                self.advance();
                Ok(Expression::Null)
            }
            TokenType::Undefined => {
                self.advance();
                Ok(Expression::Undefined)
            }
            TokenType::This => {
                self.advance();
                Ok(Expression::This)
            }
            TokenType::Super => {
                self.advance();
                Ok(Expression::Super)
            }
            TokenType::New => {
                self.advance();
                let callee = self.parse_call()?;
                Ok(Expression::New(Box::new(callee)))
            }
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance();
                // Check for arrow function
                if self.check(TokenType::Arrow) {
                    self.advance(); // consume =>
                    let body = if self.check(TokenType::LeftBrace) {
                        self.parse_block()?
                    } else {
                        let expr = self.parse_expression()?;
                        Statement::Return(Some(expr))
                    };
                    return Ok(Expression::Arrow {
                        params: vec![name],
                        body: Box::new(body),
                    });
                }
                Ok(Expression::Identifier(name))
            }
            TokenType::LeftParen => {
                self.advance();
                // Could be grouping or arrow function params
                if self.check(TokenType::RightParen) {
                    self.advance();
                    // Arrow function with no params
                    if self.match_token(TokenType::Arrow) {
                        let body = if self.check(TokenType::LeftBrace) {
                            self.parse_block()?
                        } else {
                            let expr = self.parse_expression()?;
                            Statement::Return(Some(expr))
                        };
                        return Ok(Expression::Arrow {
                            params: vec![],
                            body: Box::new(body),
                        });
                    }
                    return Err(JsError::SyntaxError("Unexpected token )".into()));
                }
                
                let expr = self.parse_expression()?;
                
                // Check if this is arrow function params
                if self.check(TokenType::Comma) {
                    // Multiple params - likely arrow function
                    let mut params = vec![self.expr_to_param(&expr)?];
                    while self.match_token(TokenType::Comma) {
                        let param_expr = self.parse_expression()?;
                        params.push(self.expr_to_param(&param_expr)?);
                    }
                    self.expect(TokenType::RightParen)?;
                    if self.match_token(TokenType::Arrow) {
                        let body = if self.check(TokenType::LeftBrace) {
                            self.parse_block()?
                        } else {
                            let expr = self.parse_expression()?;
                            Statement::Return(Some(expr))
                        };
                        return Ok(Expression::Arrow { params, body: Box::new(body) });
                    }
                }
                
                self.expect(TokenType::RightParen)?;
                
                // Check for arrow after (expr)
                if self.check(TokenType::Arrow) {
                    self.advance();
                    let params = vec![self.expr_to_param(&expr)?];
                    let body = if self.check(TokenType::LeftBrace) {
                        self.parse_block()?
                    } else {
                        let expr = self.parse_expression()?;
                        Statement::Return(Some(expr))
                    };
                    return Ok(Expression::Arrow { params, body: Box::new(body) });
                }
                
                Ok(expr)
            }
            TokenType::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();
                if !self.check(TokenType::RightBracket) {
                    loop {
                        // Handle spread operator
                        if self.match_token(TokenType::Spread) {
                            let expr = self.parse_expression()?;
                            elements.push(Expression::Spread(Box::new(expr)));
                        } else {
                            elements.push(self.parse_expression()?);
                        }
                        if !self.match_token(TokenType::Comma) {
                            break;
                        }
                        // Allow trailing comma
                        if self.check(TokenType::RightBracket) {
                            break;
                        }
                    }
                }
                self.expect(TokenType::RightBracket)?;
                Ok(Expression::Array(elements))
            }
            TokenType::LeftBrace => {
                self.advance();
                let mut properties = Vec::new();
                if !self.check(TokenType::RightBrace) {
                    loop {
                        // Handle spread in objects
                        if self.match_token(TokenType::Spread) {
                            let expr = self.parse_expression()?;
                            properties.push(("...spread".to_string(), expr));
                        } else {
                            let key = match self.peek_type() {
                                TokenType::String(s) => {
                                    self.advance();
                                    s
                                }
                                TokenType::Number(n) => {
                                    self.advance();
                                    n.to_string()
                                }
                                TokenType::LeftBracket => {
                                    // Computed property
                                    self.advance();
                                    let key_expr = self.parse_expression()?;
                                    self.expect(TokenType::RightBracket)?;
                                    format!("[{}]", key_expr.to_string())
                                }
                                _ => self.expect_identifier_or_keyword()?
                            };
                            
                            // Shorthand property { foo } or method { foo() {} }
                            if self.check(TokenType::Comma) || self.check(TokenType::RightBrace) {
                                properties.push((key.clone(), Expression::Identifier(key)));
                            } else if self.check(TokenType::LeftParen) {
                                // Method shorthand
                                self.expect(TokenType::LeftParen)?;
                                let params = self.parse_parameters()?;
                                self.expect(TokenType::RightParen)?;
                                self.expect(TokenType::LeftBrace)?;
                                let body = self.parse_block_statements()?;
                                self.expect(TokenType::RightBrace)?;
                                properties.push((key, Expression::Function { name: None, params, body }));
                            } else {
                                self.expect(TokenType::Colon)?;
                                let value = self.parse_expression()?;
                                properties.push((key, value));
                            }
                        }
                        if !self.match_token(TokenType::Comma) {
                            break;
                        }
                        // Allow trailing comma
                        if self.check(TokenType::RightBrace) {
                            break;
                        }
                    }
                }
                self.expect(TokenType::RightBrace)?;
                Ok(Expression::Object(properties))
            }
            TokenType::Function => {
                self.advance();
                let name = if let TokenType::Identifier(_) = self.peek_type() {
                    Some(self.expect_identifier()?)
                } else {
                    None
                };
                self.expect(TokenType::LeftParen)?;
                let params = self.parse_parameters()?;
                self.expect(TokenType::RightParen)?;
                self.expect(TokenType::LeftBrace)?;
                let body = self.parse_block_statements()?;
                self.expect(TokenType::RightBrace)?;
                Ok(Expression::Function { name, params, body })
            }
            TokenType::Async => {
                self.advance();
                // async function or async arrow
                if self.check(TokenType::Function) {
                    self.advance();
                    let name = if let TokenType::Identifier(_) = self.peek_type() {
                        Some(self.expect_identifier()?)
                    } else {
                        None
                    };
                    self.expect(TokenType::LeftParen)?;
                    let params = self.parse_parameters()?;
                    self.expect(TokenType::RightParen)?;
                    self.expect(TokenType::LeftBrace)?;
                    let body = self.parse_block_statements()?;
                    self.expect(TokenType::RightBrace)?;
                    Ok(Expression::AsyncFunction { name, params, body })
                } else {
                    // async arrow - just parse as regular expression for now
                    self.parse_call()
                }
            }
            // Skip unknown tokens gracefully
            _ => {
                self.advance();
                Ok(Expression::Undefined)
            }
        }
    }

    fn expr_to_param(&self, expr: &Expression) -> JsResult<String> {
        match expr {
            Expression::Identifier(name) => Ok(name.clone()),
            _ => Ok("_".to_string()), // Default param for complex patterns
        }
    }

    fn expect_identifier_or_keyword(&mut self) -> JsResult<String> {
        let token = self.advance();
        match token.token_type {
            TokenType::Identifier(name) => Ok(name),
            // Allow keywords as property names
            TokenType::Get => Ok("get".to_string()),
            TokenType::Set => Ok("set".to_string()),
            TokenType::Async => Ok("async".to_string()),
            TokenType::Static => Ok("static".to_string()),
            TokenType::Class => Ok("class".to_string()),
            TokenType::Function => Ok("function".to_string()),
            TokenType::Return => Ok("return".to_string()),
            TokenType::If => Ok("if".to_string()),
            TokenType::Else => Ok("else".to_string()),
            TokenType::For => Ok("for".to_string()),
            TokenType::While => Ok("while".to_string()),
            TokenType::Do => Ok("do".to_string()),
            TokenType::In => Ok("in".to_string()),
            TokenType::Of => Ok("of".to_string()),
            TokenType::True => Ok("true".to_string()),
            TokenType::False => Ok("false".to_string()),
            TokenType::Null => Ok("null".to_string()),
            TokenType::Undefined => Ok("undefined".to_string()),
            TokenType::New => Ok("new".to_string()),
            TokenType::This => Ok("this".to_string()),
            TokenType::Try => Ok("try".to_string()),
            TokenType::Catch => Ok("catch".to_string()),
            TokenType::Finally => Ok("finally".to_string()),
            TokenType::Throw => Ok("throw".to_string()),
            TokenType::Switch => Ok("switch".to_string()),
            TokenType::Case => Ok("case".to_string()),
            TokenType::Default => Ok("default".to_string()),
            TokenType::Break => Ok("break".to_string()),
            TokenType::Continue => Ok("continue".to_string()),
            TokenType::Typeof => Ok("typeof".to_string()),
            TokenType::Delete => Ok("delete".to_string()),
            TokenType::Void => Ok("void".to_string()),
            TokenType::Import => Ok("import".to_string()),
            TokenType::Export => Ok("export".to_string()),
            TokenType::From => Ok("from".to_string()),
            TokenType::As => Ok("as".to_string()),
            TokenType::Instanceof => Ok("instanceof".to_string()),
            TokenType::Extends => Ok("extends".to_string()),
            TokenType::Super => Ok("super".to_string()),
            TokenType::Yield => Ok("yield".to_string()),
            TokenType::Await => Ok("await".to_string()),
            other => Err(JsError::SyntaxError(format!("Expected identifier, got {:?}", other))),
        }
    }

    // Helper methods
    
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_type(&self) -> TokenType {
        self.peek().token_type.clone()
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens[self.current - 1].clone()
    }

    fn advance_type(&mut self) -> TokenType {
        self.advance().token_type
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn check(&self, token_type: TokenType) -> bool {
        std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(&token_type)
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, token_type: TokenType) -> JsResult<()> {
        if self.check(token_type.clone()) {
            self.advance();
            Ok(())
        } else {
            Err(JsError::SyntaxError(format!("Expected {:?}, got {:?}", token_type, self.peek().token_type)))
        }
    }

    fn expect_identifier(&mut self) -> JsResult<String> {
        match self.advance().token_type {
            TokenType::Identifier(name) => Ok(name),
            other => Err(JsError::SyntaxError(format!("Expected identifier, got {:?}", other))),
        }
    }

    fn consume_semicolon(&mut self) {
        // Automatic Semicolon Insertion (ASI) rules:
        // 1. A semicolon is inserted if the parser encounters a token not allowed
        //    by the grammar and that token is separated by at least one line terminator.
        // 2. A semicolon is inserted if the parser encounters } or EOF.
        // 3. "Restricted productions" - return, throw, break, continue, yield, ++, --
        //    CANNOT have a line terminator between them and their operand.
        //
        // LIMITATION: Our tokenizer currently strips newlines, so we can't fully
        // implement rule 1 and 3. For now, we're lenient and allow omitted semicolons
        // before }, EOF, and certain keywords that typically start statements.
        //
        // For proper ASI, the tokenizer would need to track line terminators.
        
        if self.check(TokenType::Semicolon) {
            self.advance();
        } else if self.check(TokenType::RightBrace) 
            || self.is_at_end() 
            || self.check(TokenType::Var)
            || self.check(TokenType::Let)
            || self.check(TokenType::Const)
            || self.check(TokenType::Function)
            || self.check(TokenType::Class)
            || self.check(TokenType::If)
            || self.check(TokenType::While)
            || self.check(TokenType::For)
            || self.check(TokenType::Return)
            || self.check(TokenType::Try)
            || self.check(TokenType::Switch)
        {
            // ASI applies - don't require semicolon
        }
        // Otherwise, lenient mode - accept missing semicolon
        // In strict mode, we would error here
    }
}
