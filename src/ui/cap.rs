use crate::utils::{Buffer, Dimension};
use super::{Message, ZoomableImage};

pub struct Cap {
    width: Dimension,
    height: Dimension,
    buffer: Buffer,
    zoom: bool,
    scroll_state: iced::scrollable::State,
}

impl Cap {
    pub fn new(width: Dimension, height: Dimension, buffer: Buffer, zoom: bool) -> Self {
        Cap {
            width,
            height,
            buffer,
            zoom,
            scroll_state: iced::scrollable::State::new(),
        }
    }

    pub fn view(&mut self) -> iced::Container<crate::ui::Message> {
        let image_scrollable = iced::Scrollable::new(&mut self.scroll_state)
            .push(ZoomableImage::new(
                self.buffer.clone(),
                self.width,
                self.height,
                self.zoom,
                Message::CaptureZoomChanged(true),
                Message::CaptureZoomChanged(false),
            ))
            .scrollbar_width(8)
            .scroller_width(8)
            .scrollbar_margin(4);

        iced::Container::new(image_scrollable)
    }

    #[allow(dead_code)]
    pub fn w(&self) -> Dimension {
        self.width
    }

    #[allow(dead_code)]
    pub fn h(&self) -> Dimension {
        self.height
    }

    #[allow(dead_code)]
    pub fn buffer(&self) -> Buffer {
        self.buffer.clone()
    }

    pub fn put_buffer(&mut self, buffer: Buffer) {
        self.buffer = buffer;
    }

    pub fn toggle_zoom(&mut self) {
        self.zoom = !self.zoom;
    }
}

