use glow::HasContext;

pub struct Texture {
    pub name: String,
    pub texture: glow::NativeTexture,
}

impl Texture {
    pub fn new<T: ToString>(context: &glow::Context, name: String, path: T) -> Self {
        Self {
            name,
            texture: Self::create_texture(context, path.to_string().as_str()),
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
