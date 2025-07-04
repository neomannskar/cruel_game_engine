use glow::HasContext;

use crate::data::LoadedTexture;

pub struct Texture {
    pub name: String,
    pub texture: glow::NativeTexture,
    pub width: u32,
    pub height: u32,
    pub data: Option<Vec<u8>>, // raw image data
}

impl Texture {
    pub fn from_loaded_data(
        context: &glow::Context,
        name: Option<String>,
        data: LoadedTexture,
    ) -> Self {
        unsafe {
            let texture = context.create_texture().unwrap();
            context.bind_texture(glow::TEXTURE_2D, Some(texture));

            context.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            context.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
            context.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            context.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );

            context.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                data.width as i32,
                data.height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(&data.data)),
            );

            context.generate_mipmap(glow::TEXTURE_2D);

            let name = match name {
                Some(n) => n,
                None => data.name,
            };

            Texture {
                name,
                texture,
                width: data.width,
                height: data.height,
                data: Some(data.data),
            }
        }
    }

    fn create_texture(gl: &glow::Context, image_path: &str) -> glow::NativeTexture {
        let img = image::open(image_path).unwrap().flipv().to_rgba8();
        let (width, height) = img.dimensions();
        let data = img.into_raw();

        unsafe {
            let texture = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));

            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width as i32,
                height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(&data)),
            );

            gl.generate_mipmap(glow::TEXTURE_2D);

            texture
        }
    }
}
