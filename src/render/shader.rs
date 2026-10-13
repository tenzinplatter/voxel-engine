use gl33::{global_loader::*, *};
use glam::{Mat4, Vec3};

pub(crate) trait ShaderUniformType<T> {
    /// Sets a uniform value in the given shader program.
    fn set_uniform(&self, name: &str, val: T);
}

/// The types of shader object.
enum ShaderType {
    /// Vertex shaders determine the position of geometry within the screen.
    Vertex = GL_VERTEX_SHADER.0 as isize,
    /// Fragment shaders determine the color output of geometry.
    ///
    /// Also other values, but mostly color.
    Fragment = GL_FRAGMENT_SHADER.0 as isize,
}
/// A handle to a [Shader
/// Object](https://www.khronos.org/opengl/wiki/GLSL_Object#Shader_objects)
struct Shader(u32);

/// A handle to a [Program
/// Object](https://www.khronos.org/opengl/wiki/GLSL_Object#Program_objects)
pub(crate) struct ShaderProgram(u32);

impl ShaderProgram {
    /// Allocates a new program object.
    ///
    /// Prefer [`ShaderProgram::from_vert_frag`](ShaderProgram::from_vert_frag),
    /// it makes a complete program from the vertex and fragment sources all at
    /// once.
    fn new() -> Option<Self> {
        let prog = glCreateProgram();
        if prog != 0 { Some(Self(prog)) } else { None }
    }

    /// Attaches a shader object to this program object.
    fn attach_shader(&self, shader: &Shader) {
        glAttachShader(self.0, shader.0);
    }

    /// Links the various attached, compiled shader objects into a usable program.
    fn link_program(&self) {
        glLinkProgram(self.0);
    }

    /// Checks if the last linking operation was successful.
    fn link_success(&self) -> bool {
        let mut success = 0;
        unsafe { glGetProgramiv(self.0, GL_LINK_STATUS, &mut success) };
        success == GL_TRUE.0 as i32
    }

    /// Gets the log data for this program.
    ///
    /// This is usually used to check the message when a program failed to link.
    fn info_log(&self) -> String {
        let mut needed_len = 0;
        unsafe { glGetProgramiv(self.0, GL_INFO_LOG_LENGTH, &mut needed_len) };
        let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
        let mut len_written = 0_i32;
        unsafe {
            glGetProgramInfoLog(
                self.0,
                v.capacity().try_into().unwrap(),
                &mut len_written,
                v.as_mut_ptr().cast(),
            );
            v.set_len(len_written.try_into().unwrap());
        }
        String::from_utf8_lossy(&v).into_owned()
    }

    /// Sets the program as the program to use when drawing.
    pub(crate) fn use_program(&self) {
        glUseProgram(self.0);
    }

    /// Marks the program for deletion.
    ///
    /// Note: This _does not_ immediately delete the program. If the program is
    /// currently in use it won't be deleted until it's not the active program.
    /// When a program is finally deleted and attached shaders are unattached.
    fn delete(self) {
        glDeleteProgram(self.0);
    }

    /// Takes a vertex shader source string and a fragment shader source string
    /// and either gets you a working program object or gets you an error message.
    ///
    /// This is the preferred way to create a simple shader program in the common
    /// case. It's just less error prone than doing all the steps yourself.
    pub(crate) fn from_vert_frag(vert: &str, frag: &str) -> Result<Self, String> {
        let p = Self::new().ok_or_else(|| "Couldn't allocate a program".to_string())?;
        let v = Shader::from_source(ShaderType::Vertex, vert)
            .map_err(|e| format!("Vertex Compile Error: {}", e))?;
        let f = Shader::from_source(ShaderType::Fragment, frag)
            .map_err(|e| format!("Fragment Compile Error: {}", e))?;
        p.attach_shader(&v);
        p.attach_shader(&f);
        p.link_program();
        v.delete();
        f.delete();
        if p.link_success() {
            Ok(p)
        } else {
            let out = format!("Program Link Error: {}", p.info_log());
            p.delete();
            Err(out)
        }
    }
}

impl ShaderUniformType<Vec3> for ShaderProgram {
    fn set_uniform(&self, name: &str, val: Vec3) {
        let cstr = std::ffi::CString::new(name).unwrap();
        unsafe {
            let location = glGetUniformLocation(self.0, cstr.as_bytes().as_ptr());
            glUniform3f(location, val.x, val.y, val.z);
        }
    }
}

impl ShaderUniformType<Mat4> for ShaderProgram {
    fn set_uniform(&self, name: &str, val: Mat4) {
        let cstr = std::ffi::CString::new(name).unwrap();
        unsafe {
            let location = glGetUniformLocation(self.0, cstr.as_bytes().as_ptr());
            glUniformMatrix4fv(location, 1, GL_FALSE.0 as u8, val.as_ref().as_ptr());
        }
    }
}

impl Shader {
    /// Makes a new shader.
    ///
    /// Prefer the [`Shader::from_source`](Shader::from_source) method.
    ///
    /// Possibly skip the direct creation of the shader object and use
    /// [`ShaderProgram::from_vert_frag`](ShaderProgram::from_vert_frag).
    fn new(ty: ShaderType) -> Option<Self> {
        let shader = glCreateShader(GLenum(ty as u32));
        if shader != 0 {
            Some(Self(shader))
        } else {
            None
        }
    }

    /// Assigns a source string to the shader.
    ///
    /// Replaces any previously assigned source.
    fn set_source(&self, src: &str) {
        unsafe {
            glShaderSource(
                self.0,
                1,
                &(src.as_bytes().as_ptr().cast()),
                &(src.len().try_into().unwrap()),
            );
        }
    }

    /// Compiles the shader based on the current source.
    fn compile(&self) {
        glCompileShader(self.0);
    }

    /// Checks if the last compile was successful or not.
    fn compile_success(&self) -> bool {
        let mut compiled = 0;
        unsafe { glGetShaderiv(self.0, GL_COMPILE_STATUS, &mut compiled) };
        compiled == GL_TRUE.0 as i32
    }

    /// Gets the info log for the shader.
    ///
    /// Usually you use this to get the compilation log when a compile failed.
    fn info_log(&self) -> String {
        let mut needed_len = 0;
        unsafe { glGetShaderiv(self.0, GL_INFO_LOG_LENGTH, &mut needed_len) };
        let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
        let mut len_written = 0_i32;
        unsafe {
            glGetShaderInfoLog(
                self.0,
                v.capacity().try_into().unwrap(),
                &mut len_written,
                v.as_mut_ptr().cast(),
            );
            v.set_len(len_written.try_into().unwrap());
        }
        String::from_utf8_lossy(&v).into_owned()
    }

    /// Marks a shader for deletion.
    ///
    /// Note: This _does not_ immediately delete the shader. It only marks it for
    /// deletion. If the shader has been previously attached to a program then the
    /// shader will stay allocated until it's unattached from that program.
    fn delete(self) {
        glDeleteShader(self.0);
    }

    /// Takes a shader type and source string and produces either the compiled
    /// shader or an error message.
    ///
    /// Prefer [`ShaderProgram::from_vert_frag`](ShaderProgram::from_vert_frag),
    /// it makes a complete program from the vertex and fragment sources all at
    /// once.
    fn from_source(ty: ShaderType, source: &str) -> Result<Self, String> {
        let id = Self::new(ty).ok_or_else(|| "Couldn't allocate new shader".to_string())?;
        id.set_source(source);
        id.compile();
        if id.compile_success() {
            Ok(id)
        } else {
            let out = id.info_log();
            id.delete();
            Err(out)
        }
    }
}
