#![allow(unused)]

use bytemuck::{Pod, Zeroable};
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;

type Vec3 = [f32; 3];
type Vec4 = [f32; 4];

type Position = Vec3;
type Color = Vec4;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct Vertex {
    pub position: Position,
    pub color: Color,
}
vulkano::impl_vertex!(Vertex, position, color);

type Angle = f32;
type Center = Vec3;
type Up = Vec3;
type Fovy = f32;
type AspectRatio = f32;
type ZNear = f32;
type ZFar = f32;

type PerspectiveMat4 = [[f32; 4]; 4];

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    position: Position,
    center: Center,
    up: Up,
    perspective: (Fovy, AspectRatio, ZNear, ZFar),
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: [2.0, 2.0, 2.0],
            center: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            perspective: (0.75, 4.0 / 3.0, 0.1, 10.0),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Transform {
    translate: Vec3,
    scale: Vec3,
    rotate: (Angle, Vec3),
    camera: Camera,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translate: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            rotate: (0.0, [0.0, 0.0, 0.0]),
            camera: Camera::default(),
        }
    }
}

type VertexBuffer<'a> = Vec<&'a str>;
type IndexBuffer = Vec<u32>;

#[derive(Clone, Debug)]
pub struct Model {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    topology: PrimitiveTopology,
    transforms: Transform,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            topology: PrimitiveTopology::TriangleList,
            transforms: Transform::default(),
        }
    }
}

#[derive (Clone, Debug)]
pub enum BasicOp<'a> {
    Add(Vec<TvkObject<'a>>),
    Sub(Vec<TvkObject<'a>>),
    Mul(Vec<TvkObject<'a>>),
    Div(Vec<TvkObject<'a>>),
}

#[derive(Clone, Debug)]
pub enum TvkObject<'a> {
    FloatLiteral(f32),
    Atom(&'a str),
    UIntLiteral(u32),
    Color(Color),
    List(Vec<TvkObject<'a>>),
    //Vertex(Vertex),
    //VertexBuffer(VertexBuffer<'a>),
    //IndexBuffer(IndexBuffer),
    //Perspective(Fovy, AspectRatio, ZNear, ZFar),
    //Camera(Camera),
    //Transform(Transform),
    //Model(Model),
}


#[derive (Clone, Debug)]
pub enum Drawable<'a> {
    VertexBuffer(VertexBuffer<'a>),
    Model(Model),
}

#[derive (Clone, Debug)]
pub enum Expr<'a> {
    Identifier(&'a str),
    FloatLiteral(f32),
    UIntLiteral(u32),
    List(Vec<Expr<'a>>),
    Construct(TvkObject<'a>),
    BasicOp(BasicOp<'a>),
    Define(&'a str, Vec<Expr<'a>>),
    Draw(&'a Drawable<'a>),
}
