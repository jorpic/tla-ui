
use super::base::Lexer;
use super::token_type::{TokenType, OPERATORS};


#[derive(Debug, PartialEq)]
pub enum Error {
    Unicode(unicode_segmentation::GraphemeIncomplete),
    UnclosedBlockComment,
}


pub trait TlaCombinators {
    // FIXME: add short descriptions
    fn skip_whitespace(&mut self) -> Result<bool, Error>;
    fn skip_until(&mut self, s: &str) -> Result<bool, Error>;
    fn skip(&mut self, s: &str) -> Result<bool, Error>;
    fn skip_many(&mut self, s: &str) -> Result<bool, Error>;
    fn ident(&mut self) -> Result<bool, Error>;
    fn line_comment(&mut self) -> Result<bool, Error>;
    fn block_comment(&mut self) -> Result<bool, Error>;
    fn operator(&mut self) -> Result<Option<TokenType>, Error>;
}

impl TlaCombinators for Lexer<'_> {
    fn skip_whitespace(&mut self) -> Result<bool, Error> {
        loop {
            let c = self.current_char();
            if c != " " && c != "\t" {
                break;
            }
            let res = self.next_char();
            if res != Ok(true) {
                return res.map_err(Error::Unicode);
            }
        }
        Ok(true)
    }

    // NB: Does not revert state in case of error.
    fn skip_until(&mut self, s: &str) -> Result<bool, Error> {
        loop {
            let str = &self.str[self.pos.byte_offset..];
            if str.starts_with(s) {
                return Ok(true);
            }
            match self.next_char() {
                Ok(true) => {},
                res => return res.map_err(Error::Unicode),
            }
        }
    }

    // NB. Does not support complex graphemes in `s`.
    fn skip(&mut self, s: &str) -> Result<bool, Error> {
        let mut premature_end_of_string = false;
        let save_pos = self.pos;
        for c in s.chars() {
            if premature_end_of_string {
                self.pos = save_pos;
                return Ok(false);
            }
            if self.current_char() != c.to_string() {
                self.pos = save_pos;
                return Ok(false);
            }
            match self.next_char() {
                Ok(true) => {}
                Ok(false) => {
                    premature_end_of_string = true;
                }
                Err(err) => return Err(Error::Unicode(err)),
            }
        }
        Ok(true)
    }

    fn skip_many(&mut self, s: &str) -> Result<bool, Error> {
        loop {
            match self.skip(s) {
                Ok(true) => {}
                Ok(false) => return Ok(true),
                err => return err,
            }
        }
    }

    fn ident(&mut self) -> Result<bool, Error> {
        let save_pos = self.pos;
        // FIXME: TLA+ actually allows identifiers starting with a digit.
        if !self
            .current_char()
            .chars()
            .all(|c| c.is_ascii_alphabetic() || c == '_')
        {
            self.pos = save_pos;
            return Ok(false);
        }
        loop {
            if let Err(err) = self.next_char() {
                return Err(Error::Unicode(err));
            }
            let valid_char = self
                .current_char()
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_');
            if !valid_char {
                return Ok(true);
            }
        }
    }

    fn line_comment(&mut self) -> Result<bool, Error> {
        match self.skip("\\*") {
            Ok(true) => self.skip_until("\n").map(|_| true), // end of line is a valid terminator
            res => res,
        }
    }

    fn block_comment(&mut self) -> Result<bool, Error> {
        match self.skip("(*") {
            Ok(true) => {
                let mut depth = 1;
                while depth > 0 {
                    match self.next_char() {
                        Ok(true) => {},
                        Ok(false) => return Err(Error::UnclosedBlockComment),
                        Err(err) => return Err(Error::Unicode(err)),
                    }
                    match self.skip("(*") {
                        Ok(true) => depth += 1,
                        Ok(false) => {},
                        Err(err) => return Err(err),
                    }
                    match self.skip("*)") {
                        Ok(true) => depth -= 1,
                        Ok(false) => {},
                        Err(err) => return Err(err),
                    }
                }
                Ok(true)
            }
            res => res,
        }
    }

    fn operator(&mut self) -> Result<Option<TokenType>, Error> {
        let start = self.pos;
        let mut prev_pos = self.pos;
        let mut op = self.current_char();
        let mut res = None;
        loop {
            match OPERATORS.binary_search_by_key(&op, |t| t.0) {
                Ok(i) => {
                    res = Some(OPERATORS[i].1);
                    prev_pos = self.pos;
                    match self.next_char() {
                        Ok(true) => {
                            op = self.substring(&start, &self.pos);
                        }
                        Ok(false) => return Ok(res),
                        Err(err) => return Err(Error::Unicode(err)),
                    }
                }
                _ => {
                    // continue if is prefix
                    self.pos = prev_pos;
                    return Ok(res);
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::super::base::*;
    use super::*;

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
        assert_eq!(lx.skip_many("+"), Ok(true));
        assert_eq!(lx.current_char(), "*");

        let mut lx = Lexer::new("++++");
        assert_eq!(lx.skip_many("+"), Ok(true));
        assert_eq!(lx.current_char(), "");
    }

    #[test]
    fn skip_until() -> Result<(), Error> {
        let mut lx = Lexer::new("++++xyz");
        assert_eq!(lx.skip_until("xyz"), Ok(true));
        assert_eq!(lx.current_char(), "x");

        let mut lx = Lexer::new("");
        assert_eq!(lx.skip_until("xyz"), Ok(false));
        Ok(())
    }


    #[test]
    fn line_comment() {
        let mut lx = Lexer::new("\\* hello world");
        assert_eq!(lx.line_comment(), Ok(true));
    }

    #[test]
    fn block_comment() {
        let mut lx = Lexer::new("(* hello (*worl*)d*)");
        assert_eq!(lx.block_comment(), Ok(true));

        let mut lx = Lexer::new("(* hello (*world*)");
        assert_eq!(lx.block_comment(), Err(Error::UnclosedBlockComment));
    }

    #[test]
    fn operator() {
        let mut lx = Lexer::new("\\/");
        let start = lx.pos;
        assert_eq!(lx.operator(), Ok(Some(TokenType::InfixOperator)));
        let end = lx.pos;
        assert_eq!(lx.substring(&start, &end), "\\/");

        let mut lx = Lexer::new("---+->");
        let start = lx.pos;
        assert_eq!(lx.operator(), Ok(Some(TokenType::InfixOperator)));
        let end = lx.pos;
        assert_eq!(lx.substring(&start, &end), "--");
        let start = lx.pos;
        assert_eq!(lx.operator(), Ok(Some(TokenType::InfixOperator)));
        let end = lx.pos;
        assert_eq!(lx.substring(&start, &end), "-+->");
    }
}
