use gl::types::*;
use std::ffi::CString;
use std::ptr;
use std::str;
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use std::io::{self, Write};
use clap::{App, Arg};

macro_rules! gl_safe {
    ($block:block) => {
        gl_safe!($block, "")
    };
    ($block:block, $step_name:expr) => {{
        let result = unsafe { (|| $block)() };
        unsafe {
            let err_code = gl::GetError();
            if err_code != gl::NO_ERROR {
                if $step_name.is_empty() {
                    eprintln!("OpenGL error code {}", err_code);
                    panic!("OpenGL error code {}", err_code);
                } else {
                    eprintln!("OpenGL error code {} at {}", err_code, $step_name);
                    panic!("OpenGL error code {} at {}", err_code, $step_name);
                }
            }
        }
        result
    }};
}

// Same as gl_safe, but without unsafe block
macro_rules! gl_saint {
    ($block:block) => {
        gl_saint!($block, "")
    };
    ($block:block, $step_name:expr) => {{
        let result = (|| $block)();
        unsafe {
            let err_code = gl::GetError();
            if err_code != gl::NO_ERROR {
                if $step_name.is_empty() {
                    eprintln!("OpenGL error code {}", err_code);
                    panic!("OpenGL error code {}", err_code);
                } else {
                    eprintln!("OpenGL error code {} at {}", err_code, $step_name);
                    panic!("OpenGL error code {} at {}", err_code, $step_name);
                }
            }
        }
        result
    }};
}

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader = gl_safe!({ gl::CreateShader(ty) }, "creating shader");
    let c_str = CString::new(src.as_bytes()).unwrap();
    gl_safe!({ gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null()) }, "setting shader source");
    gl_safe!({ gl::CompileShader(shader) }, "compiling shader");

    // Check for compilation errors
    let mut success = gl::FALSE as GLint;
    gl_safe!({ gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success) }, "checking shader compile status");
    if success != gl::TRUE as GLint {
        let mut len = 0;
        gl_safe!({ gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len) }, "getting shader info log length");
        let mut buffer = vec![0u8; len as usize];
        gl_safe!({ gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut GLchar) }, "getting shader info log");
        
        eprintln!("Shader compilation failed: {}", str::from_utf8(&buffer).unwrap());
        panic!("Shader compilation failed");
    }
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
    gl_saint!({ gl::load_with(|symbol| windowed_context.get_proc_address(symbol) as *const _); }, "loading OpenGL functions");

    // Load fragment shader source from file
    let fs_src = std::fs::read_to_string(shader_path).expect("Failed to read shader file");

    // Compile fragment shader
    let fs = compile_shader(&fs_src, gl::FRAGMENT_SHADER);

    // Create a program and attach the fragment shader
    let program = gl_safe!({ gl::CreateProgram() }, "creating program");
    gl_safe!({ gl::AttachShader(program, fs); }, "attaching shader to program");
    gl_safe!({ gl::LinkProgram(program); }, "linking program");

    // Check for linking errors
    let mut success = gl::FALSE as GLint;
    gl_safe!({ gl::GetProgramiv(program, gl::LINK_STATUS, &mut success); }, "checking program link status");
    if success != gl::TRUE as GLint {
        let mut len = 0;
        gl_safe!({ gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len); }, "getting program info log length");
        let mut buffer = vec![0u8; len as usize];
        gl_safe!({ gl::GetProgramInfoLog(program, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut GLchar); }, "getting program info log");
        eprintln!("Program linking failed: {}", str::from_utf8(&buffer).unwrap());
        panic!("Program linking failed");
    }

    // Use the shader program
    gl_safe!({ gl::UseProgram(program); }, "using shader program");

    // Create a framebuffer
    let mut framebuffer = 0;
    gl_safe!({ gl::GenFramebuffers(1, &mut framebuffer); }, "generating framebuffer");
    gl_safe!({ gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer); }, "binding framebuffer");

    // Create a texture to render to
    let mut texture = 0;
    gl_safe!({ gl::GenTextures(1, &mut texture); }, "generating texture");
    gl_safe!({ gl::BindTexture(gl::TEXTURE_2D, texture); }, "binding texture");
    gl_safe!({ gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, width as i32, height as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, ptr::null()); }, "creating texture image");
    gl_safe!({ gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32); }, "setting texture min filter");
    gl_safe!({ gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32); }, "setting texture mag filter");
    gl_safe!({ gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture, 0); }, "attaching texture to framebuffer");

    // Check if framebuffer is complete
    gl_safe!({
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            eprintln!("Framebuffer is not complete");
            panic!("Framebuffer is not complete");
        }
    }, "checking framebuffer status");

    // Set iResolution uniform
    let i_resolution_cstr = CString::new("iResolution").unwrap();
    let i_resolution_loc = gl_safe!({ gl::GetUniformLocation(program, i_resolution_cstr.as_ptr()) }, "getting uniform location for iResolution");
    if i_resolution_loc == -1 {
        eprintln!("Failed to get uniform location for iResolution");
        panic!("Failed to get uniform location for iResolution");
    }
    gl_safe!({ gl::Uniform2f(i_resolution_loc, width as f32, height as f32); }, "setting iResolution uniform");

    // Создаем вектор для пикселей один раз до начала цикла
    let mut pixels = vec![0u8; (width * height * 3) as usize];

    // Создаем и настраиваем буфер вершин для прямоугольника
    let vertices: [f32; 12] = [
        -1.0, -1.0, 0.0,
         1.0, -1.0, 0.0,
         1.0,  1.0, 0.0,
        -1.0,  1.0, 0.0,
    ];

    let mut vbo = 0;
    let mut vao = 0;
    gl_safe!({ gl::GenVertexArrays(1, &mut vao); }, "generating VAO");
    gl_safe!({ gl::GenBuffers(1, &mut vbo); }, "generating VBO");

    gl_safe!({ gl::BindVertexArray(vao); }, "binding VAO");
    gl_safe!({ gl::BindBuffer(gl::ARRAY_BUFFER, vbo); }, "binding VBO");
    gl_safe!({ gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr, vertices.as_ptr() as *const _, gl::STATIC_DRAW); }, "buffering vertex data");
    gl_safe!({ gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * std::mem::size_of::<f32>() as GLsizei, ptr::null()); }, "setting vertex attrib pointer");
    gl_safe!({ gl::EnableVertexAttribArray(0); }, "enabling vertex attrib array");

    // Main rendering loop
    for frame in 0..(fps * duration) {
        let i_time = frame as f32 / fps as f32;
        let i_time_cstr = CString::new("iTime").unwrap();
        let i_time_loc = gl_safe!({ gl::GetUniformLocation(program, i_time_cstr.as_ptr()) }, "getting uniform location for iTime");
        if i_time_loc == -1 {
            eprintln!("Failed to get uniform location for iTime");
            panic!("Failed to get uniform location for iTime");
        }
        gl_safe!({ gl::Uniform1f(i_time_loc, i_time); }, "setting uniform value for iTime");

        // Render to the framebuffer
        gl_safe!({ gl::Viewport(0, 0, width as i32, height as i32); }, "setting viewport");
        gl_safe!({ gl::ClearColor(0.0, 0.0, 0.0, 1.0); }, "setting clear color");
        gl_safe!({ gl::Clear(gl::COLOR_BUFFER_BIT); }, "clearing framebuffer");

        // Рендерим прямоугольник
        gl_safe!({ gl::BindVertexArray(vao); }, "binding vertex array");
        gl_safe!({ gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4); }, "drawing arrays");

        // Read pixels from the framebuffer
        gl_safe!({ gl::ReadPixels(0, 0, width as i32, height as i32, gl::RGB, gl::UNSIGNED_BYTE, pixels.as_mut_ptr() as *mut _); }, "reading pixels");

        // Check if all pixels are black
        if pixels.iter().all(|&p| p == 0) {
            eprintln!("All pixels are black");
            panic!("All pixels are black");
        }

        // Write pixels to stdout
        io::stdout().write_all(&pixels).unwrap();
    }
}
