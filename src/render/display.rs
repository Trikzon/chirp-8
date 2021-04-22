use crate::render::gl;
use crate::render::{Buffer, ProgramBuilder, ShaderError, ShaderProgram, VertexArray};
use glutin::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
    ContextBuilder, ContextWrapper, PossiblyCurrent,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DisplayError {
    #[error("failed to create the glutin window")]
    WindowCreation,
    #[error("failed to make the glutin window current")]
    ContextCurrent,
    #[error("failed to swap glutin window's buffers")]
    SwapBuffers,
    #[error("encountered a shader error")]
    ShaderError(#[from] ShaderError),
}

const TEMP_SHADER: &str = include_str!("./shader/chip-8.glsl");

pub struct Display {
    context: ContextWrapper<PossiblyCurrent, Window>,
    clear_color: (f32, f32, f32),
    gl: gl::Gl,
    shader: ShaderProgram,
    vertex_array: VertexArray,
    indice_count: usize,
}

impl Display {
    pub fn new<T>(
        builder: DisplayBuilder,
        event_loop: &EventLoop<T>,
    ) -> Result<Self, DisplayError> {
        let title = builder.title.unwrap_or("CHIRP-8".to_string());
        let size = builder.size.unwrap_or((640, 480));

        let context = ContextBuilder::new()
            .build_windowed(
                WindowBuilder::new()
                    .with_title(title)
                    .with_inner_size(LogicalSize::new(size.0, size.1)),
                event_loop,
            )
            .map_err(|_| DisplayError::WindowCreation)?;
        let context = unsafe {
            context
                .make_current()
                .map_err(|_| DisplayError::ContextCurrent)?
        };

        let gl = gl::Gl::load_with(|ptr| context.get_proc_address(ptr) as *const _);

        let mut shader = ProgramBuilder::new().with_combo(TEMP_SHADER)?.build(&gl)?;

        shader.bind();
        shader.define_uniform("uPixels")?;
        let mut pixels = [0; 64];
        pixels[0] = 0b01111100000000000000000000000000;
        pixels[1] = 0b00010000000000000000000000000000;
        pixels[2] = 0b01111100000000000000000000000000;
        pixels[3] = 0b00000000000000000000000000000000;
        pixels[4] = 0b01111100000000000000000000000000;
        pixels[5] = 0b01010100000000000000000000000000;
        pixels[6] = 0b01000100000000000000000000000000;
        pixels[7] = 0b00000000000000000000000000000000;
        pixels[8] = 0b01111100000000000000000000000000;
        pixels[9] = 0b00000100000000000000000000000000;
        pixels[10] = 0b00000100000000000000000000000000;
        pixels[11] = 0b00000000000000000000000000000000;
        pixels[12] = 0b01111100000000000000000000000000;
        pixels[13] = 0b00000100000000000000000000000000;
        pixels[14] = 0b00000100000000000000000000000000;
        pixels[15] = 0b00000000000000000000000000000000;
        pixels[16] = 0b01111100000000000000000000000000;
        pixels[17] = 0b01000100000000000000000000000000;
        pixels[18] = 0b01111100000000000000000000000000;
        shader.upload_uniform("uPixels", &pixels)?;
        shader.unbind();

        let vertices: [f32; 12] = [
            -1.0, 1.0, 0.0, // top left
            -1.0, -1.0, 0.0, // bottom left
            1.0, -1.0, 0.0, // bottom right
            1.0, 1.0, 0.0, // top right
        ];
        let pixel_pos: [f32; 8] = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0];
        let indices = [0, 1, 3, 1, 2, 3];

        let vertices_buffer = Buffer::new_array_buffer(&gl, &vertices, 3);
        let pixel_pos_buffer = Buffer::new_array_buffer(&gl, &pixel_pos, 2);
        let indices_buffer = Buffer::new_element_buffer(&gl, &indices);

        let mut vertex_array = VertexArray::new(&gl);
        vertex_array.put_element_buffer(indices_buffer);
        vertex_array.put_array_buffer(0, vertices_buffer);
        vertex_array.put_array_buffer(1, pixel_pos_buffer);

        Ok(Self {
            context,
            clear_color: (0.0, 0.0, 0.0),
            gl,
            shader,
            vertex_array,
            indice_count: indices.len(),
        })
    }

    pub fn resize(&self, width: u32, height: u32) {
        self.context.resize(PhysicalSize::new(width, height));
        self.gl.set_view_port(0, 0, width, height);
    }

    pub fn request_redraw(&self) {
        self.context.window().request_redraw();
    }

    pub fn update(&self) -> Result<(), DisplayError> {
        self.context
            .swap_buffers()
            .map_err(|_| DisplayError::SwapBuffers)?;

        self.gl.set_clear_color(
            self.clear_color.0,
            self.clear_color.1,
            self.clear_color.2,
            1.0,
        );
        self.gl
            .clear(&[gl::ClearFlag::COLOR_BUFFER, gl::ClearFlag::DEPTH_BUFFER]);

        Ok(())
    }

    pub fn render(&self) {
        self.shader.bind();
        self.vertex_array.bind();
        self.vertex_array.enable_attrib_arrays();

        self.gl.draw_elements(self.indice_count);

        self.vertex_array.disable_attrib_arrays();
        self.vertex_array.unbind();
        self.shader.unbind();

        self.gl.debug_print_error();
    }
}

pub struct DisplayBuilder {
    title: Option<String>,
    size: Option<(u32, u32)>,
}

impl DisplayBuilder {
    pub fn new() -> Self {
        Self {
            title: None,
            size: None,
        }
    }

    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.size = Some((width, height));
        self
    }

    pub fn build<T>(self, event_loop: &EventLoop<T>) -> Result<Display, DisplayError> {
        Display::new(self, event_loop)
    }
}
