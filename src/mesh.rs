use cgmath::SquareMatrix;
use glow::HasContext;

use crate::{data::{DynamicRenderData, StaticRenderData}, viewport::Viewport};

#[derive(Debug, Clone)]
pub struct StaticMesh {
    pub name: String,
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,

    pub render_data: Option<StaticRenderData>,
    
    // Alternatively pub model_matrix: cgmath::Matrix4<f32>
    
    pub translation: cgmath::Vector3<f32>,
    pub rotation: cgmath::Vector3<f32>, // Change to cgmath::Quaternion<f32> later,
    pub scale: cgmath::Vector3<f32>,
}

impl StaticMesh {
    pub fn new(name: String, vertices: Vec<f32>, indices: Vec<u32>) -> Self {
        Self {
            name,
            vertices,
            indices,
            render_data: None,
            translation: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rotation: cgmath::Vector3::new(0.0, 0.0, 0.0),
            scale: cgmath::Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn set_render_data(&mut self, render_data: StaticRenderData) {
        self.render_data = Some(render_data);
    }

    pub fn render(&self, context: &glow::Context,) {
        unsafe {
            if let Some(rd) = &self.render_data {
                rd.bind(context);
                if rd.ebo.is_some() {
                    context.draw_elements(glow::TRIANGLES, self.indices.len() as i32, glow::UNSIGNED_INT, 0);
                } else {
                    todo!("Static mesh rendering without EBO is not implemented yet");
                    // context.draw_arrays(glow::TRIANGLES, 0, 0);
                }
            } else {
                return;
            }
        }
    }
}

pub struct DynamicMesh {
    pub name: String,
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,

    pub render_data: Option<DynamicRenderData>,
    
    // Alternatively pub model_matrix: cgmath::Matrix4<f32>,

    pub translation: cgmath::Vector3<f32>,
    pub rotatation: cgmath::Quaternion<f32>,
    pub scale: cgmath::Vector3<f32>,
}

impl DynamicMesh {
    pub fn new(name: String, vertices: Vec<f32>, indices: Vec<u32>) -> Self {
        Self {
            name,
            vertices,
            indices,
            render_data: None,
            translation: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rotatation: cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0),
            scale: cgmath::Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn set_render_data(&mut self, render_data: DynamicRenderData) {
        self.render_data = Some(render_data);
    }

    pub fn render(&self, context: &glow::Context) {
        unsafe {
            if let Some(rd) = &self.render_data {
                rd.bind(context);
                if rd.ebo.is_some() {
                    context.draw_elements(glow::TRIANGLES, self.indices.len() as i32, glow::UNSIGNED_INT, 0);
                } else {
                    todo!("Static mesh rendering without EBO is not implemented yet");
                    // context.draw_arrays(glow::TRIANGLES, 0, 0);
                }
            } else {
                return;
            }
        }
    }
}
