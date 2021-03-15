extern crate base64;
extern crate image;
extern crate scrap;

use std::io::ErrorKind;
use std::thread;
use std::time::Duration;

use iced::Application;

#[derive(Default)]
struct Ui {
    /// A buffer of png-encoded bytes representing the last full screen
    /// capture.
    screen_buffer: Option<Vec<u8>>,
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
            println!(
                "kyoyu: ui: rendering buffer len={}",
                buffer.len()
            );
            // We can't pass a reference to the buffer to the image, so we
            // have to clone it here :<
            iced::Container::new(iced::Image::new(iced::image::Handle::from_memory(
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
                println!("kyoyu: ui: capture requested");
                iced::Command::perform(capture_screenshot(), |result| {
                    println!("kyoyu: ui: async capture complete");
                    match result {
                        Ok(buffer) => Message::CaptureComplete(buffer),
                        Err(err) => Message::CaptureFailed(String::from(&format!(
                            "couldn't capture displays: {:#?}",
                            err
                        ))),
                    }
                })
            }
            Message::CaptureComplete(buffer) => {
                println!("kyoyu: ui: capture_complete");
                self.screen_buffer = Some(buffer);
                self.last_error = "displays captured without error".to_string();
                iced::Command::none()
            }
            Message::CaptureFailed(err) => {
                println!("kyoyu: ui: capture_failed");
                self.last_error = err;
                iced::Command::none()
            } // _ => iced::Command::none(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    CaptureRequested,
    CaptureComplete(Vec<u8>),
    CaptureFailed(String),
}

fn main() -> iced::Result {
    Ui::run(iced::Settings::default())
}

/// Capture a screenshot of the entire display, returning a vec of png-encoded
/// bytes that represent the captured image.
async fn capture_screenshot() -> Result<Vec<u8>, image::ImageError> {
    println!("kyoyu: setup: start");

    match scrap::Display::all() {
        Ok(displays) => {
            let captures = capture_all_displays(displays);
            let x = captures
                .iter()
                .min_by(|(_, x1, _, _, _), (_, x2, _, _, _)| x1.cmp(x2))
                .map(|(_, x, _, _, _)| x)
                .unwrap();
            let y = captures
                .iter()
                .min_by(|(_, _, y1, _, _), (_, _, y2, _, _)| y1.cmp(y2))
                .map(|(_, _, y, _, _)| y)
                .unwrap();
            let w = captures
                .iter()
                .max_by(|(_, x1, _, w1, _), (_, x2, _, w2, _)| (x1 + w1).cmp(&(x2 + w2)))
                .map(|(_, x, _, w, _)| x + w)
                .unwrap();
            let h = captures
                .iter()
                .max_by(|(_, _, y1, _, h1), (_, _, y2, _, h2)| (y1 + h1).cmp(&(y2 + h2)))
                .map(|(_, _, y, _, h)| y + h)
                .unwrap();

            println!("kyoyu: output: dimensions: {}x{} {}x{}", x, y, w, h);

            let mut canvas: image::RgbImage = image::ImageBuffer::new(w as u32, h as u32);
            // BGR buffers
            for (buffer, ix, iy, iw, ih) in captures {
                let stride = buffer.len() / ih;
                println!(
                    "kyoyu: output: buffer: processing {}x{} {}x{}",
                    ix, iy, iw, ih
                );
                for nx in 0..iw {
                    for ny in 0..ih {
                        let i = stride * ny + 3 * nx;
                        let pixel = image::Rgb([buffer[i], buffer[i + 1], buffer[i + 2]]);
                        canvas.put_pixel((ix + nx) as u32, (iy + ny) as u32, pixel);
                    }
                }
            }

            let mut out: Vec<u8> = Vec::with_capacity(canvas.len());
            let writer = std::io::BufWriter::new(&mut out);
            let encoder = image::png::PngEncoder::new(writer);
            encoder.encode(&canvas, w as u32, h as u32, image::ColorType::Rgb8)?;

            Ok(out)
        }
        Err(_) => panic!("kyoyu: display: couldn't get all displays"),
    }
}

/// Capture all provided displays, returning a vec of tuples of image buffers +
/// bounds
fn capture_all_displays(
    displays: Vec<scrap::Display>,
) -> Vec<(Vec<u8>, usize, usize, usize, usize)> {
    let mut captures = vec![];
    for display in displays {
        let x = display.x();
        let y = display.y();
        let w = display.width();
        let h = display.height();
        captures.push((capture_display(display), x, y, w, h));
    }
    captures
}

/// Capture the display. Returns a buffer.
fn capture_display(display: scrap::Display) -> Vec<u8> {
    let one_second = Duration::new(1, 0);
    let one_frame = one_second / 60;

    let mut capturer = match scrap::Capturer::new(display) {
        Ok(capturer) => capturer,
        Err(_) => panic!("kyoyu: capturer: couldn't create"),
    };
    let (w, h) = (capturer.width(), capturer.height());
    println!("kyoyu: setup: finish");

    // Capture our single frame
    let buffer = loop {
        match capturer.frame() {
            Ok(buffer) => break buffer,
            Err(err) => {
                if err.kind() == ErrorKind::WouldBlock {
                    // Spin until we can get a frame
                    thread::sleep(one_frame);
                    continue;
                }
            }
        }
    };

    println!("kyoyu: frame: captured");

    let mut flipped = Vec::with_capacity(w * h * 3);
    let stride = buffer.len() / h;
    for y in 0..h {
        for x in 0..w {
            let i = stride * y + 4 * x;
            flipped.extend_from_slice(&[buffer[i + 2], buffer[i + 1], buffer[i]]);
        }
    }

    println!("kyoyu: buffer: flipped (size={})", flipped.len());

    flipped
}
