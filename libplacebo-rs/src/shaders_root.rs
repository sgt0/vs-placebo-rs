use std::ptr::{null_mut, NonNull};

use foreign_types::{foreign_type, ForeignTypeRef};
use libplacebo_sys::{
  pl_deband_params, pl_dither_params, pl_gpu, pl_sample_src, pl_shader, pl_shader_alloc,
  pl_shader_deband, pl_shader_dither, pl_shader_free, pl_shader_obj, pl_shader_obj_destroy,
  pl_shader_obj_t, pl_shader_obj_type, pl_shader_params, pl_shader_reset,
};

use crate::log::Log;

pub struct Shader(pl_shader, pl_shader_obj);

impl Shader {
  /// Creates a new, blank, mutable `pl_shader` object.
  ///
  /// Note: Rather than allocating and destroying many shaders, users are
  /// encouraged to reuse them (using `pl_shader_reset`) for efficiency.
  ///
  /// # Panics
  ///
  /// Will panic if `pl_shader_alloc()` returns a null pointer.
  #[must_use]
  pub fn new(log: &Log, params: &pl_shader_params) -> Self {
    debug_assert!(!log.0.is_null());

    unsafe {
      let ptr = pl_shader_alloc(log.0, params);
      debug_assert!(!ptr.is_null());
      Self(ptr, null_mut())
    }
  }

  #[must_use]
  pub fn from_ptr(ptr: pl_shader) -> Self {
    debug_assert!(!ptr.is_null());
    Self(ptr, null_mut())
  }

  #[must_use]
  pub const fn as_ptr(&self) -> pl_shader {
    self.0
  }

  /// Resets a `pl_shader` to a blank slate, without releasing internal memory.
  /// If you're going to be re-generating shaders often, this function will let
  /// you skip the re-allocation overhead.
  pub fn reset(&mut self, params: &pl_shader_params) {
    debug_assert!(!params.gpu.is_null());

    unsafe {
      pl_shader_reset(self.as_ptr(), params);
    }
  }

  /// Debands a given texture.
  pub fn deband(&mut self, src: &pl_sample_src, params: &pl_deband_params) {
    debug_assert!(!src.tex.is_null());

    unsafe {
      pl_shader_deband(self.as_ptr(), src, params);
    }
  }

  /// Dither the colors to a lower depth, given in bits.
  pub fn dither(
    &mut self,
    new_depth: i32,
    dither_state: &ShaderObjectRef,
    params: &pl_dither_params,
  ) {
    // debug_assert!(!dither_state.is_null());

    unsafe {
      pl_shader_dither(self.as_ptr(), new_depth, &mut self.1, params);
    }
  }
}

// Let `pl_dispatch` free its own shaders.
// impl Drop for Shader {
//   fn drop(&mut self) {
//     println!("dropping Shader");
//     unsafe { pl_shader_free(&mut self.0) }
//   }
// }

foreign_type! {
  /// Shader objects represent abstract resources that shaders need to manage in
  /// order to ensure their operation. This could include shader storage
  /// buffers, generated lookup textures, or other sorts of configured state.
  /// The body of a shader object is fully opaque
  pub unsafe type ShaderObject
  {
      type CType = pl_shader_obj;
      fn drop = |x: *mut pl_shader_obj| if !x.is_null() { pl_shader_obj_destroy(x) };
  }
}

impl ShaderObject {
  #[must_use]
  pub fn new(gpu: pl_gpu, r#type: pl_shader_obj_type) -> Self {
    debug_assert!(!gpu.is_null());

    unsafe {
      let mut obj: pl_shader_obj = &mut pl_shader_obj_t { gpu, r#type };
      Self(NonNull::new_unchecked(&mut obj))
    }
  }
}
