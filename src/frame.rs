use napi_derive::napi;
use napi::bindgen_prelude::*;
use crate::util::hex_to_rgba;
use crate::enums::{DisposalMethod, FrameBufType};

/// A GIF frame.
#[napi]
pub struct Frame {
  /// The delay for this frame in units of 10ms (e.g., a value of `10` equals 100ms).
  pub delay: u16,
  /// How the current frame should be disposed of when moving to the next one.
  pub dispose: DisposalMethod,
  /// An optional index of a transparent color in the palette.
  pub transparent: Option<u8>,
  /// Indicates whether this frame requires user input to proceed.
  pub needs_user_input: bool,
  /// The vertical offset of the frame relative to the top of the canvas.
  pub top: u16,
  /// The horizontal offset of the frame relative to the left of the canvas.
  pub left: u16,
  /// Width of the frame.
  pub width: u16,
  /// Height of the frame.
  pub height: u16,
  /// True if the image is interlaced.
  pub interlaced: bool,

  buf_type: FrameBufType,
  palette: Option<Vec<u8>>,
  buf: Vec<u8>, speed: Option<u8>
}

#[napi]
impl Frame {
  fn new(
    width: u16, height: u16,
    buf: Vec<u8>, palette: Option<Vec<u8>>,
    buf_type: FrameBufType, speed: Option<u8>,
  ) -> napi::Result<Frame> {
    if buf.len() != width as usize * height as usize * (match buf_type {
      FrameBufType::Rgba | FrameBufType::Hex => 4,
      FrameBufType::Rgb => 3,
      FrameBufType::IndexedPixels => 1
    }) { return Err(Error::new(Status::InvalidArg, "Buffer size mismatch")) }

    Ok(Self {
      delay: 1, dispose: DisposalMethod::Keep,
      transparent: None, needs_user_input: false,
      top: 0, left: 0, width, height,
      interlaced: false, palette,
      buf, buf_type, speed
    })
  }

  /// The frame's palette.
  #[napi(getter)]
  pub fn get_palette(&self) -> Option<Buffer> {
    self.palette.clone().map(|p| Buffer::from(p))
  }

  /// Sets the frame's palette.
  #[napi]
  pub fn set_palette(
    &mut self, val: Option<&[u8]>,
  ) { self.palette = val.map(|p| p.to_vec()); }

  /// The buffer of this frame.
  #[napi(getter)]
  pub fn get_buffer(&self) -> Buffer { Buffer::from(self.buf.clone()) }
  #[napi]
  pub fn set_buffer(&mut self, val: &[u8]) { self.buf = val.to_vec(); }

  /// Creates a frame from RGBA pixel data.
  ///
  /// ### Notes:
  /// - Speed needs to be in the range 1-30. Higher is faster, lower CPU usage but worse quality.
  /// - The size of `buffer` should match the expected size based on `width`, `height`.
  #[napi]
  pub fn from_rgba(
    width: u16, height: u16,
    buffer: &[u8], speed: Option<u8>
  ) -> napi::Result<Frame> {
    if let Some(speed) = speed {
      if speed < 1 || speed > 30 {
        return Err(Error::new(Status::InvalidArg, "Speed needs to be in the range 1-30"));
      }
    }

    Ok(Self::new(
      width, height, buffer.to_vec(),
      None, FrameBufType::Rgba, speed
    )?)
  }

  /// Creates a frame from RGB pixel data.
  ///
  /// ### Notes:
  /// - Speed needs to be in the range 1-30. Higher is faster, lower CPU usage but worse quality.
  /// - The size of `buffer` should match the expected size based on `width`, `height`.
  #[napi]
  pub fn from_rgb(
    width: u16, height: u16,
    buffer: &[u8], speed: Option<u8>
  ) -> napi::Result<Frame> {
    if let Some(speed) = speed {
      if speed < 1 || speed > 30 {
        return Err(Error::new(Status::InvalidArg, "Speed needs to be in the range 1-30"));
      }
    }

    Ok(Self::new(
      width, height, buffer.to_vec(),
      None, FrameBufType::Rgb, speed
    )?)
  }

  /// Creates a frame from hex pixel data.
  ///
  /// ### Notes:
  /// - Speed needs to be in the range 1-30. Higher is faster, lower CPU usage but worse quality.
  /// - The size of `buffer` should match the expected size based on `width`, `height`.
  /// - The color output will be in Rgba.
  #[napi]
  pub fn from_hex(
    width: u16, height: u16,
    buffer: Vec<String>, speed: Option<u8>
  ) -> napi::Result<Frame> {
    if let Some(speed) = speed {
      if speed < 1 || speed > 30 {
        return Err(Error::new(Status::InvalidArg, "Speed needs to be in the range 1-30"));
      }
    }

    Ok(Self::new(
      width, height, hex_to_rgba(buffer)?.to_vec(),
      None, FrameBufType::Hex, speed
    )?)
  }

  /// Creates a frame from indexed pixel data.
  ///
  /// ### Notes:
  /// - The size of `buffer` should match the expected size based on `width`, `height`.
  #[napi]
  pub fn from_indexed_pixels(
    width: u16, height: u16,
    pixels: &[u8], palette: Option<&[u8]>,
    transparent: Option<u8>,
  ) -> napi::Result<Frame> {
    let mut frame = Self::new(
      width, height, pixels.to_vec(),
      palette.map(|p| p.to_vec()), FrameBufType::IndexedPixels,
      None
    )?;
    frame.transparent = transparent;
    Ok(frame)
  }

  pub fn from_gif_frame(f: &gif::Frame, buf_type: FrameBufType) -> Frame {
    Frame {
      delay: f.delay, dispose: match f.dispose {
        gif::DisposalMethod::Any => DisposalMethod::Any,
        gif::DisposalMethod::Keep => DisposalMethod::Keep,
        gif::DisposalMethod::Background => DisposalMethod::Background,
        gif::DisposalMethod::Previous => DisposalMethod::Previous,
      }, transparent: f.transparent,
      needs_user_input: f.needs_user_input,
      top: f.top, left: f.left,
      width: f.width, height: f.height,
      interlaced: f.interlaced,
      palette: f.palette.to_owned(),
      buf: (&*f.buffer).to_owned().to_vec(),
      buf_type, speed: None
    }
  }

  pub fn to_gif_frame(&self) -> gif::Frame<'static> {
    let mut frame = match self.buf_type {
      FrameBufType::Rgba | FrameBufType::Hex => gif::Frame::from_rgba_speed(
        self.width, self.height,
        &mut self.buf.clone(), self.speed.unwrap_or(10).into(),
      ),
      FrameBufType::Rgb => gif::Frame::from_rgb_speed(
        self.width, self.height,
        &mut self.buf.clone(), self.speed.unwrap_or(10).into(),
      ),
      FrameBufType::IndexedPixels => gif::Frame::from_indexed_pixels(
        self.width, self.height, self.buf.clone(), self.transparent
      )
    };

    if self.palette.is_some() { frame.palette = self.palette.clone() }
    if frame.transparent.is_some() { frame.transparent = self.transparent }

    frame.delay = self.delay;
    frame.dispose = self.to_gif_disposal();
    frame.needs_user_input = self.needs_user_input;
    frame.top = self.top;
    frame.left = self.left;
    frame
  }

  fn to_gif_disposal(&self) -> gif::DisposalMethod {
    match self.dispose {
      DisposalMethod::Any => gif::DisposalMethod::Any,
      DisposalMethod::Keep => gif::DisposalMethod::Keep,
      DisposalMethod::Background => gif::DisposalMethod::Background,
      DisposalMethod::Previous => gif::DisposalMethod::Previous,
    }
  }
}