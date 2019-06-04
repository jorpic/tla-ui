#![allow(dead_code)]

mod base;
mod combinators;
mod token_type;

use base::{Pos, Lexer, Error};
use combinators::TlaCombinators;
use token_type::{TokenType, KEYWORDS};


pub fn next_token(mut lx: &mut Lexer) -> Result<(Pos, Pos, TokenType), Error> {
    let start = lx.pos;
    match lx.current_char() {
        "" => Err(Error::EndOfString),
        " " | "\t" => lx.skip_whitespace()
            .and_then(|_| next_token(&mut lx)),
        "\n" | "\r\n" => lx.next_char().and_then(|_| {
            let another_start = lx.pos; // Don't include CR in the token span.
            lx.skip_whitespace()
                .map(|_| (another_start, lx.pos, TokenType::Indent))
        }),
        "-" => match lx.skip("----") {
            Ok(true) => lx
                .skip_many("-")
                .map(|_| (start, lx.pos, TokenType::Separator)),
            Ok(false) => match lx.operator() {
                Ok(Some(op)) => Ok((start, lx.pos, op)),
                Ok(None) => Err(Error::NotRecognized),
                Err(err) => Err(err),
            }
            Err(err) => Err(err),
        }
        "\\" => match lx.line_comment() {
            Ok(true) => Ok((start, lx.pos, TokenType::Comment)),
            Err(err) => Err(err),
            Ok(false) => match lx.operator() {
                Ok(Some(op)) => Ok((start, lx.pos, op)),
                Ok(None) => Err(Error::NotRecognized),
                Err(err) => Err(err),
            }
        }
        // "*" => match lx.skip("*)") => Unpaired comment closing
        "(" => match lx.block_comment() {
            Ok(true) => Ok((start, lx.pos, TokenType::Comment)),
            Ok(false) => Ok((start, lx.pos, TokenType::ParenOpen)), // Move this to ops
            Err(err) => Err(err),
        }
        _ => match lx.ident() {
            Ok(true) => {
                let end = lx.pos;
                let name = lx.substring(&start, &end);
                match KEYWORDS.binary_search_by_key(&name, |t| t.0) {
                    Ok(i) => Ok((start, end, KEYWORDS[i].1)),
                    _ => Ok((start, end, TokenType::Identifier)),
                }
            }
            Ok(false) => Err(Error::NotRecognized),
            Err(err) => Err(err),
        }
    }
}
