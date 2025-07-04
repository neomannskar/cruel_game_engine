use crate::handles::ShaderHandle;

#[derive(Debug)]
pub struct ShaderProgram {
    pub name: String,
    pub handle: ShaderHandle,
}
