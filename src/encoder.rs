use crate::frame::Frame;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::io::Cursor;

#[napi]
pub struct Encoder {
  w: gif::Encoder<Cursor<Vec<u8>>>,
  /// The gif width.
  #[napi(readonly)]
  pub width: u16,
  /// The gif height.
  #[napi(readonly)]
  pub height: u16,
}

#[napi]
impl Encoder {
  /// Create a new encoder.
  #[napi(constructor)]
  pub fn new(width: u16, height: u16, palette: Option<&[u8]>) -> napi::Result<Encoder> {
    Ok(Encoder {
      width,
      height,
      w: gif::Encoder::new(
        Cursor::new(Vec::new()),
        width, height,
        &palette.unwrap_or(&[]),
      )
      .map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to create a GIF Encoder: {}", e),
        )
      })?,
    })
  }

  /// Add a frame to the gif.
  ///
  /// ### Notes:
  /// - The size of `buffer` should match the expected size based on `width`, `height`.
  #[napi]
  pub fn add_frame(&mut self, frame: &Frame) -> napi::Result<()> {
    if frame.w.width != self.width || frame.w.height != self.height {
      return Err(Error::new(
        Status::GenericFailure,
        format!(
          "Frame dimensions {}x{} do not match encoder dimensions {}x{}",
          frame.w.width, frame.w.height, self.width, self.height
        ),
      ));
    }

    self.w.write_frame(&frame.w).map_err(|e| {
      Error::new(
        Status::GenericFailure,
        format!("Failed to write a frame: {}", e),
      )
    })?;
    Ok(())
  }

  /// Sets the repeat count for the gif. If the value is -1, the gif will repeat infinitely; otherwise, the gif will repeat a `value` number of times.
  #[napi]
  pub fn set_repeat(&mut self, value: i16) {
    let _ = self.w.set_repeat(if value <= -1 {
      gif::Repeat::Infinite
    } else {
      gif::Repeat::Finite(value as u16)
    });
  }

  /// Returns the gif buffer.
  #[napi]
  pub fn get_buffer(&mut self) -> napi::Result<Buffer> {
    let mut buf = self.w.get_mut().clone().into_inner();
    buf.push(0x3B);
    Ok(Buffer::from(buf))
  }
}
