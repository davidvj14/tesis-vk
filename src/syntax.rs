use bytemuck::{Pod, Zeroable};
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;

#[derive(Clone, Copy, Debug)]
pub enum InterpretingMode {
    Continuous,
    Manual,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Zeroable, Pod)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

vulkano::impl_vertex!(Vertex, position, color);

//Vector of vertex names. Might change as the design evolves
pub type VertexBuffer = Vec<String>;

#[derive(Debug, Clone, Copy)]
pub struct ModelTransform {
    pub translate: [f32; 3],
    pub scale: [f32; 3],
    pub rotate: (f32, [f32; 3]),
}

impl ModelTransform {
    pub fn identity() -> Self {
        ModelTransform {
            translate: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            rotate: (0.0, [0.0, 0.0, 1.0]),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub model: Option<ModelTransform>,
    pub view: Option<([f32; 3], [f32; 3], [f32; 3])>,
    pub projection: Option<(f32, f32, f32, f32)>,
}

impl Transform {
    pub fn identity() -> Self {
        Transform {
            model: Some(ModelTransform::identity()),
            view: None,
            projection: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ModelOperation {
    MkModel(String, String, Option<PrimitiveTopology>),
    LoadModel(String),
    AssocIndexBuffer(String, Vec<u32>),
    ApplyTransform(String, String),
    ChangeTopology(String, PrimitiveTopology),
}

#[derive(Debug, Clone)]
pub enum Command {
    GlobalConf(PrimitiveTopology, InterpretingMode),
    DefVertex(String, Vertex), //Binding position/color data to a name
    MkVertexBuffer(String, VertexBuffer), //Binding a VertexBuffer to a name
    Draw(Option<PrimitiveTopology>, String), //Asking the engine to draw a vertex buffer with an optional drawing mode. Might change
    ModelOp(ModelOperation),
    MkTransform(String, Transform),
    MkPerspective(String, Transform),
    MkCamera(String, Transform),
}
