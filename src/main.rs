use gl::types::*;
use std::ffi::CString;
use std::ptr;
use std::str;
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use std::io::{self, Write};
use clap::{App, Arg};

// Universal approach to OpenGL error handling
// Wrap every all into gl_safe!(...) instead of unsafe { ... }
macro_rules! gl_safe {
    (gl::CompileShader(_shader:expr), $step_name:expr) => {{
        let $shader = _shader; // compute expression once
        let result = unsafe { gl::CompileShader($shader) };
        
        // Check for compilation errors
        let mut success = gl::FALSE as gl::types::GLint;
        unsafe { gl::GetShaderiv($shader, gl::COMPILE_STATUS, &mut success);}
        if success != gl::TRUE as gl::types::GLint {
            let mut len = 0;
            unsafe { gl::GetShaderiv($shader, gl::INFO_LOG_LENGTH, &mut len); }
            let mut buffer = vec![0u8; len as usize];
            unsafe { gl::GetShaderInfoLog($shader, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut GLchar); }
            
            panic!("Shader compilation failed at \"{}\": {}. Check the shader source code for errors.", $step_name, str::from_utf8(&buffer).unwrap());
        }
        result
    }};
    (gl::load_with($func:expr), $step_name:expr) => {{
        let result = gl::load_with($func); // safe call

        // Check for errors
        let err_code = unsafe { gl::GetError() };
        if err_code != gl::NO_ERROR {
            panic!("OpenGL error code {} at \"{}\"", err_code, $step_name);
        }
        result
    }};
    ($block:expr, $step_name:expr) => {{
        let result = unsafe { $block };

        // Check for errors
        let err_code = unsafe { gl::GetError() };
        if err_code != gl::NO_ERROR {
            panic!("OpenGL error code {} at \"{}\"", err_code, $step_name);
        }
        result
    }};
}

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader = gl_safe!(gl::CreateShader(ty), "create shader: initialize a new shader object. Ensure the shader type is correct.");
    let c_str = CString::new(src.as_bytes()).unwrap();
    gl_safe!(gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null()), "set shader source: provide source code to shader. Ensure the source is valid GLSL.");
    gl_safe!(gl::CompileShader(shader), "compile shader: compile the shader source code.");
    shader
}

fn main() {
    // Parse command line arguments
    let matches = App::new("Shader Renderer")
        .version("1.0")
        .author("Anton Bukov <k06aaa@gmail.com>")
        .about("Renders a shader to a video file")
        .arg(Arg::new("shader")
            .help("Path to the shader file")
            .required(true)
            .index(1))
        .arg(Arg::new("width")
            .help("Width of the video")
            .required(true)
            .index(2))
        .arg(Arg::new("height")
            .help("Height of the video")
            .required(true)
            .index(3))
        .arg(Arg::new("fps")
            .help("Frames per second")
            .required(true)
            .index(4))
        .arg(Arg::new("duration")
            .help("Duration of the video in seconds")
            .required(true)
            .index(5))
        .get_matches();

    let shader_path = matches.value_of("shader").unwrap();
    let width: u32 = matches.value_of("width").unwrap().parse().expect("Invalid width");
    let height: u32 = matches.value_of("height").unwrap().parse().expect("Invalid height");
    let fps: u32 = matches.value_of("fps").unwrap().parse().expect("Invalid fps");
    let duration: u32 = matches.value_of("duration").unwrap().parse().expect("Invalid duration");

    // Create an invisible OpenGL context
    let el = glutin::event_loop::EventLoop::new();
    let wb = WindowBuilder::new().with_visible(false);
    let windowed_context = ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(wb, &el)
        .unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    // Load OpenGL functions
    gl_safe!(gl::load_with(|symbol| windowed_context.get_proc_address(symbol) as *const _), "loading OpenGL functions");

    // Load and compile vertex shader source from constant
    let vertex_shader_src = "#version 330 core\nlayout (location = 0) in vec3 aPos;\nvoid main() {\ngl_Position = vec4(aPos, 1.0);\n}";
    let vs = compile_shader(vertex_shader_src, gl::VERTEX_SHADER);

    // Load and compile fragment shader source from file
    let fs_src = std::fs::read_to_string(shader_path).expect("Failed to read shader file");
    let fs = compile_shader(&fs_src, gl::FRAGMENT_SHADER);

    // Create a program and attach the fragment shader
    let program = gl_safe!(gl::CreateProgram(), "create program");
    gl_safe!(gl::AttachShader(program, vs), "attach vertex shader: link vertex shader to program");
    gl_safe!(gl::AttachShader(program, fs), "attach fragment shader: link fragment shader to program");
    gl_safe!(gl::LinkProgram(program), "link program: link all attached shaders");
    gl_safe!(gl::UseProgram(program), "use program: activate the shader program");

    // Check for linking errors
    let mut success = gl::FALSE as GLint;
    gl_safe!(gl::GetProgramiv(program, gl::LINK_STATUS, &mut success), "check link status: verify program linking success");
    if success != gl::TRUE as GLint {
        let mut len = 0;
        gl_safe!(gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len), "get program info log length: determine length of linking log");
        let mut buffer = vec![0u8; len as usize];
        gl_safe!(gl::GetProgramInfoLog(program, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut GLchar), "get program info log: retrieve linking log");
        
        panic!("Program linking failed: {}. Verify that all shaders are correctly attached and compiled.", str::from_utf8(&buffer).unwrap());
    }

    // Use the shader program
    gl_safe!(gl::UseProgram(program), "use shader program");

    // Create a framebuffer
    let mut framebuffer = 0;
    gl_safe!(gl::GenFramebuffers(1, &mut framebuffer), "generate framebuffer: create a new framebuffer object");
    gl_safe!(gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer), "bind framebuffer: set the framebuffer as active");

    // Create a texture to render to
    let mut texture = 0;
    gl_safe!(gl::GenTextures(1, &mut texture), "generate texture: create a new texture object");
    gl_safe!(gl::BindTexture(gl::TEXTURE_2D, texture), "bind texture: set the texture as active");
    gl_safe!(gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, width as i32, height as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, ptr::null()), "create texture image: allocate storage for texture");
    gl_safe!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32), "set texture min filter: define texture minification filter");
    gl_safe!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32), "set texture mag filter: define texture magnification filter");
    gl_safe!(gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture, 0), "attach texture to framebuffer: link texture to framebuffer");

    // Check if framebuffer is complete
    if gl_safe!(gl::CheckFramebufferStatus(gl::FRAMEBUFFER), "check framebuffer status: verify framebuffer completeness") != gl::FRAMEBUFFER_COMPLETE {
        panic!("Framebuffer is not complete. Check framebuffer attachments and ensure they are correctly configured.");
    }

    // Set iResolution uniform
    let i_resolution_cstr = CString::new("iResolution").unwrap();
    let i_resolution_loc = gl_safe!(gl::GetUniformLocation(program, i_resolution_cstr.as_ptr()), "get iResolution location: find uniform location");
    if i_resolution_loc == -1 {
        panic!("Failed to get uniform location for iResolution. Ensure the uniform variable is declared in the shader.");
    }
    gl_safe!(gl::Uniform3f(i_resolution_loc, width as f32, height as f32, 0.0), "set iResolution uniform: set uniform value");

    // Create a vector for pixels once before the loop
    let mut pixels = vec![0u8; (width * height * 3) as usize];

    // Create and configure a vertex buffer for the rectangle
    let vertices: [f32; 12] = [
        -1.0, -1.0, 0.0,
         1.0, -1.0, 0.0,
         1.0,  1.0, 0.0,
        -1.0,  1.0, 0.0,
    ];

    let mut vbo = 0;
    let mut vao = 0;
    gl_safe!(gl::GenVertexArrays(1, &mut vao), "generating VAO");
    gl_safe!(gl::GenBuffers(1, &mut vbo), "generating VBO");

    gl_safe!(gl::BindVertexArray(vao), "binding VAO");
    gl_safe!(gl::BindBuffer(gl::ARRAY_BUFFER, vbo), "binding VBO");
    gl_safe!(gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr, vertices.as_ptr() as *const _, gl::STATIC_DRAW), "buffering vertex data");
    gl_safe!(gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * std::mem::size_of::<f32>() as GLsizei, ptr::null()), "setting vertex attrib pointer");
    gl_safe!(gl::EnableVertexAttribArray(0), "enabling vertex attrib array");

    // Main rendering loop
    for frame in 0..(fps * duration) {
        let i_time = frame as f32 / fps as f32;
        let i_time_cstr = CString::new("iTime").unwrap();
        let i_time_loc = gl_safe!(gl::GetUniformLocation(program, i_time_cstr.as_ptr()), "getting uniform location for iTime");
        if i_time_loc == -1 {
            panic!("Failed to get uniform location for iTime. Ensure the uniform variable is declared in the shader.");
        }
        gl_safe!(gl::Uniform1f(i_time_loc, i_time), "setting uniform value for iTime");

        // Render to the framebuffer
        gl_safe!(gl::Viewport(0, 0, width as i32, height as i32), "setting viewport");
        gl_safe!(gl::ClearColor(0.0, 0.0, 0.0, 1.0), "setting clear color");
        gl_safe!(gl::Clear(gl::COLOR_BUFFER_BIT), "clearing framebuffer");

        // Render the rectangle
        gl_safe!(gl::BindVertexArray(vao), "binding vertex array");
        gl_safe!(gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4), "drawing arrays");

        // Read pixels from the framebuffer
        gl_safe!(gl::ReadPixels(0, 0, width as i32, height as i32, gl::RGB, gl::UNSIGNED_BYTE, pixels.as_mut_ptr() as *mut _), "reading pixels");

        // Write pixels to stdout
        io::stdout().write_all(&pixels).unwrap();
    }
}
