//! CFML Parser - Converts tokens to AST

use crate::ast::*;
use crate::lexer::{Lexer, TokenWithLoc};
use crate::token::Token;
use cfml_common::position::SourceLocation;
use std::convert::TryFrom;

pub struct Parser {
    tokens: Vec<TokenWithLoc>,
    current: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl Parser {
    pub fn new(source: String) -> Self {
        let tokens = Lexer::new(source).tokenize();
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        Ok(Program {
            statements,
            location: SourceLocation::default(),
        })
    }

    fn is_at_end(&self) -> bool {
        matches!(&self.tokens[self.current].token, Token::Eof)
    }

    fn peek(&self, offset: usize) -> &Token {
        let idx = self.current + offset;
        if idx >= self.tokens.len() {
            return &Token::Eof;
        }
        &self.tokens[idx].token
    }

    fn current_location(&self) -> SourceLocation {
        if self.current < self.tokens.len() {
            self.tokens[self.current].location
        } else {
            SourceLocation::default()
        }
    }

    fn advance(&mut self) -> TokenWithLoc {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> TokenWithLoc {
        self.tokens[self.current - 1].clone()
    }

    fn check(&self, token: &Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(self.peek(0)) == std::mem::discriminant(token)
    }

    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            return true;
        }
        false
    }

    #[allow(dead_code)]
    fn match_any(&mut self, tokens: &[Token]) -> Option<Token> {
        for token in tokens {
            if self.check(token) {
                let t = self.advance().token.clone();
                return Some(t);
            }
        }
        None
    }

    fn parse_error(&self, message: &str) -> ParseError {
        let loc = self.current_location();
        ParseError {
            message: format!("{} (found {:?})", message, self.peek(0)),
            line: loc.start.line,
            column: loc.start.column,
        }
    }

    // ---- Statement Parsing ----

    fn parse_statement(&mut self) -> Result<CfmlNode, ParseError> {
        // Check for access modifiers before function
        if matches!(
            self.peek(0),
            Token::Public | Token::Private | Token::Remote | Token::Package
        ) {
            let access = self.parse_access_modifier();
            if self.match_token(&Token::Function) {
                let mut func = self.parse_function()?;
                func.access = access;
                return Ok(CfmlNode::Statement(Statement::FunctionDecl(FunctionDecl {
                    func,
                })));
            }
            if self.match_token(&Token::Static) {
                if self.match_token(&Token::Function) {
                    let mut func = self.parse_function()?;
                    func.access = access;
                    func.is_static = true;
                    return Ok(CfmlNode::Statement(Statement::FunctionDecl(FunctionDecl {
                        func,
                    })));
                }
            }
        }

        if self.match_token(&Token::Var) {
            return Ok(CfmlNode::Statement(Statement::Var(self.parse_var()?)));
        }

        if self.match_token(&Token::If) {
            return Ok(CfmlNode::Statement(Statement::If(self.parse_if()?)));
        }

        if self.match_token(&Token::For) {
            return self.parse_for_statement();
        }

        if self.match_token(&Token::While) {
            return Ok(CfmlNode::Statement(Statement::While(self.parse_while()?)));
        }

        if self.match_token(&Token::Do) {
            return Ok(CfmlNode::Statement(Statement::Do(self.parse_do()?)));
        }

        if self.match_token(&Token::Switch) {
            return Ok(CfmlNode::Statement(Statement::Switch(self.parse_switch()?)));
        }

        if self.match_token(&Token::Try) {
            return Ok(CfmlNode::Statement(Statement::Try(self.parse_try()?)));
        }

        if self.match_token(&Token::Throw) {
            return Ok(CfmlNode::Statement(Statement::Throw(self.parse_throw()?)));
        }

        if self.match_token(&Token::Return) {
            return Ok(CfmlNode::Statement(Statement::Return(self.parse_return()?)));
        }

        if self.match_token(&Token::Break) {
            self.match_token(&Token::Semicolon);
            return Ok(CfmlNode::Statement(Statement::Break(Break {
                label: None,
                location: SourceLocation::default(),
            })));
        }

        if self.match_token(&Token::Continue) {
            self.match_token(&Token::Semicolon);
            return Ok(CfmlNode::Statement(Statement::Continue(Continue {
                label: None,
                location: SourceLocation::default(),
            })));
        }

        if self.match_token(&Token::Function) {
            return Ok(CfmlNode::Statement(Statement::FunctionDecl(FunctionDecl {
                func: self.parse_function()?,
            })));
        }

        if self.match_token(&Token::Component) {
            return Ok(CfmlNode::Statement(Statement::ComponentDecl(
                ComponentDecl {
                    component: self.parse_component()?,
                },
            )));
        }

        if self.match_token(&Token::Include) {
            let path = self.parse_expression()?;
            self.match_token(&Token::Semicolon);
            return Ok(CfmlNode::Statement(Statement::Include(Include {
                path,
                location: SourceLocation::default(),
            })));
        }

        if self.match_token(&Token::Import) {
            let path = self.extract_identifier()?;
            let alias = if self.match_token(&Token::Identifier("as".into())) {
                Some(self.extract_identifier()?)
            } else {
                None
            };
            self.match_token(&Token::Semicolon);
            return Ok(CfmlNode::Statement(Statement::Import(Import {
                path,
                alias,
                location: SourceLocation::default(),
            })));
        }

        // Expression statement (may be assignment)
        let expr = self.parse_expression()?;

        // Check for compound assignment on expressions
        if let Some(assign_op) = self.check_assignment_op() {
            self.advance(); // consume the operator
            let value = self.parse_expression()?;
            self.match_token(&Token::Semicolon);

            let target = self.expression_to_assign_target(&expr)?;
            return Ok(CfmlNode::Statement(Statement::Assignment(Assignment {
                target,
                value,
                operator: assign_op,
                location: SourceLocation::default(),
            })));
        }

        // Check for postfix ++ / --
        if self.match_token(&Token::PlusPlus) || self.match_token(&Token::MinusMinus) {
            let op = match self.previous().token {
                Token::PlusPlus => PostfixOpType::Increment,
                _ => PostfixOpType::Decrement,
            };
            self.match_token(&Token::Semicolon);
            return Ok(CfmlNode::Statement(Statement::Expression(
                ExpressionStatement {
                    expr: Expression::PostfixOp(Box::new(PostfixOp {
                        operand: Box::new(expr),
                        operator: op,
                        location: SourceLocation::default(),
                    })),
                    location: SourceLocation::default(),
                },
            )));
        }

        self.match_token(&Token::Semicolon);

        Ok(CfmlNode::Statement(Statement::Expression(
            ExpressionStatement {
                expr,
                location: SourceLocation::default(),
            },
        )))
    }

    fn check_assignment_op(&self) -> Option<AssignOp> {
        match self.peek(0) {
            Token::PlusEqual => Some(AssignOp::PlusEqual),
            Token::MinusEqual => Some(AssignOp::MinusEqual),
            Token::StarEqual => Some(AssignOp::StarEqual),
            Token::SlashEqual => Some(AssignOp::SlashEqual),
            Token::AmpEqual => Some(AssignOp::ConcatEqual),
            _ => None,
        }
    }

    fn expression_to_assign_target(&self, expr: &Expression) -> Result<AssignTarget, ParseError> {
        match expr {
            Expression::Identifier(id) => Ok(AssignTarget::Variable(id.name.clone())),
            Expression::ArrayAccess(acc) => Ok(AssignTarget::ArrayAccess(
                acc.array.clone(),
                acc.index.clone(),
            )),
            Expression::MemberAccess(acc) => {
                Ok(AssignTarget::StructAccess(acc.object.clone(), acc.member.clone()))
            }
            _ => Err(self.parse_error("Invalid assignment target")),
        }
    }

    fn parse_access_modifier(&mut self) -> AccessModifier {
        let tok = self.advance().token.clone();
        match tok {
            Token::Public => AccessModifier::Public,
            Token::Private => AccessModifier::Private,
            Token::Remote => AccessModifier::Remote,
            Token::Package => AccessModifier::Package,
            _ => AccessModifier::Public,
        }
    }

    fn parse_var(&mut self) -> Result<Var, ParseError> {
        let name = self.extract_identifier()?;
        let value = if self.match_token(&Token::Equal) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.match_token(&Token::Semicolon);

        Ok(Var {
            name,
            value,
            location: SourceLocation::default(),
        })
    }

    fn parse_if(&mut self) -> Result<If, ParseError> {
        self.consume(&Token::LParen)?;
        let condition = self.parse_expression()?;
        self.consume(&Token::RParen)?;

        let then_branch = if self.check(&Token::LBrace) {
            self.parse_block()?
        } else {
            // Single statement without braces
            let stmt = self.parse_statement()?;
            if let CfmlNode::Statement(s) = stmt {
                vec![s]
            } else {
                Vec::new()
            }
        };

        let mut else_if = Vec::new();
        let mut else_branch = None;

        // Handle else if / elseif chains
        while self.match_token(&Token::Else) {
            if self.match_token(&Token::If) || self.match_token(&Token::ElseIf) {
                // else if
                self.consume(&Token::LParen)?;
                let cond = self.parse_expression()?;
                self.consume(&Token::RParen)?;
                let body = if self.check(&Token::LBrace) {
                    self.parse_block()?
                } else {
                    let stmt = self.parse_statement()?;
                    if let CfmlNode::Statement(s) = stmt {
                        vec![s]
                    } else {
                        Vec::new()
                    }
                };
                else_if.push(ElseIf {
                    condition: cond,
                    body,
                });
            } else if self.match_token(&Token::ElseIf) {
                // elseif (single keyword)
                self.consume(&Token::LParen)?;
                let cond = self.parse_expression()?;
                self.consume(&Token::RParen)?;
                let body = if self.check(&Token::LBrace) {
                    self.parse_block()?
                } else {
                    let stmt = self.parse_statement()?;
                    if let CfmlNode::Statement(s) = stmt {
                        vec![s]
                    } else {
                        Vec::new()
                    }
                };
                else_if.push(ElseIf {
                    condition: cond,
                    body,
                });
            } else {
                // else
                else_branch = Some(if self.check(&Token::LBrace) {
                    self.parse_block()?
                } else {
                    let stmt = self.parse_statement()?;
                    if let CfmlNode::Statement(s) = stmt {
                        vec![s]
                    } else {
                        Vec::new()
                    }
                });
                break;
            }
        }

        // Handle standalone elseif (without else keyword prefix)
        while self.match_token(&Token::ElseIf) {
            self.consume(&Token::LParen)?;
            let cond = self.parse_expression()?;
            self.consume(&Token::RParen)?;
            let body = if self.check(&Token::LBrace) {
                self.parse_block()?
            } else {
                let stmt = self.parse_statement()?;
                if let CfmlNode::Statement(s) = stmt {
                    vec![s]
                } else {
                    Vec::new()
                }
            };
            else_if.push(ElseIf {
                condition: cond,
                body,
            });
        }

        Ok(If {
            condition,
            then_branch,
            else_if,
            else_branch,
            location: SourceLocation::default(),
        })
    }

    fn parse_for_statement(&mut self) -> Result<CfmlNode, ParseError> {
        self.consume(&Token::LParen)?;

        // Check for for-in: for (var x in collection) or for (x in collection)
        let has_var = self.match_token(&Token::Var);

        if let Token::Identifier(name) = self.peek(0).clone() {
            if matches!(self.peek(1), Token::In) {
                // for-in loop
                self.advance(); // consume identifier
                self.advance(); // consume 'in'
                let iterable = self.parse_expression()?;
                self.consume(&Token::RParen)?;
                let body = self.parse_block()?;
                return Ok(CfmlNode::Statement(Statement::ForIn(ForIn {
                    variable: name,
                    iterable,
                    body,
                    location: SourceLocation::default(),
                })));
            }
        }

        // Standard C-style for loop: for (init; condition; increment)
        let init = if has_var {
            Some(Box::new(Statement::Var(self.parse_var_no_semicolon()?)))
        } else if !self.check(&Token::Semicolon) {
            let expr = self.parse_expression()?;
            // Check if it's an assignment
            if self.match_token(&Token::Equal) {
                let value = self.parse_expression()?;
                if let Expression::Identifier(ident) = &expr {
                    Some(Box::new(Statement::Var(Var {
                        name: ident.name.clone(),
                        value: Some(value),
                        location: SourceLocation::default(),
                    })))
                } else {
                    Some(Box::new(Statement::Expression(ExpressionStatement {
                        expr,
                        location: SourceLocation::default(),
                    })))
                }
            } else {
                Some(Box::new(Statement::Expression(ExpressionStatement {
                    expr,
                    location: SourceLocation::default(),
                })))
            }
        } else {
            None
        };

        self.consume(&Token::Semicolon)?;

        let condition = if !self.check(&Token::Semicolon) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume(&Token::Semicolon)?;

        let increment = if !self.check(&Token::RParen) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        self.consume(&Token::RParen)?;

        let body = if self.check(&Token::LBrace) {
            self.parse_block()?
        } else {
            let stmt = self.parse_statement()?;
            if let CfmlNode::Statement(s) = stmt {
                vec![s]
            } else {
                Vec::new()
            }
        };

        Ok(CfmlNode::Statement(Statement::For(For {
            init,
            condition,
            increment,
            body,
            location: SourceLocation::default(),
        })))
    }

    fn parse_var_no_semicolon(&mut self) -> Result<Var, ParseError> {
        let name = self.extract_identifier()?;
        let value = if self.match_token(&Token::Equal) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(Var {
            name,
            value,
            location: SourceLocation::default(),
        })
    }

    fn parse_while(&mut self) -> Result<While, ParseError> {
        self.consume(&Token::LParen)?;
        let condition = self.parse_expression()?;
        self.consume(&Token::RParen)?;

        let body = if self.check(&Token::LBrace) {
            self.parse_block()?
        } else {
            let stmt = self.parse_statement()?;
            if let CfmlNode::Statement(s) = stmt {
                vec![s]
            } else {
                Vec::new()
            }
        };

        Ok(While {
            condition,
            body,
            location: SourceLocation::default(),
        })
    }

    fn parse_do(&mut self) -> Result<Do, ParseError> {
        let body = self.parse_block()?;
        self.consume(&Token::While)?;
        self.consume(&Token::LParen)?;
        let condition = self.parse_expression()?;
        self.consume(&Token::RParen)?;
        self.match_token(&Token::Semicolon);

        Ok(Do {
            body,
            condition,
            location: SourceLocation::default(),
        })
    }

    fn parse_switch(&mut self) -> Result<Switch, ParseError> {
        self.consume(&Token::LParen)?;
        let expression = self.parse_expression()?;
        self.consume(&Token::RParen)?;
        self.consume(&Token::LBrace)?;

        let mut cases = Vec::new();
        let mut default_case = None;

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            if self.match_token(&Token::Case) {
                let mut values = vec![self.parse_expression()?];
                while self.match_token(&Token::Comma) {
                    values.push(self.parse_expression()?);
                }
                self.consume(&Token::Colon)?;

                let mut body = Vec::new();
                while !self.check(&Token::Case)
                    && !self.check(&Token::Default)
                    && !self.check(&Token::RBrace)
                    && !self.is_at_end()
                {
                    let node = self.parse_statement()?;
                    if let CfmlNode::Statement(s) = node {
                        body.push(s);
                    }
                }

                cases.push(SwitchCase { values, body });
            } else if self.match_token(&Token::Default) {
                self.consume(&Token::Colon)?;

                let mut body = Vec::new();
                while !self.check(&Token::Case)
                    && !self.check(&Token::RBrace)
                    && !self.is_at_end()
                {
                    let node = self.parse_statement()?;
                    if let CfmlNode::Statement(s) = node {
                        body.push(s);
                    }
                }

                default_case = Some(body);
            } else {
                self.advance(); // skip unknown token
            }
        }

        self.consume(&Token::RBrace)?;

        Ok(Switch {
            expression,
            cases,
            default_case,
            location: SourceLocation::default(),
        })
    }

    fn parse_try(&mut self) -> Result<Try, ParseError> {
        let body = self.parse_block()?;
        let mut catches = Vec::new();
        let mut finally_body = None;

        while self.match_token(&Token::Catch) {
            self.consume(&Token::LParen)?;

            // catch (type varname) or catch (varname) or catch (any e)
            let first = self.extract_identifier()?;

            let (var_type, var_name) = if self.check(&Token::RParen) {
                (None, first)
            } else {
                let name = self.extract_identifier()?;
                (Some(first), name)
            };

            self.consume(&Token::RParen)?;
            let catch_body = self.parse_block()?;

            catches.push(Catch {
                var_type,
                var_name,
                body: catch_body,
            });
        }

        if self.match_token(&Token::Finally) {
            finally_body = Some(self.parse_block()?);
        }

        Ok(Try {
            body,
            catches,
            finally_body,
            location: SourceLocation::default(),
        })
    }

    fn parse_throw(&mut self) -> Result<Throw, ParseError> {
        let message = if !self.check(&Token::Semicolon) && !self.is_at_end() {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.match_token(&Token::Semicolon);

        Ok(Throw {
            message,
            type_: None,
            location: SourceLocation::default(),
        })
    }

    fn parse_return(&mut self) -> Result<Return, ParseError> {
        let value = if !self.check(&Token::Semicolon)
            && !self.check(&Token::RBrace)
            && !self.is_at_end()
        {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.match_token(&Token::Semicolon);

        Ok(Return {
            value,
            location: SourceLocation::default(),
        })
    }

    fn parse_function(&mut self) -> Result<Function, ParseError> {
        // Optional return type before function name
        let mut return_type = None;
        let name;

        let first = self.extract_identifier()?;

        // If the next token is an identifier, then `first` is the return type
        if let Token::Identifier(_) = self.peek(0) {
            return_type = Some(first);
            name = self.extract_identifier()?;
        } else {
            name = first;
        }

        self.consume(&Token::LParen)?;
        let params = self.parse_param_list()?;
        self.consume(&Token::RParen)?;

        // Parse function metadata attributes (e.g., httpmethod="GET" restpath="/users")
        let mut metadata = Vec::new();
        while let Token::Identifier(_) = self.peek(0) {
            if matches!(self.peek(1), Token::Equal) {
                let key = self.extract_identifier()?;
                self.consume(&Token::Equal)?;
                if let Token::String(val) = self.peek(0).clone() {
                    self.advance();
                    metadata.push((key, val));
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let body = if self.check(&Token::LBrace) {
            self.parse_block()?
        } else {
            Vec::new()
        };

        Ok(Function {
            name,
            params,
            return_type,
            access: AccessModifier::Public,
            is_static: false,
            is_abstract: false,
            body,
            location: SourceLocation::default(),
            metadata,
        })
    }

    fn parse_component(&mut self) -> Result<Component, ParseError> {
        let name = self
            .extract_identifier()
            .unwrap_or_else(|_| "Anonymous".to_string());

        let mut extends = None;
        let mut implements = Vec::new();

        if self.match_token(&Token::Extends) {
            extends = self.extract_dotted_identifier().ok();
        }

        if self.match_token(&Token::Implements) {
            loop {
                if let Ok(iface) = self.extract_identifier() {
                    implements.push(iface);
                }
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        // Parse component metadata attributes (e.g., taffy_uri="/users/{id}")
        let mut metadata = Vec::new();
        while let Token::Identifier(_) = self.peek(0) {
            if matches!(self.peek(1), Token::Equal) {
                let key = self.extract_identifier()?;
                self.consume(&Token::Equal)?;
                if let Token::String(val) = self.peek(0).clone() {
                    self.advance();
                    metadata.push((key, val));
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        self.consume(&Token::LBrace)?;

        let mut properties = Vec::new();
        let mut functions = Vec::new();
        let mut body = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Check for access modifiers
            let access = if matches!(
                self.peek(0),
                Token::Public | Token::Private | Token::Remote | Token::Package
            ) {
                self.parse_access_modifier()
            } else {
                AccessModifier::Public
            };

            let is_static = self.match_token(&Token::Static);

            if self.match_token(&Token::Property) {
                properties.push(self.parse_property()?);
            } else if self.match_token(&Token::Function) {
                let mut func = self.parse_function()?;
                func.access = access;
                func.is_static = is_static;
                functions.push(func);
            } else if self.match_token(&Token::Var) {
                body.push(Statement::Var(self.parse_var()?));
            } else {
                let node = self.parse_statement()?;
                if let CfmlNode::Statement(s) = node {
                    body.push(s);
                }
            }
        }

        self.consume(&Token::RBrace)?;

        Ok(Component {
            name,
            extends,
            implements,
            properties,
            functions,
            body,
            location: SourceLocation::default(),
            metadata,
        })
    }

    fn parse_property(&mut self) -> Result<Property, ParseError> {
        let mut prop_type = None;
        let mut required = false;

        // Handle attributes: property type name; or property name;
        // or property required type name;
        if self.match_token(&Token::Required) {
            required = true;
        }

        let first = self
            .extract_identifier()
            .unwrap_or_else(|_| "unknown".to_string());

        let name = if let Token::Identifier(_) = self.peek(0) {
            prop_type = Some(first);
            self.extract_identifier()
                .unwrap_or_else(|_| "unknown".to_string())
        } else {
            first
        };

        let default = if self.match_token(&Token::Equal) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.match_token(&Token::Semicolon);

        Ok(Property {
            name,
            prop_type,
            default,
            required,
            location: SourceLocation::default(),
        })
    }

    fn parse_param_list(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();

        if self.check(&Token::RParen) {
            return Ok(params);
        }

        loop {
            let required = self.match_token(&Token::Required);
            let mut param_type = None;

            let first = self
                .extract_identifier()
                .unwrap_or_else(|_| "arg".to_string());

            // If next is also an identifier, then first was the type
            let name = if let Token::Identifier(_) = self.peek(0) {
                param_type = Some(first);
                self.extract_identifier()
                    .unwrap_or_else(|_| "arg".to_string())
            } else {
                first
            };

            let default = if self.match_token(&Token::Equal) {
                Some(self.parse_expression()?)
            } else {
                None
            };

            params.push(Param {
                name,
                param_type,
                default,
                required,
            });

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        Ok(params)
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, ParseError> {
        self.consume(&Token::LBrace)?;
        let mut statements = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let node = self.parse_statement()?;
            if let CfmlNode::Statement(s) = node {
                statements.push(s);
            }
        }

        self.consume(&Token::RBrace)?;
        Ok(statements)
    }

    fn extract_identifier(&mut self) -> Result<String, ParseError> {
        match self.peek(0) {
            Token::Identifier(_) => {
                if let Token::Identifier(id) = self.advance().token {
                    Ok(id)
                } else {
                    unreachable!()
                }
            }
            _ => Err(self.parse_error("Expected identifier")),
        }
    }

    fn extract_dotted_identifier(&mut self) -> Result<String, ParseError> {
        let mut path = self.extract_identifier()?;
        while self.match_token(&Token::Dot) {
            let next = self.extract_identifier()?;
            path.push('.');
            path.push_str(&next);
        }
        Ok(path)
    }

    fn consume(&mut self, token: &Token) -> Result<(), ParseError> {
        if self.check(token) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError {
                message: format!("Expected {:?}, found {:?}", token, self.peek(0)),
                line: self.current_location().start.line,
                column: self.current_location().start.column,
            })
        }
    }

    // ---- Expression Parsing (Pratt-style precedence climbing) ----

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_assignment_expr()
    }

    fn parse_assignment_expr(&mut self) -> Result<Expression, ParseError> {
        let expr = self.parse_ternary()?;

        if self.check(&Token::Equal) {
            if let Expression::Identifier(ref ident) = expr {
                let name = ident.name.clone();
                self.advance(); // consume =
                let value = self.parse_expression()?;
                return Ok(Expression::BinaryOp(Box::new(BinaryOp {
                    left: Box::new(Expression::Identifier(Identifier {
                        name,
                        location: SourceLocation::default(),
                    })),
                    operator: BinaryOpType::Assign,
                    right: Box::new(value),
                    location: SourceLocation::default(),
                })));
            } else if let Expression::MemberAccess(_) | Expression::ArrayAccess(_) = &expr {
                self.advance(); // consume =
                let value = self.parse_expression()?;
                return Ok(Expression::BinaryOp(Box::new(BinaryOp {
                    left: Box::new(expr),
                    operator: BinaryOpType::Assign,
                    right: Box::new(value),
                    location: SourceLocation::default(),
                })));
            }
        }

        Ok(expr)
    }

    fn parse_ternary(&mut self) -> Result<Expression, ParseError> {
        let expr = self.parse_imp()?;

        if self.match_token(&Token::Question) {
            let then_expr = Box::new(self.parse_expression()?);
            self.consume(&Token::Colon)?;
            let else_expr = Box::new(self.parse_expression()?);

            return Ok(Expression::Ternary(Box::new(Ternary {
                condition: Box::new(expr),
                then_expr,
                else_expr,
                location: SourceLocation::default(),
            })));
        }

        // Elvis operator ?: (null coalescing)
        if self.match_token(&Token::QuestionColon) {
            let right = Box::new(self.parse_expression()?);
            return Ok(Expression::Elvis(Box::new(Elvis {
                left: Box::new(expr),
                right,
                location: SourceLocation::default(),
            })));
        }

        Ok(expr)
    }

    fn parse_imp(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_eqv()?;

        while self.match_token(&Token::ImpKeyword) {
            let right = Box::new(self.parse_eqv()?);
            left = Expression::BinaryOp(Box::new(BinaryOp {
                left: Box::new(left),
                operator: BinaryOpType::Imp,
                right,
                location: SourceLocation::default(),
            }));
        }

        Ok(left)
    }

    fn parse_eqv(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_xor()?;

        while self.match_token(&Token::EqvKeyword) {
            let right = Box::new(self.parse_xor()?);
            left = Expression::BinaryOp(Box::new(BinaryOp {
                left: Box::new(left),
                operator: BinaryOpType::Eqv,
                right,
                location: SourceLocation::default(),
            }));
        }

        Ok(left)
    }

    fn parse_xor(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_or()?;

        while self.match_token(&Token::XorKeyword) {
            let right = Box::new(self.parse_or()?);
            left = Expression::BinaryOp(Box::new(BinaryOp {
                left: Box::new(left),
                operator: BinaryOpType::Xor,
                right,
                location: SourceLocation::default(),
            }));
        }

        Ok(left)
    }

    fn parse_or(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_and()?;

        while self.match_token(&Token::BarBar) || self.match_token(&Token::OrKeyword) {
            let right = Box::new(self.parse_and()?);
            left = Expression::BinaryOp(Box::new(BinaryOp {
                left: Box::new(left),
                operator: BinaryOpType::Or,
                right,
                location: SourceLocation::default(),
            }));
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_not()?;

        while self.match_token(&Token::AmpAmp) || self.match_token(&Token::AndKeyword) {
            let right = Box::new(self.parse_not()?);
            left = Expression::BinaryOp(Box::new(BinaryOp {
                left: Box::new(left),
                operator: BinaryOpType::And,
                right,
                location: SourceLocation::default(),
            }));
        }

        Ok(left)
    }

    fn parse_not(&mut self) -> Result<Expression, ParseError> {
        if self.match_token(&Token::NotKeyword) || self.match_token(&Token::Bang) {
            let operand = Box::new(self.parse_not()?);
            return Ok(Expression::UnaryOp(Box::new(UnaryOp {
                operator: UnaryOpType::Not,
                operand,
                location: SourceLocation::default(),
            })));
        }

        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_comparison()?;

        loop {
            if self.match_token(&Token::EqualEqual) || self.match_token(&Token::EqKeyword) || self.match_token(&Token::IsKeyword) {
                let right = Box::new(self.parse_comparison()?);
                left = Expression::BinaryOp(Box::new(BinaryOp {
                    left: Box::new(left),
                    operator: BinaryOpType::Equal,
                    right,
                    location: SourceLocation::default(),
                }));
            } else if self.match_token(&Token::BangEqual) || self.match_token(&Token::NeqKeyword) {
                let right = Box::new(self.parse_comparison()?);
                left = Expression::BinaryOp(Box::new(BinaryOp {
                    left: Box::new(left),
                    operator: BinaryOpType::NotEqual,
                    right,
                    location: SourceLocation::default(),
                }));
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_contains()?;

        loop {
            if self.match_token(&Token::Greater) || self.match_token(&Token::GtKeyword) {
                let right = Box::new(self.parse_contains()?);
                left = Expression::BinaryOp(Box::new(BinaryOp {
                    left: Box::new(left),
                    operator: BinaryOpType::Greater,
                    right,
                    location: SourceLocation::default(),
                }));
            } else if self.match_token(&Token::GreaterEqual) || self.match_token(&Token::GteKeyword) {
                let right = Box::new(self.parse_contains()?);
                left = Expression::BinaryOp(Box::new(BinaryOp {
                    left: Box::new(left),
                    operator: BinaryOpType::GreaterEqual,
                    right,
                    location: SourceLocation::default(),
                }));
            } else if self.match_token(&Token::Less) || self.match_token(&Token::LtKeyword) {
                let right = Box::new(self.parse_contains()?);
                left = Expression::BinaryOp(Box::new(BinaryOp {
                    left: Box::new(left),
                    operator: BinaryOpType::Less,
                    right,
                    location: SourceLocation::default(),
                }));
            } else if self.match_token(&Token::LessEqual) || self.match_token(&Token::LteKeyword) {
                let right = Box::new(self.parse_contains()?);
                left = Expression::BinaryOp(Box::new(BinaryOp {
                    left: Box::new(left),
                    operator: BinaryOpType::LessEqual,
                    right,
                    location: SourceLocation::default(),
                }));
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_contains(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_concatenation()?;

        if self.match_token(&Token::Contains) {
            let right = Box::new(self.parse_concatenation()?);
            left = Expression::BinaryOp(Box::new(BinaryOp {
                left: Box::new(left),
                operator: BinaryOpType::Contains,
                right,
                location: SourceLocation::default(),
            }));
        } else if self.match_token(&Token::NotKeyword) {
            // "NOT CONTAINS" as two-word operator
            if self.match_token(&Token::Contains) {
                let right = Box::new(self.parse_concatenation()?);
                left = Expression::BinaryOp(Box::new(BinaryOp {
                    left: Box::new(left),
                    operator: BinaryOpType::DoesNotContain,
                    right,
                    location: SourceLocation::default(),
                }));
            } else {
                // It was just NOT used as unary, put it back
                self.current -= 1;
            }
        }

        Ok(left)
    }

    fn parse_concatenation(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_term()?;

        while self.match_token(&Token::Amp) {
            let right = Box::new(self.parse_term()?);
            left = Expression::BinaryOp(Box::new(BinaryOp {
                left: Box::new(left),
                operator: BinaryOpType::Concat,
                right,
                location: SourceLocation::default(),
            }));
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_factor()?;

        while self.match_token(&Token::Plus) || self.match_token(&Token::Minus) {
            let operator = match self.previous().token {
                Token::Plus => BinaryOpType::Add,
                _ => BinaryOpType::Sub,
            };
            let right = Box::new(self.parse_factor()?);
            left = Expression::BinaryOp(Box::new(BinaryOp {
                left: Box::new(left),
                operator,
                right,
                location: SourceLocation::default(),
            }));
        }

        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_power()?;

        while self.match_token(&Token::Star)
            || self.match_token(&Token::Slash)
            || self.match_token(&Token::Percent)
            || self.match_token(&Token::ModKeyword)
            || self.match_token(&Token::Backslash)
        {
            let operator = match self.previous().token {
                Token::Star => BinaryOpType::Mul,
                Token::Slash => BinaryOpType::Div,
                Token::Backslash => BinaryOpType::IntDiv,
                _ => BinaryOpType::Mod,
            };
            let right = Box::new(self.parse_power()?);
            left = Expression::BinaryOp(Box::new(BinaryOp {
                left: Box::new(left),
                operator,
                right,
                location: SourceLocation::default(),
            }));
        }

        Ok(left)
    }

    fn parse_power(&mut self) -> Result<Expression, ParseError> {
        let left = self.parse_unary()?;

        if self.match_token(&Token::Caret) {
            let right = Box::new(self.parse_unary()?);
            return Ok(Expression::BinaryOp(Box::new(BinaryOp {
                left: Box::new(left),
                operator: BinaryOpType::Pow,
                right,
                location: SourceLocation::default(),
            })));
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        if self.match_token(&Token::Minus) {
            let operand = Box::new(self.parse_unary()?);
            return Ok(Expression::UnaryOp(Box::new(UnaryOp {
                operator: UnaryOpType::Minus,
                operand,
                location: SourceLocation::default(),
            })));
        }

        // Prefix ++ / --
        if self.match_token(&Token::PlusPlus) {
            let operand = Box::new(self.parse_call()?);
            return Ok(Expression::UnaryOp(Box::new(UnaryOp {
                operator: UnaryOpType::Minus, // We'll handle prefix increment at compile time
                operand,
                location: SourceLocation::default(),
            })));
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_call()?;

        // Postfix ++ / --
        if self.match_token(&Token::PlusPlus) {
            expr = Expression::PostfixOp(Box::new(PostfixOp {
                operand: Box::new(expr),
                operator: PostfixOpType::Increment,
                location: SourceLocation::default(),
            }));
        } else if self.match_token(&Token::MinusMinus) {
            expr = Expression::PostfixOp(Box::new(PostfixOp {
                operand: Box::new(expr),
                operator: PostfixOpType::Decrement,
                location: SourceLocation::default(),
            }));
        }

        Ok(expr)
    }

    fn parse_call(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&Token::Dot) {
                let method = self.extract_identifier().unwrap_or_default();
                if self.match_token(&Token::LParen) {
                    let args = self.parse_arguments()?;
                    self.consume(&Token::RParen)?;
                    expr = Expression::MethodCall(Box::new(MethodCall {
                        object: Box::new(expr),
                        method,
                        arguments: args,
                        null_safe: false,
                        location: SourceLocation::default(),
                    }));
                } else {
                    expr = Expression::MemberAccess(Box::new(MemberAccess {
                        object: Box::new(expr),
                        member: method,
                        null_safe: false,
                        location: SourceLocation::default(),
                    }));
                }
            } else if self.match_token(&Token::LParen) {
                let args = self.parse_arguments()?;
                self.consume(&Token::RParen)?;
                expr = Expression::FunctionCall(Box::new(FunctionCall {
                    name: Box::new(expr),
                    arguments: args,
                    location: SourceLocation::default(),
                }));
            } else if self.match_token(&Token::LBracket) {
                let index = Box::new(self.parse_expression()?);
                self.consume(&Token::RBracket)?;
                expr = Expression::ArrayAccess(Box::new(ArrayAccess {
                    array: Box::new(expr),
                    index,
                    location: SourceLocation::default(),
                }));
            } else if self.match_token(&Token::QuestionDot) {
                // Null-safe navigation: obj?.method() or obj?.property
                let member = self.extract_identifier().unwrap_or_default();
                if self.match_token(&Token::LParen) {
                    let args = self.parse_arguments()?;
                    self.consume(&Token::RParen)?;
                    expr = Expression::MethodCall(Box::new(MethodCall {
                        object: Box::new(expr),
                        method: member,
                        arguments: args,
                        null_safe: true,
                        location: SourceLocation::default(),
                    }));
                } else {
                    expr = Expression::MemberAccess(Box::new(MemberAccess {
                        object: Box::new(expr),
                        member,
                        null_safe: true,
                        location: SourceLocation::default(),
                    }));
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_arguments(&mut self) -> Result<Vec<Expression>, ParseError> {
        let mut args = Vec::new();

        if self.check(&Token::RParen) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_expression()?);
            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        let token = self.advance().token.clone();

        match token {
            Token::True => Ok(Expression::Literal(Literal {
                value: LiteralValue::Bool(true),
                location: SourceLocation::default(),
            })),
            Token::False => Ok(Expression::Literal(Literal {
                value: LiteralValue::Bool(false),
                location: SourceLocation::default(),
            })),
            Token::Null => Ok(Expression::Literal(Literal {
                value: LiteralValue::Null,
                location: SourceLocation::default(),
            })),
            Token::Integer(i) => Ok(Expression::Literal(Literal {
                value: LiteralValue::Int(i),
                location: SourceLocation::default(),
            })),
            Token::Double(d) => Ok(Expression::Literal(Literal {
                value: LiteralValue::Double(d),
                location: SourceLocation::default(),
            })),
            Token::String(s) => Ok(Expression::Literal(Literal {
                value: LiteralValue::String(s),
                location: SourceLocation::default(),
            })),
            Token::InterpolatedStringStart => {
                let mut parts: Vec<Expression> = Vec::new();
                while !self.is_at_end() && !self.check(&Token::InterpolatedStringEnd) {
                    let part_token = self.advance().token.clone();
                    match part_token {
                        Token::String(s) => {
                            parts.push(Expression::Literal(Literal {
                                value: LiteralValue::String(s),
                                location: SourceLocation::default(),
                            }));
                        }
                        Token::InterpolatedExpr(expr_str) => {
                            // Add semicolon so sub-parser can parse as a statement
                            let mut sub_parser = Parser::new(format!("{};", expr_str));
                            if let Ok(program) = sub_parser.parse() {
                                let expr = program.statements.into_iter().next().and_then(|node| {
                                    match node {
                                        CfmlNode::Statement(Statement::Expression(es)) => Some(es.expr),
                                        CfmlNode::Expression(expr) => Some(expr),
                                        _ => None,
                                    }
                                });
                                parts.push(expr.unwrap_or(Expression::Empty));
                            } else {
                                // Fallback: treat as identifier
                                parts.push(Expression::Identifier(Identifier {
                                    name: expr_str.trim().to_string(),
                                    location: SourceLocation::default(),
                                }));
                            }
                        }
                        _ => break,
                    }
                }
                self.match_token(&Token::InterpolatedStringEnd);
                Ok(Expression::StringInterpolation(StringInterpolation {
                    parts,
                    location: SourceLocation::default(),
                }))
            }
            Token::Identifier(id) => Ok(Expression::Identifier(Identifier {
                name: id,
                location: SourceLocation::default(),
            })),
            Token::This => Ok(Expression::This(This {
                location: SourceLocation::default(),
            })),
            Token::Super => Ok(Expression::Super(Super {
                location: SourceLocation::default(),
            })),
            Token::New => {
                let class = Box::new(self.parse_call()?);
                let args = if self.match_token(&Token::LParen) {
                    let a = self.parse_arguments()?;
                    self.consume(&Token::RParen)?;
                    a
                } else {
                    // The call parser might have already consumed the parens
                    // if the primary was parsed as a function call
                    Vec::new()
                };
                Ok(Expression::New(Box::new(NewExpression {
                    class,
                    arguments: args,
                    location: SourceLocation::default(),
                })))
            }
            Token::Function => self.parse_closure(),
            Token::LParen => {
                // Arrow function check: (params) => expr
                // or regular grouping: (expr)
                let expr = self.parse_expression()?;
                self.consume(&Token::RParen)?;

                if self.match_token(&Token::FatArrow) {
                    // Arrow function - single param from the grouped expression
                    let params = vec![Param {
                        name: if let Expression::Identifier(id) = &expr {
                            id.name.clone()
                        } else {
                            "arg".to_string()
                        },
                        param_type: None,
                        default: None,
                        required: false,
                    }];
                    let body = self.parse_expression()?;
                    return Ok(Expression::ArrowFunction(Box::new(ArrowFunction {
                        params,
                        body: Box::new(body),
                        location: SourceLocation::default(),
                    })));
                }

                Ok(expr)
            }
            Token::LBracket => self.parse_array_literal(),
            Token::LBrace => self.parse_struct_literal(),
            _ => Ok(Expression::Empty),
        }
    }

    fn parse_closure(&mut self) -> Result<Expression, ParseError> {
        // Optional name for named closures
        let _name = if let Token::Identifier(_) = self.peek(0) {
            Some(self.extract_identifier()?)
        } else {
            None
        };

        self.consume(&Token::LParen)?;
        let params = self.parse_param_list()?;
        self.consume(&Token::RParen)?;

        let body = self.parse_block()?;

        Ok(Expression::Closure(Box::new(Closure {
            params,
            body,
            location: SourceLocation::default(),
        })))
    }

    fn parse_array_literal(&mut self) -> Result<Expression, ParseError> {
        let mut elements = Vec::new();

        if !self.check(&Token::RBracket) {
            loop {
                if self.check(&Token::RBracket) {
                    break; // trailing comma
                }
                elements.push(self.parse_expression()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        self.consume(&Token::RBracket)?;

        Ok(Expression::Array(Array {
            elements,
            location: SourceLocation::default(),
        }))
    }

    fn parse_struct_literal(&mut self) -> Result<Expression, ParseError> {
        let mut pairs = Vec::new();

        if !self.check(&Token::RBrace) {
            loop {
                if self.check(&Token::RBrace) {
                    break; // trailing comma
                }
                let key = self.parse_expression()?;

                // Support both : and = for struct initialization
                if self.match_token(&Token::Colon) || self.match_token(&Token::Equal) {
                    let value = self.parse_expression()?;
                    pairs.push((key, value));
                } else {
                    // Shorthand {x} means {x: x}
                    pairs.push((key.clone(), key));
                }

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        self.consume(&Token::RBrace)?;

        Ok(Expression::Struct(Struct {
            pairs,
            ordered: false,
            location: SourceLocation::default(),
        }))
    }
}

impl TryFrom<CfmlNode> for Statement {
    type Error = ();

    fn try_from(node: CfmlNode) -> Result<Self, Self::Error> {
        match node {
            CfmlNode::Statement(s) => Ok(s),
            _ => Err(()),
        }
    }
}
