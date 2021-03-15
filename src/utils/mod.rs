pub async fn encode_buffer_to_png(buffer: Vec<u8>, w: u32, h: u32) -> Result<Vec<u8>, image::ImageError> {
    let mut out: Vec<u8> = Vec::with_capacity(buffer.len());
    let writer = std::io::BufWriter::new(&mut out);
    let encoder = image::png::PngEncoder::new(writer);
    encoder.encode(&buffer, w, h, image::ColorType::Rgb8)?;
    Ok(out)
}