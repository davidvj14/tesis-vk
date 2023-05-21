use std::collections::HashMap;

use crate::language::types::*;
use crate::rendering_pipeline::MSAAPipeline;

pub struct Interpreter<'a> {
    pub bindings: HashMap<&'a str, InnerType>,
}

impl<'a> Interpreter<'a> {
    pub fn eval(
        &mut self,
        exprs: &TvkObject<'a>,
        pipeline: &mut MSAAPipeline,
    ) -> Option<InnerType> {
        match exprs {
            TvkObject::List(l) => {
                if l.len() < 1 {
                    return None;
                }
                match &l[0] {
                    TvkObject::Atom("def") => {
                        if let Some(TvkObject::Atom(key)) = &l.get(1) {
                            if let Some(e) = &l.get(2) {
                                if let Some(val) = self.eval(e, pipeline) {
                                    self.bindings.insert(key, val);
                                }
                            }
                        }
                        return None;
                    },
                    TvkObject::Atom("position") => {
                        return self.eval_pos(l, pipeline);
                    },
                    TvkObject::Atom("vec3") => {
                        return self.eval_vec3(l, pipeline);
                    },
                    TvkObject::Atom("center") => {
                        return self.eval_center(l, pipeline);
                    },
                    TvkObject::Atom("up") => {
                        return self.eval_up(l, pipeline);
                    },
                    TvkObject::Atom("color") => {
                        return match &l.get(1) {
                            Some(TvkObject::Color(c)) => Some(InnerType::Color(*c)),
                            Some(TvkObject::Atom(ident)) => self.bindings.get(ident).cloned(),
                            _ => None,
                        }
                    },
                    TvkObject::Atom("vertex") => {
                        return self.eval_vertex(l, pipeline);
                    },
                    TvkObject::Atom("vertex-buffer") => {
                        return self.eval_vertex_buffer(l, pipeline);
                    },
                    TvkObject::Atom("index-buffer") => {
                        return self.eval_index_buffer(l, pipeline);
                    },
                    TvkObject::Atom("fovy") => {
                        if let Some(e) = &l.get(1) {
                            return self.eval(e, pipeline);
                        } else {
                            return None;
                        }
                    },
                    TvkObject::Atom("z-near") => {
                        if let Some(e) = &l.get(1) {
                            return self.eval(e, pipeline);
                        } else {
                            return None;
                        }
                    },
                    TvkObject::Atom("z-far") => {
                        if let Some(e) = &l.get(1) {
                            return self.eval(e, pipeline);
                        } else {
                            return None;
                        }
                    },
                    TvkObject::Atom("perspective") => {
                        return self.eval_perspective(l, pipeline);
                    },
                    TvkObject::Atom("camera") => {
                        return self.eval_camera(l, pipeline);
                    },
                    TvkObject::Atom("transform") => {
                        return self.eval_transform(l, pipeline);
                    },
                    TvkObject::Atom("translate") => {
                        if l.len() == 2 {
                            if let Some(InnerType::Vec3(vector)) = self.eval(&l[1], pipeline) {
                                return Some(InnerType::Vec3(vector));
                            }
                        } else {
                            return None;
                        }
                    },
                    TvkObject::Atom("scale") => {
                        if l.len() == 2 {
                            if let Some(InnerType::Vec3(vector)) = self.eval(&l[1], pipeline) {
                                return Some(InnerType::Vec3(vector));
                            }
                        } else {
                            return None;
                        }
                    },
                    TvkObject::Atom("rotate") => {
                        if l.len() == 2 {
                            if let Some(InnerType::Vec3(vector)) = self.eval(&l[1], pipeline) {
                                return Some(InnerType::Vec3(vector));
                            }
                        } else {
                            return None;
                        }
                    },
                    TvkObject::Atom("topology") => {
                        return self.eval_topology(l);
                    },
                    TvkObject::Atom("model") => {
                        return self.eval_model(l, pipeline);
                    },
                    TvkObject::Atom("draw") => {
                        if l.len() > 1 {
                            for drawable in &l[1..] {
                                match self.eval(drawable, pipeline) {
                                    Some(InnerType::Model(m)) => {
                                        pipeline.receive_model(m);
                                        return None;
                                    },
                                    Some(InnerType::VertexBuffer(vb)) => {
                                        pipeline.receive_vertex_buffer(vb);
                                        return None;
                                    },
                                    _ => return None,
                                }
                            }
                        } else {
                            return None;
                        }
                    },
                    TvkObject::Atom("texture") => {
                        if l.len() == 2 {
                            if let TvkObject::Atom(path) = &l[1] {
                                let (data, dims) = pipeline.load_texture_image(path);
                                return Some(InnerType::Texture((data, dims)));
                            }
                        }
                    }
                    _ => return None,
                }
                return None;
            },
            TvkObject::Atom(_) => {
                return self.eval_identifier(exprs);
            },
            TvkObject::FloatLiteral(f) => return Some(InnerType::Float(*f)),
            TvkObject::UIntLiteral(i) => return Some(InnerType::UInt(*i)),
            _ => return None,
        }
    }

    fn eval_pos(
        &mut self,
        expr: &Vec<TvkObject<'a>>,
        pipeline: &mut MSAAPipeline,
        ) -> Option<InnerType> {
        if let (
            Some(TvkObject::List(x_list)),
            Some(TvkObject::List(y_list)),
            Some(TvkObject::List(z_list)),
            ) = (&expr.get(1), &expr.get(2), &expr.get(3))
        {
            let mut vec3: [f32; 3] = [0.0, 0.0, 0.0];
            let mut i = 0;
            for list in [&x_list, &y_list, &z_list] {
                match list.get(1) {
                    Some(TvkObject::FloatLiteral(n)) => vec3[i] = *n,
                    Some(TvkObject::UIntLiteral(n)) => vec3[i] = *n as f32,
                    _ => return None,
                }
                i += 1;
            }
            return Some(InnerType::Position(vec3));
        } else {
            if let Some(e) = &expr.get(1) {
                return self.eval(&e, pipeline);
            } else {
                return None;
            }
        }
    }
    
    fn eval_vec3(
        &mut self,
        expr: &Vec<TvkObject<'a>>,
        pipeline: &mut MSAAPipeline
        ) -> Option<InnerType> {
        if let Some(TvkObject::List(list)) = &expr.get(1) {
            let mut vec3 = [0.0, 0.0, 0.0];
            for i in 0..=2 {
                    if let Some(x) = &list.get(i) {
                    match self.eval(&x, pipeline) {
                        Some(InnerType::Float(n)) => vec3[i] = n,
                        Some(InnerType::UInt(n)) => vec3[i] = n as f32,
                        _ => return None,
                    }
                }
            }
            return Some(InnerType::Vec3(vec3));
        } else {
            if let Some(e) = &expr.get(1) {
                return self.eval(&e, pipeline);
            } else {
                return None;
            }
        }
    }

    fn eval_center(
        &mut self,
        expr: &Vec<TvkObject<'a>>,
        pipeline: &mut MSAAPipeline
        ) -> Option<InnerType> {
        if let (
            Some(TvkObject::List(x_list)),
            Some(TvkObject::List(y_list)),
            Some(TvkObject::List(z_list)),
            ) = (&expr.get(1), &expr.get(2), &expr.get(3))
        {
            let mut vec3: [f32; 3] = [0.0, 0.0, 0.0];
            let mut i = 0;
            for list in [&x_list, &y_list, &z_list] {
                match list.get(1) {
                    Some(TvkObject::FloatLiteral(n)) => vec3[i] = *n,
                    Some(TvkObject::UIntLiteral(n)) => vec3[i] = *n as f32,
                    _ => return None,
                }
                i += 1;
            }
            return Some(InnerType::Position(vec3));
        } else {
            if let Some(e) = &expr.get(1) {
                return self.eval(&e, pipeline);
            } else {
                return None;
            }
        }
    }

    fn eval_up(
        &mut self,
        expr: &Vec<TvkObject<'a>>,
        pipeline: &mut MSAAPipeline
        ) -> Option<InnerType> {
        if let (
            Some(TvkObject::List(x_list)),
            Some(TvkObject::List(y_list)),
            Some(TvkObject::List(z_list)),
            ) = (&expr.get(1), &expr.get(2), &expr.get(3))
        {
            let mut vec3: [f32; 3] = [0.0, 0.0, 0.0];
            let mut i = 0;
            for list in [&x_list, &y_list, &z_list] {
                match list.get(1) {
                    Some(TvkObject::FloatLiteral(n)) => vec3[i] = *n,
                    Some(TvkObject::UIntLiteral(n)) => vec3[i] = *n as f32,
                    _ => return None,
                }
                i += 1;
            }
            return Some(InnerType::Position(vec3));
        } else {
            if let Some(e) = &expr.get(1) {
                return self.eval(&e, pipeline);
            } else {
                return None;
            }
        }
    }

    fn eval_vertex(
        &mut self,
        expr: &Vec<TvkObject<'a>>,
        pipeline: &mut MSAAPipeline
        ) -> Option<InnerType> {
        match &expr.get(1) {
            Some(e1) => {
                match &expr.get(2) {
                    Some(e2) => {
                        if let Some(InnerType::Position(position)) = self.eval(&e1, pipeline) {
                            return match self.eval(&e2, pipeline) {
                                Some(InnerType::Color(color)) =>
                                    Some(InnerType::Vertex(Vertex { position, color})),
                                Some(InnerType::Position(uv)) => {
                                    Some(InnerType::TextureVertex(
                                            TextureVertex { position , uv: [uv[0], uv[1]]}))
                                },
                                _ => None,
                            };
                        }
                        return None;
                    },
                    _ => return None,
                }
            },
            _ => return None,
        }
    }

    fn eval_vertex_buffer(
        &mut self,
        expr: &Vec<TvkObject<'a>>,
        pipeline: &mut MSAAPipeline
        ) -> Option<InnerType> {
        if let Some(TvkObject::List(vertices)) = &expr.get(1) {
            let mut vb = Vec::new();
            let mut tvb = Vec::new();
            for vertex in vertices {
                match self.eval(vertex, pipeline) {
                    Some(InnerType::Vertex(v)) => vb.push(v),
                    Some(InnerType::TextureVertex(v)) => tvb.push(v),
                    _ => return None,
                }
            }
            if !vb.is_empty() && tvb.is_empty() {
                return Some(InnerType::VertexBuffer(vb));
            } else if !tvb.is_empty() && vb.is_empty() {
                return Some(InnerType::TexVertexBuffer(tvb));
            } else {
                return None;
            }
        }
        return None;
    }

    fn eval_index_buffer(
        &mut self,
        expr: &Vec<TvkObject<'a>>,
        pipeline: &mut MSAAPipeline,
        ) -> Option<InnerType> {
        if let Some(TvkObject::List(indices)) = &expr.get(1) {
            let mut ib = Vec::new();
            for index in indices {
                if let Some(InnerType::UInt(i)) = self.eval(index, pipeline) {
                    ib.push(i);
                } else {
                    return None;
                }
            }
            return Some(InnerType::IndexBuffer(ib));
        }
        return None;
    }

    fn eval_perspective(
        &mut self,
        expr: &Vec<TvkObject<'a>>,
        pipeline: &mut MSAAPipeline
        ) -> Option<InnerType> {
        if expr.len() == 4 {
            if let Some(InnerType::Float(fovy)) = self.eval(&expr[1], pipeline) {
                if let Some(InnerType::Float(z_near)) = self.eval(&expr[2], pipeline) {
                    if let Some(InnerType::Float(z_far)) = self.eval(&expr[3], pipeline) {
                        return Some(InnerType::Perspective([fovy, z_near, z_far]));
                    }
                }
            }
        } else if expr.len() == 2 {
            return self.eval(&expr[1], pipeline);
        }
        return None;
    }

    fn eval_camera(
        &mut self,
        expr: &Vec<TvkObject<'a>>,
        pipeline: &mut MSAAPipeline
        ) -> Option<InnerType> {
        if expr.len() == 5 {
            if let Some(InnerType::Position(position)) = self.eval(&expr[1], pipeline) {
                if let Some(InnerType::Position(center)) = self.eval(&expr[2], pipeline) {
                    if let Some(InnerType::Position(up)) = self.eval(&expr[3], pipeline) {
                        if let Some(InnerType::Perspective(perspective)) =
                            self.eval(&expr[4], pipeline)
                        {
                            return Some(InnerType::Camera(Camera {
                                position,
                                center,
                                up,
                                perspective,
                            }));
                        }
                    }
                }
            }
        } else if expr.len() == 2 {
            if let Some(InnerType::Camera(c)) = self.eval(&expr[1], pipeline){
                return Some(InnerType::Camera(c));
            }
        }
        return None;
    }

    fn eval_transform(
        &mut self,
        expr: &Vec<TvkObject<'a>>,
        pipeline: &mut MSAAPipeline
        ) -> Option<InnerType> {
        if expr.len() == 4 {
            if let Some(InnerType::Vec3(translate)) = self.eval(&expr[1], pipeline) {
                if let Some(InnerType::Vec3(scale)) = self.eval(&expr[2], pipeline) {
                    if let Some(InnerType::Rotate(rotate)) = self.eval(&expr[3], pipeline) {
                        return Some(InnerType::Transform(Transform {
                            translate,
                            scale,
                            rotate,
                        }));
                    }
                }
            } else if let TvkObject::Atom("default") = &expr[1] {
                return Some(InnerType::Transform(Transform::default()));
            }
            return self.eval(&expr[1], pipeline);
        } else if expr.len() == 2 {
            match &expr[1] {
                TvkObject::Atom("default") =>
                    return Some(InnerType::Transform(Transform::default())),
                TvkObject::Atom(ident) => {
                    if let Some(InnerType::Transform(t)) = self.bindings.get(ident) {
                        return Some(InnerType::Transform(t.clone()));
                    }
                },
                _ => return None,
            }
        }
        return None;
    }
    
    fn eval_topology(
        &mut self,
        expr: &Vec<TvkObject<'a>>,
        ) -> Option<InnerType> {
        if expr.len() == 2 {
            return match &expr[1] {
                TvkObject::Atom("default") => {
                    Some(InnerType::Topology("RESERVED_TRIANGLE_LIST".to_string()))
                }
                TvkObject::Atom("triangle-list") => {
                    Some(InnerType::Topology("RESERVED_TRIANGLE_LIST".to_string()))
                }
                TvkObject::Atom("triangle-strip") => {
                    Some(InnerType::Topology("RESERVED_TRIANGLE_STRIP".to_string()))
                }
                TvkObject::Atom("line-list") => {
                    Some(InnerType::Topology("RESERVED_LINE_LIST".to_string()))
                }
                TvkObject::Atom("line-strip") => {
                    Some(InnerType::Topology("RESERVED_LINE_STRIP".to_string()))
                }
                TvkObject::Atom("point-list") => {
                    Some(InnerType::Topology("RESERVED_POINT_LIST".to_string()))
                }
                _ => None,
            };
        } else {
            return None;
        }
    }

    fn eval_model(
        &mut self,
        expr: &Vec<TvkObject<'a>>,
        pipeline: &mut MSAAPipeline
        ) -> Option<InnerType> {
        if expr.len() == 7 {
            if let Some(InnerType::TexVertexBuffer(vertices)) = self.eval(&expr[1], pipeline) {
                if let Some(InnerType::IndexBuffer(indices)) = self.eval(&expr[2], pipeline) {
                    if let Some(InnerType::Topology(topology)) = self.eval(&expr[3], pipeline) {
                        if let Some(InnerType::Transform(transforms)) =
                            self.eval(&expr[4], pipeline) {
                                if let Some(InnerType::Camera(camera)) =
                                    self.eval(&expr[5], pipeline) {
                                        if let Some(InnerType::Texture(texture_data)) =
                                            self.eval(&expr[6], pipeline) {
                                                return Some(InnerType::Model(Model {
                                                    vertices,
                                                    indices,
                                                    topology: "RESERVED_TRIANGLE_LIST_TEX".to_string(),
                                                    transforms,
                                                    camera,
                                                    texture_data,
                                                    texture: None,
                                                }))
                                            };
                                    }
                            }
                    }
                }
            }
        }
        return None;
    }

    fn eval_identifier(
        &self,
        expr: &TvkObject<'a>) -> Option<InnerType> {
        if let TvkObject::Atom(identifier) = expr {
            return self.bindings.get(identifier).cloned();
        } else {
            return None;
        }
    }
}
