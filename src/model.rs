use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use crate::tvk_glm::*;

use crate::syntax::{Transform, Vertex, ModelTransform};

#[derive(Clone, Debug)]
pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Option<Vec<u32>>,
    pub topology: Option<PrimitiveTopology>,
    pub transforms: Transform,
}

impl Model {
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Option<Vec<u32>>,
        topology: Option<PrimitiveTopology>,
        ) -> Self {
        Model {
            vertices,
            indices,
            topology,
            transforms: Transform::identity(),
        }
    }
}

#[derive(Debug)]
pub struct TransUBO {
    pub model: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
}

impl TransUBO{
    pub fn generate_ubo(trans: &Transform) -> Self {
        if let Some(m) = trans.model {
            let model = Self::generate_model_matrix(&m);
            TransUBO {
                model,
                view: identity_mat4(),
                projection: identity_mat4()
            }
        } else{
            TransUBO {
                model: identity_mat4(),
                view: identity_mat4(),
                projection: identity_mat4()
            }
        }
    }

    pub fn generate_model_matrix(trans: &ModelTransform) -> [[f32; 4]; 4] {
        let translate = translate_mat4(identity_mat4(), trans.translate);
        let rotation = rotate_mat4(identity_mat4(), trans.rotate.0, trans.rotate.1);
        let scale = scale_mat4(identity_mat4(), trans.scale);
        let result = mult_mat4(translate, rotation);
        mult_mat4(result, scale)
    }
}
