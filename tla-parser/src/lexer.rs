use std::str;
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
    Module,
    Extends,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    Separator,
    Indent,
    Identifier,
    Wildcard,
    Keyword(Keyword),
    Unknown,
}

static KEYWORDS: &'static [(&'static str, TokenType)] = &[
    ("EXTENDS", TokenType::Keyword(Keyword::Extends)),
    ("MODULE", TokenType::Keyword(Keyword::Module)),
    ("_", TokenType::Wildcard),
];


/// Specifies a position in a string.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pos {
    /// Line number.
    pub line: usize,
    /// Column number from the start of the line.
    pub col: usize,
    /// Byte offset from the start of the string.
    pub byte_offset: usize,
    /// Character size in bytes.
    pub char_size: usize,
}


impl Pos {
    // FIXME: explain why we represent current char with &str.
    // This is because current char is actually a grapheme that may be represented as a unicode
    // point with a modifier. E.g. 'e' and acute.
    pub fn current_char<'a>(&self, str: &'a str) -> &'a str {
        &str[self.byte_offset .. self.byte_offset + self.char_size]
    }
}


pub enum State {
    BeforeModuleHeader,
    ModuleBody,
}


pub struct Lexer<'a> {
    str: &'a str,
    grc: GraphemeCursor,
    state: State,
    pos: Pos,
    errors: Vec<(Pos, Error)>,
    snapshots: Vec<LexSnapshot>,
}


struct LexSnapshot {
    // current state: Char('-'), ident, comment(depth)
    pos: Pos,
}


#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidUnicodeChar,
    UnicodeGrapheme(GraphemeIncomplete),
    FailedExpectation(&'static str)
}


impl<'a> Lexer<'a> {
    pub fn new(s: &'a str) -> Self {
        let mut lex = Lexer {
            str: s,
            state: State::BeforeModuleHeader,
            // Pos {col = 0, char_size = 0} represents position before first character.
            // NB. This may lead to unexpected side effects.
            pos: Pos{col: 0, line: 1, byte_offset: 0, char_size: 0},
            grc: GraphemeCursor::new(0, s.len(), true),
            errors: Vec::new(),
            snapshots: Vec::new(),
        };
        lex.next_char();
        lex
    }

    pub fn current_char(&self) -> &str { // FIXME: return error if EOF or MalformedUnicode
        self.pos.current_char(self.str)
    }


    pub fn substring(&self, start: &Pos, end: &Pos) -> &str {
        &self.str[start.byte_offset .. end.byte_offset]
    }


    // FIXME: return Result<&str, (EOS | MalformedUnicode)>
    pub fn next_char(&mut self) -> bool {
        let mut pos = self.pos;
        let prev_char = pos.current_char(self.str); // returns "" if there is no prev char
        pos.byte_offset += pos.char_size;
        self.grc.set_cursor(pos.byte_offset); // move cursor behind the current char
        let next_offset = match self.grc.next_boundary(self.str, 0) {
            Ok(Some(end)) => end,
            Ok(None) => return false, // End of string reached
            Err(err) => {
                self.errors.push((pos, Error::UnicodeGrapheme(err)));
                return false;
            }
        };

        pos.char_size = next_offset - pos.byte_offset;
        match prev_char {
            "\n" | "\r\n" => {
                pos.line += 1;
                pos.col = 1;
            },
            "\t" => {
                pos.col += 4; // FIXME:
            },
            _ => {
                pos.col += 1;
            }
        }

        self.pos = pos;
        return true;
    }


    pub fn save_snapshot(&mut self) {
        self.snapshots.push(LexSnapshot{pos: self.pos});
    }


    pub fn restore_snapshot(&mut self) {
        debug_assert!(self.snapshots.len() > 0);
        let LexSnapshot{pos} = self.snapshots.pop().expect("no snapshot to restore");
        self.pos = pos;
    }


    pub fn drop_snapshot(&mut self) {
        debug_assert!(self.snapshots.len() > 0);
        self.snapshots.pop();
    }


    pub fn skip_whitespace(&mut self) {
        loop {
            let c = self.current_char();
            if c != " " && c != "\t" {
                break;
            }
            if !self.next_char() { break; } // FIXME: end of string
        }
    }


    pub fn skip_until(&mut self, s: &str) -> bool {
        self.save_snapshot();
        loop {
            let str = &self.str[self.pos.byte_offset..];
            if str.starts_with(s) {
                self.drop_snapshot();
                return true;
            }
            if !self.next_char() {
                // FIXME: restore snapshot in case of end of string
                // Fail hard and save error position in case of malformed unicode
                self.restore_snapshot();
                return false;
            }
        }
    }


    // Does not support complex graphemes in `s`.
    pub fn skip(&mut self, s: &str) -> bool {
        let mut premature_end_of_string = false;
        self.save_snapshot();
        for c in s.chars() {
            if premature_end_of_string || self.current_char() != c.to_string() {
                self.restore_snapshot();
                return false;
            }
            if !self.next_char() { // FIXME: EOS vs MalformedUnicode
                premature_end_of_string = true;
            }
        }
        self.drop_snapshot();
        return true;
    }


    // Same as skip, but generates error if expected string is not found.
    pub fn expect(&mut self, s: &'static str) {
        if !self.skip(s) {
            self.errors.push((self.pos, Error::FailedExpectation(s)));
        }
    }

    pub fn ident(&mut self) -> bool {
        self.save_snapshot();
        // FIXME: TLA+ actually allows identifiers starting with a digit.
        if !self.current_char().chars().all(|c| c.is_ascii_alphabetic() || c == '_') {
            self.restore_snapshot();
            return false;
        }
        loop {
            if !self.next_char() { break; }
            let valid_char = self.current_char().chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_');
            if !valid_char { break; }
        }
        self.drop_snapshot();
        return true;
    }


    // FIXME: do not push errors to vector, just return them here.
    // Delegate responsibility to collect errors to the parser.
    pub fn next_token(&mut self) -> Option<(Pos, Pos, TokenType)> {
        match self.state {
            State::BeforeModuleHeader => {
                if self.skip_until("----") {
                    self.state = State::ModuleBody;
                    let start = self.pos;
                    loop {
                        if !self.skip("-") {
                            return Some((start, self.pos, TokenType::Separator));
                        }
                    }
                } else {
                    return None; // FIXME: push error
                }
            },
            _ => match self.current_char() {
                " " | "\t" => {
                    self.skip_whitespace();
                    return self.next_token();
                },
                "\n" | "\r\n" => {
                    self.next_char();
                    let start = self.pos;
                    self.skip_whitespace(); // FIXME: return span
                    let end = self.pos;
                    return Some((start, end, TokenType::Indent));
                }

                _ => {
                    let start = self.pos;
                    if self.ident() {
                        let end = self.pos;
                        let name = self.substring(&start, &end);
                        match KEYWORDS.binary_search_by_key(&name, |t| t.0) {
                            Ok(i) => return Some((start, end, KEYWORDS[i].1)),
                            _ => return Some((start, end, TokenType::Identifier)),
                        }
                    }
                    return None;
                }
            }
        }
    }
}


// TODO:
// - Handle end of string
// - struct Lexeme {start, end, token}
// - drop errors
// - better lexer tests


#[cfg(test)]
mod tests {
    use super::Lexer;

    #[test]
    pub fn xxx() {
        let s = "x         ---- MODULE hello\r\ny̆es❤\r\nh";
        let mut lx = Lexer::new(s);
        assert_eq!(lx.current_char(), "x");
        assert_eq!(lx.pos.col, 1);
        assert_eq!(lx.pos.line, 1);

        lx.save_snapshot();
        lx.next_char();
        lx.skip_whitespace();
        assert_eq!(lx.current_char(), "-");
        assert_eq!(lx.pos.col, 11);
        loop { if !lx.skip("-") { break; }};
        lx.skip_whitespace();
        assert_eq!(lx.skip("MODULE"), true);
        lx.skip_whitespace();

        lx.restore_snapshot();
        assert_eq!(lx.current_char(), "x");
    }

    #[test]
    pub fn yyy() {
        let s = "x         ---- \n    MODULE hello";
        let mut lx = Lexer::new(s);
        lx.next_token();
        lx.next_token();
        lx.next_token();
        lx.next_token();
        assert_eq!(lx.next_token(), None);
    }
}

