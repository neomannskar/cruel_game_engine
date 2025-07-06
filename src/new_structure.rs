pub struct AssetLoader { // Holds where to find assets (paths)
    meshes: HashMap<MeshHandle, PathBuf>,
}

pub struct ResourceManager { // Holds loaded assets (meshes)
    asset_loader: AssetLoader,
    meshes: HashMap<MeshHandle, Mesh>,
}

pub struct Renderer { // Holds render-specific representation (RenderData)
    render_data: HashMap<MeshHandle, RenderData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshHandle(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaterialHandle(pub usize);

pub struct MeshInstance {
    pub name: String,
    pub handle: MeshHandle,
}

pub struct MaterialInstance {
    pub name: String,
    pub handle: MaterialHandle,
}

pub struct Level {
    pub name: String,
    
    pub mesh_instances: Vec<MeshInstance>,
    pub material_instances: Vec<MaterialInstance>,
}

pub struct LevelHandle(pub usize);

pub struct World {
    pub level_handle: LevelHandle,
    pub levels: Vec<Box<Level>>,
}
