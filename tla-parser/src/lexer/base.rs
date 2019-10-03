use std::str;
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};


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
    pub str: &'a str,
    grc: GraphemeCursor,
    pub pos: Pos,
}

/// This is a basic Unicode-aware grapheme stream.
/// It is able to return current grapheme.
/// It is able to advance pointer forward by one grapheme.
impl<'a> Lexer<'a> {
    pub fn new(s: &'a str) -> Self {
        let mut lex = Lexer {
            str: s,
            // Pos {col = 0, char_size = 0} represents position before first character.
            // NB. This may lead to unexpected side effects.
            pos: Pos {
                col: 1,
                line: 1,
                byte_offset: 0,
                char_size: 0,
            },
            grc: GraphemeCursor::new(0, s.len(), true),
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

    /// Extracts part of the underlying string.
    pub fn substring(&self, start: &Pos, end: &Pos) -> &str {
        &self.str[start.byte_offset..end.byte_offset]
    }

    /// Advances cursor forward.
    /// Returns false if EOF, returns error if next grapheme is malformed.
    pub fn next_char(&mut self) -> Result<bool, GraphemeIncomplete> {
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
                Ok(true)
            }
            Ok(None) => {
                self.pos.char_size = 0;
                Ok(false)
            }
            Err(err) => {
                self.pos.char_size = 0;
                Err(err)
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
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_char() {
        // Empty string returns Ok(false)
        // Empty string current_char is ""
        // Malformed char returns error
        // next_char at EOF is idempotent
        // next_char at malformed grapheme is idempotent
        // next_char at simple char
        // mext_char at grapheme
        let mut lx = Lexer::new("");
        assert_eq!(lx.current_char(), "");
        assert_eq!(lx.next_char(), Ok(false));
        let eos_pos = lx.pos;
        assert_eq!(lx.next_char(), Ok(false)); // idempotent
        assert_eq!(lx.pos, eos_pos); // position does not change

        let mut lx = Lexer::new("x");
        assert_eq!(lx.current_char(), "x");
        assert_eq!(lx.next_char(), Ok(false));
        assert_eq!(lx.current_char(), "");
        assert_eq!(lx.next_char(), Ok(false));
    }
}
