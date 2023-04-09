#![allow(unused)]
use std::{str::{self, from_utf8}, collections::HashMap};
use nom::{
    IResult,
    sequence::{delimited, Tuple},
    branch::alt,
    character::{
        is_alphanumeric, is_space,
        complete::char, is_alphabetic,
    },
    bytes::complete::{tag, take_while, take_while1, take_while_m_n, take_until1},
    number::complete::float,
    combinator::map_res, multi::separated_list1,
};
use crate::language::types::{
    Vertex,
    Expr::{self, *},
    TvkObject::{self, *},
};

pub struct Parser<'a> {
    source: &'a str,
    pub bindings: HashMap<&'a str, TvkObject<'a>>,
    pub exprs: Vec<Expr<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new (src: &'a str) -> Self {
        Self {
            source: src,
            bindings: HashMap::new(),
            exprs: Vec::new(),
        }
    }

    fn parse_object(&self, src: &'a str) -> IResult<&'a str, TvkObject> {
        for p in [Self::parse_position(self, src),
                    Self::parse_color(self, src),
                    Self::parse_vertex(self, src)] {
            if let Ok((sr, obj)) = p {
                return Ok((sr, obj))
            }
        }
        Err(nom::Err::Incomplete(nom::Needed::Unknown))
    }

    fn consume_space(src: &str) -> &str {
        let (rest, _) =
            take_while::<_, _, ()>(Self::is_whitespace)(src.as_bytes()).unwrap();
        from_utf8(rest).unwrap()
    }

    fn is_whitespace(c: u8) -> bool {
        is_space(c) || c == '\n' as u8
    }

    fn parens<T>(src: &'a str, parser: impl Fn(&'a str) -> IResult<&'a str, T>)
        -> IResult<&'a str, T> {
            delimited(char('('), parser, char(')'))(src)
    }

    fn is_identifier_char(c : u8) -> bool {
        is_alphanumeric(c) || c == b'-' || c == b'_'
    }

    fn parse_identifier(src: &'a str) -> IResult<&'a str, &'a str> {
        let src = Self::consume_space(src);
        if let Ok((src, ident)) =
            take_while1::<_, _, ()>(Self::is_identifier_char)(src.as_bytes()){
                let src = from_utf8(src).unwrap();
                let ident = from_utf8(ident).unwrap();
                return Ok((src, ident));
        }
        Err(nom::Err::Incomplete(nom::Needed::Unknown))
    }

    fn parse_tagged_float(src: &'a str, t: &'a str) -> IResult<&'a str, f32> {
        let src = Self::consume_space(src);
        Self::parens(src, |src| {
            let (src, _) = tag(t)(src)?;
            let src = Self::consume_space(src);
            let (src, value) = float(src)?;
            Ok((src, value))
        })
    }

    fn parse_position(&self, src: &'a str) -> IResult<&'a str, TvkObject> {
        let src = Self::consume_space(src);
        Self::parens(src, |src| {
            let (src, _) = tag("position")(src)?;
            let src = Self::consume_space(src);

            if let Ok((src, ident)) = Self::parse_identifier(src) {
                return Ok((src, Position(Default::default())))
            }

            let (src, x) = Self::parse_tagged_float(src, "x")?;
            let src = Self::consume_space(src);
            let (src, y) = Self::parse_tagged_float(src, "y")?;
            let src = Self::consume_space(src);
            let (src, z) = Self::parse_tagged_float(src, "z")?;
            let src = Self::consume_space(src);
            Ok((src, Position([x, y, z])))
        })
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

    fn parse_color(&self, src: &'a str) -> IResult<&'a str, TvkObject> {
        let src = Self::consume_space(src);
        Self::parens(src, |src| {
            let src = Self::consume_space(src);
            let (src, _) = tag("color")(src)?;
            let src = Self::consume_space(src);

            if let Ok((src, ident)) = Self::parse_identifier(src){
                return Ok((src, Color(Default::default())))
            }

            let (src, _) = tag("#")(src)?;
            let (src, (red, green, blue, alpha)) =
                (Self::hex_primary,
                 Self::hex_primary,
                 Self::hex_primary,
                 Self::hex_primary).parse(src)?;
            Ok((src,
                    Color([red as f32 / 255.0,
                        green as f32 / 255.0,
                        blue as f32 / 255.0,
                        alpha as f32 / 255.0])))
                        
        })
    }

    fn parse_vertex(&self, src: &'a str) -> IResult<&'a str, TvkObject> {
        let src = Self::consume_space(src);
        Self::parens(src, |src| {
            let src = Self::consume_space(src);
            let (src, _) = tag("vertex")(src)?;
            let src = Self::consume_space(src);

            if let Ok((src, ident)) = Self::parse_identifier(src){
                return Ok((src, Vertex(Default::default())))
            }

            let (src, position) = self.parse_position(src)?;
            let src = Self::consume_space(src);
            let (src, color) = self.parse_color(src)?;
            let src = Self::consume_space(src);

            if let (Position([x, y, z]), Color([r, g, b, a])) =
                (position, color) {
                    let v = Vertex {
                        position: [x, y, z],
                        color: [r, g, b, a],
                    };
                    return Ok((src, Vertex(v)))
            }
            Err(nom::Err::Incomplete(nom::Needed::Unknown))
        })
    }

    fn parse_binding(&self, src: &'a str) -> IResult<&'a str, Expr> {
        let src = Self::consume_space(src);
        Self::parens(src, |src| {
            let src = Self::consume_space(src);
            let (src, _) = tag("def")(src)?;
            let src = Self::consume_space(src);
            let (src, ident) = Self::parse_identifier(src)?;
            let src = Self::consume_space(src);
            let (src, obj) = self.parse_object(src)?;
            Ok((src, Define(ident, obj)))
        })
    }
}
