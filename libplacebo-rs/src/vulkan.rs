use std::ptr::NonNull;

use libplacebo_sys::{
  pl_fmt_t, pl_gpu, pl_plane, pl_plane_data, pl_plane_find_fmt, pl_tex_create, pl_tex_destroy,
  pl_tex_download, pl_tex_params, pl_tex_transfer_params, pl_tex_upload, pl_upload_plane,
  pl_vulkan, pl_vulkan_create, pl_vulkan_destroy, pl_vulkan_params,
};

use crate::{gpu::Tex, log::Log};
use miette::{miette, Result};

pub struct Vulkan(pub(crate) pl_vulkan);

impl Vulkan {
  /// # Panics
  ///
  /// Will panic if `pl_vulkan_create()` returns a null pointer.
  #[must_use]
  pub fn new(log: &Log, params: &pl_vulkan_params) -> Self {
    debug_assert!(!log.0.is_null());

    unsafe {
      let ptr = pl_vulkan_create(log.0, params);
      assert!(!ptr.is_null());
      Self(ptr)
    }
  }

  #[must_use]
  pub fn gpu(&self) -> pl_gpu {
    unsafe { (*self.0).gpu }
  }

  /// Helper function to find a suitable `pl_fmt` based on a `pl_plane_data`'s
  /// requirements. This is called internally by `pl_upload_plane`, but it's
  /// exposed to users both as a convenience and so they may preemptively check
  /// if a format would be supported without actually having to attempt the
  /// upload.
  #[must_use]
  pub fn plane_find_fmt(&self, data: &pl_plane_data) -> Option<NonNull<pl_fmt_t>> {
    unsafe {
      let format = pl_plane_find_fmt(self.gpu(), &mut 0, data);
      if format.is_null() {
        None
      } else {
        // Note that this *must* return the raw pointer, because the precise,
        // unchanged address is necessary later. libplacebo has a `PL_PRIV`
        // function that stores public and private structs next to each other in
        // memory, and it is used during Vulkan texture generation.
        // Some(format)

        Some(NonNull::new_unchecked(format.cast_mut()))
      }
    }
  }

  /// Create a texture (with undefined contents). This is assumed to be an
  /// expensive/rare operation, and may need to perform memory allocation or
  /// framebuffer creation.
  ///
  /// # Panics
  ///
  /// Will panic if `pl_tex_create()` returns a null pointer, which indicates a
  /// failure.
  #[must_use]
  pub fn tex_create(&self, params: &pl_tex_params) -> Tex {
    debug_assert!(!self.0.is_null());

    unsafe {
      let gpu = self.gpu();
      let tex = pl_tex_create(gpu, params);
      assert!(!tex.is_null());
      Tex::new_unchecked(tex.cast_mut())
    }
  }

  pub fn tex_destroy(&self, tex: &Tex) {
    unsafe {
      pl_tex_destroy(self.gpu(), &mut tex.as_ptr());
    }
  }

  /// # Errors
  ///
  /// TODO
  ///
  /// # Panics
  ///
  /// TODO
  pub fn tex_download(&self, params: &pl_tex_transfer_params) -> Result<()> {
    assert!(!self.0.is_null());
    assert!(!params.ptr.is_null());
    assert!(!params.tex.is_null());

    unsafe {
      let gpu = self.gpu();
      assert!(!gpu.is_null());
      let success = pl_tex_download(gpu, params);
      if success {
        Ok(())
      } else {
        Err(miette!("Failed to download texture."))
      }
    }
  }

  /// # Errors
  ///
  /// TODO
  pub fn tex_upload(&self, params: &pl_tex_transfer_params) -> Result<()> {
    unsafe {
      let gpu = self.gpu();
      let success = pl_tex_upload(gpu, params);
      if success {
        Ok(())
      } else {
        Err(miette!("Failed to upload texture."))
      }
    }
  }

  /// Upload an image plane to a texture. `tex` will be destroyed and
  /// reinitialized if it is incompatible incompatible.
  ///
  /// # Errors
  ///
  /// Will return `Err` if `pl_upload_plane()` is unsuccessful.
  ///
  /// # Panics
  ///
  /// Will panic if `gpu` or `tex` are null.
  pub fn upload_plane(&self, tex: &mut Tex, data: &pl_plane_data) -> Result<pl_plane> {
    assert!(!self.0.is_null());
    assert!(!tex.as_ptr().is_null());

    let mut plane = pl_plane::default();

    // HACK: why is this coercion like this?
    // let box_tex: *mut *const pl_tex_t = Box::into_raw(Box::new(tex.as_ptr()));

    let result = unsafe { pl_upload_plane(self.gpu(), &mut plane, &mut tex.as_ptr(), data) };
    if result {
      Ok(plane)
    } else {
      Err(miette!("Failed to upload plane."))
    }
  }
}

impl Drop for Vulkan {
  fn drop(&mut self) {
    unsafe {
      pl_vulkan_destroy(&mut self.0);
    }
  }
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn can_create_vulkan() {
    let log = Log::default();
    let _vk = Vulkan::new(&log, &pl_vulkan_params::default());
  }
}
