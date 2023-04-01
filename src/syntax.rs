use bytemuck::{Pod, Zeroable};
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;

#[derive(Clone, Copy, Debug)]
pub enum InterpretingMode{
    Continuous,
    Manual
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Zeroable, Pod)]
pub struct Vertex{
    pub position: [f32; 3],
    pub color: [f32; 4],
}

vulkano::impl_vertex!(Vertex, position, color);

//Vector of vertex names. Might change as the design evolves
pub type VertexBuffer = Vec<String>;


#[derive(Debug, Clone)]
pub enum Command{
    GlobalConf(PrimitiveTopology, InterpretingMode),
    DefVertex(String, Vertex), //Binding position/color data to a name
    MkVertexBuffer(String, Box<VertexBuffer>), //Binding a VertexBuffer to a name
    Draw(Option<PrimitiveTopology>, String) //Asking the engine to draw a vertex buffer with an optional
                                      //drawing mode. Might change
}
