use crate::retro;
use std::mem;
use std::slice::{ChunksExact, ChunksExactMut};

/// A video frame that can be passed to the libretro `retro_video_refresh_t`
/// callback, consisting of:
/// * a pixel buffer
/// * the width and height of the frame
/// * the pitch of the buffer (i.e. the gap between the start of two rows)
///
/// This trait is unsafe to implement since incorrectly reporting the size and
/// shape of the framebuffer may result in undefined behavior. You probably
/// don't need to implement this trait yourself. If you know your buffer's
/// dimensions at compile time, use an [ArrayFrameBuffer]; otherwise, use a
/// [SliceFrameBuffer]. Both types implement [Deref] and [DerefMut], so they can
/// be treated as an array or slice of pixels.
///
/// In order to implement this trait correctly, the following invariants must
/// be maintained:
/// * `self.data().len() == self.pitch() * self.height()`
/// * `self.width() * size_of::<Pixel>() <= self.pitch()`
///
/// Types implementing this trait should not allow direct mutable access to the
/// pixel buffer if its type is resizable; all mutation should be done through a
/// slice to prevent resizing. Implementors should also provide a consuming
/// `into_inner` method so the buffer can be resized and used to construct a new
/// instance.
pub unsafe trait FrameBuffer {
  /// The pixel format of the buffer.
  type Pixel: retro::pixel::format::Format;

  /// Returns a byte slice containing the frame buffer data. The data must be
  /// in the format specified by [Self::Pixel].
  fn data(&self) -> &[u8];

  /// Returns the width of the frame buffer, in pixels.
  fn width(&self) -> u16;

  /// Returns the height of the frame buffer, in pixels.
  fn height(&self) -> u16;

  /// Returns the width of the frame buffer, in bytes.
  ///
  /// The default implementation returns `width * size_of::<Pixel>()`.
  fn pitch(&self) -> usize {
    self.width() as usize * mem::size_of::<Self::Pixel>()
  }
}

/// A [FrameBuffer] that is always packed (i.e. `width == pitch * size_of::<Pixel>()`).
pub unsafe trait PackedFrameBuffer: FrameBuffer + AsRef<[Self::Pixel]> {
  /// Returns a slice containing all the frame buffer's pixels.
  fn pixels(&self) -> &[Self::Pixel] {
    self.as_ref()
  }

  /// Read-only iterator over the rows of pixels in the buffer. The slices are
  /// guaranteed to have the same length as the framebuffer's width.
  fn rows(&self) -> ChunksExact<'_, Self::Pixel> {
    self.as_ref().chunks_exact(self.pitch())
  }
}

/// A packed [FrameBuffer] that allows mutation.
pub unsafe trait PackedFrameBufferMut: PackedFrameBuffer + AsMut<[Self::Pixel]> {
  /// Returns a mutable slice containing the frame buffer data.
  fn pixels_mut(&mut self) -> &mut [Self::Pixel] {
    self.as_mut()
  }

  /// Mutable iterator over the rows of pixels in the buffer. The slices are
  /// guaranteed to have the same length as the framebuffer's width.
  fn rows_mut(&mut self) -> ChunksExactMut<'_, Self::Pixel> {
    let pitch = self.pitch();
    self.as_mut().chunks_exact_mut(pitch)
  }
}

pub use err::*;
mod err {
  use crate::retro::error::CoreError;
  use thiserror::Error;

  #[derive(Debug, Error)]
  #[error("invalid combination of framebuffer size, width, height and/or pitch")]
  pub struct FrameBufferError(pub(crate) ());

  impl From<FrameBufferError> for CoreError {
    fn from(_: FrameBufferError) -> Self {
      CoreError::new()
    }
  }
}

pub use array::ArrayFrameBuffer;
mod array {
  use super::{FrameBuffer, PackedFrameBuffer, PackedFrameBufferMut};
  use crate::retro::pixel;
  use std::fmt::Debug;
  use std::ops::{Deref, DerefMut};

  /// A frame buffer whose dimensions are known at compile time. Automatically
  /// dereferences to the array it wraps.
  ///
  /// # Examples
  /// ```
  /// use libretro_rs::retro::pixel::format::XRGB8888;
  /// use libretro_rs::retro::video::ArrayFrameBuffer;
  /// let mut buf = ArrayFrameBuffer::<XRGB8888, {320*240}, 240>::default();
  /// // Set the top left pixel to blue.
  /// buf[0] = XRGB8888::new_with_raw_value(0x000000FF);
  /// ```
  #[repr(transparent)]
  #[derive(Clone, Debug)]
  pub struct ArrayFrameBuffer<P, const LEN: usize, const W: u16>([P; LEN]);

  impl<P, const LEN: usize, const W: u16> ArrayFrameBuffer<P, LEN, W>
  where
    P: pixel::format::Format,
  {
    /// The height of the framebuffer.
    pub const HEIGHT: u16 = (LEN as u32 / W as u32) as u16;

    /// Creates a new frame buffer from an array of pixels. If the array length
    /// isn't divisible by `W`, a compile time error will occur.
    ///
    /// # Examples
    /// ```
    /// use libretro_rs::retro::pixel::format::XRGB8888;
    /// use libretro_rs::retro::video::ArrayFrameBuffer;
    /// _ = ArrayFrameBuffer::<_, {512*512}, 512>::new(Box::new([XRGB8888::default(); 512*512]));
    /// ```
    pub const fn new(pixels: [P; LEN]) -> Self {
      // This is a workaround to perform validation on const generic parameters.
      // Since using a const param from an outer scope in an expression isn't
      // allowed, the validation is performed while defining an associated
      // constant for a trait this type implements.
      // See https://stackoverflow.com/a/72588268
      _ = <Self as ValidFramebuffer>::IS_VALID;
      Self(pixels)
    }

    /// Consumes this [FrameBuffer], returning the wrapped pixel buffer.
    pub fn into_inner(self) -> [P; LEN] {
      self.0
    }
  }

  unsafe impl<P, const LEN: usize, const W: u16> FrameBuffer for ArrayFrameBuffer<P, LEN, W>
  where
    P: pixel::format::Format,
  {
    type Pixel = P;

    fn data(&self) -> &[u8] {
      super::as_bytes(&self.0[..])
    }

    fn width(&self) -> u16 {
      W
    }

    fn height(&self) -> u16 {
      Self::HEIGHT
    }
  }

  unsafe impl<P, const LEN: usize, const W: u16> PackedFrameBuffer for ArrayFrameBuffer<P, LEN, W> where
    P: pixel::format::Format
  {
  }

  unsafe impl<P, const LEN: usize, const W: u16> PackedFrameBufferMut for ArrayFrameBuffer<P, LEN, W> where
    P: pixel::format::Format
  {
  }

  impl<P, const LEN: usize, const W: u16> Deref for ArrayFrameBuffer<P, LEN, W> {
    type Target = [P; LEN];

    fn deref(&self) -> &Self::Target {
      &self.0
    }
  }

  impl<P, const LEN: usize, const W: u16> DerefMut for ArrayFrameBuffer<P, LEN, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
      &mut self.0
    }
  }

  impl<P, const LEN: usize, const W: u16> AsRef<[P]> for ArrayFrameBuffer<P, LEN, W> {
    fn as_ref(&self) -> &[P] {
      &self.0[..]
    }
  }

  impl<P, const LEN: usize, const W: u16> AsMut<[P]> for ArrayFrameBuffer<P, LEN, W> {
    fn as_mut(&mut self) -> &mut [P] {
      &mut self.0[..]
    }
  }

  impl<P, const LEN: usize, const W: u16> Into<[P; LEN]> for ArrayFrameBuffer<P, LEN, W> {
    fn into(self) -> [P; LEN] {
      self.0
    }
  }

  impl<P, const LEN: usize, const W: u16> Default for ArrayFrameBuffer<P, LEN, W>
  where
    P: pixel::format::Format + Copy + Default,
  {
    fn default() -> Self {
      Self::new([P::default(); LEN])
    }
  }

  unsafe trait ValidFramebuffer {
    const IS_VALID: ();
  }

  unsafe impl<P, const LEN: usize, const W: u16> ValidFramebuffer for ArrayFrameBuffer<P, LEN, W> {
    const IS_VALID: () = {
      assert!(
        LEN % W as usize == 0,
        "ArrayFramebuffer length must be evenly divisible by its width."
      );
      assert!(
        LEN / W as usize <= u16::MAX as usize,
        "ArrayFramebuffer (LEN/W) must fit in a u16."
      );
    };
  }
}

pub use packed::SliceFrameBuffer;
mod packed {
  use super::{
    FrameBuffer, FrameBufferError, PackedFrameBuffer, PackedFrameBufferMut, PixelBuffer,
  };
  use std::ops::{Deref, DerefMut};

  /// A frame buffer backed by a slice of pixels. Automatically dereferences to
  /// a slice of pixels.
  ///
  /// # Examples
  /// ```
  /// use libretro_rs::prelude::*;
  ///
  /// fn frame_buffer_example() -> Result<(), CoreError> {
  ///   let pixels = vec![XRGB8888::default(); 320*240];
  ///   // Don't use unwrap in your code; libretro cores must not panic.
  ///   let mut buffer = SliceFrameBuffer::with_width(pixels, 240)?;
  ///   // Set the top left pixel to red.
  ///   buffer[0] = XRGB8888::new_with_raw_value(0x00FF0000);
  ///   Ok(())
  /// }
  ///
  /// assert!(frame_buffer_example().is_ok())
  /// ```
  #[derive(Clone, Debug)]
  pub struct SliceFrameBuffer<T> {
    buffer: T,
    width: u16,
  }

  impl<T> SliceFrameBuffer<T>
  where
    T: PixelBuffer,
  {
    /// Returns a new frame buffer with the given width; the height is derived
    /// from the length of the buffer. Returns an error if the buffer length
    /// isn't divisible by `width` or the height would not fit in a `u16`.
    pub fn with_width(buffer: T, width: u16) -> Result<Self, FrameBufferError> {
      let (height, remainder) = (
        buffer.as_ref().len() / width as usize,
        buffer.as_ref().len() % width as usize,
      );
      if remainder != 0 || u16::try_from(height).is_err() {
        return Err(FrameBufferError(()));
      }
      Ok(Self { buffer, width })
    }

    /// Returns a shared reference to the underlying pixel buffer.
    pub fn buffer(&self) -> &T {
      &self.buffer
    }

    // buffer_mut is intentionally omitted to prevent resizing

    /// Consumes this frame buffer and returns the underlying pixel buffer.
    pub fn into_inner(self) -> T {
      self.buffer
    }
  }

  unsafe impl<T> FrameBuffer for SliceFrameBuffer<T>
  where
    T: PixelBuffer,
  {
    type Pixel = T::Pixel;

    fn data(&self) -> &[u8] {
      super::as_bytes(self.buffer.as_ref())
    }

    fn width(&self) -> u16 {
      self.width
    }

    fn height(&self) -> u16 {
      // The height is guaranteed to fit in a u16, therefore
      // buffer.len() <= u16::MAX * u16::MAX < u32::MAX
      (self.buffer.as_ref().len() as u32 / self.width as u32) as u16
    }
  }

  unsafe impl<T> PackedFrameBuffer for SliceFrameBuffer<T> where T: PixelBuffer {}

  unsafe impl<T> PackedFrameBufferMut for SliceFrameBuffer<T> where
    T: PixelBuffer + AsMut<[Self::Pixel]>
  {
  }

  impl<T> Deref for SliceFrameBuffer<T>
  where
    T: PixelBuffer,
  {
    type Target = [<Self as FrameBuffer>::Pixel];

    fn deref(&self) -> &Self::Target {
      self.buffer.as_ref()
    }
  }

  impl<T> DerefMut for SliceFrameBuffer<T>
  where
    T: PixelBuffer + AsMut<[T::Pixel]>,
  {
    fn deref_mut(&mut self) -> &mut Self::Target {
      self.buffer.as_mut()
    }
  }

  impl<T> AsRef<[T::Pixel]> for SliceFrameBuffer<T>
  where
    T: PixelBuffer,
  {
    fn as_ref(&self) -> &[T::Pixel] {
      self.buffer.as_ref()
    }
  }

  impl<T> AsMut<[T::Pixel]> for SliceFrameBuffer<T>
  where
    T: PixelBuffer + AsMut<[T::Pixel]>,
  {
    fn as_mut(&mut self) -> &mut [T::Pixel] {
      self.buffer.as_mut()
    }
  }
}

fn as_bytes<T>(slice: &[T]) -> &[u8] {
  // Safety: Aligning to u8 will always succeed since the size of a type is
  // always a multiple of its alignment. u8 having a size of 1 byte implies an
  // alignment of 1 as well.
  let (prefix, bytes, suffix) = unsafe { slice.align_to::<u8>() };
  assert_eq!(prefix.len(), 0);
  assert_eq!(suffix.len(), 0);
  bytes
}

pub use pixel_buffer::*;
mod pixel_buffer {
  use crate::retro::pixel;
  use std::rc::Rc;
  use std::sync::Arc;

  /// A slice of pixels.
  pub trait PixelBuffer: AsRef<[Self::Pixel]> {
    type Pixel: pixel::format::Format;
  }

  impl<P, const LEN: usize> PixelBuffer for [P; LEN]
  where
    P: pixel::format::Format,
  {
    type Pixel = P;
  }

  impl<P> PixelBuffer for [P]
  where
    P: pixel::format::Format,
  {
    type Pixel = P;
  }

  impl<P> PixelBuffer for &[P]
  where
    P: pixel::format::Format,
  {
    type Pixel = P;
  }

  impl<P> PixelBuffer for Vec<P>
  where
    P: pixel::format::Format,
  {
    type Pixel = P;
  }

  impl<P> PixelBuffer for Box<[P]>
  where
    P: pixel::format::Format,
  {
    type Pixel = P;
  }

  impl<P> PixelBuffer for Rc<[P]>
  where
    P: pixel::format::Format,
  {
    type Pixel = P;
  }

  impl<P> PixelBuffer for Arc<[P]>
  where
    P: pixel::format::Format,
  {
    type Pixel = P;
  }
}
