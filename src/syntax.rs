#![allow(dead_code)]
use bytemuck::{Pod, Zeroable};

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

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, Zeroable, Pod)]
pub struct Position{
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, Pod, Zeroable)]
pub struct Color{
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Zeroable, Pod)]
pub struct Vertex{
    pub position: [f32; 3],
    pub color: [f32; 4],
}

vulkano::impl_vertex!(Vertex, position, color);

pub type VertexBuffer = Vec<String>;

#[derive(Debug, Clone)]
pub enum Command{
    GlobalConf,
    DefVertex(String, Vertex),
    MkVertexBuffer(String, Box<VertexBuffer>),
    Draw(String)
}
