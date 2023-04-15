#![allow(unreachable_code)]
use std::collections::HashMap;

use crate::language::types::*;
use crate::rendering_pipeline::MSAAPipeline;
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;

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
                        if let (
                            Some(TvkObject::List(x_list)),
                            Some(TvkObject::List(y_list)),
                            Some(TvkObject::List(z_list)),
                            ) = (&l.get(1), &l.get(2), &l.get(3))
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
                        if let Some(e) = &l.get(1) {
                            return self.eval(&e, pipeline);
                        } else {
                            return None;
                        }
                    }
                    },
                    TvkObject::Atom("vec3") => {
                        if let Some(TvkObject::List(list)) = &l.get(1) {
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
                            if let Some(e) = &l.get(1) {
                                return self.eval(&e, pipeline);
                            } else {
                                return None;
                            }
                        }
                    },
                    TvkObject::Atom("center") => {
                        if let (
                            Some(TvkObject::List(x_list)),
                            Some(TvkObject::List(y_list)),
                            Some(TvkObject::List(z_list)),
                            ) = (&l.get(1), &l.get(2), &l.get(3))
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
                        if let Some(e) = &l.get(1) {
                            return self.eval(&e, pipeline);
                        } else {
                            return None;
                        }
                    }
                    },
                    TvkObject::Atom("up") => {
                        if let (
                            Some(TvkObject::List(x_list)),
                            Some(TvkObject::List(y_list)),
                            Some(TvkObject::List(z_list)),
                            ) = (&l.get(1), &l.get(2), &l.get(3))
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
                        if let Some(e) = &l.get(1) {
                            return self.eval(&e, pipeline);
                        } else {
                            return None;
                        }
                    }
                    },
                    TvkObject::Atom("color") => {
                        return match &l.get(1) {
                            Some(TvkObject::Color(c)) => Some(InnerType::Color(*c)),
                            Some(TvkObject::Atom(ident)) => self.bindings.get(ident).cloned(),
                            _ => None,
                        }
                    },
                    TvkObject::Atom("vertex") => {
                        match &l.get(1) {
                            Some(e1) => {
                                match &l.get(2) {
                                    Some(e2) => {
                                        if let Some(InnerType::Position(position)) = self.eval(&e1, pipeline) {
                                            if let Some(InnerType::Color(color)) = self.eval(&e2, pipeline) {
                                                return Some(InnerType::Vertex(Vertex { position, color }));
                                            }
                                        }
                                        return None;
                                    },
                                    _ => return None,
                                }
                            },
                            _ => return None,
                        }
                    },
                    TvkObject::Atom("vertex-buffer") => {
                        if let Some(TvkObject::List(vertices)) = &l.get(1) {
                            let mut vb = Vec::new();
                            for vertex in vertices {
                                if let Some(InnerType::Vertex(v)) = self.eval(vertex, pipeline) {
                                    vb.push(v);
                                } else {
                                    return None;
                                }
                            }
                            return Some(InnerType::VertexBuffer(vb));
                        }
                        return None;
                    },
                    TvkObject::Atom("index-buffer") => {
                        if let Some(TvkObject::List(indices)) = &l.get(1) {
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
                        if l.len() == 4 {
                            if let Some(InnerType::Float(fovy)) = self.eval(&l[1], pipeline) {
                                if let Some(InnerType::Float(z_near)) = self.eval(&l[2], pipeline) {
                                    if let Some(InnerType::Float(z_far)) = self.eval(&l[3], pipeline) {
                                        return Some(InnerType::Perspective([fovy, z_near, z_far]));
                                    }
                                }
                            }
                        } else if l.len() == 2 {
                            return self.eval(&l[1], pipeline);
                        }
                        return None;
                    },
                    TvkObject::Atom("camera") => {
                        if l.len() == 5 {
                            if let Some(InnerType::Position(position)) = self.eval(&l[1], pipeline) {
                                if let Some(InnerType::Position(center)) = self.eval(&l[2], pipeline) {
                                    if let Some(InnerType::Position(up)) = self.eval(&l[3], pipeline) {
                                        if let Some(InnerType::Perspective(perspective)) =
                                            self.eval(&l[4], pipeline)
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
                        } else if l.len() == 2 {
                            if let Some(InnerType::Camera(c)) = self.eval(&l[1], pipeline){
                                return Some(InnerType::Camera(c));
                            }
                        } else {
                            return None;
                        }
                    },
                    TvkObject::Atom("transform") => {
                        if l.len() == 4 {
                            if let Some(InnerType::Vec3(translate)) = self.eval(&l[1], pipeline) {
                                if let Some(InnerType::Vec3(scale)) = self.eval(&l[2], pipeline) {
                                    if let Some(InnerType::Rotate(rotate)) = self.eval(&l[3], pipeline) {
                                        return Some(InnerType::Transform(Transform {
                                            translate,
                                            scale,
                                            rotate,
                                        }));
                                    }
                                }
                            } else if let TvkObject::Atom("default") = &l[1] {
                                return Some(InnerType::Transform(Transform::default()));
                            }
                            return self.eval(&l[1], pipeline);
                        } else if l.len() == 2 {
                            match &l[1] {
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
                        if l.len() == 2 {
                            return match &l[1] {
                                TvkObject::Atom("default") => {
                                    Some(InnerType::Topology(PrimitiveTopology::TriangleList))
                                }
                                TvkObject::Atom("triangle-list") => {
                                    Some(InnerType::Topology(PrimitiveTopology::TriangleList))
                                }
                                TvkObject::Atom("triangle-strip") => {
                                    Some(InnerType::Topology(PrimitiveTopology::TriangleStrip))
                                }
                                TvkObject::Atom("line-list") => {
                                    Some(InnerType::Topology(PrimitiveTopology::LineList))
                                }
                                TvkObject::Atom("line-strip") => {
                                    Some(InnerType::Topology(PrimitiveTopology::LineStrip))
                                }
                                TvkObject::Atom("point-list") => {
                                    Some(InnerType::Topology(PrimitiveTopology::PointList))
                                }
                                _ => None,
                            };
                        } else {
                            return None;
                        }
                    },
                    TvkObject::Atom("model") => {
                        if l.len() == 6 {
                            if let Some(InnerType::VertexBuffer(vertices)) = self.eval(&l[1], pipeline) {
                                if let Some(InnerType::IndexBuffer(indices)) = self.eval(&l[2], pipeline) {
                                    if let Some(InnerType::Topology(topology)) = self.eval(&l[3], pipeline)
                                    {
                                        if let Some(InnerType::Transform(transforms)) =
                                            self.eval(&l[4], pipeline)
                                        {
                                            if let Some(InnerType::Camera(camera)) =
                                                self.eval(&l[5], pipeline)
                                            {
                                                return Some(InnerType::Model(Model {
                                                    vertices,
                                                    indices,
                                                    topology,
                                                    transforms,
                                                    camera,
                                                }));
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            return None;
                        }
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
                    }
                    _ => return None,
                }
                return None;
            },
            TvkObject::Atom(identifier) => {
                return self.bindings.get(identifier).cloned();
            },
            TvkObject::FloatLiteral(f) => return Some(InnerType::Float(*f)),
            TvkObject::UIntLiteral(i) => return Some(InnerType::UInt(*i)),
            _ => return None,
        }
    }
}
