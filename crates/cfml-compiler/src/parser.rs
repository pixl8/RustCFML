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
            location: self.current_location(),
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
        let stmt_loc = self.current_location();

        // Check for access modifiers before function
        if matches!(
            self.peek(0),
            Token::Public | Token::Private | Token::Remote | Token::Package
        ) {
            let access = self.parse_access_modifier();
            // Skip optional return type annotation (e.g. "private array function ..."
            // or "public MachII.framework.AppManager function ...")
            if matches!(self.peek(0), Token::Identifier(_)) {
                // Look ahead past dotted name: Ident.Ident.Ident... then Function
                let mut lookahead = 1;
                while matches!(self.peek(lookahead), Token::Dot) && matches!(self.peek(lookahead + 1), Token::Identifier(_)) {
                    lookahead += 2;
                }
                if matches!(self.peek(lookahead), Token::Function) {
                    for _ in 0..lookahead {
                        self.advance(); // skip return type tokens
                    }
                }
            }
            if self.match_token(&Token::Function) {
                let mut func = self.parse_function()?;
                func.access = access;
                return Ok(CfmlNode::Statement(Statement::FunctionDecl(FunctionDecl {
                    func,
                })));
            }
            if self.match_token(&Token::Static) {
                // Skip optional return type after static (including dotted names)
                if matches!(self.peek(0), Token::Identifier(_)) {
                    let mut lookahead = 1;
                    while matches!(self.peek(lookahead), Token::Dot) && matches!(self.peek(lookahead + 1), Token::Identifier(_)) {
                        lookahead += 2;
                    }
                    if matches!(self.peek(lookahead), Token::Function) {
                        for _ in 0..lookahead {
                            self.advance();
                        }
                    }
                }
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
            // throw(...) with parens = function call form (VM-intercepted)
            if self.check(&Token::LParen) {
                self.consume(&Token::LParen)?;

                // Check if first arg is named (identifier followed by =)
                let is_named = matches!(self.peek(0), Token::Identifier(_))
                    && matches!(self.peek(1), Token::Equal);

                let arguments = if is_named {
                    // Parse named args like throw(message="oops", type="custom")
                    // Convert to positional: throw("oops", "custom", "", "")
                    let mut named: Vec<(String, Expression)> = Vec::new();
                    loop {
                        let key = self.extract_identifier()?.to_lowercase();
                        self.consume(&Token::Equal)?;
                        let value = self.parse_expression()?;
                        named.push((key, value));
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                        // Check if next arg is also named
                        if !matches!(self.peek(0), Token::Identifier(_)) || !matches!(self.peek(1), Token::Equal) {
                            break;
                        }
                    }
                    // Map to positional: message, type, detail, errorcode
                    let get_arg = |name: &str| -> Expression {
                        named.iter()
                            .find(|(k, _)| k == name)
                            .map(|(_, v)| v.clone())
                            .unwrap_or(Expression::Literal(Literal {
                                value: LiteralValue::String(String::new()),
                                location: stmt_loc,
                            }))
                    };
                    vec![get_arg("message"), get_arg("type"), get_arg("detail"), get_arg("errorcode")]
                } else {
                    self.parse_arguments()?
                };

                self.consume(&Token::RParen)?;
                self.match_token(&Token::Semicolon);

                let throw_ident = Expression::Identifier(Identifier {
                    name: "throw".to_string(),
                    location: stmt_loc,
                });
                let call_expr = Expression::FunctionCall(Box::new(FunctionCall {
                    name: Box::new(throw_ident),
                    arguments,
                    location: stmt_loc,
                }));
                return Ok(CfmlNode::Statement(Statement::Expression(ExpressionStatement {
                    expr: call_expr,
                    location: stmt_loc,
                })));
            }
            return Ok(CfmlNode::Statement(Statement::Throw(self.parse_throw()?)));
        }

        if self.match_token(&Token::Rethrow) {
            self.match_token(&Token::Semicolon);
            return Ok(CfmlNode::Statement(Statement::Rethrow(stmt_loc)));
        }

        if self.match_token(&Token::Return) {
            return Ok(CfmlNode::Statement(Statement::Return(self.parse_return()?)));
        }

        if self.match_token(&Token::Break) {
            self.match_token(&Token::Semicolon);
            return Ok(CfmlNode::Statement(Statement::Break(Break {
                label: None,
                location: stmt_loc,
            })));
        }

        if self.match_token(&Token::Continue) {
            self.match_token(&Token::Semicolon);
            return Ok(CfmlNode::Statement(Statement::Continue(Continue {
                label: None,
                location: stmt_loc,
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

        if self.match_token(&Token::Interface) {
            return Ok(CfmlNode::Statement(Statement::InterfaceDecl(
                InterfaceDecl {
                    interface: self.parse_interface()?,
                },
            )));
        }

        // cfscript param statement: param name="varName" default="value";
        // or shorthand: param varName = defaultValue;
        if self.match_token(&Token::Param) {
            return self.parse_param_statement(stmt_loc);
        }

        // cfscript lock block: lock name="x" type="exclusive" timeout="5" { body }
        if self.match_token(&Token::Lock) {
            return self.parse_lock(stmt_loc);
        }

        if self.match_token(&Token::Include) {
            let path = self.parse_expression()?;
            self.match_token(&Token::Semicolon);
            return Ok(CfmlNode::Statement(Statement::Include(Include {
                path,
                location: stmt_loc,
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
                location: stmt_loc,
            })));
        }

        // Handle 'savecontent variable="varname" { body }' in CFScript
        if matches!(self.peek(0), Token::Identifier(ref s) if s.to_lowercase() == "savecontent") {
            self.advance(); // consume 'savecontent'
            // Parse attributes: variable = "name"
            let mut var_name = "__savecontent_result".to_string();
            while !self.check(&Token::LBrace) && !self.is_at_end() {
                if self.is_identifier_like() && matches!(self.peek(1), Token::Equal) {
                    let attr_name = self.extract_identifier()?;
                    self.advance(); // consume =
                    let attr_value = self.parse_expression()?;
                    if attr_name.to_lowercase() == "variable" {
                        if let Expression::Literal(ref lit) = attr_value {
                            if let LiteralValue::String(ref s) = lit.value {
                                var_name = s.clone();
                            }
                        } else if let Expression::Identifier(ref id) = attr_value {
                            var_name = id.name.clone();
                        }
                    }
                } else {
                    break;
                }
            }
            // Parse body block
            let body = self.parse_block()?;
            // Convert to: __cfsavecontent_start(); body; varname = __cfsavecontent_end();
            let mut stmts = Vec::new();
            stmts.push(Statement::Expression(ExpressionStatement {
                expr: Expression::FunctionCall(Box::new(FunctionCall {
                    name: Box::new(Expression::Identifier(Identifier {
                        name: "__cfsavecontent_start".to_string(),
                        location: stmt_loc.clone(),
                    })),
                    arguments: vec![],
                    location: stmt_loc.clone(),
                })),
                location: stmt_loc.clone(),
            }));
            stmts.extend(body);
            stmts.push(Statement::Assignment(Assignment {
                target: AssignTarget::Variable(var_name),
                value: Expression::FunctionCall(Box::new(FunctionCall {
                    name: Box::new(Expression::Identifier(Identifier {
                        name: "__cfsavecontent_end".to_string(),
                        location: stmt_loc.clone(),
                    })),
                    arguments: vec![],
                    location: stmt_loc.clone(),
                })),
                operator: AssignOp::Equal,
                location: stmt_loc.clone(),
            }));
            return Ok(CfmlNode::Statement(Statement::Output(Output {
                body: stmts,
                location: stmt_loc,
            })));
        }

        // Handle 'abort' keyword as __cfabort() call
        if matches!(self.peek(0), Token::Identifier(ref s) if s.to_lowercase() == "abort") {
            self.advance(); // consume 'abort'
            self.match_token(&Token::Semicolon);
            // Build a function call expression to __cfabort()
            let abort_call = Expression::FunctionCall(Box::new(FunctionCall {
                name: Box::new(Expression::Identifier(Identifier {
                    name: "__cfabort".to_string(),
                    location: stmt_loc.clone(),
                })),
                arguments: vec![],
                location: stmt_loc.clone(),
            }));
            return Ok(CfmlNode::Statement(Statement::Expression(ExpressionStatement {
                expr: abort_call,
                location: stmt_loc,
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
                location: stmt_loc,
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
                        location: stmt_loc,
                    })),
                    location: stmt_loc,
                },
            )));
        }

        self.match_token(&Token::Semicolon);

        Ok(CfmlNode::Statement(Statement::Expression(
            ExpressionStatement {
                expr,
                location: stmt_loc,
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
            Token::PercentEqual => Some(AssignOp::PercentEqual),
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
        let loc = self.current_location();
        let mut name = self.extract_identifier()?;
        // CFML allows dotted var declarations like: var local.x = 1
        while self.match_token(&Token::Dot) {
            let part = self.extract_identifier()?;
            name.push('.');
            name.push_str(&part);
        }
        let value = if self.match_token(&Token::Equal) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.match_token(&Token::Semicolon);

        Ok(Var {
            name,
            value,
            location: loc,
        })
    }

    fn parse_if(&mut self) -> Result<If, ParseError> {
        let loc = self.current_location();
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
            location: loc,
        })
    }

    fn parse_for_statement(&mut self) -> Result<CfmlNode, ParseError> {
        let loc = self.current_location();
        self.consume(&Token::LParen)?;

        // Check for for-in: for (var x in collection) or for (x in collection)
        let has_var = self.match_token(&Token::Var);

        // Lookahead to detect for-in: scan past a (possibly dotted) identifier to find 'in'
        {
            let mut la = 0;
            // First token must be an identifier or soft keyword
            let is_ident_start = matches!(self.peek(la), Token::Identifier(_) | Token::Local
                | Token::Param | Token::Output | Token::Required | Token::Default
                | Token::Include | Token::Import | Token::Property | Token::Abstract
                | Token::Final | Token::Static | Token::Lock);
            if is_ident_start {
                la += 1;
                // Skip dotted parts: .ident .ident ...
                while matches!(self.peek(la), Token::Dot) && matches!(self.peek(la + 1), Token::Identifier(_) | Token::Local
                    | Token::Param | Token::Output | Token::Required | Token::Default
                    | Token::Include | Token::Import | Token::Property | Token::Abstract
                    | Token::Final | Token::Static) {
                    la += 2;
                }
                if matches!(self.peek(la), Token::In) {
                    // It's a for-in loop — consume the dotted name
                    let mut name = self.extract_identifier()?;
                    while self.match_token(&Token::Dot) {
                        let part = self.extract_identifier()?;
                        name.push('.');
                        name.push_str(&part);
                    }
                    self.advance(); // consume 'in'
                    let iterable = self.parse_expression()?;
                    self.consume(&Token::RParen)?;
                    let body = self.parse_block_or_statement()?;
                    return Ok(CfmlNode::Statement(Statement::ForIn(ForIn {
                        variable: name,
                        iterable,
                        body,
                        location: loc,
                    })));
                }
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
                        location: self.current_location(),
                    })))
                } else {
                    Some(Box::new(Statement::Expression(ExpressionStatement {
                        expr,
                        location: self.current_location(),
                    })))
                }
            } else {
                Some(Box::new(Statement::Expression(ExpressionStatement {
                    expr,
                    location: self.current_location(),
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
            location: loc,
        })))
    }

    fn parse_var_no_semicolon(&mut self) -> Result<Var, ParseError> {
        let loc = self.current_location();
        let mut name = self.extract_identifier()?;
        // CFML allows dotted var declarations like: var local.i = 1
        while self.match_token(&Token::Dot) {
            let part = self.extract_identifier()?;
            name.push('.');
            name.push_str(&part);
        }
        let value = if self.match_token(&Token::Equal) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(Var {
            name,
            value,
            location: loc,
        })
    }

    fn parse_while(&mut self) -> Result<While, ParseError> {
        let loc = self.current_location();
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
            location: loc,
        })
    }

    fn parse_do(&mut self) -> Result<Do, ParseError> {
        let loc = self.current_location();
        let body = self.parse_block()?;
        self.consume(&Token::While)?;
        self.consume(&Token::LParen)?;
        let condition = self.parse_expression()?;
        self.consume(&Token::RParen)?;
        self.match_token(&Token::Semicolon);

        Ok(Do {
            body,
            condition,
            location: loc,
        })
    }

    fn parse_switch(&mut self) -> Result<Switch, ParseError> {
        let loc = self.current_location();
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
            location: loc,
        })
    }

    fn parse_try(&mut self) -> Result<Try, ParseError> {
        let loc = self.current_location();
        let body = self.parse_block()?;
        let mut catches = Vec::new();
        let mut finally_body = None;

        while self.match_token(&Token::Catch) {
            self.consume(&Token::LParen)?;

            // catch (type varname) or catch (varname) or catch (any e)
            // Exception types can be dotted: catch (FW1.AbortControllerException e)
            let mut first = self.extract_identifier()?;
            while self.check(&Token::Dot) && self.is_identifier_like_at(1) {
                self.advance(); // consume dot
                let part = self.extract_identifier()?;
                first = format!("{}.{}", first, part);
            }

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
            location: loc,
        })
    }

    /// Parse cfscript lock block: lock name="x" type="exclusive" timeout="5" { body }
    /// Desugars to: __cflock_start({name:"x", type:"exclusive", timeout:5}); try { body } finally { __cflock_end("x"); }
    /// Parse cfscript `param` statement:
    ///   param name="varName" default="value" type="string";
    ///   param varName = defaultValue;
    /// Converts to: if (!isDefined("varName")) varName = defaultValue;
    fn parse_param_statement(&mut self, loc: SourceLocation) -> Result<CfmlNode, ParseError> {
        // Check if it's the named-attribute form: param name="..." default="..."
        let is_named_form = matches!(self.peek(0), Token::Identifier(ref s) if s.to_lowercase() == "name")
            && matches!(self.peek(1), Token::Equal);

        if is_named_form {
            // Parse name=value attributes and emit __cfparam(name, default) call
            let mut name_expr: Option<Expression> = None;
            let mut default_expr: Option<Expression> = None;
            while (self.is_identifier_like() || matches!(self.peek(0), Token::Identifier(_)))
                && matches!(self.peek(1), Token::Equal) {
                let attr_name = self.extract_identifier()?.to_lowercase();
                self.advance(); // consume =
                let attr_value = self.parse_expression()?;
                match attr_name.as_str() {
                    "name" => name_expr = Some(attr_value),
                    "default" => default_expr = Some(attr_value),
                    _ => {} // ignore type, etc.
                }
            }
            self.match_token(&Token::Semicolon);

            let name_val = name_expr.unwrap_or(Expression::Literal(Literal {
                value: LiteralValue::String(String::new()),
                location: loc,
            }));

            // For simple string literal names, try to do compile-time expansion
            if let Expression::Literal(ref lit) = name_val {
                if let LiteralValue::String(ref var_name) = lit.value {
                    if !var_name.is_empty() {
                        let default_val = default_expr.unwrap_or(Expression::Literal(Literal {
                            value: LiteralValue::String(String::new()),
                            location: loc,
                        }));
                        let condition = Expression::FunctionCall(Box::new(FunctionCall {
                            name: Box::new(Expression::Identifier(Identifier {
                                name: "isDefined".to_string(),
                                location: loc,
                            })),
                            arguments: vec![Expression::Literal(Literal {
                                value: LiteralValue::String(var_name.clone()),
                                location: loc,
                            })],
                            location: loc,
                        }));

                        let assign_stmt = if let Some(dot_pos) = var_name.find('.') {
                            let root = var_name[..dot_pos].to_string();
                            let rest = &var_name[dot_pos + 1..];
                            let parts: Vec<&str> = rest.split('.').collect();
                            let mut expr = Expression::Identifier(Identifier {
                                name: root.clone(),
                                location: loc,
                            });
                            for (i, part) in parts.iter().enumerate() {
                                if i < parts.len() - 1 {
                                    expr = Expression::MemberAccess(Box::new(MemberAccess {
                                        object: Box::new(expr),
                                        member: part.to_string(),
                                        null_safe: false,
                                        location: loc,
                                    }));
                                }
                            }
                            let last_part = parts.last().unwrap().to_string();
                            Statement::Assignment(Assignment {
                                target: if parts.len() == 1 {
                                    AssignTarget::StructAccess(Box::new(Expression::Identifier(Identifier {
                                        name: root,
                                        location: loc,
                                    })), last_part)
                                } else {
                                    AssignTarget::StructAccess(Box::new(expr), last_part)
                                },
                                value: default_val,
                                operator: AssignOp::Equal,
                                location: loc,
                            })
                        } else {
                            Statement::Assignment(Assignment {
                                target: AssignTarget::Variable(var_name.clone()),
                                value: default_val,
                                operator: AssignOp::Equal,
                                location: loc,
                            })
                        };

                        return Ok(CfmlNode::Statement(Statement::If(If {
                            condition: Expression::UnaryOp(Box::new(UnaryOp {
                                operator: UnaryOpType::Not,
                                operand: Box::new(condition),
                                location: loc,
                            })),
                            then_branch: vec![assign_stmt],
                            else_if: vec![],
                            else_branch: None,
                            location: loc,
                        })));
                    }
                }
            }

            // Dynamic name (e.g., string interpolation) — emit __cfparam(nameExpr, defaultExpr)
            let default_val = default_expr.unwrap_or(Expression::Literal(Literal {
                value: LiteralValue::String(String::new()),
                location: loc,
            }));
            let call = Expression::FunctionCall(Box::new(FunctionCall {
                name: Box::new(Expression::Identifier(Identifier {
                    name: "__cfparam".to_string(),
                    location: loc,
                })),
                arguments: vec![name_val, default_val],
                location: loc,
            }));
            return Ok(CfmlNode::Statement(Statement::Expression(ExpressionStatement {
                expr: call,
                location: loc,
            })));
        }

        // Shorthand form: param varName = defaultValue;
        // or: param type varName = defaultValue;
        let _type = if self.is_identifier_like() && !matches!(self.peek(1), Token::Equal | Token::Semicolon) {
            Some(self.extract_identifier()?)
        } else {
            None
        };
        let var_name = self.extract_identifier()?;
        let default_value = if self.match_token(&Token::Equal) {
            self.parse_expression()?
        } else {
            Expression::Literal(Literal {
                value: LiteralValue::String(String::new()),
                location: loc,
            })
        };
        self.match_token(&Token::Semicolon);

        let condition = Expression::FunctionCall(Box::new(FunctionCall {
            name: Box::new(Expression::Identifier(Identifier {
                name: "isDefined".to_string(),
                location: loc,
            })),
            arguments: vec![Expression::Literal(Literal {
                value: LiteralValue::String(var_name.clone()),
                location: loc,
            })],
            location: loc,
        }));

        Ok(CfmlNode::Statement(Statement::If(If {
            condition: Expression::UnaryOp(Box::new(UnaryOp {
                operator: UnaryOpType::Not,
                operand: Box::new(condition),
                location: loc,
            })),
            then_branch: vec![Statement::Assignment(Assignment {
                target: AssignTarget::Variable(var_name),
                value: default_value,
                operator: AssignOp::Equal,
                location: loc,
            })],
            else_if: vec![],
            else_branch: None,
            location: loc,
        })))
    }

    fn parse_lock(&mut self, loc: SourceLocation) -> Result<CfmlNode, ParseError> {
        // Parse key=value attributes before the block
        let mut attrs: Vec<(String, Expression)> = Vec::new();
        while let Token::Identifier(_) = self.peek(0) {
            if matches!(self.peek(1), Token::Equal) {
                let key = self.extract_identifier()?;
                self.consume(&Token::Equal)?;
                let value = self.parse_expression()?;
                attrs.push((key, value));
            } else {
                break;
            }
        }

        // Parse the block body
        let body = self.parse_block()?;

        // Extract lock name for __cflock_end
        let lock_name_expr = attrs.iter()
            .find(|(k, _)| k.to_lowercase() == "name")
            .map(|(_, v)| v.clone())
            .unwrap_or(Expression::Literal(Literal {
                value: LiteralValue::String("default".to_string()),
                location: loc,
            }));

        // Build struct literal for __cflock_start argument
        let struct_pairs: Vec<(Expression, Expression)> = attrs.iter().map(|(k, v)| {
            (Expression::Literal(Literal {
                value: LiteralValue::String(k.clone()),
                location: loc,
            }), v.clone())
        }).collect();

        let attrs_struct = Expression::Struct(Struct {
            pairs: struct_pairs,
            ordered: false,
            location: loc,
        });

        // __cflock_start(attrs)
        let lock_start = Statement::Expression(ExpressionStatement {
            expr: Expression::FunctionCall(Box::new(FunctionCall {
                name: Box::new(Expression::Identifier(Identifier {
                    name: "__cflock_start".to_string(),
                    location: loc,
                })),
                arguments: vec![attrs_struct],
                location: loc,
            })),
            location: loc,
        });

        // __cflock_end(name)
        let lock_end = Statement::Expression(ExpressionStatement {
            expr: Expression::FunctionCall(Box::new(FunctionCall {
                name: Box::new(Expression::Identifier(Identifier {
                    name: "__cflock_end".to_string(),
                    location: loc,
                })),
                arguments: vec![lock_name_expr],
                location: loc,
            })),
            location: loc,
        });

        // try { body } finally { __cflock_end(name) }
        let try_stmt = Statement::Try(Try {
            body,
            catches: vec![],
            finally_body: Some(vec![lock_end]),
            location: loc,
        });

        // Wrap as Output block: __cflock_start; try { ... } finally { __cflock_end }
        let output = Statement::Output(Output {
            body: vec![lock_start, try_stmt],
            location: loc,
        });

        Ok(CfmlNode::Statement(output))
    }

    fn parse_throw(&mut self) -> Result<Throw, ParseError> {
        let loc = self.current_location();
        let message = if !self.check(&Token::Semicolon) && !self.is_at_end() {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.match_token(&Token::Semicolon);

        Ok(Throw {
            message,
            type_: None,
            location: loc,
        })
    }

    fn parse_return(&mut self) -> Result<Return, ParseError> {
        let loc = self.current_location();
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
            location: loc,
        })
    }

    fn parse_function(&mut self) -> Result<Function, ParseError> {
        let loc = self.current_location();
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
            location: loc,
            metadata,
        })
    }

    fn parse_component(&mut self) -> Result<Component, ParseError> {
        let loc = self.current_location();
        // Only consume an identifier as the name if it's NOT followed by '=' (which
        // would indicate a metadata attribute like output="false" or hint="...").
        let name = if matches!(self.peek(0), Token::Identifier(_))
            && !matches!(self.peek(1), Token::Equal)
            && !matches!(self.peek(0), Token::Extends | Token::Implements)
        {
            self.extract_identifier().unwrap_or_else(|_| "Anonymous".to_string())
        } else {
            "Anonymous".to_string()
        };

        let mut extends = None;
        let mut implements = Vec::new();

        if self.match_token(&Token::Extends) {
            // Handle both `extends Animal` and `extends="Animal"` syntax
            if self.match_token(&Token::Equal) {
                if let Token::String(val) = self.peek(0).clone() {
                    self.advance();
                    extends = Some(val);
                }
            } else {
                extends = self.extract_dotted_identifier().ok();
            }
        }

        if self.match_token(&Token::Implements) {
            // Handle both `implements IFoo` and `implements="IFoo"` syntax
            if self.match_token(&Token::Equal) {
                if let Token::String(val) = self.peek(0).clone() {
                    self.advance();
                    // May be comma-separated: "IFoo,IBar"
                    for iface in val.split(',') {
                        let trimmed = iface.trim().to_string();
                        if !trimmed.is_empty() {
                            implements.push(trimmed);
                        }
                    }
                }
            } else {
                loop {
                    if let Ok(iface) = self.extract_dotted_identifier() {
                        implements.push(iface);
                    }
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
        }

        // Parse component metadata attributes (e.g., taffy_uri="/users/{id}", output="false", hint="...")
        // Accepts both identifiers and keyword tokens as attribute keys.
        let mut metadata = Vec::new();
        loop {
            let is_attr_key = matches!(self.peek(1), Token::Equal)
                && (matches!(self.peek(0), Token::Identifier(_))
                    || self.token_as_string(&self.peek(0).clone()).is_some());
            if !is_attr_key {
                break;
            }
            let key = if let Token::Identifier(ref s) = self.peek(0) {
                let s = s.clone();
                self.advance();
                s
            } else if let Some(s) = self.token_as_string(&self.peek(0).clone()) {
                self.advance();
                s
            } else {
                break;
            };
            self.consume(&Token::Equal)?;
            if let Token::String(val) = self.peek(0).clone() {
                self.advance();
                metadata.push((key, val));
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

            // Skip optional return type annotation (e.g. "array function ...")
            if matches!(self.peek(0), Token::Identifier(_)) && matches!(self.peek(1), Token::Function) {
                self.advance(); // skip return type
            }

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
            location: loc,
            metadata,
        })
    }

    fn parse_interface(&mut self) -> Result<Interface, ParseError> {
        let loc = self.current_location();
        // Optional name (same logic as component — skip if followed by '=')
        let name = if matches!(self.peek(0), Token::Identifier(_))
            && !matches!(self.peek(1), Token::Equal)
            && !matches!(self.peek(0), Token::Extends)
        {
            self.extract_identifier().unwrap_or_else(|_| "Anonymous".to_string())
        } else {
            "Anonymous".to_string()
        };

        // interfaces can extend multiple other interfaces
        let mut extends = Vec::new();
        if self.match_token(&Token::Extends) {
            loop {
                if let Ok(parent) = self.extract_dotted_identifier() {
                    extends.push(parent);
                }
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        // Parse metadata attributes (same as component)
        let mut metadata = Vec::new();
        loop {
            let is_attr_key = matches!(self.peek(1), Token::Equal)
                && (matches!(self.peek(0), Token::Identifier(_))
                    || self.token_as_string(&self.peek(0).clone()).is_some());
            if !is_attr_key {
                break;
            }
            let key = if let Token::Identifier(ref s) = self.peek(0) {
                let s = s.clone();
                self.advance();
                s
            } else if let Some(s) = self.token_as_string(&self.peek(0).clone()) {
                self.advance();
                s
            } else {
                break;
            };
            self.consume(&Token::Equal)?;
            if let Token::String(val) = self.peek(0).clone() {
                self.advance();
                metadata.push((key, val));
            } else {
                break;
            }
        }

        self.consume(&Token::LBrace)?;

        let mut functions = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Consume optional semicolons between signatures
            if self.match_token(&Token::Semicolon) {
                continue;
            }

            // Parse access modifier
            let access = if matches!(
                self.peek(0),
                Token::Public | Token::Private | Token::Remote | Token::Package
            ) {
                self.parse_access_modifier()
            } else {
                AccessModifier::Public
            };

            // Skip optional return type annotation
            if matches!(self.peek(0), Token::Identifier(_)) && matches!(self.peek(1), Token::Function) {
                self.advance();
            }

            if self.match_token(&Token::Function) {
                let mut func = self.parse_function()?;
                func.access = access;
                functions.push(func);
            } else {
                // Skip unexpected tokens
                self.advance();
            }
        }

        self.consume(&Token::RBrace)?;

        Ok(Interface {
            name,
            extends,
            functions,
            metadata,
            location: loc,
        })
    }

    fn parse_property(&mut self) -> Result<Property, ParseError> {
        let loc = self.current_location();
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
            location: loc,
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

            // If next is also an identifier (or soft keyword usable as identifier),
            // then first was the type annotation and next is the param name.
            let name = if self.is_identifier_like() {
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

    /// Parse either a braced block or a single statement (CFML allows braceless for/if/while bodies)
    fn parse_block_or_statement(&mut self) -> Result<Vec<Statement>, ParseError> {
        if self.check(&Token::LBrace) {
            self.parse_block()
        } else {
            let node = self.parse_statement()?;
            if let CfmlNode::Statement(s) = node {
                Ok(vec![s])
            } else {
                Ok(Vec::new())
            }
        }
    }

    /// Check if the next token can be used as an identifier (true Identifier or soft keyword).
    fn is_identifier_like(&self) -> bool {
        self.is_identifier_like_at(0)
    }

    /// Check if the token at offset can be used as an identifier.
    fn is_identifier_like_at(&self, offset: usize) -> bool {
        matches!(self.peek(offset),
            Token::Identifier(_) | Token::Local | Token::Param | Token::Output
            | Token::Required | Token::Default | Token::Include | Token::Import
            | Token::Property | Token::Abstract | Token::Final | Token::Static | Token::Lock
            | Token::Function | Token::Var | Token::Throw | Token::Component
            | Token::Interface | Token::Package | Token::Remote
        )
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
            // CFML soft keywords — can be used as identifiers in most contexts
            Token::Local => { self.advance(); Ok("local".to_string()) }
            Token::Param => { self.advance(); Ok("param".to_string()) }
            Token::Output => { self.advance(); Ok("output".to_string()) }
            Token::Required => { self.advance(); Ok("required".to_string()) }
            Token::Default => { self.advance(); Ok("default".to_string()) }
            Token::Include => { self.advance(); Ok("include".to_string()) }
            Token::Import => { self.advance(); Ok("import".to_string()) }
            Token::Property => { self.advance(); Ok("property".to_string()) }
            Token::Abstract => { self.advance(); Ok("abstract".to_string()) }
            Token::Final => { self.advance(); Ok("final".to_string()) }
            Token::Static => { self.advance(); Ok("static".to_string()) }
            Token::Lock => { self.advance(); Ok("lock".to_string()) }
            Token::Function => { self.advance(); Ok("function".to_string()) }
            Token::Var => { self.advance(); Ok("var".to_string()) }
            Token::Throw => { self.advance(); Ok("throw".to_string()) }
            Token::Component => { self.advance(); Ok("component".to_string()) }
            Token::Interface => { self.advance(); Ok("interface".to_string()) }
            Token::Package => { self.advance(); Ok("package".to_string()) }
            Token::Remote => { self.advance(); Ok("remote".to_string()) }
            _ => Err(self.parse_error("Expected identifier")),
        }
    }

    /// Extract a property name after a dot — any keyword or identifier is valid in CFML.
    fn extract_property_name(&mut self) -> Result<String, ParseError> {
        // First try normal identifier extraction (handles identifiers + soft keywords)
        if let Ok(name) = self.extract_identifier() {
            return Ok(name);
        }
        // After a dot, any keyword can be used as a property name in CFML
        let name = match self.peek(0) {
            Token::If => "if", Token::Else => "else", Token::ElseIf => "elseif",
            Token::For => "for", Token::In => "in", Token::While => "while",
            Token::Do => "do", Token::Break => "break", Token::Continue => "continue",
            Token::Return => "return", Token::Switch => "switch", Token::Case => "case",
            Token::Try => "try", Token::Catch => "catch", Token::Finally => "finally",
            Token::Throw => "throw", Token::Rethrow => "rethrow", Token::Function => "function", Token::Var => "var",
            Token::New => "new", Token::This => "this", Token::Super => "super",
            Token::Component => "component", Token::Extends => "extends",
            Token::Implements => "implements", Token::Interface => "interface",
            Token::Public => "public", Token::Private => "private",
            Token::Remote => "remote", Token::Package => "package",
            Token::True => "true", Token::False => "false", Token::Null => "null",
            Token::Contains => "contains", Token::NotKeyword => "not",
            Token::AndKeyword => "and", Token::OrKeyword => "or",
            Token::EqKeyword => "eq", Token::NeqKeyword => "neq",
            Token::GtKeyword => "gt", Token::GteKeyword => "gte",
            Token::LtKeyword => "lt", Token::LteKeyword => "lte",
            Token::ModKeyword => "mod", Token::IsKeyword => "is",
            _ => return Err(self.parse_error("Expected property name")),
        };
        self.advance();
        Ok(name.to_string())
    }

    /// Convert a keyword token to its string representation for use as metadata keys.
    fn token_as_string(&self, token: &Token) -> Option<String> {
        match token {
            Token::Output => Some("output".to_string()),
            Token::Public => Some("public".to_string()),
            Token::Private => Some("private".to_string()),
            Token::Remote => Some("remote".to_string()),
            Token::Package => Some("package".to_string()),
            Token::Static => Some("static".to_string()),
            Token::Abstract => Some("abstract".to_string()),
            Token::Final => Some("final".to_string()),
            Token::Required => Some("required".to_string()),
            Token::Default => Some("default".to_string()),
            Token::Lock => Some("lock".to_string()),
            _ => None,
        }
    }

    fn extract_dotted_identifier(&mut self) -> Result<String, ParseError> {
        let mut path = self.extract_identifier()?;
        while self.match_token(&Token::Dot) {
            let next = self.extract_property_name()?;
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
                        location: self.current_location(),
                    })),
                    operator: BinaryOpType::Assign,
                    right: Box::new(value),
                    location: self.current_location(),
                })));
            } else if let Expression::MemberAccess(_) | Expression::ArrayAccess(_) = &expr {
                self.advance(); // consume =
                let value = self.parse_expression()?;
                return Ok(Expression::BinaryOp(Box::new(BinaryOp {
                    left: Box::new(expr),
                    operator: BinaryOpType::Assign,
                    right: Box::new(value),
                    location: self.current_location(),
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
                location: self.current_location(),
            })));
        }

        // Elvis operator ?: (null coalescing) and ?? (null coalescing alias)
        if self.match_token(&Token::QuestionColon) || self.match_token(&Token::QuestionQuestion) {
            let right = Box::new(self.parse_expression()?);
            return Ok(Expression::Elvis(Box::new(Elvis {
                left: Box::new(expr),
                right,
                location: self.current_location(),
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
                location: self.current_location(),
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
                location: self.current_location(),
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
                location: self.current_location(),
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
                location: self.current_location(),
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
                location: self.current_location(),
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
                location: self.current_location(),
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
                    location: self.current_location(),
                }));
            } else if self.match_token(&Token::BangEqual) || self.match_token(&Token::NeqKeyword) {
                let right = Box::new(self.parse_comparison()?);
                left = Expression::BinaryOp(Box::new(BinaryOp {
                    left: Box::new(left),
                    operator: BinaryOpType::NotEqual,
                    right,
                    location: self.current_location(),
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
                    location: self.current_location(),
                }));
            } else if self.match_token(&Token::GreaterEqual) || self.match_token(&Token::GteKeyword) {
                let right = Box::new(self.parse_contains()?);
                left = Expression::BinaryOp(Box::new(BinaryOp {
                    left: Box::new(left),
                    operator: BinaryOpType::GreaterEqual,
                    right,
                    location: self.current_location(),
                }));
            } else if self.match_token(&Token::Less) || self.match_token(&Token::LtKeyword) {
                let right = Box::new(self.parse_contains()?);
                left = Expression::BinaryOp(Box::new(BinaryOp {
                    left: Box::new(left),
                    operator: BinaryOpType::Less,
                    right,
                    location: self.current_location(),
                }));
            } else if self.match_token(&Token::LessEqual) || self.match_token(&Token::LteKeyword) {
                let right = Box::new(self.parse_contains()?);
                left = Expression::BinaryOp(Box::new(BinaryOp {
                    left: Box::new(left),
                    operator: BinaryOpType::LessEqual,
                    right,
                    location: self.current_location(),
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
                location: self.current_location(),
            }));
        } else if self.match_token(&Token::NotKeyword) {
            // "NOT CONTAINS" as two-word operator
            if self.match_token(&Token::Contains) {
                let right = Box::new(self.parse_concatenation()?);
                left = Expression::BinaryOp(Box::new(BinaryOp {
                    left: Box::new(left),
                    operator: BinaryOpType::DoesNotContain,
                    right,
                    location: self.current_location(),
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
                location: self.current_location(),
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
                location: self.current_location(),
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
                location: self.current_location(),
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
                location: self.current_location(),
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
                location: self.current_location(),
            })));
        }

        // Prefix ++ / --
        if self.match_token(&Token::PlusPlus) {
            let operand = Box::new(self.parse_call()?);
            return Ok(Expression::UnaryOp(Box::new(UnaryOp {
                operator: UnaryOpType::Minus, // We'll handle prefix increment at compile time
                operand,
                location: self.current_location(),
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
                location: self.current_location(),
            }));
        } else if self.match_token(&Token::MinusMinus) {
            expr = Expression::PostfixOp(Box::new(PostfixOp {
                operand: Box::new(expr),
                operator: PostfixOpType::Decrement,
                location: self.current_location(),
            }));
        }

        Ok(expr)
    }

    fn parse_call(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&Token::Dot) {
                let method = self.extract_property_name().unwrap_or_default();
                if self.match_token(&Token::LParen) {
                    let args = self.parse_arguments()?;
                    self.consume(&Token::RParen)?;
                    expr = Expression::MethodCall(Box::new(MethodCall {
                        object: Box::new(expr),
                        method,
                        arguments: args,
                        null_safe: false,
                        location: self.current_location(),
                    }));
                } else {
                    expr = Expression::MemberAccess(Box::new(MemberAccess {
                        object: Box::new(expr),
                        member: method,
                        null_safe: false,
                        location: self.current_location(),
                    }));
                }
            } else if self.match_token(&Token::LParen) {
                let args = self.parse_arguments()?;
                self.consume(&Token::RParen)?;
                expr = Expression::FunctionCall(Box::new(FunctionCall {
                    name: Box::new(expr),
                    arguments: args,
                    location: self.current_location(),
                }));
            } else if self.match_token(&Token::LBracket) {
                let index = Box::new(self.parse_expression()?);
                self.consume(&Token::RBracket)?;
                expr = Expression::ArrayAccess(Box::new(ArrayAccess {
                    array: Box::new(expr),
                    index,
                    location: self.current_location(),
                }));
            } else if self.match_token(&Token::QuestionDot) {
                // Null-safe navigation: obj?.method() or obj?.property
                let member = self.extract_property_name().unwrap_or_default();
                if self.match_token(&Token::LParen) {
                    let args = self.parse_arguments()?;
                    self.consume(&Token::RParen)?;
                    expr = Expression::MethodCall(Box::new(MethodCall {
                        object: Box::new(expr),
                        method: member,
                        arguments: args,
                        null_safe: true,
                        location: self.current_location(),
                    }));
                } else {
                    expr = Expression::MemberAccess(Box::new(MemberAccess {
                        object: Box::new(expr),
                        member,
                        null_safe: true,
                        location: self.current_location(),
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
            if self.match_token(&Token::DotDotDot) {
                let expr = self.parse_expression()?;
                args.push(Expression::Spread(Box::new(expr)));
            } else {
                // Check for named argument: identifier = value
                // CFML supports foo(name = value, name2 = value2)
                // We must detect this before parse_expression consumes `=` as assignment.
                let is_named_arg = (matches!(self.peek(0), Token::Identifier(_)) || self.is_identifier_like())
                    && matches!(self.peek(1), Token::Equal);
                if is_named_arg {
                    let name = self.extract_identifier()?;
                    self.advance(); // consume =
                    let value = self.parse_expression()?;
                    // Encode named arg as a struct entry: argumentCollection-style
                    // Use a NamedArgument expression node or encode as key:value
                    args.push(Expression::NamedArgument(Box::new(NamedArgument {
                        name,
                        value: Box::new(value),
                        location: self.current_location(),
                    })));
                } else {
                    args.push(self.parse_expression()?);
                }
            }
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
                location: self.current_location(),
            })),
            Token::False => Ok(Expression::Literal(Literal {
                value: LiteralValue::Bool(false),
                location: self.current_location(),
            })),
            Token::Null => Ok(Expression::Literal(Literal {
                value: LiteralValue::Null,
                location: self.current_location(),
            })),
            Token::Integer(i) => Ok(Expression::Literal(Literal {
                value: LiteralValue::Int(i),
                location: self.current_location(),
            })),
            Token::Double(d) => Ok(Expression::Literal(Literal {
                value: LiteralValue::Double(d),
                location: self.current_location(),
            })),
            Token::String(s) => Ok(Expression::Literal(Literal {
                value: LiteralValue::String(s),
                location: self.current_location(),
            })),
            Token::InterpolatedStringStart => {
                let mut parts: Vec<Expression> = Vec::new();
                while !self.is_at_end() && !self.check(&Token::InterpolatedStringEnd) {
                    let part_token = self.advance().token.clone();
                    match part_token {
                        Token::String(s) => {
                            parts.push(Expression::Literal(Literal {
                                value: LiteralValue::String(s),
                                location: self.current_location(),
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
                                    location: self.current_location(),
                                }));
                            }
                        }
                        _ => break,
                    }
                }
                self.match_token(&Token::InterpolatedStringEnd);
                Ok(Expression::StringInterpolation(StringInterpolation {
                    parts,
                    location: self.current_location(),
                }))
            }
            Token::Identifier(id) => Ok(Expression::Identifier(Identifier {
                name: id,
                location: self.current_location(),
            })),
            // CFML soft keywords used as variables in expressions
            Token::Local => Ok(Expression::Identifier(Identifier {
                name: "local".to_string(),
                location: self.current_location(),
            })),
            Token::Param => Ok(Expression::Identifier(Identifier {
                name: "param".to_string(),
                location: self.current_location(),
            })),
            Token::Output => Ok(Expression::Identifier(Identifier {
                name: "output".to_string(),
                location: self.current_location(),
            })),
            Token::Required => Ok(Expression::Identifier(Identifier {
                name: "required".to_string(),
                location: self.current_location(),
            })),
            Token::Default => Ok(Expression::Identifier(Identifier {
                name: "default".to_string(),
                location: self.current_location(),
            })),
            Token::Include => Ok(Expression::Identifier(Identifier {
                name: "include".to_string(),
                location: self.current_location(),
            })),
            Token::Import => Ok(Expression::Identifier(Identifier {
                name: "import".to_string(),
                location: self.current_location(),
            })),
            Token::Property => Ok(Expression::Identifier(Identifier {
                name: "property".to_string(),
                location: self.current_location(),
            })),
            Token::Abstract => Ok(Expression::Identifier(Identifier {
                name: "abstract".to_string(),
                location: self.current_location(),
            })),
            Token::Final => Ok(Expression::Identifier(Identifier {
                name: "final".to_string(),
                location: self.current_location(),
            })),
            Token::Static => Ok(Expression::Identifier(Identifier {
                name: "static".to_string(),
                location: self.current_location(),
            })),
            Token::Lock => Ok(Expression::Identifier(Identifier {
                name: "lock".to_string(),
                location: self.current_location(),
            })),
            Token::This => Ok(Expression::This(This {
                location: self.current_location(),
            })),
            Token::Super => Ok(Expression::Super(Super {
                location: self.current_location(),
            })),
            Token::New => {
                // After `new`, collect a dotted class name: Ident(.Ident)*
                // e.g. `new framework.one()`, `new com.myapp.Service()`
                // Then parse arguments in parens.
                if let Token::Identifier(_) = self.peek(0) {
                    let mut name_parts = Vec::new();
                    if let Token::Identifier(first) = self.advance().token.clone() {
                        name_parts.push(first);
                    }
                    while self.check(&Token::Dot) {
                        if let Token::Identifier(_) = self.peek(1) {
                            self.advance(); // consume dot
                            if let Token::Identifier(part) = self.advance().token.clone() {
                                name_parts.push(part);
                            }
                        } else {
                            break;
                        }
                    }
                    let class_name = name_parts.join(".");
                    let class = Box::new(Expression::Identifier(Identifier {
                        name: class_name,
                        location: self.current_location(),
                    }));
                    let args = if self.match_token(&Token::LParen) {
                        let a = self.parse_arguments()?;
                        self.consume(&Token::RParen)?;
                        a
                    } else {
                        Vec::new()
                    };
                    Ok(Expression::New(Box::new(NewExpression {
                        class,
                        arguments: args,
                        location: self.current_location(),
                    })))
                } else {
                    // Fallback for non-identifier new (e.g. new (expr)())
                    let class = Box::new(self.parse_call()?);
                    let args = if self.match_token(&Token::LParen) {
                        let a = self.parse_arguments()?;
                        self.consume(&Token::RParen)?;
                        a
                    } else {
                        Vec::new()
                    };
                    Ok(Expression::New(Box::new(NewExpression {
                        class,
                        arguments: args,
                        location: self.current_location(),
                    })))
                }
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
                        location: self.current_location(),
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

        // Skip optional closure metadata attributes (e.g., `localmode = "classic"`)
        // These appear between the RParen and LBrace
        while !self.check(&Token::LBrace) && !self.is_at_end() {
            if self.is_identifier_like() && matches!(self.peek(1), Token::Equal) {
                self.advance(); // skip attribute name
                self.advance(); // skip =
                self.parse_expression()?; // skip attribute value
            } else {
                break;
            }
        }

        let body = self.parse_block()?;

        Ok(Expression::Closure(Box::new(Closure {
            params,
            body,
            location: self.current_location(),
        })))
    }

    fn parse_array_literal(&mut self) -> Result<Expression, ParseError> {
        let mut elements = Vec::new();

        if !self.check(&Token::RBracket) {
            loop {
                if self.check(&Token::RBracket) {
                    break; // trailing comma
                }
                if self.match_token(&Token::DotDotDot) {
                    let expr = self.parse_expression()?;
                    elements.push(Expression::Spread(Box::new(expr)));
                } else {
                    elements.push(self.parse_expression()?);
                }
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        self.consume(&Token::RBracket)?;

        Ok(Expression::Array(Array {
            elements,
            location: self.current_location(),
        }))
    }

    fn parse_struct_literal(&mut self) -> Result<Expression, ParseError> {
        let mut pairs = Vec::new();

        if !self.check(&Token::RBrace) {
            loop {
                if self.check(&Token::RBrace) {
                    break; // trailing comma
                }
                if self.match_token(&Token::DotDotDot) {
                    // Spread: ...expr merges another struct
                    let expr = self.parse_expression()?;
                    // Use a sentinel key to mark this as a spread entry
                    pairs.push((Expression::Spread(Box::new(expr.clone())), expr));
                } else {
                    // In struct literals, `=` is a key-value separator (like `:`),
                    // NOT an assignment operator. We must parse the key without
                    // consuming `=` as assignment.
                    // Check for simple `identifier =` pattern first (most common case).
                    let is_key_eq = matches!(self.peek(0), Token::Identifier(_))
                        && matches!(self.peek(1), Token::Equal);
                    let key = if is_key_eq {
                        // Parse just the identifier, don't let parse_expression consume `=`
                        self.parse_ternary()?
                    } else {
                        self.parse_expression()?
                    };

                    // Support both : and = for struct initialization
                    if self.match_token(&Token::Colon) || self.match_token(&Token::Equal) {
                        let value = self.parse_expression()?;
                        pairs.push((key, value));
                    } else {
                        // Shorthand {x} means {x: x}
                        pairs.push((key.clone(), key));
                    }
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
            location: self.current_location(),
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
