use super::Vec2;
use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Window as WinitWindow, WindowBuilder},
};

const WINDOW_NAME: &'static str = "The Clockwork";

pub struct Window {
    pub name: &'static str,
    pub events_loop: EventLoop<()>,
    pub window: WinitWindow,
}

impl Window {
    pub fn new(size: Vec2) -> Result<Self, winit::error::OsError> {
        let events_loop = EventLoop::new();
        let output = WindowBuilder::new()
            .with_title(WINDOW_NAME)
            .with_inner_size(LogicalSize {
                width: size.x as f64,
                height: size.y as f64,
            })
            .with_resizable(false)
            .build(&events_loop);

        output.map(|window| {
            // if let Some(window_size) = window.get_inner_size() {
            //     error!("Original Logical Window size is {:?}", window_size);

            //     let size = window.get_hidpi_factor();
            //     error!("Size is {}", size);
            //     let window_size = window_size.to_physical(size);
            //     error!("Original Physical Window size is {:?}", window_size);

            //     let new_size = Vec2::new(window_size.width as f32, window_size.height as f32) / size as f32;
            //     error!("New Physical Size is {}", new_size);

            //     // window.set_inner_size(LogicalSize {
            //     //     width: new_size.x as f64,
            //     //     height: new_size.y as f64,
            //     // });
            // }

            // if let Some(window_size) = window.get_outer_size() {
            //     error!("Outer Window Size {:?}", window_size);
            // }

            Self {
                events_loop,
                window,
                name: WINDOW_NAME,
            }
        })
    }

    pub fn get_window_size(&self) -> Vec2 {
        let window_client_area = self.window.inner_size();
        Vec2::new(window_client_area.width as f32, window_client_area.height as f32)
    }
}
