use std::str::{self, from_utf8};
use nom::{
    IResult,
    sequence::{delimited, Tuple},
    branch::alt,
    character::{is_alphanumeric, is_space},
    bytes::complete::{tag, take_while, take_while1, take_while_m_n, take_until1},
    number::complete::float,
    combinator::map_res, multi::separated_list1,
};
use crate::syntax::{
    Command, Command::*, Vertex,
    InterpretingMode, InterpretingMode::*,
};
use vulkano::pipeline::graphics::input_assembly::{
    PrimitiveTopology, PrimitiveTopology::*
};

pub struct Parser{
    pub source: String,
    pub cmds: Vec<Command>,
}

//TODO
//Generate error messages

impl Parser{
    pub fn new(code: &str) -> Self{
        Parser{
            source: code.to_string(),
            cmds: Vec::new(),
        }
    }

    //Parse s-exprs in real-time, safely failing when facing incomplete/invalid input
    pub fn parse(&mut self){
        while self.source.len() > 0 {
            if let Ok((input, cmd)) = Parser::parse_cmd(self.source.as_str()){
                self.source = input.to_string();
                self.cmds.push(cmd);
                continue;
            }
            break;
        }
    }

    //Parser combinator of s-exprs
    pub fn parse_cmd(code: &str) -> IResult<&str, Command> {
        alt((
                Parser::parse_config,
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

    //Remove leading whitespace from input
    fn space(input: &str) -> &str {
        let (res, _) = take_while::<_, _, ()>(Parser::is_whitespace)(input.as_bytes()).unwrap();
        from_utf8(res).unwrap()
    }

    fn is_whitespace(c: u8) -> bool {
        is_space(c) || c == '\n' as u8
    }

    //Parse an f32 value preceded by a dimension (x, y or z)
    pub fn parse_pos_dim_value(src: &str, dim: String) -> IResult<&str, f32> {
        Self::parens(src, |src| {
            let (input, _) = tag(dim.as_str())(src)?;
            let input = Self::space(input);
            let (input, value) = float(input)?;
            Ok((input, value))
        })
    }

    //Parse the x, y, z position of a vertex
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
    
    //The next three functions are used to convert a 2-digit hex value to a u8 value.
    
    fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
        u8::from_str_radix(input, 16)
    }

    fn is_hex_digit(c: char) -> bool {
        c.is_digit(16)
    }

    fn hex_primary(input: &str) -> IResult<&str, u8> {
        map_res(take_while_m_n(2, 2, Self::is_hex_digit), Self::from_hex)(input)
    } 

    //Parse the RGBA color of a vertex and normalizes it.
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
            if let Ok((input, name)) = take_while1::<_, _, ()>(is_alphanumeric)(input.as_bytes()){
                let input = from_utf8(input).unwrap();
                let name = from_utf8(name).unwrap();
                let input = Parser::space(input);
                let (input, position) = Self::parse_pos(input)?;
                let input = Parser::space(input);
                let (input, color) = Parser::parse_color(input)?;
                let vertex = Vertex { position, color };
                return Ok((input, DefVertex(name.to_string(), vertex)));
            }
            Err(nom::Err::Incomplete(nom::Needed::Unknown))
        })
    }

    //Required for the separated_list1 in the next function
    fn space_consumer(src: &[u8]) -> Result<(&[u8], ()), nom::Err<()>> {
        let s = Parser::space(from_utf8(src).unwrap());
        Ok((s.as_bytes(), ()))
    }

    //Parse a list of vertex names and return the vector containing the names
    fn parse_vertices_list(src: &str) -> IResult<&str, Vec<String>> {
        Parser::parens(src, |src|{
            let input = Parser::space(src);
            let (rest, input) = take_until1::<_, _, ()>(")")(input).unwrap();
            let input = input.to_string() + " ";
            if let Ok((_, verts)) = separated_list1(
                Parser::space_consumer,
                take_while1::<_, _, ()>(is_alphanumeric)
                )(&input.as_bytes()){
                let mut vec_verts : Vec<String> = Vec::new();
                for vert in verts{
                    vec_verts.push(from_utf8(vert).unwrap().to_string());
                }
                return Ok((rest, vec_verts));
            }
            Err(nom::Err::Incomplete(nom::Needed::Unknown))
        })
    }

    //Parse the creation of a vertex buffer
    pub fn parse_mk_vb(src: &str) -> IResult<&str, Command> {
        let input = Parser::space(src);
        Parser::parens(input, |src| {
            let input = Parser::space(src);
            let (input, _) = tag("mk-vertex-buffer")(input)?;
            let input = Parser::space(input);
            if let Ok((input, name)) = take_while1::<_, _, ()>(is_alphanumeric)(input.as_bytes()){
                let input = Parser::space(from_utf8(input).unwrap());
                let name = from_utf8(name).unwrap();
                if let Ok((input, verts)) = Parser::parse_vertices_list(input){
                    let vb : Box<Vec<String>> = Box::new(verts);
                    return Ok((input,
                            MkVertexBuffer(name.to_string(), vb)));
                }
            }
            Err(nom::Err::Incomplete(nom::Needed::Unknown))
        })
    }

    //Parse a draw command
    fn parse_draw(src: &str) -> IResult<&str, Command> {
        let input = Parser::space(src);
        Parser::parens(input, |src| {
            let input = Parser::space(src);
            let (input, _) = tag("draw")(input)?;
            let input = Parser::space(input);
            let mode = None;
            if let Ok((input, dmode)) = Parser::parse_drawing_mode(input){
                let mode = Some(dmode);
                let input = Parser::space(input);
                if let Ok((input, name)) =
                    take_while1::<_, _, ()>(is_alphanumeric)(input.as_bytes()){
                        let input = Parser::space(from_utf8(input).unwrap());
                        let name = from_utf8(name).unwrap().to_string();
                        return Ok((input, Draw(mode, name)));
                }
            }
            if let Ok((input, name)) =
                take_while1::<_, _, ()>(is_alphanumeric)(input.as_bytes()){
                    let input = Parser::space(from_utf8(input).unwrap());
                    let name = from_utf8(name).unwrap().to_string();
                    return Ok((input, Draw(mode, name)));
            }
            Err(nom::Err::Incomplete(nom::Needed::Unknown))
        })
    }

    fn parse_config(src: &str) -> IResult<&str, Command>{
        let input = Parser::space(src);
        Parser::parens(input, |src| {
            let input = Parser::space(src);
            let (input, _) = tag("config")(input)?;
            let input = Parser::space(input);
            let (input, drawing_mode) = Parser::parse_drawing_mode(input)?;
            let input = Parser::space(input);
            let (input, interpreting_mode) = Parser::parse_interpreting_mode(input)?;
            return Ok((input,
                    GlobalConf(drawing_mode, interpreting_mode)))
        })
    }

    fn parse_drawing_mode(src: &str) -> IResult<&str, PrimitiveTopology> {
        let input = Parser::space(src);
        Parser::parens(input, |src| {
            let input = Parser::space(src);
            let (input, _) = tag("primitive")(input)?;
            let input = Parser::space(input);
            if let Ok((input, name)) =
                take_while1::<_, _, ()>(Parser::is_identifier_char)(input.as_bytes()){
                    match from_utf8(name).unwrap() {
                        "point" => 
                            return Ok((from_utf8(input).unwrap(), PointList)),
                        "line-list" => 
                            return Ok((from_utf8(input).unwrap(), LineList)),
                        "line-strip" => 
                            return Ok((from_utf8(input).unwrap(), LineStrip)),
                        "triangle-list" => 
                            return Ok((from_utf8(input).unwrap(), TriangleList)),
                        "triangle-strip" => 
                            return Ok((from_utf8(input).unwrap(), TriangleStrip)),
                        _ => return Err(nom::Err::Incomplete(nom::Needed::Unknown))
                    }
            }
            Err(nom::Err::Incomplete(nom::Needed::Unknown))
        })
    }

    fn is_identifier_char(c: u8) -> bool {
        is_alphanumeric(c) || c == b'-'
    }

    fn parse_interpreting_mode(src: &str) -> IResult<&str, InterpretingMode> {
        let input = Parser::space(src);
        Parser::parens(input, |src| {
            let input = Parser::space(src);
            let (input, _) = tag("interpreting-mode")(input)?;
            let input = Parser::space(input);
            if let Ok((input, name)) =
                take_while1::<_, _, ()>(Parser::is_identifier_char)(input.as_bytes()){
                    match from_utf8(name).unwrap() {
                        "continuous" => 
                            return Ok((from_utf8(input).unwrap(), Continuous)),
                        "manual" => 
                            return Ok((from_utf8(input).unwrap(), Manual)),
                        _ => return Err(nom::Err::Incomplete(nom::Needed::Unknown))
                    }
            }
            Err(nom::Err::Incomplete(nom::Needed::Unknown))
        })

    }
}
