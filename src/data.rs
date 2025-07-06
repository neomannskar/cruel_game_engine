use std::path::PathBuf;

use crate::opengl::{DynamicRenderData, StaticRenderData};

#[derive(Debug, Clone)]
pub enum Color {
    Rgb([f32; 3]),
    Rgba([f32; 4]),
}

#[derive(Debug)]
pub struct Uv(pub [f32; 2]);

#[derive(Debug)]
pub struct VertexData {
    pub positions: Vec<[f32; 3]>, // Required

    pub normals: Option<Vec<[f32; 3]>>,  // Optional
    pub tangents: Option<Vec<[f32; 4]>>, // Optional

    /// Supports multiple texcoord sets (TEXCOORD_0, TEXCOORD_1, etc.)
    pub texcoords: Vec<Uv>, // Optional; empty Vec if none

    /// Supports multiple color sets (COLOR_0, COLOR_1, etc.)
    pub colors: Vec<Color>, // Optional; empty Vec if none

    pub joints: Option<Vec<[u16; 4]>>,  // Optional (skinning)
    pub weights: Option<Vec<[f32; 4]>>, // Optional (skinning)                // Optional; None = non-indexed
}

#[derive(Debug)]
pub struct LoadedTexture {
    pub name: String,
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // RGBA8 pixels
}

#[derive(Debug)]
pub struct LoadedMaterial {
    pub base_color_texture: Option<PathBuf>,
    pub metallic_roughness_texture: Option<PathBuf>,
    pub normal_texture: Option<PathBuf>,
    pub occlusion_texture: Option<PathBuf>,
    pub emissive_texture: Option<PathBuf>,

    pub base_color_factor: Color, // fallback if no texture
    pub metallic_factor: f32,
    pub roughness_factor: f32,

    pub alpha_mode: bool,
    pub double_sided: bool,
}

#[derive(Debug)]
pub struct LoadedPrimitive {
    pub vertex_data: Box<VertexData>,
    pub material: Option<LoadedMaterial>,
    pub indices: Option<Vec<u32>>,
}

#[derive(Debug, Clone)]
pub struct StaticPrimitiveInstance {
    pub primitive_index: usize, // Index into LoadedMesh.primitives
    pub render_data: Option<StaticRenderData>, // VAO/VBO/EBO for this primitive
}

#[derive(Debug, Clone)]
pub struct DynamicPrimitiveInstance {
    pub primitive_index: usize, // Index into LoadedMesh.primitives
    pub render_data: Option<DynamicRenderData>, // VAO/VBO/EBO for this primitive
}

// StreamPrimitiveInstance

#[derive(Debug)]
pub struct LoadedMesh {
    pub name: String,
    pub path: PathBuf,
    pub primitives: Vec<LoadedPrimitive>,
}

#[derive(Debug)]
pub struct CompiledShaderProgram {
    pub name: String,
    pub path: PathBuf,
}
