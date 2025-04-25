#![deny(clippy::all, clippy::pedantic, clippy::nursery, clippy::perf)]
#![allow(clippy::module_name_repetitions)]
#![feature(const_pointer_is_aligned)]
#![feature(negative_impls)]

pub mod shaders;
pub mod utils;

pub mod colorspace;
pub mod dispatch;
pub mod gpu;
pub mod log;
pub mod options;
pub mod renderer;
pub mod shaders_root;
pub mod vulkan;

#[cfg(test)]
mod tests {
  use foreign_types::{foreign_type, ForeignType, ForeignTypeRef};

  pub struct Bar {
    level: u8,
  }

  foreign_type! {
      pub unsafe type Foo: Sync + Send {
          type CType = Bar;
          fn drop = |_| {};
      }
  }

  impl Default for Foo {
    fn default() -> Self {
      unsafe { Self::from_ptr(Box::into_raw(Box::new(Bar { level: 9 }))) }
    }
  }

  impl FooRef {
    pub fn level(&self) -> u8 {
      unsafe { (*self.as_ptr()).level }
    }
  }

  fn assert_thru_ref(x: &FooRef) {
    assert_eq!(x.level(), 9);
  }

  #[test]
  fn insane() {
    let log = Foo::default();
    assert!(!log.as_ptr().is_null());
    assert_thru_ref(&log);
    assert!(!log.as_ptr().is_null());
  }
}
