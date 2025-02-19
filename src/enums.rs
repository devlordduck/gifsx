use napi_derive::napi;

/// Output mode for the image data.
#[derive(PartialEq)]
#[napi]
pub enum ColorOutput {
  /// RGBA color mode, where each pixel is represented with red, green, blue, and alpha components.
  /// Suitable for detailed color rendering.
  Rgba,
  /// Indexed pixel mode, where each pixel references a color in a palette.
  /// Useful for memory-efficient representations with limited color ranges.
  IndexedPixels
}

#[derive(PartialEq, Clone)]
pub enum FrameBufType {
  Rgba, Rgb, IndexedPixels, Hex
}

/// Disposal method, describing how the next frame should be drawn over the current one.
#[napi]
pub enum DisposalMethod {
  /// Decoder is not required to take any specific action.
  Any,
  /// Retain the current frame as it is.
  Keep,
  /// Clear the frame and restore the canvas to its background color.
  Background,
  /// Restore the canvas to the previous frame's state.
  Previous
}