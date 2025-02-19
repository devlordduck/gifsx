use napi_derive::napi;
use napi::bindgen_prelude::*;
use std::io::Cursor;
use std::num::NonZero;
use crate::frame::Frame;
use crate::enums::{FrameBufType, ColorOutput};

#[derive(Clone)]
struct CustomOptions {
  pub(crate) frame_buf_type: FrameBufType
}

/// The GIF Decoder.
#[napi]
pub struct Decoder {
  w: gif::Decoder<Cursor<Vec<u8>>>,
  custom_options: CustomOptions
}

#[napi]
impl Decoder {
  /// Create a new decoder.
  /// @param buffer - The GIF buffer to decode.
  /// @param options - The options to use for decoding.
  #[napi(constructor)]
  pub fn new(
    buffer: &[u8], options: Option<&DecodeOptions>,
  ) -> napi::Result<Decoder> {
    if let Some(options) = options {
      return options.read_info(buffer);
    }

    Ok(Decoder {
      w: gif::Decoder::new(Cursor::new(buffer.to_vec()))
        .map_err(|e| Error::new(
          Status::GenericFailure, format!("Failed to create a GIF decoder: {}", e),
        ))?,
      custom_options: CustomOptions { frame_buf_type: FrameBufType::IndexedPixels }
    })
  }

  /// Returns the next frame info. (skips the buffer)
  #[napi]
  pub fn next_frame_info(&mut self) -> napi::Result<Option<Frame>> {
    let f = self.w.next_frame_info().map_err(|e| Error::new(
      Status::GenericFailure, format!("Failed to get next frame info: {}", e)
    ))?;
    Ok(f.map(|f| Frame::from_gif_frame(f, self.custom_options.frame_buf_type.clone())))
  }

  /// Reads the next frame from the GIF.
  /// Do not call `<Decoder>.nextFrameInfo` beforehand. Deinterlaces the result.
  #[napi]
  pub fn read_next_frame(&mut self) -> napi::Result<Option<Frame>> {
    let f = self.w.read_next_frame().map_err(|e| Error::new(
      Status::GenericFailure, format!("Failed to get next frame info: {}", e)
    ))?;
    Ok(f.map(|f| Frame::from_gif_frame(f, self.custom_options.frame_buf_type.clone())))
  }

  /// Output buffer size.
  #[napi(getter)]
  pub fn buffer_size(&self) -> u32 { self.w.buffer_size() as u32 }

  /// Line length of the current frame.
  #[napi(getter)]
  pub fn line_length(&self) -> u32 { self.w.line_length() as u32 }

  /// The color palette relevant for the frame that has been decoded.
  #[napi(getter)]
  pub fn palette(&self) -> napi::Result<Buffer> {
    Ok(Buffer::from(self.w.palette().map_err(|e| Error::new(
      Status::GenericFailure, format!("Failed to get the palette: {}", e),
    ))?))
  }

  /// The global color palette.
  #[napi(getter)]
  pub fn global_palette(&self) -> Option<Buffer> {
    self.w.global_palette().map(|p| Buffer::from(p))
  }

  /// Width of the GIF.
  #[napi(getter)]
  pub fn width(&self) -> u16 { self.w.width() }

  /// Height of the GIF.
  #[napi(getter)]
  pub fn height(&self) -> u16 { self.w.height() }

  /// Index of the background color in the global palette
  /// In practice this is not used, and the background is always transparent
  #[napi(getter)]
  pub fn bg_color(&self) -> Option<u16> {
    self.w.bg_color().map(|c| c as u16)
  }

  /// Number of loop repetitions.
  #[napi(getter)]
  pub fn loops(&self) -> i16 {
    match self.w.repeat() {
      gif::Repeat::Finite(v) => v as i16,
      gif::Repeat::Infinite => -1,
    }
  }
}

/// Options for opening a GIF decoder. `<DecodeOptions>.readInfo` will create a decoder with these options.
#[napi]
pub struct DecodeOptions {
  w: gif::DecodeOptions,
  custom_options: CustomOptions
}

#[napi]
impl DecodeOptions {
  /// Create new decode options.
  #[napi(constructor)]
  pub fn new() -> DecodeOptions {
    Self {
      w: gif::DecodeOptions::new(),
      custom_options: CustomOptions { frame_buf_type: FrameBufType::IndexedPixels }
    }
  }

  /// Configure how color data is decoded.
  #[napi]
  pub fn set_color_output(&mut self, value: ColorOutput) {
    self.custom_options.frame_buf_type = match value {
      ColorOutput::Rgba => FrameBufType::Rgba,
      ColorOutput::IndexedPixels => FrameBufType::IndexedPixels
    };

    self.w.set_color_output(match value {
      ColorOutput::Rgba => gif::ColorOutput::RGBA,
      ColorOutput::IndexedPixels => gif::ColorOutput::Indexed
    });
  }

  /// Configure a memory limit for decoding.
  /// @param value - The memory limit in bytes. Negative values are treated as unlimited. (e.g. -1)
  /// If the provided value is `-1`, the memory limit is set to unlimited. If a positive integer is provided,
  /// the memory limit will be set in bytes. A non-zero integer is required for this case, and any non-integer or
  /// invalid value will return an error.
  #[napi]
  pub fn set_memory_limit(&mut self, value: i64) -> napi::Result<()> {
    if value <= -1 {
      self.w.set_memory_limit(gif::MemoryLimit::Unlimited);
    } else {
      self.w.set_memory_limit(gif::MemoryLimit::Bytes(
        NonZero::new(value as u64).ok_or_else(|| Error::new(
          Status::InvalidArg, "Limit must be a positive non-zero integer".to_string(),
        ))?
      ));
    }
    Ok(())
  }

  /// Configure if frames must be within the screen descriptor.
  /// @param value - Whether to check frame consistency.
  /// The default is `false`.
  /// When turned on, all frame descriptors being read must fit within the screen descriptor or otherwise an error is returned and the stream left in an unspecified state.
  /// When turned off, frames may be arbitrarily larger or offset in relation to the screen. Many other decoder libraries handle this in highly divergent ways. This moves all checks to the caller, for example to emulate a specific style.
  #[napi]
  pub fn check_frame_consistency(&mut self, value: bool) {
    self.w.check_frame_consistency(value);
  }

  /// Configure whether to skip decoding frames.
  /// @param value - Whether to skip frame decoding.
  /// The default is `false`.
  /// When turned on, LZW decoding is skipped. `<Decoder>.readNextFrame` will return compressed LZW bytes in frame’s data. `<Decoder>.nextFrameInfo` will return the metadata of the next frame as usual. This is useful to count frames without incurring the overhead of decoding.
  #[napi]
  pub fn skip_frame_decoding(&mut self, value: bool) {
    self.w.check_frame_consistency(value);
  }

  /// Configure if LZW encoded blocks must end with a marker end code.
  /// @param value - Whether to check for the end code.
  /// The default is `false`.
  /// When turned on, all image data blocks—which are LZW encoded—must contain a special bit sequence signalling the end of the data. LZW processing terminates when this code is encountered. The specification states that it must be the last code output by the encoder for an image.
  /// When turned off then image data blocks can simply end. Note that this might silently ignore some bits of the last or second to last byte.
  #[napi]
  pub fn check_lzw_end_code(&mut self, value: bool) {
    self.w.check_lzw_end_code(value);
  }

  /// Configure if unknown blocks are allowed to be decoded.
  /// @param value - Whether to allow unknown blocks.
  /// The default is `false`.
  /// When turned on, the decoder will allow unknown blocks to be in the `BlockStart` position.
  /// When turned off, decoded block starts must mark an `Image`, `Extension`, or `Trailer` block. Otherwise, the decoded image will return an error. If an unknown block error is returned from decoding, enabling this setting may allow for a further state of decoding on the next attempt.
  #[napi]
  pub fn allow_unknown_blocks(&mut self, value: bool) {
    self.w.allow_unknown_blocks(value);
  }

  /// Reads the logical screen descriptor including the global color palette
  /// Returns a Decoder. All decoder configuration has to be done beforehand.
  /// @param buffer - The GIF buffer to decode.
  #[napi]
  pub fn read_info(
    &self,
    buffer: &[u8],
  ) -> napi::Result<Decoder> {
    Ok(Decoder {
      w: self.w.clone().read_info(Cursor::new(buffer.to_vec()))
        .map_err(|e| Error::new(
          Status::GenericFailure, format!("Failed to create a GIF decoder: {}", e.to_string()),
        ))?, custom_options: self.custom_options.clone()
    })
  }
}