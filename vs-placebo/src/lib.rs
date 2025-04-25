#![deny(clippy::all, clippy::pedantic, clippy::nursery, clippy::perf)]
#![allow(clippy::too_many_lines)]
#![feature(iterator_try_collect)]

mod deband;

use crate::deband::Filter as DebandFilter;
use const_str::cstr;
use vapoursynth4_rs::declare_plugin;

declare_plugin!(
  "sh.cosm.placebors",
  "placebors",
  "some description",
  (1, 0),
  VAPOURSYNTH_API_VERSION,
  0,
  (DebandFilter, None)
);
