#[derive(Debug)]
pub struct Transform {
    pub translation: cgmath::Vector3<f32>,
    pub rotation: cgmath::Vector3<f32>,     // Later: cgmath::Quaternion<f32>,
    pub scale: cgmath::Vector3<f32>,
}


