pub mod cap;
pub mod zoomable_image;

pub use zoomable_image::ZoomableImage;

use crate::capture::screenshot;
use crate::utils;
use crate::utils::CaptureStatus;
use crate::utils::{Buffer, Dimension};

use self::cap::Cap;

use iced::Application;

#[derive(Clone, Debug)]
pub enum Message {
    CaptureRequested,
    CaptureComplete(Buffer, Dimension, Dimension),
    CaptureEncoded(Buffer, Dimension, Dimension),
    CaptureFailed(String),

    CaptureZoomChanged(bool),
}

pub fn run() -> iced::Result {
    Ui::run(iced::Settings::default())
}

#[derive(Default)]
struct Ui {
    /// A buffer of png-encoded bytes representing the last full screen
    /// capture.
    cap: Option<Cap>,
    /// The state of the capture button.
    capture_button: iced::button::State,
    /// The last error message
    last_error: String,
    capture_status: CaptureStatus,
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
        let image = if let Some(cap) = &mut self.cap {
            iced::Container::new(cap.view())
        } else {
            iced::Container::new(iced::Text::new("no screenshot yet"))
        };

        let capture_status = match self.capture_status {
            CaptureStatus::Ready => "ready to capture!",
            CaptureStatus::CapturingDisplays => "capturing displays...",
            CaptureStatus::EncodingBuffer => "encoding capture...",
            CaptureStatus::Captured => "captured!",
        };

        iced::Column::new()
            .push(iced::Text::new("kyoyu"))
            .push(iced::Text::new(format!(
                "capture status: {}",
                capture_status
            )))
            .push(iced::Text::new(format!("last error: {}", self.last_error)))
            .push(
                iced::Button::new(&mut self.capture_button, iced::Text::new("capture"))
                    .on_press(Message::CaptureRequested),
            )
            .push(image)
            .spacing(20)
            .padding(20)
            .into()
    }

    fn update(&mut self, msg: Self::Message) -> iced::Command<Self::Message> {
        match msg {
            Message::CaptureRequested => {
                info!("kyoyu: ui: capture requested");
                self.capture_status = CaptureStatus::CapturingDisplays;

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
                self.capture_status = CaptureStatus::EncodingBuffer;
                self.last_error = "displays captured without error".to_string();

                // TODO: Can we avoid cloning this buffer?
                iced::Command::perform(
                    utils::encode_buffer_to_png(buffer.clone(), w, h),
                    |result| {
                        info!("kyoyu: ui: encoded buffer");
                        match result {
                            Ok((buffer, w, h)) => Message::CaptureEncoded(buffer, w, h),
                            Err(err) => Message::CaptureFailed(String::from(&format!(
                                "couldn't encode buffer: {:#?}",
                                err
                            ))),
                        }
                    },
                )
            }
            Message::CaptureEncoded(buffer, w, h) => {
                info!("kyoyu: ui: buffer encoded");
                self.cap = Some(Cap::new(w, h, buffer.clone(), false));
                self.capture_status = CaptureStatus::Captured;
                self.last_error = "buffer encoded without error".to_string();

                iced::Command::none()
            }
            Message::CaptureFailed(err) => {
                info!("kyoyu: ui: capture_failed");
                self.last_error = err;

                iced::Command::none()
            }
            Message::CaptureZoomChanged(state) => {
                info!("kyoyu: ui: capture: zoom: {}", state);
                if let Some(cap) = &mut self.cap {
                    cap.toggle_zoom();
                }
                iced::Command::none()
            }
            // _ => iced::Command::none(),
        }
    }
}
