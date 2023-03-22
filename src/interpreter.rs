#![allow(unused)]
use crate::parser::*;
use crate::syntax::*;
use crate::syntax::Command::*;
use std::collections::HashMap;
use crate::rendering_pipeline::MSAAPipeline;

pub struct Interpreter<'a>{
    pub parsed_commands: Vec<Command>,
    pub vertex_bindings: HashMap<String, Vertex>,
    pub vb_bindings: HashMap<String, Vec<Vertex>>,
    pub pipeline: &'a mut MSAAPipeline,
}

impl<'a> Interpreter<'a>{
    pub fn new(cmds: &'a Vec <Command>, pipeline: &'a mut MSAAPipeline) -> Self{
        Interpreter{
            parsed_commands: cmds.to_vec(),
            vertex_bindings: HashMap::new(),
            vb_bindings: HashMap::new(),
            pipeline
        }
    }

    pub fn interpret(&mut self){
        for cmd in &self.parsed_commands{
            match cmd{
                DefVertex(name, v) => {
                    self.vertex_bindings.insert(name.to_string(), *v);
                },
                MkVertexBuffer(name, vb) => {
                    let mut vert_buf: Vec<Vertex> = Vec::new();
                    for v in &**vb{
                        vert_buf.push(*self.vertex_bindings.get(v).unwrap());
                    }
                    self.vb_bindings.insert(name.to_string(), vert_buf);
                },
                Draw(name) => {
                    let vb: Vec<Vertex> = self.vb_bindings.get(name).unwrap().clone();
                    self.pipeline.set_vertex_buffer(vb);
                },
                _ => (),
            };
        }
    }

}
