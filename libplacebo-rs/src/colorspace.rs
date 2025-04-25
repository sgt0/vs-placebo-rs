use libplacebo_sys::{pl_bit_encoding, pl_color_repr, pl_color_system};

/// The underlying bit-wise representation of a color sample.
pub struct BitEncoding(pl_bit_encoding);

impl BitEncoding {
  /// A representational bit shift applied to the color.
  #[must_use]
  pub const fn bit_shift(mut self, x: i32) -> Self {
    self.0.bit_shift = x;
    self
  }

  /// The effective number of bits of the color information.
  #[must_use]
  pub const fn color_depth(mut self, x: i32) -> Self {
    self.0.color_depth = x;
    self
  }

  /// The number of bits the color is stored/sampled as.
  #[must_use]
  pub const fn sample_depth(mut self, x: i32) -> Self {
    self.0.sample_depth = x;
    self
  }
}

/// Describes the underlying color system and representation.
pub struct ColorRepr(pl_color_repr);

impl ColorRepr {
  // pub fn new() -> Self {
  //   Self(pl_color_repr {})
  // }

  #[must_use]
  pub const fn bits(mut self, x: &BitEncoding) -> Self {
    self.0.bits = x.0;
    self
  }

  #[must_use]
  pub const fn sys(mut self, x: pl_color_system) -> Self {
    self.0.sys = x;
    self
  }
}
