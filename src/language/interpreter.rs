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
                if l.len() < 2 {
                    return None;
                }
                if let TvkObject::Atom("def") = &l[0] {
                    if let Some(TvkObject::Atom(key)) = &l.get(1) {
                        if let Some(e) = &l.get(2) {
                            if let Some(val) = self.eval(e, pipeline) {
                                self.bindings.insert(key, val);
                            }
                        }
                    }
                    return None;
                }
                if let TvkObject::Atom("position") = &l[0] {
                    if let TvkObject::Atom(ident) = &l[1] {
                        if let Some(InnerType::Position(p)) = self.bindings.get(ident) {
                            return Some(InnerType::Position(*p));
                        }
                    } else if let (
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
                    }
                }
                if let TvkObject::Atom("vec3") = &l[0] {
                    if let Some(TvkObject::List(list)) = &l.get(1) {
                        println!("\nvec3 list {:?}\n", list);
                        if let (
                            Some(TvkObject::FloatLiteral(x)),
                            Some(TvkObject::FloatLiteral(y)),
                            Some(TvkObject::FloatLiteral(z)),
                        ) = (&list.get(0), &list.get(1), &list.get(2))
                        {
                            return Some(InnerType::Vec3([*x, *y, *z]));
                        }
                    }
                }
                if let TvkObject::Atom("center") = &l[0] {
                    if let TvkObject::Atom(ident) = &l[1] {
                        if let Some(InnerType::Position(p)) = self.bindings.get(ident) {
                            return Some(InnerType::Position(*p));
                        }
                    } else if let (
                        TvkObject::List(x_list),
                        TvkObject::List(y_list),
                        TvkObject::List(z_list),
                    ) = (&l[1], &l[2], &l[3])
                    {
                        if let (
                            TvkObject::FloatLiteral(x),
                            TvkObject::FloatLiteral(y),
                            TvkObject::FloatLiteral(z),
                        ) = (&x_list[1], &y_list[1], &z_list[1])
                        {
                            return Some(InnerType::Position([*x, *y, *z]));
                        }
                    }
                }
                if let TvkObject::Atom("up") = &l[0] {
                    if let TvkObject::Atom(ident) = &l[1] {
                        if let Some(InnerType::Position(p)) = self.bindings.get(ident) {
                            return Some(InnerType::Position(*p));
                        }
                    } else if let (
                        TvkObject::List(x_list),
                        TvkObject::List(y_list),
                        TvkObject::List(z_list),
                    ) = (&l[1], &l[2], &l[3])
                    {
                        if let (
                            TvkObject::FloatLiteral(x),
                            TvkObject::FloatLiteral(y),
                            TvkObject::FloatLiteral(z),
                        ) = (&x_list[1], &y_list[1], &z_list[1])
                        {
                            return Some(InnerType::Position([*x, *y, *z]));
                        }
                    }
                }
                if let TvkObject::Atom("color") = &l[0] {
                    if let TvkObject::Color(c) = &l[1] {
                        return Some(InnerType::Color(*c));
                    } else if let TvkObject::Atom(ident) = &l[1] {
                        if let Some(InnerType::Color(c)) = self.bindings.get(ident) {
                            return Some(InnerType::Color(*c));
                        }
                    }
                }
                if let TvkObject::Atom("vertex") = &l[0] {
                    if let Some(InnerType::Position(position)) = self.eval(&l[1], pipeline) {
                        if let Some(InnerType::Color(color)) = self.eval(&l[2], pipeline) {
                            return Some(InnerType::Vertex(Vertex { position, color }));
                        }
                    } else if let Some(InnerType::Color(color)) = self.eval(&l[1], pipeline) {
                        if let Some(InnerType::Position(position)) = self.eval(&l[2], pipeline) {
                            return Some(InnerType::Vertex(Vertex { position, color }));
                        }
                    }
                }
                if let TvkObject::Atom("vertex-buffer") = &l[0] {
                    if let TvkObject::List(vs) = &l[1] {
                        let mut vb = Vec::new();
                        for v in vs {
                            if let Some(InnerType::Vertex(ver)) = self.eval(v, pipeline) {
                                vb.push(ver);
                            } else {
                                return None;
                            }
                        }
                        return Some(InnerType::VertexBuffer(vb));
                    }
                }
                if let TvkObject::Atom("index-buffer") = &l[0] {
                    let mut ib = Vec::new();
                    if let TvkObject::List(indices) = &l[1] {
                        for i in indices {
                            if let Some(InnerType::UInt(ui)) = self.eval(i, pipeline) {
                                ib.push(ui);
                            } else {
                                return None;
                            }
                        }
                        return Some(InnerType::IndexBuffer(ib));
                    }
                }
                if let TvkObject::Atom("fovy") = &l[0] {
                    return self.eval(&l[1], pipeline);
                }
                if let TvkObject::Atom("z-near") = &l[0] {
                    return self.eval(&l[1], pipeline);
                }
                if let TvkObject::Atom("z-far") = &l[0] {
                    return self.eval(&l[1], pipeline);
                }
                if let TvkObject::Atom("perspective") = &l[0] {
                    if let Some(InnerType::Float(fovy)) = self.eval(&l[1], pipeline) {
                        if let Some(InnerType::Float(z_near)) = self.eval(&l[2], pipeline) {
                            if let Some(InnerType::Float(z_far)) = self.eval(&l[3], pipeline) {
                                return Some(InnerType::Perspective([fovy, z_near, z_far]));
                            }
                        }
                    }
                    return self.eval(&l[1], pipeline);
                }
                if let TvkObject::Atom("camera") = &l[0] {
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
                }
                if let TvkObject::Atom("transform") = &l[0] {
                    println!("\nl1 {:?}\n", &l[1]);
                    if let Some(InnerType::Vec3(translate)) = self.eval(&l[1], pipeline) {
                        println!("\nl2 {:?}\n", &l[2]);
                        if let Some(InnerType::Vec3(scale)) = self.eval(&l[2], pipeline) {
                            println!("\nl3 {:?}\n", &l[3]);
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
                }
                if let TvkObject::Atom("translate") = &l[0] {
                    println!("\ntranslate l: {:?}\n", l);
                    println!("eval'd {:?}\n", self.eval(&l[1], pipeline));
                    if let Some(InnerType::Vec3(vector)) = self.eval(&l[1], pipeline) {
                        return Some(InnerType::Vec3(vector));
                    }
                }
                if let TvkObject::Atom("scale") = &l[0] {
                    if let Some(InnerType::Vec3(vector)) = self.eval(&l[1], pipeline) {
                        return Some(InnerType::Vec3(vector));
                    }
                }
                if let TvkObject::Atom("rotate") = &l[0] {
                    if let Some(InnerType::Float(angle)) = self.eval(&l[1], pipeline) {
                        if let Some(InnerType::Vec3(axis)) = self.eval(&l[2], pipeline) {
                            return Some(InnerType::Rotate((angle, axis)));
                        }
                    }
                }
                if let TvkObject::Atom("topology") = &l[0] {
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
                        _ => Some(InnerType::Topology(PrimitiveTopology::TriangleList)),
                    };
                }
                if let TvkObject::Atom("model") = &l[0] {
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
                }
                if let TvkObject::Atom("draw") = &l[0] {
                    match self.eval(&l[1], pipeline) {
                        Some(InnerType::Model(m)) => {
                            pipeline.receive_model(m);
                            return None;
                        }
                        Some(InnerType::VertexBuffer(vb)) => {
                            pipeline.receive_vertex_buffer(vb);
                            return None;
                        }
                        _ => return None,
                    }
                }
                return None;
            }
            TvkObject::Atom(identifier) => {
                return self.bindings.get(identifier).cloned();
            }
            TvkObject::FloatLiteral(f) => return Some(InnerType::Float(*f)),
            TvkObject::UIntLiteral(i) => return Some(InnerType::UInt(*i)),
            _ => return None,
        }
    }
}
