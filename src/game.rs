/// Wrapps all game-related things, this is what gets serialized and deserialized
#[derive(Debug)]
pub struct Game {
    pub resource_manager: ResourceManager,
    pub world: World, 
}
