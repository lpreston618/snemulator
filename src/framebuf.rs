use std::ops::{Deref, DerefMut};

use libretro_rs::retro::{pixel, video::{FrameBuffer, PackedFrameBuffer, PackedFrameBufferMut}};



#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct VecFrameBuffer<P, const LEN: usize, const W: u16>(Vec<P>);

impl<P, const LEN: usize, const W: u16> VecFrameBuffer<P, LEN, W>
where
    P: pixel::format::Format,
{
    /// The height of the framebuffer.
    pub const HEIGHT: u16 = (LEN as u32 / W as u32) as u16;


    pub const fn new(pixels: Vec<P>) -> Self {
        // This is a workaround to perform validation on const generic parameters.
        // Since using a const param from an outer scope in an expression isn't
        // allowed, the validation is performed while defining an associated
        // constant for a trait this type implements.
        // See https://stackoverflow.com/a/72588268
        _ = <Self as ValidFramebuffer>::IS_VALID;
        Self(pixels)
    }

    /// Consumes this [FrameBuffer], returning the wrapped pixel buffer.
    pub fn into_inner(self) -> Vec<P> {
        self.0
    }
}

unsafe impl<P, const LEN: usize, const W: u16> FrameBuffer for VecFrameBuffer<P, LEN, W>
where
    P: pixel::format::Format,
{
    type Pixel = P;

    fn data(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.0.as_ptr() as *const u8,
                self.0.len() * std::mem::size_of::<P>(),
            )
        }
    }

    fn width(&self) -> u16 {
        W
    }

    fn height(&self) -> u16 {
        Self::HEIGHT
    }
}

unsafe impl<P, const LEN: usize, const W: u16> PackedFrameBuffer for VecFrameBuffer<P, LEN, W> where
    P: pixel::format::Format
{
}

unsafe impl<P, const LEN: usize, const W: u16> PackedFrameBufferMut for VecFrameBuffer<P, LEN, W> where
    P: pixel::format::Format
{
}

impl<P, const LEN: usize, const W: u16> Deref for VecFrameBuffer<P, LEN, W> {
    type Target = Vec<P>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<P, const LEN: usize, const W: u16> DerefMut for VecFrameBuffer<P, LEN, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<P, const LEN: usize, const W: u16> AsRef<[P]> for VecFrameBuffer<P, LEN, W> {
    fn as_ref(&self) -> &[P] {
        &self.0[..]
    }
}

impl<P, const LEN: usize, const W: u16> AsMut<[P]> for VecFrameBuffer<P, LEN, W> {
    fn as_mut(&mut self) -> &mut [P] {
        &mut self.0[..]
    }
}

impl<P, const LEN: usize, const W: u16> Into<Vec<P>> for VecFrameBuffer<P, LEN, W> {
    fn into(self) -> Vec<P> {
        self.0
    }
}

impl<P, const LEN: usize, const W: u16> Default for VecFrameBuffer<P, LEN, W>
where
    P: pixel::format::Format + Copy + Default,
{
    fn default() -> Self {
        Self::new(vec![P::default(); LEN])
    }
}

unsafe trait ValidFramebuffer {
    const IS_VALID: ();
}

unsafe impl<P, const LEN: usize, const W: u16> ValidFramebuffer for VecFrameBuffer<P, LEN, W> {
    const IS_VALID: () = {
        assert!(
            LEN % W as usize == 0,
            "VecFrameBuffer length must be evenly divisible by its width."
        );
        assert!(
            LEN / W as usize <= u16::MAX as usize,
            "VecFrameBuffer (LEN/W) must fit in a u16."
        );
    };
}


mod tests {
    use std::panic::catch_unwind;

    use libretro_rs::retro::{pixel::format::XRGB8888, video::ArrayFrameBuffer};

    use super::VecFrameBuffer;

    const WIDTH: usize = 1024;
    const HEIGHT: usize = 1024;
    const LENGTH: usize = WIDTH*HEIGHT;


    // THIS TEST WILL ERROR!!!
    // This is demonstrating that even though we are putting the array on the heap,
    // the array is still allocated on the stack first, overflowing the thread's stack.
    // Thus we need to use VecFrameBuffer instead, which is initially allocated
    // on the heap.
    #[test]
    fn large_arr_frame_buf() {
        let frame_buf: Box<ArrayFrameBuffer<XRGB8888, LENGTH, {WIDTH as u16}>>;
    
        frame_buf = Box::new(ArrayFrameBuffer::new([XRGB8888::default(); LENGTH]));
    }

    #[test]
    fn large_vec_frame_buf() {
        let frame_buf: VecFrameBuffer<XRGB8888, LENGTH, {WIDTH as u16}>;
        
        frame_buf = VecFrameBuffer::new(vec![XRGB8888::default(); LENGTH]);
    }
}