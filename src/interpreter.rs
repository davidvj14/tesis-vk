#![allow(unused)]
use crate::parser::*;
use crate::syntax::*;
use crate::syntax::Command::*;
use crate::syntax::InterpretingMode::*;
use crate::rendering_pipeline::MSAAPipeline;
use std::collections::HashMap;
use vulkano::pipeline::graphics::input_assembly::{
    PrimitiveTopology, PrimitiveTopology::*
};

pub struct Interpreter{
    pub parser: Parser,
    pub vertex_bindings: HashMap<String, Vertex>,
    pub vb_bindings: HashMap<String, Vec<Vertex>>,
    vbuffers_to_send: HashMap<String, (Option<PrimitiveTopology>, Vec<Vertex>)>,
    sent_buffers: Vec<String>,
    drawing_mode: PrimitiveTopology,
    interpreting_mode: InterpretingMode,
}

impl Interpreter{
    pub fn new(src: &str) -> Self {
        Interpreter{
            parser: Parser::new(src),
            vertex_bindings: HashMap::new(),
            vb_bindings: HashMap::new(),
            vbuffers_to_send: HashMap::new(),
            sent_buffers: Vec::new(),
            drawing_mode: TriangleList,
            interpreting_mode: Continuous,
        }
    }

    pub fn interpret(&mut self, pipeline: &mut MSAAPipeline){
        self.parser.parse();
        for cmd in &self.parser.cmds{
            match cmd{
                GlobalConf(draw_mode, int_mode) => {
                    self.drawing_mode = *draw_mode;
                    pipeline.change_topology(*draw_mode);
                    self.interpreting_mode = *int_mode;
                },
                DefVertex(name, v) => {
                    self.vertex_bindings.insert(name.to_string(), *v);
                },
                MkVertexBuffer(name, vb) => {
                    let mut vert_buf: Vec<Vertex> = Vec::new();
                    for v in &**vb{
                        if let (Some(vert)) = self.vertex_bindings.get(v){
                            vert_buf.push(*vert);
                        }
                    }
                    if self.sent_buffers.contains(name){
                        self.sent_buffers
                            .remove(self.sent_buffers.binary_search(name).unwrap());
                    }
                    self.vb_bindings.insert(name.to_string(), vert_buf);
                },
                Draw(mode, name) => {
                    if let Some(vb) = self.vb_bindings.get(name){
                        let mut send: Vec<(Option<PrimitiveTopology>, Vec<Vertex>)> = Vec::new();
                        if !(self.sent_buffers.contains(name)){
                            self.vbuffers_to_send.insert(name.to_string(), (*mode, vb.to_vec()));
                            self.sent_buffers.push(name.clone());
                        }
                        for (_, v) in &self.vbuffers_to_send{
                            send.push(v.clone());
                        }
                        pipeline.set_vertex_buffer(send);
                    }
                },
                _ => (),
            };
        }
    }

}
