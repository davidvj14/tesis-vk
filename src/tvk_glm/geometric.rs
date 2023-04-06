#![allow(dead_code)]

use std::f32::consts::PI;

pub fn dot_vec3(a: [f32; 3], b: [f32;3]) -> f32 {
    a[0] * b[0] +
    a[1] * b[1] +
    a[2] * b[2] 
}

pub fn len_vec3(a: [f32; 3]) -> f32 {
    dot_vec3(a, a).sqrt()
}

pub fn cross_vec3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - b[1] * a[2],
    a[2] * b[0] - b[2] * a[0],
    a[0] * b[1] - b[0] * a[1]]
}

pub fn normalize_vec3(vec: [f32; 3]) -> [f32; 3] {
    let isqrt = 1.0 / dot_vec3(vec, vec).sqrt();
    [ vec[0] * isqrt
    , vec[1] * isqrt
    , vec[2] * isqrt]
}

pub fn vector_scalar_mult3(vec: [f32; 3], scalar: f32) -> [f32; 3] {
    [vec[0] * scalar,
    vec[1] * scalar,
    vec[2] * scalar]
}

pub fn vector_scalar_mult4(vec: [f32; 4], scalar: f32) -> [f32; 4] {
    [vec[0] * scalar,
    vec[1] * scalar,
    vec[2] * scalar,
    vec[3] * scalar]
}

pub fn vec3_subs(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0],
    a[1] - b[1],
    a[2] - b[2]]
}

pub fn vec4_addition(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0] + b[0],
    a[1] + b[1],
    a[2] + b[2],
    a[3] + b[3]]
}

pub fn identity_mat4() -> [[f32; 4]; 4] {
    [[1.0, 0.0, 0.0, 0.0],
     [0.0, 1.0, 0.0, 0.0],
     [0.0, 0.0, 1.0, 0.0],
     [0.0, 0.0, 0.0, 1.0],]
}

pub fn zero_mat4() -> [[f32; 4]; 4] {
    [[0.0, 0.0, 0.0, 0.0],
     [0.0, 0.0, 0.0, 0.0],
     [0.0, 0.0, 0.0, 0.0],
     [0.0, 0.0, 0.0, 0.0],]
}

pub fn translate_mat4(mat: [[f32; 4]; 4], vec: [f32; 3]) -> [[f32; 4]; 4]{
    let mut result = mat;
    result[3] = vector_scalar_mult4(mat[0], vec[0]);
    result[3] = vec4_addition(result[3], vector_scalar_mult4(mat[1], vec[1]));
    result[3] = vec4_addition(result[3], vector_scalar_mult4(mat[2], vec[2]));
    result[3] = vec4_addition(result[3], mat[3]);
    result
}

pub fn scale_mat4(mat: [[f32; 4]; 4], vec: [f32; 3]) -> [[f32; 4]; 4]{
    let mut result = mat;
    result[0] = vector_scalar_mult4(mat[0], vec[0]);
    result[1] = vector_scalar_mult4(mat[1], vec[1]);
    result[2] = vector_scalar_mult4(mat[2], vec[2]);
    result[3] = mat[3];
    result
}

pub fn rotate_mat4(mat4: [[f32; 4]; 4], angle: f32, axis: [f32; 3]) -> [[f32; 4] ; 4]{
    let c = angle.cos();
    let s = angle.sin();
    let norm_axis = normalize_vec3(axis);
    let temp: [f32; 3] = vector_scalar_mult3(norm_axis, 1.0 - c); 
    let mut rotate = identity_mat4();

    rotate[0][0] = c + temp[0] * norm_axis[0];
		rotate[0][1] = temp[0] * norm_axis[1] + s * norm_axis[2];
		rotate[0][2] = temp[0] * norm_axis[2] - s * norm_axis[1];

		rotate[1][0] = temp[1] * norm_axis[0] - s * norm_axis[2];
		rotate[1][1] = c + temp[1] * norm_axis[1];
		rotate[1][2] = temp[1] * norm_axis[2] + s * norm_axis[0];

		rotate[2][0] = temp[2] * norm_axis[0] + s * norm_axis[1];
		rotate[2][1] = temp[2] * norm_axis[1] - s * norm_axis[0];
		rotate[2][2] = c + temp[2] * norm_axis[2];
    
    let mut result = identity_mat4();

    result[0] = vector_scalar_mult4(mat4[0], rotate[0][0]);
    result[0] = vec4_addition(result[0], vector_scalar_mult4(mat4[1], rotate[0][1]));
    result[0] = vec4_addition(result[0], vector_scalar_mult4(mat4[2], rotate[0][2]));

    result[1] = vector_scalar_mult4(mat4[0], rotate[1][0]);
    result[1] = vec4_addition(result[1], vector_scalar_mult4(mat4[1], rotate[1][1]));
    result[1] = vec4_addition(result[1], vector_scalar_mult4(mat4[2], rotate[1][2]));

    result[2] = vector_scalar_mult4(mat4[0], rotate[2][0]);
    result[2] = vec4_addition(result[2], vector_scalar_mult4(mat4[1], rotate[2][1]));
    result[2] = vec4_addition(result[2], vector_scalar_mult4(mat4[2], rotate[2][2]));

    result[3] = mat4[3];
    result
}

pub fn look_at_rh(eye: [f32; 3], center: [f32; 3], up: [f32; 3]) -> [[f32; 4]; 4]{
    let f = normalize_vec3(vec3_subs(center, eye));
    let s =  normalize_vec3(cross_vec3(f, up));
    let u = cross_vec3(s, f);
    
    let mut result = identity_mat4();
    result[0][0] = s[0];
    result[1][0] = s[1];
    result[2][0] = s[2];
    result[0][1] = u[0];
    result[1][1] = u[1];
    result[2][1] = u[2];
    result[0][2] = -f[0];
    result[1][2] = -f[1];
    result[2][2] = -f[2];
    result[3][0] = -dot_vec3(s, eye);
    result[3][1] = -dot_vec3(u, eye);
    result[3][2] = dot_vec3(f, eye);

    result
}

pub fn perspective_rh_no (fovy: f32, aspect: f32, z_near: f32, z_far: f32)
    -> [[f32; 4]; 4] {
        let tan_half_fovy = (fovy / 2.0).tan();
        let mut result = zero_mat4();

        result[0][0] = 1.0 / (aspect * tan_half_fovy);
        result[1][1] = -1.0 / tan_half_fovy;
        result[2][2] = - (z_far + z_near) / (z_far - z_near);
        result[2][3] = -1.0;
        result[3][2] = - (2.0 * z_far * z_near) / (z_far - z_near);
        result
}

pub fn radians(degrees: f32) -> f32 {
    degrees * (PI / 180.0)
}

pub fn mult_mat4(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut result = zero_mat4();
    for i in 0..4 {
        for j in 0..4{
            for k in 0..4{
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    result
}
