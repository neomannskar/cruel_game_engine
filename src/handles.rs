use glow::Texture;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshHandle(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaterialHandle(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderHandle(pub usize);

#[derive(Debug)]
pub enum AssetHandle {
    Texture(TextureHandle),
    Mesh(MeshHandle),
    Material(MaterialHandle),
    Shader(ShaderHandle),
}

impl AssetHandle {
    pub fn as_mesh_handle(&self) -> Option<MeshHandle> {
        if let AssetHandle::Mesh(handle) = *self {
            Some(handle)
        } else {
            None
        }
    }

    pub fn as_texture_handle(&self) -> Option<TextureHandle> {
        if let AssetHandle::Texture(handle) = *self {
            Some(handle)
        } else {
            None
        }
    }

    pub fn as_material_handle(&self) -> Option<MaterialHandle> {
        if let AssetHandle::Material(handle) = *self {
            Some(handle)
        } else {
            None
        }
    }

    pub fn as_shader_handle(&self) -> Option<ShaderHandle> {
        if let AssetHandle::Shader(handle) = *self {
            Some(handle)
        } else {
            None
        }
    }
}
