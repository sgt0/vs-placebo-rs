use foreign_types::{foreign_type, ForeignType};
use libplacebo_sys::{pl_options, pl_options_alloc, pl_options_free};

use crate::log::Log;

foreign_type! {
  /// Options for configuring the behavior of a renderer.
  pub unsafe type Options
  {
    type CType = pl_options;
    fn drop = pl_options_free;
  }
}

impl Options {
  #[must_use]
  pub fn new(log: &Log) -> Self {
    unsafe { Self::from_ptr(&mut pl_options_alloc(log.0)) }
  }
}

impl Default for Options {
  fn default() -> Self {
    Self::new(&Log::default())
  }
}
