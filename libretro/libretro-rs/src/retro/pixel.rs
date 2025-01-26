pub mod format {
  use std::marker::PhantomData;

  #[derive(Debug, PartialEq, Eq, Hash)]
  pub struct ActiveFormat<P>(pub(crate) PhantomData<P>);

  mod private {
    pub trait Sealed {}
  }

  pub trait Format: private::Sealed {}

  pub use orgb1555::*;
  mod orgb1555 {
    use super::private::Sealed;
    use crate::retro::pixel::format::Format;
    use arbitrary_int::u5;
    use bitbybit::bitfield;

    #[bitfield(u16, default: 0)]
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub struct ORGB1555 {
      #[bits(10..=14, rw)]
      r: u5,
      #[bits(5..=9, rw)]
      g: u5,
      #[bits(0..=4, rw)]
      b: u5,
    }

    impl Sealed for ORGB1555 {}
    impl Format for ORGB1555 {}
  }

  pub use xrgb8888::*;
  mod xrgb8888 {
    use super::private::Sealed;
    use crate::retro::pixel::format::Format;
    use bitbybit::bitfield;

    #[bitfield(u32, default: 0)]
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub struct XRGB8888 {
      #[bits(24..=31, rw)]
      x: u8,
      #[bits(16..=23, rw)]
      r: u8,
      #[bits(8..=15, rw)]
      g: u8,
      #[bits(0..=7, rw)]
      b: u8,
    }

    impl Sealed for XRGB8888 {}
    impl Format for XRGB8888 {}
  }

  pub use rgb565::*;
  mod rgb565 {
    use super::private::Sealed;
    use crate::retro::pixel::format::Format;
    use arbitrary_int::{u5, u6};
    use bitbybit::bitfield;

    #[bitfield(u16, default: 0)]
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub struct RGB565 {
      #[bits(11..=15, rw)]
      r: u5,
      #[bits(5..=10, rw)]
      g: u6,
      #[bits(0..=4, rw)]
      b: u5,
    }

    impl Sealed for RGB565 {}
    impl Format for RGB565 {}
  }
}
