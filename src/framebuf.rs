use std::ops::{Deref, DerefMut};

use libretro_rs::retro::{pixel, video::{FrameBuffer, PackedFrameBuffer, PackedFrameBufferMut}};

#[derive(Clone, Debug)]
pub struct ResizableFrameBuffer<P, const MAX_LEN: usize> {
    data: Vec<P>,
    width: u16,
    height: u16,
    length: usize,   
}

impl<P, const MAX_LEN: usize> ResizableFrameBuffer<P, MAX_LEN>
where
    P: pixel::format::Format,
{
    pub fn new() -> Self {
        Self {
            data: vec![P::default(); MAX_LEN],
            width: 0,
            height: 0,
            length: 0,
        }
    }

    /// Resized the internal pixel vector to allow for dynamic screen resolution.
    /// Returns an error if the length provided is greater than MAX_LEN or if
    /// length is not divisible by width.
    pub fn resize(&mut self, width: u16, height: u16) -> Result<(), String> {
        let length = (width as usize) * (height as usize);
        
        if length > MAX_LEN {
            return Err(String::from("witdth*height must not exceed frame buffer MAX_LEN"));
        }

        if length % (width as usize) != 0 {
            return Err(String::from("frame buffer length must be divisible by width"));
        }

        self.width = width;
        self.height = height;
        self.length = length;

        Ok(())
    }

    // /// Consumes this [FrameBuffer], returning the wrapped pixel buffer.
    // pub fn into_inner(self) -> Vec<P> {
    //     self.0
    // }
}

impl<P, const LEN: usize> From<Vec<P>> for ResizableFrameBuffer<P, LEN> {
    fn from(value: Vec<P>) -> Self {
        Self {
            data: value,
            width: 0,
            height: 0,
            length: 0,
        }
    }
}

unsafe impl<P, const LEN: usize> FrameBuffer for ResizableFrameBuffer<P, LEN>
where
    P: pixel::format::Format,
{
    type Pixel = P;

    fn data(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr() as *const u8,
                self.length * std::mem::size_of::<P>(),
            )
        }
    }

    fn width(&self) -> u16 {
        self.width
    }

    fn height(&self) -> u16 {
        self.height
    }
}

unsafe impl<P, const LEN: usize> PackedFrameBuffer for ResizableFrameBuffer<P, LEN> where
    P: pixel::format::Format
{
}

unsafe impl<P, const LEN: usize> PackedFrameBufferMut for ResizableFrameBuffer<P, LEN> where
    P: pixel::format::Format
{
}

impl<P, const LEN: usize> Deref for ResizableFrameBuffer<P, LEN> {
    type Target = Vec<P>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<P, const LEN: usize> DerefMut for ResizableFrameBuffer<P, LEN> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<P, const LEN: usize> AsRef<[P]> for ResizableFrameBuffer<P, LEN> {
    fn as_ref(&self) -> &[P] {
        &self.data[..]
    }
}

impl<P, const LEN: usize> AsMut<[P]> for ResizableFrameBuffer<P, LEN> {
    fn as_mut(&mut self) -> &mut [P] {
        &mut self.data[..]
    }
}

impl<P, const LEN: usize> Into<Vec<P>> for ResizableFrameBuffer<P, LEN> {
    fn into(self) -> Vec<P> {
        self.data
    }
}

impl<P, const LEN: usize> Default for ResizableFrameBuffer<P, LEN>
where
    P: pixel::format::Format + Copy + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

// unsafe trait ValidFramebuffer {
//     const IS_VALID: ();
// }

// unsafe impl<P, const LEN: usize> ValidFramebuffer for ResizableFrameBuffer<P, LEN> {
//     const IS_VALID: () = {
//         assert!(
//             LEN % W as usize == 0,
//             "VecFrameBuffer length must be evenly divisible by its width."
//         );
//         assert!(
//             LEN / W as usize <= u16::MAX as usize,
//             "VecFrameBuffer (LEN/W) must fit in a u16."
//         );
//     };
// }


mod tests {
    use std::panic::catch_unwind;

    use libretro_rs::retro::{pixel::format::XRGB8888, video::ArrayFrameBuffer};

    use super::ResizableFrameBuffer;

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
        let mut frame_buf: ResizableFrameBuffer<XRGB8888, LENGTH> = ResizableFrameBuffer::new();
        
        frame_buf.resize(WIDTH as u16, HEIGHT as u16);
    }
}