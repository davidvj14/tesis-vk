use crate::language::types::TvkObject::{self, *};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1, take_while_m_n},
    character::{
        complete::{char, space1, u32},
        is_alphanumeric, is_space,
    },
    combinator::map_res,
    error::ErrorKind,
    multi::{many1, separated_list1},
    number::complete::float,
    sequence::{delimited, terminated, Tuple},
    IResult,
};
use std::str::{self, from_utf8};

pub struct Parser<'a> {
    source: &'a str,
    pub exprs: Vec<TvkObject<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            source: src,
            exprs: Vec::new(),
        }
    }

    pub fn parse(&'a mut self) -> Vec<TvkObject> {
        while self.source.len() > 0 {
            let r = Self::parse_tvk(self.source);
            match r {
                Ok((src, expr)) => {
                    self.source = src;
                    self.exprs.push(expr);
                }
                Err(nom::Err::Incomplete(_)) => {
                    if self.source.len() > 0 {
                        self.source = &self.source[1..];
                        continue;
                    }
                    break;
                }
                _ => self.source = &self.source[1..],
            }
        }
        self.exprs.clone()
    }

    fn parse_tvk(src: &'a str) -> IResult<&'a str, TvkObject<'a>> {
        let src = Self::consume_space(src);
        alt((
            delimited(char('('), Self::parse_list, char(')')),
            Self::parse_color,
            Self::parse_uint_literal,
            Self::parse_float_literal,
            Self::parse_atom,
        ))(src)
    }

    fn consume_space(src: &str) -> &str {
        let (rest, _) = take_while::<_, _, ()>(Self::is_whitespace)(src.as_bytes()).unwrap();
        from_utf8(rest).unwrap()
    }

    fn is_whitespace(c: u8) -> bool {
        is_space(c) || c == '\n' as u8
    }

    fn is_atom_char(c: u8) -> bool {
        is_alphanumeric(c) || c == b'-' || c == b'_'
    }

    fn parse_atom(src: &'a str) -> IResult<&'a str, TvkObject<'a>> {
        let src = Self::consume_space(src);
        if let Ok((src, ident)) = take_while1::<_, _, ()>(Self::is_atom_char)(src.as_bytes()) {
            let src = from_utf8(src).unwrap();
            let ident = from_utf8(ident).unwrap();
            return Ok((src, Atom(ident)));
        }
        Err(nom::Err::Error(nom::error::Error {
            input: src,
            code: ErrorKind::Tag,
        }))
    }

    fn parse_float_literal(src: &'a str) -> IResult<&'a str, TvkObject<'a>> {
        let src = Self::consume_space(src);
        if let Ok((src, f)) = float::<&str, ()>(src) {
            return Ok((src, FloatLiteral(f)));
        }
        Err(nom::Err::Error(nom::error::Error {
            input: src,
            code: ErrorKind::Tag,
        }))
    }

    fn parse_uint_literal(src: &'a str) -> IResult<&'a str, TvkObject<'a>> {
        let src = Self::consume_space(src);
        if let Ok((_, _)) = terminated(u32::<&str, ()>, char('.'))(src) {
            return Err(nom::Err::Error(nom::error::Error {
                input: src,
                code: ErrorKind::Tag,
            }));
        }
        if let Ok((src, n)) = u32::<&str, ()>(src) {
            return Ok((src, UIntLiteral(n)));
        }
        Err(nom::Err::Error(nom::error::Error {
            input: src,
            code: ErrorKind::Tag,
        }))
    }

    fn my_space1(src: &'a str) -> IResult<&'a str, ()> {
        if let Ok((src, _)) = many1(alt((space1::<_, ()>, tag("\n"))))(src) {
            return Ok((src, ()));
        }
        Err(nom::Err::Error(nom::error::Error {
            input: src,
            code: ErrorKind::Tag,
        }))
    }

    fn parse_list(src: &'a str) -> IResult<&'a str, TvkObject<'a>> {
        let src = Self::consume_space(src);
        if let Ok((src, list)) = separated_list1(Self::my_space1, Self::parse_tvk)(src) {
            let src = Self::consume_space(src);
            return Ok((src, List(list)));
        }
        Err(nom::Err::Incomplete(nom::Needed::Unknown))
    }

    fn from_hex(src: &str) -> Result<u8, std::num::ParseIntError> {
        u8::from_str_radix(src, 16)
    }

    fn is_hex_digit(c: char) -> bool {
        c.is_digit(16)
    }

    fn hex_primary(src: &str) -> IResult<&str, u8> {
        map_res(take_while_m_n(2, 2, Self::is_hex_digit), Self::from_hex)(src)
    }

    fn parse_color(src: &'a str) -> IResult<&'a str, TvkObject<'a>> {
        let src = Self::consume_space(src);
        let (src, _) = tag("#")(src)?;
        if let Ok((src, (red, green, blue, alpha))) = (
            Self::hex_primary,
            Self::hex_primary,
            Self::hex_primary,
            Self::hex_primary,
        )
            .parse(src)
        {
            return Ok((
                src,
                Color([
                    red as f32 / 255.0,
                    green as f32 / 255.0,
                    blue as f32 / 255.0,
                    alpha as f32 / 255.0,
                ]),
            ));
        }
        Err(nom::Err::Error(nom::error::Error {
            input: src,
            code: ErrorKind::TooLarge,
        }))
    }
}
