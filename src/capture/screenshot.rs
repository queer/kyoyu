use std::io::ErrorKind;
use std::thread;
use std::time::Duration;

/// Capture a screenshot of the entire display, returning a vec of png-encoded
/// bytes that represent the captured image.
pub async fn capture_screenshot() -> Result<(Vec<u8>, u32, u32), image::ImageError> {
    info!("kyoyu: setup: start");

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

            info!("kyoyu: output: dimensions: {}x{} {}x{}", x, y, w, h);

            let mut canvas: image::RgbImage = image::ImageBuffer::new(w as u32, h as u32);
            // BGR buffers
            for (buffer, ix, iy, iw, ih) in captures {
                let stride = buffer.len() / ih;
                info!(
                    "kyoyu: output: buffer: processing {}x{} {}x{}",
                    ix, iy, iw, ih
                );
                for nx in 0..iw {
                    for ny in 0..ih {
                        // 4 = bytes per pixel
                        let bpp = 3; // 4;
                        let i = stride * ny + bpp * nx;
                        // let pixel =
                        //     image::Rgba([buffer[i], buffer[i + 1], buffer[i + 2], buffer[i + 3]]);
                        let pixel =
                            image::Rgb([buffer[i], buffer[i + 1], buffer[i + 2]]);
                        canvas.put_pixel((ix + nx) as u32, (iy + ny) as u32, pixel);
                    }
                }
            }

            Ok((canvas.into_raw(), w as u32, h as u32))
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
    info!("kyoyu: setup: finish");

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

    info!("kyoyu: frame: captured");

    info!(
        "kyoyu: buffer: flipping ARGB -> BGR (size={})",
        buffer.len()
    );

    flip_buffer(&buffer.to_vec(), w, h, false)
    // buffer.to_vec()
}

#[allow(dead_code)]
fn flip_buffer(buffer: &Vec<u8>, w: usize, h: usize, alpha: bool) -> Vec<u8> {
    info!("kyoyu: buffer: flip: input = {}", buffer.len());
    let bytes = if alpha { 4 } else { 3 };
    let mut flipped = Vec::with_capacity(w * h * bytes);
    let stride = buffer.len() / h;

    for y in 0..h {
        for x in 0..w {
            // 4 because the input buffer always has alpha
            let i = stride * y + 4 * x;
            // [A, R, G, B]
            // [0, 1, 2, 3]
            // [B, G, R, A]
            // [3, 2, 1, 0]
            if alpha {
                flipped.extend_from_slice(&[
                    buffer[i + 3],
                    buffer[i + 2],
                    buffer[i + 1],
                    buffer[i],
                ]);
            } else {
                flipped.extend_from_slice(&[buffer[i + 2], buffer[i + 1], buffer[i]]);
            };
        }
    }

    info!("kyoyu: buffer: flip: output = {}", flipped.len());

    flipped
}
