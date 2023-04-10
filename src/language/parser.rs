#![allow(unused)]
use std::{str::{self, from_utf8}, collections::HashMap};
use nom::{
    IResult,
    sequence::{delimited, Tuple},
    branch::alt,
    character::{
        is_alphanumeric, is_space,
        complete::{char, u32, space1}, is_alphabetic, 
    },
    bytes::complete::{tag, take_while, take_while1, take_while_m_n, take_until1},
    number::complete::float,
    combinator::map_res, multi::separated_list1, error::ErrorKind,
};
use crate::language::types::{
    Vertex,
    TvkObject::{self, *},
};

pub struct Parser<'a> {
    source: &'a str,
    pub exprs: Vec<TvkObject<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new (src: &'a str) -> Self {
        Self {
            source: src,
            exprs: Vec::new(),
        }
    }

/*    pub fn parse(&'a mut self, src: &'a str) {
        let mut src = T;
        let mut bindings: HashMap<&'a str, TvkObject<'a>> = HashMap::new();
        while src.len() > 0 {
            if let Ok((sr, Define(ident, obj))) = Self::parse_binding(&bindings, src) {
                src = sr;
                continue;
            }
            break;
        }
        println!("{:?}", bindings);
    }

    fn parse_object(bindings: &HashMap<&'a str, TvkObject<'a>>, src: &'a str) -> IResult<&'a str, TvkObject<'a>> {
        for p in [Self::parse_position(bindings, src),
                    Self::parse_color(bindings, src),
                    Self::parse_vertex(bindings, src)] {
            if let Ok((sr, obj)) = p {
                return Ok((sr, obj))
            }
        }
        Err(nom::Err::Incomplete(nom::Needed::Unknown))
    }
*/

    pub fn parse(&'a mut self) {
        self.source = T;
        while self.source.len() > 0 {
            let r = Self::parse_tvk(self.source);
            if let Ok((src, expr)) = r {
                self.source = src;
                self.exprs.push(expr);
                continue;
            }
            break;
        }
        println!("{:?}", self.exprs);
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

    fn is_atom_char(c : u8) -> bool {
        is_alphanumeric(c) || c == b'-' || c == b'_'
    }

    fn parse_atom(src: &'a str) -> IResult<&'a str, TvkObject<'a>> {
        let src = Self::consume_space(src);
        let ((src, ident)) = take_while1::<_, _, ()>(Self::is_atom_char)(src.as_bytes())
            .expect("");
        let src = from_utf8(src).unwrap();
        let ident = from_utf8(ident).unwrap();
        return Ok((src, Atom(ident)));
        
    }

    fn parse_float_literal(src: &'a str) -> IResult<&'a str, TvkObject<'a>> {
        let src = Self::consume_space(src);
        if let Ok((src, f)) = float::<&str, ()>(src) {
            return Ok((src, FloatLiteral(f)));
        }
        Err(nom::Err::Error(nom::error::Error{input: src, code: ErrorKind::TooLarge}))
    }

    fn parse_uint_literal(src: &'a str) -> IResult<&'a str, TvkObject<'a>> {
        let src = Self::consume_space(src);
        if let Ok((src, n)) = u32::<&str, ()>(src) {
            return Ok((src, UIntLiteral(n)));
        }
        Err(nom::Err::Error(nom::error::Error{input: src, code: ErrorKind::TooLarge}))
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

    fn parse_list(src: &'a str) -> IResult<&'a str, TvkObject<'a>> {
        let src = Self::consume_space(src);
        let (src, list) = separated_list1(space1, Self::parse_tvk)(src)?;
        Ok((src, List(list)))
    }

/*    fn parse_position(bindings: &HashMap<&'a str, TvkObject<'a>>, src: &'a str) -> IResult<&'a str, TvkObject<'a>> {
        let src = Self::consume_space(src);
        Self::parens(src, |src| {
            let (src, _) = tag("position")(src)?;
            let src = Self::consume_space(src);

            if let Ok((src, ident)) = Self::parse_identifier(src) {
                if let Some(Position(p)) = bindings.get(ident) {
                    return Ok((src, Position(*p)))
                }
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
*/
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
/*        Self::parens(src, |src| {
            let src = Self::consume_space(src);
            let (src, _) = tag("color")(src)?;
            let src = Self::consume_space(src);

            if let Ok((src, ident)) = Self::parse_identifier(src){
                if let Some(Color(c)) = bindings.get(ident){
                    return Ok((src, Color(*c)))
                }
            }
*/
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
                        
//        })
    }

/*    fn parse_vertex(bindings: &HashMap<&'a str, TvkObject<'a>>, src: &'a str)
        -> IResult<&'a str, Expr<'a>> {
        let src = Self::consume_space(src);
        Self::parens(src, |src| {
            let src = Self::consume_space(src);
            let (src, _) = tag("vertex")(src)?;
            let src = Self::consume_space(src);

            if let Ok((src, ident)) = Self::parse_identifier(src){
                if let Some(Vertex(v)) = bindings.get(ident){
                    return Ok((src, Vertex(*v)))
                }
            }

            let (src, position) = Self::parse_position(bindings, src)?;
            let src = Self::consume_space(src);
            let (src, color) = Self::parse_color(bindings, src)?;
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

    fn parse_binding(bindings: &HashMap<&'a str, TvkObject<'a>>, src: &'a str) -> IResult<&'a str, Expr<'a>> {
        let src = Self::consume_space(src);
        Self::parens(src, |src| {
            let src = Self::consume_space(src);
            let (src, _) = tag("def")(src)?;
            let src = Self::consume_space(src);
            let (src, ident) = Self::parse_identifier(src)?;
            let src = Self::consume_space(src);
            let (src, obj) = Self::parse_object(bindings, src)?;
            let src = Self::consume_space(src);

            Ok((src, Define(ident, Box::new(Construct(obj)))))
        })
    }
*/
}


const T: &str = r#"
(def p1 (position (x 0) (y 0) (z 0)))
(def red (color #FF0000FF))
(def v1 (vertex (position p1) (color red)))
(def v2 (vertex v1))"#;
