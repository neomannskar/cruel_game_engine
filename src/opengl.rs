use glow::*;

#[derive(Debug, Clone)]
pub struct Layout {
    pub index: u32,
    pub size: i32,
    pub gl_type: u32,
    pub normalized: bool,
    pub offset: usize,
}

impl Layout {
    pub fn new(index: u32, size: i32, gl_type: u32, normalized: bool, offset: usize) -> Self {
        Self {
            index,
            size,
            gl_type,
            normalized,
            offset,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StaticRenderData {
    pub vao: NativeVertexArray,
    pub vbo: NativeBuffer,
    pub ebo: Option<NativeBuffer>,
    pub stride: i32,
    pub layouts: Vec<Layout>,

    pub vertex_count: i32,
    pub index_count: i32,
}

impl StaticRenderData {
    pub fn new(
        context: &glow::Context,
        vertices: &[f32],
        indices: &[u32],
        stride: i32,
        layouts: Vec<Layout>,
    ) -> Self {
        unsafe {
            let vao = context.create_vertex_array().expect("Failed to create VAO");
            context.bind_vertex_array(Some(vao));

            let vbo = context.create_buffer().expect("Failed to create VBO");
            context.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            context.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(vertices),
                glow::STATIC_DRAW,
            );

            let ebo = context.create_buffer().expect("Failed to create EBO");
            context.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            context.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(indices),
                glow::STATIC_DRAW,
            );

            let vertex_count = (vertices.len() as i32) / (stride / std::mem::size_of::<f32>() as i32);
            let index_count = indices.len() as i32;

            Self {
                vao,
                vbo,
                ebo: Some(ebo),
                stride,
                layouts,

                vertex_count,
                index_count,
            }
        }
    }

    pub fn bind(&self, context: &glow::Context) {
        unsafe {
            context.bind_vertex_array(Some(self.vao));
            context.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));

            for layout in &self.layouts {
                context.vertex_attrib_pointer_f32(
                    layout.index,
                    layout.size,
                    layout.gl_type,
                    layout.normalized,
                    self.stride,
                    layout.offset as i32,
                );
                context.enable_vertex_attrib_array(layout.index);
            }

            if let Some(ebo) = self.ebo {
                context.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DynamicRenderData {
    pub vao: NativeVertexArray,
    pub vbo: NativeBuffer,
    pub ebo: Option<NativeBuffer>,
    pub stride: i32,
    pub layouts: Vec<Layout>,

    pub vertex_count: i32,
    pub index_count: i32,
}

impl DynamicRenderData {
    pub fn new(
        context: &glow::Context,
        vertices: &[f32],
        indices: &[u32],
        stride: i32,
        layouts: Vec<Layout>,
    ) -> Self {
        unsafe {
            let vao = context.create_vertex_array().unwrap();
            context.bind_vertex_array(Some(vao));
            let vbo = context.create_buffer().unwrap();
            context.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            context.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(vertices),
                glow::DYNAMIC_DRAW,
            );

            let ebo = context.create_buffer().unwrap();
            context.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            context.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(indices),
                glow::DYNAMIC_DRAW,
            );

            let vertex_count = (vertices.len() as i32) / (stride / std::mem::size_of::<f32>() as i32);
            let index_count = indices.len() as i32;

            Self {
                vao,
                vbo,
                ebo: Some(ebo),
                stride,
                layouts,

                vertex_count,
                index_count,
            }
        }
    }

    pub fn bind(&self, context: &glow::Context) {
        unsafe {
            context.bind_vertex_array(Some(self.vao));
            context.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            if let Some(ebo) = self.ebo {
                context.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            }
        }
    }
}
