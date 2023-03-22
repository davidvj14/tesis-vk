#![allow(unused)]
use std::str::{self, from_utf8};
use nom::{
    IResult,
    sequence::{delimited, Tuple},
    branch::alt,
    character::{is_alphanumeric, is_space},
    bytes::complete::{tag, take_while, take_while1, take_while_m_n, take_until1},
    number::complete::float,
    combinator::{map_res, opt}, multi::{many1, separated_list1},
};
use crate::syntax::{
    Command, Command::*, Vertex, Position, Color, VertexBuffer,
};

pub struct Parser{
    pub source: String,
    pub cmds: Vec<Command>,
}

impl Parser{
    pub fn new(code: &str) -> Self{
        Parser{
            source: code.to_string(),
            cmds: Vec::new(),
        }
    }

    pub fn parse(&mut self){
        while(self.source.len() > 0){
            if let Ok((input, cmd)) = Parser::parse_cmd(self.source.as_str()){
                self.source = input.to_string();
                self.cmds.push(cmd);
                continue;
            }
            break;
        }
    }

    pub fn parse_cmd(code: &str) -> IResult<&str, Command> {
        alt((
                Parser::parse_vertex_def,
                Parser::parse_mk_vb,
                Parser::parse_draw,
                ))(code)
    }

    pub fn parens<T>(src: &str,
        parser: impl Fn(&str) -> IResult<&str, T>)
        -> IResult<&str, T> {
            delimited(tag("("), parser, tag(")"))(src)
    }

    fn space(input: &str) -> &str {
        let (res, _) = take_while::<_, _, ()>(Parser::is_whitespace)(input.as_bytes()).unwrap();
        from_utf8(res).unwrap()
    }

    fn is_whitespace(c: u8) -> bool {
        is_space(c) || c == '\n' as u8
    }

    pub fn parse_pos_dim_value(src: &str, dim: String) -> IResult<&str, f32> {
        Self::parens(src, |src| {
            let (input, _) = tag(dim.as_str())(src)?;
            let input = Self::space(input);
            let (input, value) = float(input)?;
            Ok((input, value))
        })
    }

    pub fn parse_pos(src: &str) -> IResult<&str, [f32; 3]> {
        Self::parens(src, |src| {
            let (src, _) = tag("pos")(src)?;
            let src = Self::space(src);
            let (input, x) = Self::parse_pos_dim_value(src, "x".to_string())?;
            let input = Self::space(input);
            let (input, y) = Self::parse_pos_dim_value(input, "y".to_string())?;
            let input = Self::space(input);
            let (input, z) = Self::parse_pos_dim_value(input, "z".to_string())?;
            let input = Self::space(input);
            Ok((input, [x, y, z]))
        })
    }
    
    fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
        u8::from_str_radix(input, 16)
    }

    fn is_hex_digit(c: char) -> bool {
        c.is_digit(16)
    }

    fn hex_primary(input: &str) -> IResult<&str, u8> {
        map_res(take_while_m_n(2, 2, Self::is_hex_digit), Self::from_hex)(input)
    } 

    pub fn parse_color(src: &str) -> IResult<&str, [f32; 4]> {
        Parser::parens(src, |src| {
        let input = Self::space(src);
        let (input, _) = tag("color #")(input)?;
        let (input, (red, green, blue, alpha)) =
            (Self::hex_primary,
             Self::hex_primary,
             Self::hex_primary,
             Self::hex_primary).parse(input)?;
            Ok((input, [
                    red as f32 / 255.0,
                    green as f32 / 255.0,
                    blue as f32 / 255.0,
                    alpha as f32 / 255.0,]))
        })
    }

    pub fn parse_vertex_def(src: &str) -> IResult<&str, Command> {
        let input = Parser::space(src);
        Parser::parens(input, |src| {
            let input = Parser::space(src);
            let (input, _) = tag("def-vertex")(input)?;
            let input = Parser::space(input);
            let (input, name) =
                take_while1::<_, _, ()>(is_alphanumeric)(input.as_bytes())
                .expect("");
            let input = from_utf8(input).unwrap();
            let name = from_utf8(name).unwrap();
            let input = Parser::space(input);
            let (input, position) = Self::parse_pos(input)?;
            let input = Parser::space(input);
            let (input, color) = Parser::parse_color(input)?;
            let vertex = Vertex { position, color };
            Ok((input, DefVertex(name.to_string(), vertex)))
        })
    }

    fn space_consumer(src: &[u8]) -> Result<(&[u8], ()), nom::Err<()>> {
        let s = Parser::space(from_utf8(src).unwrap());
        Ok((s.as_bytes(), ()))
    }

    fn parse_vertices_list(src: &str) -> IResult<&str, Vec<String>> {
        Parser::parens(src, |src|{
            let input = Parser::space(src);
            let (rest, input) = take_until1::<_, _, ()>(")")(input).unwrap();
            let input = input.to_string() + " ";
            let (_, verts) = 
                separated_list1(
                    Parser::space_consumer,
                    take_while1::<_, _, ()>(is_alphanumeric)
                    )(&input.as_bytes()).unwrap();
            let mut vec_verts : Vec<String> = Vec::new();
            for vert in verts{
                vec_verts.push(from_utf8(vert).unwrap().to_string());
            }
            Ok((rest, vec_verts))
        })
    }

    pub fn parse_mk_vb(src: &str) -> IResult<&str, Command> {
        let input = Parser::space(src);
        Parser::parens(input, |src| {
            let input = Parser::space(src);
            let (input, _) = tag("mk-vertex-buffer")(input)?;
            let input = Parser::space(input);
            let (input, name) =
                take_while1::<_, _, ()>(is_alphanumeric)(input.as_bytes())
                .expect("");
            let input = Parser::space(from_utf8(input).unwrap());
            let name = from_utf8(name).unwrap();
            let (input, verts) = Parser::parse_vertices_list(input).unwrap();
            let vb : Box<Vec<String>> = Box::new(verts);
            Ok((input,
                    MkVertexBuffer(name.to_string(), vb)))
        })
    }

    fn parse_draw(src: &str) -> IResult<&str, Command> {
        let input = Parser::space(src);
        Parser::parens(input, |src| {
            let input = Parser::space(src);
            let (input, _) = tag("draw")(input)?;
            let input = Parser::space(input);
            let (input, name) =
                take_while1::<_, _, ()>(is_alphanumeric)(input.as_bytes())
                .expect("");
            let input = Parser::space(from_utf8(input).unwrap());
            let name = from_utf8(name).unwrap().to_string();
            Ok((input, Draw(name)))
        })
    }
}
