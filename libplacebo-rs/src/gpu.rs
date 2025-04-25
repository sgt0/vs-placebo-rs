use std::ptr::NonNull;

use libplacebo_sys::{pl_fmt_t, pl_tex, pl_tex_params, pl_tex_t};

#[derive(Clone)]
pub struct Tex(NonNull<pl_tex_t>);

impl Tex {
  /// # Safety
  ///
  /// TODO
  pub const unsafe fn new_unchecked(ptr: *mut pl_tex_t) -> Self {
    Self(NonNull::new_unchecked(ptr))
  }

  #[must_use]
  pub const fn as_ptr(&self) -> pl_tex {
    self.0.as_ptr()
  }

  /// # Panics
  ///
  /// TODO
  #[must_use]
  pub fn format(&self) -> pl_fmt_t {
    unsafe {
      let params = self.params();
      assert!(!params.format.is_null());
      *(params.format)
    }
  }

  #[must_use]
  pub fn num_components(&self) -> i32 {
    self.format().num_components
  }

  /// # Panics
  ///
  /// TODO
  #[must_use]
  pub fn params(&self) -> pl_tex_params {
    unsafe { (*self.as_ptr()).params }
  }

  /// # Panics
  ///
  /// TODO
  #[must_use]
  pub fn sampleable(&self) -> bool {
    self.params().sampleable
  }

  #[must_use]
  pub fn width(&self) -> i32 {
    self.params().w
  }
}
