use anyhow::Result;

const ABOUT_WINDOW_WIDTH: u32 = 400;
const ABOUT_WINDOW_HEIGHT: u32 = 400;

pub struct AboutWindow {
    pub window: sdl3::video::Window,
}

impl AboutWindow {
    pub fn new(video_subsystem: &sdl3::VideoSubsystem) -> Result<Self> {
        let window = video_subsystem
            .window("About", ABOUT_WINDOW_WIDTH, ABOUT_WINDOW_HEIGHT)
            .position_centered()
            .resizable()
            .opengl()
            .build()?;
        
        Ok(Self {
            window,
        })
    }
}