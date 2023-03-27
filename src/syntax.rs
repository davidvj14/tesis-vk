use bytemuck::{Pod, Zeroable};

//How to interpret vertex buffers when drawing
enum DrawingMode{
    Point,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip
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
    GlobalConf, //Might remove or actually use later
    DefVertex(String, Vertex), //Binding position/color data to a name
    MkVertexBuffer(String, Box<VertexBuffer>), //Binding a VertexBuffer to a name
    Draw(String) //Asking the engine to draw a vertex buffer. Might change
}
