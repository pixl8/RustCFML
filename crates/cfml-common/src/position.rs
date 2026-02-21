//! Source code position tracking

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub fn start() -> Self {
        Self { line: 1, column: 1 }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::start()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceLocation {
    pub start: Position,
    pub end: Position,
}

impl SourceLocation {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn merge(self, other: Self) -> Self {
        Self {
            start: self.start,
            end: other.end,
        }
    }
}

impl Default for SourceLocation {
    fn default() -> Self {
        Self {
            start: Position::default(),
            end: Position::default(),
        }
    }
}
