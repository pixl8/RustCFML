//! CFML Lexer - Tokenizes CFML source code

use crate::token::Token;
use cfml_common::position::{Position, SourceLocation};

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
    tokens: Vec<TokenWithLoc>,
}

#[derive(Debug, Clone)]
pub struct TokenWithLoc {
    pub token: Token,
    pub location: SourceLocation,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
            tokens: Vec::new(),
        }
    }

    pub fn tokenize(&mut self) -> Vec<TokenWithLoc> {
        while !self.is_at_end() {
            self.scan_token();
        }
        self.tokens.push(TokenWithLoc {
            token: Token::Eof,
            location: SourceLocation::new(
                Position::new(self.line, self.column),
                Position::new(self.line, self.column),
            ),
        });
        self.tokens.clone()
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn current(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.pos]
        }
    }

    fn peek(&self, offset: usize) -> char {
        let idx = self.pos + offset;
        if idx >= self.source.len() {
            '\0'
        } else {
            self.source[idx]
        }
    }

    fn advance(&mut self) -> char {
        let c = self.current();
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        c
    }

    fn add_token(&mut self, token: Token) {
        let location = SourceLocation::new(
            Position::new(self.line, self.column),
            Position::new(self.line, self.column),
        );
        self.tokens.push(TokenWithLoc { token, location });
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            '(' => self.add_token(Token::LParen),
            ')' => self.add_token(Token::RParen),
            '{' => self.add_token(Token::LBrace),
            '}' => self.add_token(Token::RBrace),
            '[' => self.add_token(Token::LBracket),
            ']' => self.add_token(Token::RBracket),
            ',' => self.add_token(Token::Comma),
            '.' => {
                if self.peek(0) == '.' && self.peek(1) == '.' {
                    self.advance(); // consume second dot
                    self.advance(); // consume third dot
                    self.add_token(Token::DotDotDot);
                } else {
                    self.add_token(Token::Dot);
                }
            }
            ';' => self.add_token(Token::Semicolon),
            ':' => self.add_token(Token::Colon),
            '^' => self.add_token(Token::Caret),
            '#' => self.add_token(Token::HashSign),
            '\\' => self.add_token(Token::Backslash),

            '?' => {
                if self.match_char('.') {
                    self.add_token(Token::QuestionDot);
                } else if self.match_char(':') {
                    self.add_token(Token::QuestionColon);
                } else {
                    self.add_token(Token::Question);
                }
            }

            '+' => {
                if self.match_char('=') {
                    self.add_token(Token::PlusEqual);
                } else if self.match_char('+') {
                    self.add_token(Token::PlusPlus);
                } else {
                    self.add_token(Token::Plus);
                }
            }
            '-' => {
                if self.match_char('=') {
                    self.add_token(Token::MinusEqual);
                } else if self.match_char('-') {
                    self.add_token(Token::MinusMinus);
                } else if self.match_char('>') {
                    self.add_token(Token::Arrow);
                } else {
                    self.add_token(Token::Minus);
                }
            }
            '*' => {
                if self.match_char('=') {
                    self.add_token(Token::StarEqual);
                } else {
                    self.add_token(Token::Star);
                }
            }
            '/' => {
                if self.match_char('/') {
                    self.single_line_comment();
                } else if self.match_char('*') {
                    self.multi_line_comment();
                } else if self.match_char('=') {
                    self.add_token(Token::SlashEqual);
                } else {
                    self.add_token(Token::Slash);
                }
            }
            '%' => self.add_token(Token::Percent),

            '&' => {
                if self.match_char('&') {
                    self.add_token(Token::AmpAmp);
                } else if self.match_char('=') {
                    self.add_token(Token::AmpEqual);
                } else {
                    self.add_token(Token::Amp); // String concatenation
                }
            }
            '|' => {
                if self.match_char('|') {
                    self.add_token(Token::BarBar);
                }
                // Single | is not a valid CFML operator, ignore
            }

            '=' => {
                if self.match_char('=') {
                    self.add_token(Token::EqualEqual);
                } else if self.match_char('>') {
                    self.add_token(Token::FatArrow);
                } else {
                    self.add_token(Token::Equal);
                }
            }
            '!' => {
                if self.match_char('=') {
                    self.add_token(Token::BangEqual);
                } else {
                    self.add_token(Token::Bang);
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.add_token(Token::GreaterEqual);
                } else {
                    self.add_token(Token::Greater);
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.add_token(Token::LessEqual);
                } else if self.match_char('>') {
                    self.add_token(Token::BangEqual); // <> is != in CFML
                } else {
                    self.add_token(Token::Less);
                }
            }

            '"' => self.string('"'),
            '\'' => self.string('\''),

            '0'..='9' => self.number(c),

            'a'..='z' | 'A'..='Z' | '_' | '$' => self.identifier(c),

            ' ' | '\t' | '\r' | '\n' => {} // Whitespace already handled by advance()

            _ => {}
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.current() != expected {
            false
        } else {
            self.advance();
            true
        }
    }

    fn string(&mut self, quote: char) {
        let start_line = self.line;
        let start_column = self.column;

        // For double-quoted strings, handle #expr# interpolation
        if quote == '"' {
            let mut parts: Vec<(bool, String)> = Vec::new(); // (is_expr, content)
            let mut current_str = String::new();
            let mut has_interpolation = false;

            while !self.is_at_end() && self.current() != quote {
                if self.current() == '\\' {
                    self.advance();
                    match self.current() {
                        'n' => current_str.push('\n'),
                        't' => current_str.push('\t'),
                        'r' => current_str.push('\r'),
                        '\\' => current_str.push('\\'),
                        '"' => current_str.push('"'),
                        '#' => current_str.push('#'),
                        _ => current_str.push(self.current()),
                    }
                    self.advance();
                } else if self.current() == '#' && self.peek(1) == '#' {
                    // ## is an escaped # literal
                    current_str.push('#');
                    self.advance();
                    self.advance();
                } else if self.current() == '#' {
                    // Start of interpolation expression
                    has_interpolation = true;
                    if !current_str.is_empty() {
                        parts.push((false, current_str.clone()));
                        current_str.clear();
                    }
                    self.advance(); // skip opening #
                    let mut expr_str = String::new();
                    let mut depth = 0;
                    while !self.is_at_end() && !(self.current() == '#' && depth == 0) {
                        if self.current() == '(' { depth += 1; }
                        if self.current() == ')' { depth -= 1; }
                        expr_str.push(self.current());
                        self.advance();
                    }
                    if !self.is_at_end() {
                        self.advance(); // skip closing #
                    }
                    if !expr_str.is_empty() {
                        parts.push((true, expr_str));
                    }
                } else {
                    // CFML: doubled quote acts as escape
                    if self.current() == quote && self.peek(0) == quote {
                        current_str.push(quote);
                        self.advance();
                    } else {
                        current_str.push(self.current());
                    }
                    self.advance();
                }
            }

            if !self.is_at_end() {
                self.advance(); // closing quote
            }

            if has_interpolation {
                if !current_str.is_empty() {
                    parts.push((false, current_str));
                }
                // Emit InterpolatedStringStart, then parts, then InterpolatedStringEnd
                self.tokens.push(TokenWithLoc {
                    token: Token::InterpolatedStringStart,
                    location: SourceLocation::new(
                        Position::new(start_line, start_column),
                        Position::new(self.line, self.column),
                    ),
                });
                for (is_expr, content) in parts {
                    if is_expr {
                        self.tokens.push(TokenWithLoc {
                            token: Token::InterpolatedExpr(content),
                            location: SourceLocation::new(
                                Position::new(start_line, start_column),
                                Position::new(self.line, self.column),
                            ),
                        });
                    } else {
                        self.tokens.push(TokenWithLoc {
                            token: Token::String(content),
                            location: SourceLocation::new(
                                Position::new(start_line, start_column),
                                Position::new(self.line, self.column),
                            ),
                        });
                    }
                }
                self.tokens.push(TokenWithLoc {
                    token: Token::InterpolatedStringEnd,
                    location: SourceLocation::new(
                        Position::new(start_line, start_column),
                        Position::new(self.line, self.column),
                    ),
                });
            } else {
                // No interpolation, emit as regular string
                self.tokens.push(TokenWithLoc {
                    token: Token::String(current_str),
                    location: SourceLocation::new(
                        Position::new(start_line, start_column),
                        Position::new(self.line, self.column),
                    ),
                });
            }
        } else {
            // Single-quoted strings: no interpolation
            let mut value = String::new();
            while !self.is_at_end() && self.current() != quote {
                if self.current() == '\\' {
                    self.advance();
                    match self.current() {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        'r' => value.push('\r'),
                        '\\' => value.push('\\'),
                        '\'' => value.push('\''),
                        _ => value.push(self.current()),
                    }
                } else {
                    if self.current() == quote && self.peek(0) == quote {
                        value.push(quote);
                        self.advance();
                    } else {
                        value.push(self.current());
                    }
                }
                self.advance();
            }
            if !self.is_at_end() {
                self.advance(); // closing quote
            }
            self.tokens.push(TokenWithLoc {
                token: Token::String(value),
                location: SourceLocation::new(
                    Position::new(start_line, start_column),
                    Position::new(self.line, self.column),
                ),
            });
        }
    }

    fn number(&mut self, first: char) {
        let start_column = self.column - 1;
        let mut value = String::new();
        value.push(first);

        while !self.is_at_end() && self.current().is_ascii_digit() {
            value.push(self.current());
            self.advance();
        }

        // Check for decimal point (but not if followed by a letter - could be method call)
        if !self.is_at_end() && self.current() == '.' && self.peek(1).is_ascii_digit() {
            value.push(self.current());
            self.advance();
            while !self.is_at_end() && self.current().is_ascii_digit() {
                value.push(self.current());
                self.advance();
            }
        }

        // Scientific notation
        if !self.is_at_end() && (self.current() == 'e' || self.current() == 'E') {
            value.push(self.current());
            self.advance();
            if !self.is_at_end() && (self.current() == '+' || self.current() == '-') {
                value.push(self.current());
                self.advance();
            }
            while !self.is_at_end() && self.current().is_ascii_digit() {
                value.push(self.current());
                self.advance();
            }
        }

        let token = if value.contains('.') || value.contains('e') || value.contains('E') {
            Token::Double(value.parse().unwrap_or(0.0))
        } else {
            Token::Integer(value.parse().unwrap_or(0))
        };

        self.tokens.push(TokenWithLoc {
            token,
            location: SourceLocation::new(
                Position::new(self.line, start_column),
                Position::new(self.line, self.column),
            ),
        });
    }

    fn identifier(&mut self, first: char) {
        let start_column = self.column - 1;
        let mut value = String::new();
        value.push(first);

        while !self.is_at_end()
            && (self.current().is_ascii_alphanumeric()
                || self.current() == '_'
                || self.current() == '$')
        {
            value.push(self.current());
            self.advance();
        }

        let token = Token::keyword(&value).unwrap_or_else(|| Token::Identifier(value));

        self.tokens.push(TokenWithLoc {
            token,
            location: SourceLocation::new(
                Position::new(self.line, start_column),
                Position::new(self.line, self.column),
            ),
        });
    }

    fn single_line_comment(&mut self) {
        while !self.is_at_end() && self.current() != '\n' {
            self.advance();
        }
    }

    fn multi_line_comment(&mut self) {
        while !self.is_at_end() {
            if self.current() == '*' && self.peek(1) == '/' {
                self.advance();
                self.advance();
                break;
            }
            self.advance();
        }
    }
}

pub fn tokenize(source: String) -> Vec<TokenWithLoc> {
    let mut lexer = Lexer::new(source);
    lexer.tokenize()
}
