use crate::model::Model;
use crate::parser::*;
use crate::syntax::*;
use crate::syntax::Command::*;
use crate::syntax::InterpretingMode::*;
use crate::syntax::ModelOperation::*;
use crate::rendering_pipeline::MSAAPipeline;
use std::collections::HashMap;
use vulkano::pipeline::graphics::input_assembly::{
    PrimitiveTopology, PrimitiveTopology::*
};

pub struct Interpreter{
    pub parser: Parser,
    pub vertex_bindings: HashMap<String, Vertex>,
    pub vb_bindings: HashMap<String, Vec<Vertex>>,
    pub model_bindings: HashMap<String, Model>,
    pub trans_bindings: HashMap<String, Transform>,
    pub camera_bindings: HashMap<String, Transform>,
    pub perspective_bindings: HashMap<String, Transform>,
    models_to_send: HashMap<String, Model>,
    sent_models: Vec<String>,
    drawing_mode: PrimitiveTopology,
    interpreting_mode: InterpretingMode,
}

impl Interpreter{
    pub fn new(src: &str) -> Self {
        Interpreter{
            parser: Parser::new(src),
            vertex_bindings: HashMap::new(),
            model_bindings: HashMap::new(),
            vb_bindings: HashMap::new(),
            trans_bindings: HashMap::new(),
            camera_bindings: HashMap::new(),
            perspective_bindings: HashMap::new(),
            models_to_send: HashMap::new(),
            sent_models: Vec::new(),
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
                        if let Some(vert) = self.vertex_bindings.get(v){
                            vert_buf.push(*vert);
                        }
                    }
                    if self.sent_models.contains(name){
                        self.sent_models
                            .remove(self.sent_models.binary_search(name).unwrap());
                    }
                    self.vb_bindings.insert(name.to_string(), vert_buf);
                },
                Draw(_mode, name) => {
                    self.models_to_send.clear();
                    self.sent_models.clear();
                    if let Some(mut m) = self.model_bindings.get_mut(name){
                        let mut send= Vec::new();
                        for (_, v) in &self.perspective_bindings {
                            m.transforms.projection = v.projection;
                        }
                        for (_, v) in &self.camera_bindings {
                            m.transforms.view = v.view;
                        }
                        if !(self.sent_models.contains(name)){
                            self.models_to_send.insert(name.to_string(), m.clone());
                            self.sent_models.push(name.clone());
                        }
                        for (_, model) in &self.models_to_send{
                            send.push(model.clone());
                        }
                        pipeline.set_vertex_buffer(send);
                    }
                },
                MkTransform(name, trans) => {
                    if self.trans_bindings.contains_key(name){
                        self.trans_bindings.remove(name);
                    }
                    self.trans_bindings.insert(name.clone(), *trans);
                },
                MkPerspective(name, perspective) => {
                    if self.perspective_bindings.contains_key(name){
                        self.perspective_bindings.remove(name);
                    }
                    self.perspective_bindings.insert(name.clone(), *perspective);
                    for (_, m) in &mut self.model_bindings{
                        m.transforms.projection = Some(perspective.projection.unwrap());
                    }
                },
                MkCamera(name, cam) => {
                    if self.camera_bindings.contains_key(name){
                        self.camera_bindings.remove(name);
                    }
                    self.camera_bindings.insert(name.clone(), *cam);
                },
                ModelOp(op) => Self::interpret_model_op(
                    &mut self.model_bindings,
                    &mut self.trans_bindings,
                    &self.vb_bindings, op),
            };
        }
    }

    fn interpret_model_op(
        model_bindings: &mut HashMap<String, Model>,
        trans_bindings: &mut HashMap<String, Transform>,
        vb_bindings: &HashMap<String, Vec<Vertex>>,
        op: &ModelOperation){
        match op {
            MkModel(name, vbuffer, topology) => {
                if model_bindings.contains_key(name){
                    model_bindings.remove(name);
                }
                if let Some(vb) = vb_bindings.get(vbuffer){
                    let model = Model::new(vb.clone(), None, *topology);
                    model_bindings.insert(name.clone(), model);
                }
            },
            ApplyTransform(trans, model) => {
                if let Some(t) = trans_bindings.get(trans){
                    if let Some(mut m) = model_bindings.get_mut(model){
                        m.transforms = *t;
                    }
                }
            }
            _ => (),
        }
    }
}
