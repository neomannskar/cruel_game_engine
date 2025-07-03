use glow::*;

#[derive(Debug, Clone)]
pub struct StaticRenderData {
    pub vao: NativeVertexArray,
    pub vbo: NativeBuffer,
    pub ebo: Option<NativeBuffer>,
}

impl StaticRenderData {
    pub fn new(context: &glow::Context, vertices: &[f32], indices: &[u32]) -> Self {
        unsafe {
            let vao = context.create_vertex_array().unwrap();
            context.bind_vertex_array(Some(vao));
            let vbo = context.create_buffer().unwrap();
            context.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            context.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(vertices),
                glow::STATIC_DRAW,
            );

            let ebo = context.create_buffer().unwrap();
            context.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            context.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(indices),
                glow::STATIC_DRAW,
            );
            
            Self {
                vao,
                vbo,
                ebo: Some(ebo),
            }
        }
    }

    /* pub fn from(vao: NativeVertexArray, vbo: NativeBuffer, ebo: Option<NativeBuffer>) -> Self {
        Self {
            vao,
            vbo,
            ebo,
        }
    } */

    pub fn bind(&self, context: &glow::Context) {
        unsafe {
            context.bind_vertex_array(Some(self.vao));
            context.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            
            let stride = (8 * std::mem::size_of::<f32>()) as i32;

            context.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);
            context.enable_vertex_attrib_array(0);

            context.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, (3 * std::mem::size_of::<f32>()) as i32);
            context.enable_vertex_attrib_array(1);

            context.vertex_attrib_pointer_f32(2, 3, glow::FLOAT, false, stride, (5 * std::mem::size_of::<f32>()) as i32);
            context.enable_vertex_attrib_array(2);
            
            if let Some(ebo) = self.ebo {
                context.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            }
        }
    }
}

pub struct DynamicRenderData {
    pub vao: NativeVertexArray,
    pub vbo: NativeBuffer,
    pub ebo: Option<NativeBuffer>,
}

impl DynamicRenderData {
    pub fn new(context: &glow::Context, vertices: &[f32], indices: &[u32]) -> Self {
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
            
            Self {
                vao,
                vbo,
                ebo: Some(ebo),
            }
        }
    }

    pub fn from(vao: NativeVertexArray, vbo: NativeBuffer, ebo: Option<NativeBuffer>) -> Self {
        Self {
            vao,
            vbo,
            ebo,
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
