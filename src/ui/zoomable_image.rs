use crate::utils::{Buffer, Dimension};

use std::hash::Hash;

pub struct ZoomableImage<Message: Clone> {
    width: Dimension,
    height: Dimension,
    zoom: bool,
    handle: iced::image::Handle,
    on_zoom_in: Message,
    on_zoom_out: Message,
}

impl<Message: Clone> ZoomableImage<Message> {
    pub fn new(
        buffer: Buffer,
        width: Dimension,
        height: Dimension,
        zoom: bool,
        on_zoom_in: Message,
        on_zoom_out: Message,
    ) -> ZoomableImage<Message> {
        ZoomableImage {
            width,
            height,
            zoom,
            handle: iced::image::Handle::from_pixels(width, height, buffer),
            on_zoom_in,
            on_zoom_out,
        }
    }

    pub fn toggle_zoom(&mut self) -> bool {
        info!("kyoyu: ui: zoomable_image: zoom {} -> {}", self.zoom, !self.zoom);
        !self.zoom
    }
}

impl<Message: Clone, Renderer> iced_native::Widget<Message, Renderer> for ZoomableImage<Message>
where
    Renderer: iced_native::image::Renderer,
{
    fn width(&self) -> iced_native::Length {
        if self.zoom {
            iced_native::Length::Units(self.width as u16)
        } else {
            iced_native::Length::Shrink
        }
    }

    fn height(&self) -> iced_native::Length {
        if self.zoom {
            iced_native::Length::Units(self.height as u16)
        } else {
            iced_native::Length::Shrink
        }
    }

    fn layout(
        &self,
        _renderer: &Renderer,
        limits: &iced_native::layout::Limits,
    ) -> iced_native::layout::Node {
        info!("kyoyu: ui: zoomable_image: recalculating layout");
        let mut size = limits
            .width(iced_native::Length::Shrink)
            .height(iced_native::Length::Shrink)
            .resolve(iced_native::Size::new(
                self.width as f32,
                self.height as f32,
            ));

        // If we aren't filling, scale to correct aspect ratio.
        if !self.zoom {
            let aspect_ratio = self.width as f32 / self.height as f32;

            let viewport_aspect_ratio = size.width / size.height;

            if viewport_aspect_ratio > aspect_ratio {
                size.width = self.width as f32 * size.height / self.height as f32;
            } else {
                size.height = self.height as f32 * size.width / self.width as f32;
            }
        } else {
            size.width = self.width as f32;
            size.height = self.height as f32;
        }

        iced_native::layout::Node::new(size)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: iced_native::Layout<'_>,
        _cursor_position: iced_native::Point,
        _viewport: &iced_native::Rectangle,
    ) -> Renderer::Output {
        renderer.draw(self.handle.clone(), layout)
    }

    fn hash_layout(&self, state: &mut iced_native::Hasher) {
        info!("kyoyu: ui: zoomable_image: hashing layout");
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.handle.hash(state);
        self.zoom.hash(state);
        // self.width.hash(state);
        // self.height.hash(state);
    }

    fn on_event(
        &mut self,
        event: iced_native::Event,
        layout: iced_native::Layout<'_>,
        cursor_position: iced_native::Point,
        messages: &mut Vec<Message>,
        _renderer: &Renderer,
        _clipboard: Option<&dyn iced_native::Clipboard>,
    ) -> iced_native::event::Status {
        match event {
            iced_native::Event::Mouse(iced_native::mouse::Event::ButtonPressed(
                iced_native::mouse::Button::Left,
            )) => {
                let bounds = layout.bounds();
                if bounds.contains(cursor_position) {
                    if self.toggle_zoom() {
                        messages.push(self.on_zoom_in.clone());
                    } else {
                        messages.push(self.on_zoom_out.clone());
                    }
                    return iced_native::event::Status::Captured;
                }
            }
            // iced_native::Event::Mouse(iced_native::mouse::Event::ButtonReleased(
            //     iced_native::mouse::Button::Left,
            // )) => {
            //     let bounds = layout.bounds();
            //     if bounds.contains(cursor_position) {
            //         self.toggle_zoom();
            //         return iced_native::event::Status::Captured;
            //     }
            // }
            _ => {}
        }

        iced_native::event::Status::Ignored
    }
}

impl<'a, Message, Renderer> From<ZoomableImage<Message>> for iced_native::Element<'a, Message, Renderer>
where
    Renderer: iced_native::image::Renderer,
    Message: 'a + Clone,
{
    fn from(image: ZoomableImage<Message>) -> iced_native::Element<'a, Message, Renderer> {
        iced_native::Element::new(image)
    }
}
