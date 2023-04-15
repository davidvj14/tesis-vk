use bytemuck::{Pod, Zeroable};
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;

use crate::tvk_glm::{
    identity_mat4, look_at_rh, mult_mat4, perspective_rh_no, rotate_mat4, scale_mat4,
    translate_mat4,
};

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

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub position: Position,
    pub center: Center,
    pub up: Up,
    pub perspective: [f32; 3],
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: [2.0, 2.0, 2.0],
            center: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            perspective: [0.75, 0.1, 10.0],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Transform {
    pub translate: Vec3,
    pub scale: Vec3,
    pub rotate: (Angle, Vec3),
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translate: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            rotate: (0.0, [0.0, 1.0, 0.0]),
        }
    }
}

type VertexBuffer = Vec<Vertex>;
type IndexBuffer = Vec<u32>;

#[derive(Clone, Debug)]
pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub topology: PrimitiveTopology,
    pub transforms: Transform,
    pub camera: Camera,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            topology: PrimitiveTopology::TriangleList,
            transforms: Transform::default(),
            camera: Camera::default(),
        }
    }
}

impl Model {
    pub fn generate_mvp_mats(&self, dimensions: [u32; 2]) -> [[[f32; 4]; 4]; 3] {
        let translate = translate_mat4(identity_mat4(), self.transforms.translate);
        let scale = scale_mat4(identity_mat4(), self.transforms.scale);
        let rotate = rotate_mat4(
            identity_mat4(),
            self.transforms.rotate.0,
            self.transforms.rotate.1,
        );
        let model = mult_mat4(translate, rotate);
        let model = mult_mat4(model, scale);
        let cam = self.camera;
        let view = look_at_rh(cam.position, cam.center, cam.up);
        let aspect_ratio = dimensions[0] as f32 / dimensions[1] as f32;
        let projection = perspective_rh_no(
            cam.perspective[0],
            aspect_ratio,
            cam.perspective[1],
            cam.perspective[2],
        );
        [model, view, projection]
    }
}

/*#[derive(Clone, Debug)]
pub enum BasicOp<'a> {
    Add(Vec<TvkObject<'a>>),
    Sub(Vec<TvkObject<'a>>),
    Mul(Vec<TvkObject<'a>>),
    Div(Vec<TvkObject<'a>>),
}
*/

#[derive(Clone, Debug)]
pub enum TvkObject<'a> {
    FloatLiteral(f32),
    Atom(&'a str),
    UIntLiteral(u32),
    Color(Color),
    List(Vec<TvkObject<'a>>),
    //Vertex(Vertex)g
    //VertexBuffer(VertexBuffer<'a>),
    //IndexBuffer(IndexBuffer),
    //Perspective(Fovy, AspectRatio, ZNear, ZFar),
    //Camera(Camera),
    //Transform(Transform),
    //Model(Model),
}

#[derive(Clone, Debug)]
pub enum InnerType {
    Float(f32),
    UInt(u32),
    Position(Position),
    Rotate((Angle, Vec3)),
    Topology(PrimitiveTopology),
    Color(Color),
    Vec3([f32; 3]),
    Vertex(Vertex),
    VertexBuffer(VertexBuffer),
    IndexBuffer(IndexBuffer),
    Perspective([f32; 3]),
    Camera(Camera),
    Transform(Transform),
    Model(Model),
}