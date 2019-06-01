#![allow(dead_code)]

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
    CommentLine,
    CommentBlock,
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

pub struct Lexer<'a> {
    str: &'a str,
    grc: GraphemeCursor,
    state: LexerState,
    pos: Pos,
    snapshots: Vec<LexSnapshot>,
}

#[derive(Debug, Clone, Copy)]
pub enum LexerState {
    BeforeModuleHeader,
    ModuleBody,
    // Comment(usize), // depth
}

struct LexSnapshot {
    state: LexerState,
    pos: Pos,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    EndOfString,
    MalformedGrapheme(GraphemeIncomplete),
    NotRecognized,
}

impl<'a> Lexer<'a> {
    pub fn new(s: &'a str) -> Self {
        let mut lex = Lexer {
            str: s,
            state: LexerState::BeforeModuleHeader,
            // Pos {col = 0, char_size = 0} represents position before first character.
            // NB. This may lead to unexpected side effects.
            pos: Pos {
                col: 1,
                line: 1,
                byte_offset: 0,
                char_size: 0,
            },
            grc: GraphemeCursor::new(0, s.len(), true),
            snapshots: Vec::new(),
        };
        let _ = lex.next_char();
        lex
    }

    // Why we represent current char with &str?
    // This is because current char is actually a grapheme that may be
    // represented as a unicode point with a modifier. E.g. 'e' and acute.
    pub fn current_char(&self) -> &str {
        // FIXME: return error if EOF or MalformedUnicode
        &self.str[self.pos.byte_offset..self.pos.byte_offset + self.pos.char_size]
    }

    pub fn substring(&self, start: &Pos, end: &Pos) -> &str {
        &self.str[start.byte_offset..end.byte_offset]
    }

    pub fn next_char(&mut self) -> Result<(), Error> {
        // Next character starts immediately after the current one.
        self.pos.byte_offset = self.pos.byte_offset + self.pos.char_size;
        self.grc.set_cursor(self.pos.byte_offset);
        // Check next character and determine its size in bytes.
        // char_size == 0 encodes exceptional cases:
        //      - position before the first char
        //      - position after the last char
        //      - position at a malformed unicode grapheme
        let res = match self.grc.next_boundary(self.str, 0) {
            Ok(Some(end)) => {
                self.pos.char_size = end - self.pos.byte_offset;
                Ok(())
            }
            Ok(None) => {
                self.pos.char_size = 0;
                Err(Error::EndOfString)
            }
            Err(err) => {
                self.pos.char_size = 0;
                Err(Error::MalformedGrapheme(err))
            }
        };
        // Returns "" before the start, at the end of string, at the error.
        let prev_char = self.current_char();
        // Update line and column depending on previous char.
        match prev_char {
            "\n" | "\r\n" => {
                self.pos.line += 1;
                self.pos.col = 1;
            }
            "\t" => self.pos.col += 4, // FIXME: tab_size
            "" => {}
            _ => self.pos.col += 1,
        }
        res
    }

    pub fn save_snapshot(&mut self) {
        self.snapshots.push(LexSnapshot {
            pos: self.pos,
            state: self.state,
        });
    }

    pub fn restore_snapshot(&mut self) {
        debug_assert!(!self.snapshots.is_empty());
        let LexSnapshot { pos, state } = self.snapshots.pop().expect("no snapshot to restore");
        self.pos = pos;
        self.state = state;
    }

    pub fn drop_snapshot(&mut self) {
        debug_assert!(!self.snapshots.is_empty());
        self.snapshots.pop();
    }

    pub fn skip_whitespace(&mut self) -> Result<(), Error> {
        loop {
            let c = self.current_char();
            if c != " " && c != "\t" {
                break;
            }
            if let Err(err) = self.next_char() {
                return Err(err);
            }
        }
        Ok(())
    }

    // NB: Does not revert state in case of error.
    pub fn skip_until(&mut self, s: &str) -> Result<(), Error> {
        loop {
            let str = &self.str[self.pos.byte_offset..];
            if str.starts_with(s) {
                return Ok(());
            }
            if let Err(err) = self.next_char() {
                return Err(err);
            }
        }
    }

    // NB. Does not support complex graphemes in `s`.
    pub fn skip(&mut self, s: &str) -> Result<bool, Error> {
        let mut premature_end_of_string = false;
        self.save_snapshot();
        for c in s.chars() {
            if premature_end_of_string {
                self.restore_snapshot();
                return Ok(false);
            }
            if self.current_char() != c.to_string() {
                self.restore_snapshot();
                return Ok(false);
            }
            match self.next_char() {
                Ok(_) => {}
                Err(Error::EndOfString) => {
                    premature_end_of_string = true;
                }
                Err(err) => return Err(err), // only MalformedGrapheme is possible here
            }
        }
        self.drop_snapshot();
        Ok(true)
    }

    pub fn skip_many(&mut self, s: &str) -> Result<(), Error> {
        loop {
            match self.skip(s) {
                Ok(true) => {}
                Ok(false) | Err(Error::EndOfString) => return Ok(()),
                Err(err) => return Err(err),
            }
        }
    }

    pub fn ident(&mut self) -> Result<bool, Error> {
        self.save_snapshot();
        // FIXME: TLA+ actually allows identifiers starting with a digit.
        if !self
            .current_char()
            .chars()
            .all(|c| c.is_ascii_alphabetic() || c == '_')
        {
            self.restore_snapshot();
            return Ok(false);
        }
        loop {
            if let Err(err) = self.next_char() {
                return Err(err);
            }
            let valid_char = self
                .current_char()
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_');
            if !valid_char {
                self.drop_snapshot();
                return Ok(true);
            }
        }
    }

    pub fn next_token(&mut self) -> Result<(Pos, Pos, TokenType), Error> {
        let start = self.pos;
        match self.state {
            LexerState::BeforeModuleHeader => self.skip_until("----").and_then(|_| {
                self.state = LexerState::ModuleBody;
                self.next_token()
            }),
            _ => match self.current_char() {
                " " | "\t" => self.skip_whitespace().and_then(|_| self.next_token()),
                "\n" | "\r\n" => self.next_char().and_then(|_| {
                    let another_start = self.pos; // Don't include CR in the token span.
                    self.skip_whitespace()
                        .map(|_| (another_start, self.pos, TokenType::Indent))
                }),
                "-" => match self.skip("----") {
                    Ok(true) => self
                        .skip_many("-")
                        .map(|_| (start, self.pos, TokenType::Separator)),
                    Ok(false) => Err(Error::NotRecognized), // FIXME: various operators
                    Err(err) => Err(err),
                }
                "\\" => match self.skip("\\*") {
                    Ok(true) => match self.skip_until("\n") {
                        Ok(_) | Err(Error::EndOfString) =>
                            Ok((start, self.pos, TokenType::CommentLine)),
                        Err(err) => Err(err),
                    }
                    Ok(false) => Err(Error::NotRecognized), // FIXME: various operators
                    Err(err) => Err(err),
                }
                _ => match self.ident() {
                    Ok(true) => {
                        let end = self.pos;
                        let name = self.substring(&start, &end);
                        match KEYWORDS.binary_search_by_key(&name, |t| t.0) {
                            Ok(i) => Ok((start, end, KEYWORDS[i].1)),
                            _ => Ok((start, end, TokenType::Identifier)),
                        }
                    }
                    Ok(false) => Err(Error::NotRecognized),
                    Err(err) => Err(err),
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Lexer, Error, TokenType};

    #[test]
    fn next_char() {
        let mut lx = Lexer::new("");
        assert_eq!(lx.current_char(), "");
        assert_eq!(lx.next_char(), Err(Error::EndOfString));
        let eos_pos = lx.pos;
        assert_eq!(lx.next_char(), Err(Error::EndOfString)); // idempotent
        assert_eq!(lx.pos, eos_pos); // position does not change

        let mut lx = Lexer::new("x");
        assert_eq!(lx.current_char(), "x");
        assert_eq!(lx.next_char(), Err(Error::EndOfString));
        assert_eq!(lx.current_char(), "");
        assert_eq!(lx.next_char(), Err(Error::EndOfString));
    }

    #[test]
    fn skip() {
        let mut lx = Lexer::new("xyz");
        assert_eq!(lx.skip("x"), Ok(true));
        assert_eq!(lx.current_char(), "y"); // advance if match
        let mut lx = Lexer::new("abc");
        assert_eq!(lx.skip("x"), Ok(false));
        assert_eq!(lx.current_char(), "a"); // backtrack if no match

        let mut lx = Lexer::new("x");
        assert_eq!(lx.current_char(), "x");
        assert_eq!(lx.skip("x"), Ok(true));
        assert_eq!(lx.current_char(), "");
        assert_eq!(lx.skip("x"), Ok(false));
    }

    #[test]
    fn skip_many() {
        let mut lx = Lexer::new("++++*");
        assert_eq!(lx.skip_many("+"), Ok(()));
        assert_eq!(lx.current_char(), "*");

        let mut lx = Lexer::new("++++");
        assert_eq!(lx.skip_many("+"), Ok(()));
        assert_eq!(lx.current_char(), "");
    }

    #[test]
    fn skip_until() {
        let mut lx = Lexer::new("++++xyz");
        assert_eq!(lx.skip_until("xyz"), Ok(()));
        assert_eq!(lx.current_char(), "x");

        let mut lx = Lexer::new("");
        assert_eq!(lx.skip_until("xyz"), Err(Error::EndOfString));
    }

    #[test]
    fn module_header() -> Result<(), Error> {
        let mut lx = Lexer::new("");
        assert_eq!(lx.next_token(), Err(Error::EndOfString));
        let mut lx = Lexer::new(" --- ");
        assert_eq!(lx.next_token(), Err(Error::EndOfString));

        let mut lx = Lexer::new("----");
        assert_eq!(lx.next_token()?.2, TokenType::Separator);
        assert_eq!(lx.next_token(), Err(Error::EndOfString));
        let mut lx = Lexer::new("------------");
        assert_eq!(lx.next_token()?.2, TokenType::Separator);
        assert_eq!(lx.next_token(), Err(Error::EndOfString));
        Ok(())
    }

    #[test]
    fn line_comment() -> Result<(), Error> {
        let mut lx = Lexer::new("---- \\* hello world");
        assert_eq!(lx.next_token()?.2, TokenType::Separator);
        assert_eq!(lx.next_token()?.2, TokenType::CommentLine);
        assert_eq!(lx.next_token(), Err(Error::EndOfString));
        Ok(())
    }
}
