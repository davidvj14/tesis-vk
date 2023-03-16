#![allow(unused_imports)]
#![allow(dead_code)]
use nom::{
    IResult,
    Needed,
    Err,
    sequence::delimited,
    character::complete::char,
    bytes::complete::is_not,
};
use crate::syntax::{
    Command, Vertex,
};

pub struct Parser{
    pub source: String,
    cmds: Box<[Command]>,
}

impl Parser{
    pub fn new(code: &str) -> Self{
        Parser{
            source: code.to_string(),
            cmds: Box::new([]),
        }
    }

    pub fn parens(&mut self){
        let input = &self.source.as_str();
        if &input[0..1] == "("{
            println!("{}", self.source);
        }
    }

    fn parse_vertex_def(&mut self) {
        ()
    }
}
