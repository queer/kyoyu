use crate::capture::screenshot;
use crate::utils;

use iced::Application;

#[derive(Clone, Debug)]
pub enum Message {
    CaptureRequested,
    CaptureComplete(Vec<u8>, u32, u32),
    CaptureEncoded(Vec<u8>),
    CaptureFailed(String),
}

pub fn run() -> iced::Result {
    Ui::run(iced::Settings::default())
}

#[derive(Default)]
struct Ui {
    /// A buffer of png-encoded bytes representing the last full screen
    /// capture.
    screen_buffer: Option<Vec<u8>>,
    width: u32,
    height: u32,
    /// The state of the capture button.
    capture_button: iced::button::State,
    /// The last error message
    last_error: String,
}

impl iced::Application for Ui {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (Self::default(), iced::Command::none())
    }

    fn title(&self) -> String {
        String::from("kyoyu")
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        let image = if let Some(buffer) = &self.screen_buffer {
            info!("kyoyu: ui: rendering buffer len={}", buffer.len());
            // We can't pass a reference to the buffer to the image, so we
            // have to clone it here :<
            // iced::Container::new(iced::Image::new(iced::image::Handle::from_memory(
            //     buffer.clone(),
            // )))
            iced::Container::new(iced::Image::new(iced::image::Handle::from_pixels(
                self.width,
                self.height,
                buffer.clone(),
            )))
        } else {
            iced::Container::new(iced::Text::new("no screenshot yet"))
        };

        iced::Column::new()
            .push(iced::Text::new("kyoyu"))
            .push(iced::Text::new(format!("last error: {}", self.last_error)))
            .push(
                iced::Button::new(&mut self.capture_button, iced::Text::new("capture"))
                    .on_press(Message::CaptureRequested),
            )
            .push(image)
            .into()
    }

    fn update(&mut self, msg: Self::Message) -> iced::Command<Self::Message> {
        match msg {
            Message::CaptureRequested => {
                info!("kyoyu: ui: capture requested");
                iced::Command::perform(screenshot::capture_screenshot(), |result| {
                    info!("kyoyu: ui: async capture complete");
                    match result {
                        Ok((buffer, w, h)) => Message::CaptureComplete(buffer, w, h),
                        Err(err) => Message::CaptureFailed(String::from(&format!(
                            "couldn't capture displays: {:#?}",
                            err
                        ))),
                    }
                })
            }
            Message::CaptureComplete(buffer, w, h) => {
                info!("kyoyu: ui: capture_complete");
                self.last_error = "displays captured without error".to_string();
                self.width = w;
                self.height = h;
                // TODO: Can we avoid cloning this buffer?
                iced::Command::perform(
                    utils::encode_buffer_to_png(buffer.clone(), w, h),
                    |result| {
                        info!("kyoyu: ui: encoded buffer");
                        match result {
                            Ok(buffer) => Message::CaptureEncoded(buffer),
                            Err(err) => Message::CaptureFailed(String::from(&format!(
                                "couldn't encode buffer: {:#?}",
                                err
                            ))),
                        }
                    },
                )
            }
            Message::CaptureEncoded(buffer) => {
                info!("kyoyu: ui: buffer encoded");
                self.screen_buffer = Some(buffer);
                self.last_error = "buffer encoded without error".to_string();
                iced::Command::none()
            }
            Message::CaptureFailed(err) => {
                info!("kyoyu: ui: capture_failed");
                self.last_error = err;
                iced::Command::none()
            } // _ => iced::Command::none(),
        }
    }
}
