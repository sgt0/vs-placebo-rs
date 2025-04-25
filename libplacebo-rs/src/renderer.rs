use foreign_types::{foreign_type, ForeignType};
use libplacebo_sys::{pl_gpu, pl_renderer, pl_renderer_create, pl_renderer_destroy};

use crate::log::Log;

foreign_type! {
  /// Thread-safety: unsafe.
  pub unsafe type Renderer
  {
    type CType = pl_renderer;
    fn drop = pl_renderer_destroy;
  }
}

impl Renderer {
  /// Creates a new renderer object, which is backed by a GPU context. This is a
  /// high-level object that takes care of the rendering chain as a whole, from
  /// the source textures to the finished frame.
  ///
  /// # Panics
  ///
  /// Will panic if `pl_renderer_create()` returns a null pointer.
  #[must_use]
  pub fn new(log: &Log, gpu: &pl_gpu) -> Self {
    assert!(!log.0.is_null());
    assert!(!gpu.is_null());

    let mut ptr = unsafe { pl_renderer_create(log.0, *gpu) };
    assert!(!ptr.is_null());
    unsafe { Self::from_ptr(&mut ptr) }
  }
}
