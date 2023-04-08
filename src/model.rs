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
            let model = Self::generate_model_matrix(&trans.model.unwrap());
            let v = trans.view.unwrap();
            let p = trans.projection.unwrap();
            TransUBO {
                model,
                view: look_at_rh(v.0, v.1, v.2),
                projection: perspective_rh_no(p.0, p.1, p.2, p.3)
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
