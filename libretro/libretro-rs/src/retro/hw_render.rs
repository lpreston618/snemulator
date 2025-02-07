use libretro_rs_ffi::{
  non_null_retro_hw_get_current_framebuffer_t, non_null_retro_hw_get_proc_address_t,
  retro_hw_context_type, retro_hw_render_callback,
};
use std::ffi::c_uint;

mod private {
  pub trait Sealed {}
}

pub trait RenderType: private::Sealed {}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SoftwareRenderEnabled(pub(crate) ());

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct GLRenderEnabled(pub(crate) ());

pub trait HWRenderEnabled: private::Sealed {}

impl private::Sealed for GLRenderEnabled {}
impl HWRenderEnabled for GLRenderEnabled {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GLContextCallbacks {
  pub get_proc_address_cb: non_null_retro_hw_get_proc_address_t,
  pub get_current_framebuffer_cb: non_null_retro_hw_get_current_framebuffer_t,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GLContextType {
  OpenGL2,
  OpenGLCore3_2,
  OpenGLCore3_3,
  OpenGLCore4_0,
  OpenGLCore4_1,
  OpenGLCore4_2,
  OpenGLCore4_3,
  OpenGLCore4_4,
  OpenGLCore4_5,
  OpenGLCore4_6,
  OpenGLES2,
  OpenGLES3,
}

struct GLVersion(retro_hw_context_type, u8, u8);

impl From<GLContextType> for GLVersion {
  fn from(value: GLContextType) -> Self {
    use retro_hw_context_type::*;
    use GLContextType::*;
    match value {
      OpenGL2 => Self(RETRO_HW_CONTEXT_OPENGL, 2, 0),
      OpenGLCore3_2 => Self(RETRO_HW_CONTEXT_OPENGL_CORE, 3, 2),
      OpenGLCore3_3 => Self(RETRO_HW_CONTEXT_OPENGL_CORE, 3, 3),
      OpenGLCore4_0 => Self(RETRO_HW_CONTEXT_OPENGL_CORE, 4, 0),
      OpenGLCore4_1 => Self(RETRO_HW_CONTEXT_OPENGL_CORE, 4, 1),
      OpenGLCore4_2 => Self(RETRO_HW_CONTEXT_OPENGL_CORE, 4, 2),
      OpenGLCore4_3 => Self(RETRO_HW_CONTEXT_OPENGL_CORE, 4, 3),
      OpenGLCore4_4 => Self(RETRO_HW_CONTEXT_OPENGL_CORE, 4, 4),
      OpenGLCore4_5 => Self(RETRO_HW_CONTEXT_OPENGL_CORE, 4, 5),
      OpenGLCore4_6 => Self(RETRO_HW_CONTEXT_OPENGL_CORE, 4, 6),
      OpenGLES2 => Self(RETRO_HW_CONTEXT_OPENGLES2, 2, 0),
      OpenGLES3 => Self(RETRO_HW_CONTEXT_OPENGLES3, 3, 0),
    }
  }
}

#[repr(transparent)]
pub struct GLOptions(retro_hw_render_callback);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GLBufferOptions {
  #[default]
  None,
  DepthOnly,
  DepthAndStencil,
}

impl GLOptions {
  pub fn new(gl_type: GLContextType) -> Self {
    GLOptions(retro_hw_render_callback::default()).set_gl_type(gl_type)
  }

  pub fn set_gl_type(mut self, gl_type: GLContextType) -> Self {
    let GLVersion(context_type, major, minor) = gl_type.into();
    self.0.context_type = context_type;
    self.0.version_major = major as c_uint;
    self.0.version_minor = minor as c_uint;
    self
  }

  pub fn set_bottom_left_origin(mut self, bottom_left_origin: bool) -> Self {
    self.0.bottom_left_origin = bottom_left_origin;
    self
  }

  pub fn set_buffer_options(mut self, buffers: GLBufferOptions) -> Self {
    match buffers {
      GLBufferOptions::None => {
        self.0.depth = false;
        self.0.stencil = false;
      }
      GLBufferOptions::DepthOnly => {
        self.0.depth = true;
        self.0.stencil = false;
      }
      GLBufferOptions::DepthAndStencil => {
        self.0.depth = true;
        self.0.stencil = true;
      }
    }
    self
  }

  pub fn set_cache_context(mut self, cache_context: bool) -> Self {
    self.0.cache_context = cache_context;
    self
  }

  pub fn set_debug_context(mut self, debug_context: bool) -> Self {
    self.0.debug_context = debug_context;
    self
  }
}

impl From<GLOptions> for retro_hw_render_callback {
  fn from(value: GLOptions) -> Self {
    value.0
  }
}
