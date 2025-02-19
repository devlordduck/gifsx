use napi_derive::napi;
use napi::bindgen_prelude::*;
use std::io::Cursor;
use crate::frame::Frame;

#[napi]
pub struct Encoder {
  w: gif::Encoder<Cursor<Vec<u8>>>,
  palette: Option<Vec<u8>>,
  /// The gif width.
  #[napi(readonly)]
  pub width: u16,
  /// The gif height.
  #[napi(readonly)]
  pub height: u16
}

#[napi]
impl Encoder {
  /// Create a new encoder.
  #[napi(constructor)]
  pub fn new(
    width: u16, height: u16,
    palette: Option<&[u8]>,
  ) -> napi::Result<Encoder> {
    let palette = palette.map(|p| p.to_vec());
    Ok(Encoder {
      width, height,
      palette: palette.clone(),
      w: gif::Encoder::new(
        Cursor::new(Vec::new()),
        width, height,
        &palette.unwrap_or(Vec::new()),
      ).map_err(|e| Error::new(
        Status::GenericFailure, format!("Failed to create a GIF Encoder: {}", e),
      ))?
    })
  }

  /// Add a frame to the gif.
  ///
  /// ### Notes:
  /// - The size of `buffer` should match the expected size based on `width`, `height`.
  #[napi]
  pub fn add_frame(&mut self, frame: &Frame) -> napi::Result<()> {
    if frame.width != self.width || frame.height != self.height {
      return Err(Error::new(Status::GenericFailure, format!(
        "Frame dimensions {}x{} do not match encoder dimensions {}x{}",
        frame.width, frame.height, self.width, self.height
      )));
    }

    self.w.write_frame(&frame.to_gif_frame())
      .map_err(|e|
        Error::new(Status::GenericFailure, format!("Failed to write a frame: {}", e)
      ))?;
    Ok(())
  }

  /// The global color palette.
  #[napi(getter)]
  pub fn get_palette(&self) -> Option<Buffer> {
    self.palette.clone().map(|p| Buffer::from(p))
  }

  /// Sets the repeat count for the gif. If the value is -1, the gif will repeat infinitely; otherwise, the gif will repeat a `value` number of times.
  #[napi]
  pub fn set_repeat(&mut self, value: i16) {
    let _ = self.w.set_repeat(if value <= -1 {
      gif::Repeat::Infinite
    } else { gif::Repeat::Finite(value as u16) });
  }

  /// Returns the gif buffer.
  #[napi]
  pub fn get_buffer(&mut self) -> napi::Result<Buffer> {
    let mut buf = self.w.get_mut().clone().into_inner();
    buf.push(0x3B);
    Ok(Buffer::from(buf.to_owned()))
  }
}