pub struct ResourceManager {
    asset_loader: AssetLoader,
    
    textures: HashMap<TextureHandle, Texture>,
    meshes: HashMap<MeshHandle, Mesh>,
    materials: HashMap<MaterialHandle, Material>,
}

impl ResourceManager {
    pub fn new() -> Self {

    }
}
