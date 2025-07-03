pub struct Material {
    pub name: String,
    pub diffuse_texture: Option<String>,
    pub specular_texture: Option<String>,
    pub normal_texture: Option<String>,
    pub shader_program: Option<String>,
}
