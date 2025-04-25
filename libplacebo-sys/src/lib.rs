#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum pl_shader_obj_type {
  PL_SHADER_OBJ_INVALID = 0,
  PL_SHADER_OBJ_COLOR_MAP,
  PL_SHADER_OBJ_SAMPLER,
  PL_SHADER_OBJ_DITHER,
  PL_SHADER_OBJ_LUT,
  PL_SHADER_OBJ_AV1_GRAIN,
  PL_SHADER_OBJ_FILM_GRAIN,
  PL_SHADER_OBJ_RESHAPE,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pl_shader_obj_t {
  pub gpu: pl_gpu,
  pub r#type: pl_shader_obj_type,
}

#[deny(clippy::all, clippy::pedantic, clippy::nursery, clippy::perf)]
#[cfg(test)]
mod tests {
  use super::*;
  use std::ptr::null_mut;

  #[test]
  fn can_read_version() {
    assert_eq!(PL_MAJOR_VER, 7);
  }

  #[test]
  fn new_log() {
    let params = pl_log_params {
      log_cb: Some(pl_log_color),
      log_level: pl_log_level::PL_LOG_DEBUG,
      log_priv: null_mut(),
    };
    let mut ptr = unsafe { pl_log_create_349(1, &params) };
    assert!(!ptr.is_null());
    unsafe { pl_log_destroy(&mut ptr) };
    assert!(ptr.is_null());
  }
}
