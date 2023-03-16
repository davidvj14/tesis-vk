#![allow(dead_code)]

enum DrawingMode{
    Point,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip
}

enum RenderingMode{
    TwoD,
    ThreeD,
}

struct Position{
    x: f32,
    y: f32,
    z: f32,
}

struct Color{
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
}

pub struct Vertex{
    pos: Position,
    color: Color,
}

pub type VertexBuffer = [Vertex];

pub enum Command{
    GlobalConf,
    DefVertex(String, Vertex),
    MkVertexBuffer(String, Box<VertexBuffer>),
    Draw(String)
}
