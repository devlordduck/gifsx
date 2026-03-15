use std::borrow::Cow;

use crate::enums::{DisposalMethod, FrameBufType};
use napi::bindgen_prelude::*;
use napi_derive::napi;

/// A GIF frame.
#[napi]
pub struct Frame<'a> {
  pub buf_type: FrameBufType,
  pub(crate) w: gif::Frame<'a>,
}

#[napi]
impl<'a> Frame<'a> {
  fn new(
    width: u16,
    height: u16,
    buf: &mut [u8],
    buf_type: FrameBufType,
    speed: i32,
    transparent: Option<u8>,
  ) -> napi::Result<Frame<'a>> {
    if buf.len()
      != width as usize
        * height as usize
        * (match &buf_type {
          FrameBufType::Rgba | FrameBufType::Hex => 4,
          FrameBufType::Rgb => 3,
          FrameBufType::IndexedPixels => 1,
        })
    {
      return Err(Error::new(Status::InvalidArg, "Buffer size mismatch"));
    }

    Ok(Self {
      w: match &buf_type {
        FrameBufType::Rgba | FrameBufType::Hex => {
          gif::Frame::from_rgba_speed(width, height, buf, speed)
        }
        FrameBufType::Rgb => gif::Frame::from_rgb_speed(width, height, buf, speed),
        FrameBufType::IndexedPixels => {
          gif::Frame::from_indexed_pixels(width, height, buf, transparent)
        }
      },
      buf_type: FrameBufType::IndexedPixels,
    })
  }

  #[napi(getter)]
  pub fn width(&self) -> u16 {
    self.w.width
  }
  #[napi(getter)]
  pub fn height(&self) -> u16 {
    self.w.height
  }

  #[napi(getter)]
  pub fn delay(&self) -> u16 {
    self.w.delay
  }
  #[napi(setter)]
  pub fn set_delay(&mut self, val: u16) {
    self.w.delay = val;
  }

  #[napi(getter)]
  pub fn dispose(&self) -> DisposalMethod {
    self.w.dispose.into()
  }
  #[napi(setter)]
  pub fn set_dispose(&mut self, val: DisposalMethod) {
    self.w.dispose = match val {
      DisposalMethod::Any => gif::DisposalMethod::Any,
      DisposalMethod::Keep => gif::DisposalMethod::Keep,
      DisposalMethod::Background => gif::DisposalMethod::Background,
      DisposalMethod::Previous => gif::DisposalMethod::Previous,
    };
  }

  #[napi(getter)]
  pub fn needs_user_input(&self) -> bool {
    self.w.needs_user_input
  }
  #[napi(setter)]
  pub fn set_needs_user_input(&mut self, val: bool) {
    self.w.needs_user_input = val;
  }

  #[napi(getter)]
  pub fn transparent(&self) -> Option<u8> {
    self.w.transparent
  }
  #[napi(setter)]
  pub fn set_transparent(&mut self, val: Option<u8>) {
    self.w.transparent = val;
  }

  #[napi(getter)]
  pub fn interlaced(&self) -> bool {
    self.w.interlaced
  }
  #[napi(setter)]
  pub fn set_interlaced(&mut self, val: bool) {
    self.w.interlaced = val;
  }

  #[napi(getter)]
  pub fn top(&self) -> u16 {
    self.w.top
  }
  #[napi(setter)]
  pub fn set_top(&mut self, val: u16) {
    self.w.top = val;
  }

  #[napi(getter)]
  pub fn left(&self) -> u16 {
    self.w.left
  }
  #[napi(setter)]
  pub fn set_left(&mut self, val: u16) {
    self.w.left = val;
  }

  /// The frame's palette.
  #[napi(getter)]
  pub fn get_palette(&mut self) -> Option<Uint8Array> {
    if let Some(palette) = self.w.palette.clone() {
      Some(Uint8Array::new(palette))
    } else {
      None
    }
  }

  #[napi]
  pub fn set_palette(&mut self, val: Option<&[u8]>) {
    self.w.palette = val.map(|p| p.to_vec());
  }

  /// The frame's buffer.
  #[napi(getter)]
  pub fn get_buffer(&self) -> Buffer {
    Buffer::from(self.w.buffer.as_ref())
  }

  #[napi]
  pub fn set_buffer(&mut self, buf: &[u8]) {
    self.w.buffer = Cow::Owned(buf.to_owned());
  }

  /// Creates a frame from RGBA pixel data.
  ///
  /// ### Notes:
  /// - Speed needs to be in the range 1-30. Higher is faster, lower CPU usage but worse quality.
  /// - The size of `buffer` should match the expected size based on `width`, `height`.
  #[napi]
  pub fn from_rgba(
    width: u16,
    height: u16,
    mut buffer: Uint8Array,
    speed: Option<i32>,
  ) -> napi::Result<Frame<'a>> {
    if let Some(speed) = speed {
      if speed < 1 || speed > 30 {
        return Err(Error::new(
          Status::InvalidArg,
          "Speed needs to be in the range 1-30",
        ));
      }
    }

    unsafe {
      Self::new(
        width,
        height,
        buffer.as_mut(),
        FrameBufType::Rgba,
        speed.unwrap_or(15),
        None,
      )
    }
  }

  /// Creates a frame from RGB pixel data.
  ///
  /// ### Notes:
  /// - Speed needs to be in the range 1-30. Higher is faster, lower CPU usage but worse quality.
  /// - The size of `buffer` should match the expected size based on `width`, `height`.
  #[napi]
  pub fn from_rgb(
    width: u16,
    height: u16,
    mut buffer: Uint8Array,
    speed: Option<i32>,
  ) -> napi::Result<Frame<'a>> {
    unsafe {
      Self::new(
        width,
        height,
        buffer.as_mut(),
        FrameBufType::Rgb,
        speed.unwrap_or(15),
        None,
      )
    }
  }

  /// Creates a frame from indexed pixel data.
  ///
  /// ### Notes:
  /// - The size of `buffer` should match the expected size based on `width`, `height`.
  #[napi]
  pub fn from_indexed_pixels(
    width: u16,
    height: u16,
    mut pixels: Uint8Array,
    palette: Option<Vec<u8>>,
    transparent: Option<u8>,
  ) -> napi::Result<Frame<'a>> {
    let mut frame = unsafe {
      Self::new(
        width,
        height,
        pixels.as_mut(),
        FrameBufType::IndexedPixels,
        0,
        transparent,
      )?
    };
    frame.w.palette = palette;
    frame.w.transparent = transparent;
    Ok(frame)
  }

  pub fn from_gif_frame(f: gif::Frame, buf_type: FrameBufType) -> Frame {
    Frame { w: f, buf_type }
  }

  /*pub fn to_gif_frame(&self) -> gif::Frame<'static> {
    let mut frame = match self.buf_type {
      FrameBufType::Rgba | FrameBufType::Hex => gif::Frame::from_rgba_speed(
        self.width,
        self.height,
        &mut self.buf.clone(),
        self.speed.unwrap_or(10).into(),
      ),
      FrameBufType::Rgb => gif::Frame::from_rgb_speed(
        self.width,
        self.height,
        &mut self.buf.clone(),
        self.speed.unwrap_or(10).into(),
      ),
      FrameBufType::IndexedPixels => {
        gif::Frame::from_indexed_pixels(self.width, self.height, self.buf.clone(), self.transparent)
      }
    };

    if self.palette.is_some() {
      frame.palette = self.palette.clone()
    }
    if frame.transparent.is_some() {
      frame.transparent = self.transparent
    }

    frame.delay = self.delay;
    frame.dispose = self.dispose.clone().into();
    frame.needs_user_input = self.needs_user_input;
    frame.top = self.top;
    frame.left = self.left;
    frame
  }*/
}

impl Into<DisposalMethod> for gif::DisposalMethod {
  fn into(self) -> DisposalMethod {
    match self {
      gif::DisposalMethod::Any => DisposalMethod::Any,
      gif::DisposalMethod::Keep => DisposalMethod::Keep,
      gif::DisposalMethod::Background => DisposalMethod::Background,
      gif::DisposalMethod::Previous => DisposalMethod::Previous,
    }
  }
}
