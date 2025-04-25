use std::ptr::NonNull;

use foreign_types::{foreign_type, ForeignType};
use libplacebo_sys::{
  pl_dispatch, pl_dispatch_begin, pl_dispatch_create, pl_dispatch_destroy, pl_dispatch_finish,
  pl_dispatch_params, pl_gpu,
};
use miette::{miette, Result};

use crate::{log::Log, shaders_root::Shader};

foreign_type! {
  pub unsafe type Dispatch: Send + Sync
  {
      type CType = pl_dispatch;
      fn drop = pl_dispatch_destroy;
  }
}

impl Dispatch {
  /// Creates a new shader dispatch object. This object provides a translation
  /// layer between generated shaders (`pl_shader`) and the ra context such that
  /// it can be used to execute shaders. This dispatch object will also provide
  /// shader caching (for efficient re-use).
  ///
  /// # Panics
  ///
  /// Will panic if `pl_dispatch_create()` returns a null pointer.
  #[must_use]
  pub fn new(log: &Log, gpu: &pl_gpu) -> Self {
    unsafe {
      let mut ptr = pl_dispatch_create(log.0, *gpu);
      assert!(!ptr.is_null());
      Self(NonNull::new_unchecked(&mut ptr))
    }
  }

  /// Returns a blank `pl_shader` object, suitable for recording rendering
  /// commands.
  ///
  /// # Panics
  ///
  /// Will panic if `pl_dispatch_begin()` returns a null pointer.
  #[must_use]
  pub fn begin(&self) -> Shader {
    unsafe {
      let shader = pl_dispatch_begin(*self.as_ptr());
      assert!(!shader.is_null());
      Shader::from_ptr(shader)
    }
  }

  /// Dispatch a generated shader (via the `pl_shader` mechanism).
  ///
  /// # Errors
  ///
  /// Will return `Err` if `pl_dispatch_finish()` is unsuccessful.
  ///
  /// # Panics
  ///
  /// TODO
  pub fn finish(&self, params: &pl_dispatch_params) -> Result<()> {
    debug_assert!(!params.shader.is_null());
    debug_assert!(!params.target.is_null());

    if unsafe { pl_dispatch_finish(*self.as_ptr(), params) } {
      Ok(())
    } else {
      Err(miette!("Failed to dispatch shader."))
    }
  }
}
