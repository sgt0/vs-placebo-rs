use std::{
  ffi::CStr,
  os::raw::{c_char, c_void},
};

use libplacebo_sys::{
  pl_log, pl_log_create_349, pl_log_destroy, pl_log_level, pl_log_params, PL_API_VER,
};

/// # Safety
///
/// TODO
///
/// # Panics
///
/// Panics if the given message cannot be converted to a Rust string.
pub unsafe extern "C" fn log_cb(_stream: *mut c_void, level: pl_log_level, msg: *const c_char) {
  let c_str = unsafe { CStr::from_ptr(msg) };
  println!("[libplacebo] [{:?}] {}", level, c_str.to_str().unwrap());
}

pub struct Log(pub(crate) pl_log);

unsafe impl Send for Log {}
unsafe impl Sync for Log {}

impl Log {
  /// # Panics
  ///
  /// TODO
  #[must_use]
  pub fn new(api_ver: i32) -> Self {
    unsafe {
      let params = pl_log_params {
        log_cb: Some(log_cb),
        log_level: pl_log_level::PL_LOG_ERR,
        ..pl_log_params::default()
      };
      let ptr = pl_log_create_349(api_ver, &params);
      assert!(!ptr.is_null());
      Self(ptr)
    }
  }

  #[must_use]
  pub fn log_level(&self) -> pl_log_level {
    unsafe { (*self.0).params.log_level }
  }
}

impl Default for Log {
  #[allow(clippy::cast_possible_wrap)]
  fn default() -> Self {
    Self::new(PL_API_VER as i32)
  }
}

impl Drop for Log {
  fn drop(&mut self) {
    unsafe {
      pl_log_destroy(&mut self.0);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn assert_trace_thru_ref(log: &Log) {
    assert_eq!(log.log_level(), pl_log_level::PL_LOG_ERR);
  }

  #[test]
  fn can_create_log() {
    let log = Log::default();
    assert!(!log.0.is_null());
    assert_trace_thru_ref(&log);
    assert!(!log.0.is_null());
  }
}
