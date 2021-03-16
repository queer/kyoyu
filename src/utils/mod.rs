pub async fn encode_buffer_to_png(
    buffer: Vec<u8>,
    w: u32,
    h: u32,
) -> Result<Vec<u8>, image::ImageError> {
    info!("kyoyu: utils: encode: buffer={} {}x{}", buffer.len(), w, h);
    // Flip RGB -> BGRA
    let mut out = Vec::with_capacity(buffer.len());
    for i in 0..(buffer.len() / 3) {
        let buffer_idx = i * 3;
        out.extend_from_slice(&[
            buffer[buffer_idx + 2],
            buffer[buffer_idx + 1],
            buffer[buffer_idx],
            0xFF,
        ]);
    }
    Ok(out)
}
