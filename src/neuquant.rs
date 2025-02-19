use napi_derive::napi;
use napi::bindgen_prelude::*;

/// A NeuQuant instance.
#[napi]
pub struct NeuQuant {
    w: color_quant::NeuQuant
}

#[napi]
impl NeuQuant {
    /// Create a new NeuQuant instance.
    #[napi(constructor)]
    pub fn new(
        sample: i32,
        colors: u32,
        pixels: &[u8]
    ) -> napi::Result<NeuQuant> {
        let w = color_quant::NeuQuant::new(sample, colors as usize, pixels);
        Ok(NeuQuant { w })
    }

    /// Maps the rgba-pixel in-place to the best-matching color in the color map.
    #[napi]
    pub fn map_pixel(&self, pixel: &[u8]) {
        let mut pixel = pixel.to_vec();
        self.w.map_pixel(&mut pixel);
    }

    /// Finds the best-matching index in the color map.
    /// `pixel` is assumed to be in RGBA format.
    #[napi]
    pub fn index_of(&self, pixel: &[u8]) -> u32 {
        self.w.index_of(pixel) as u32
    }

    /// Lookup pixel values for color at `idx` in the colormap.
    #[napi]
    pub fn lookup(&self, idx: u32) -> Option<Buffer> {
        self.w.lookup(idx as usize).map(|p| Buffer::from(&p[..]))
    }

    /// Returns the RGBA color map calculated from the sample.
    #[napi]
    pub fn color_map_rgba(&self) -> napi::Result<Buffer> {
        Ok(Buffer::from(self.w.color_map_rgba()))
    }

    /// Returns the RGB color map calculated from the sample.
    #[napi]
    pub fn color_map_rgb(&self) -> napi::Result<Buffer> {
        Ok(Buffer::from(self.w.color_map_rgb()))
    }
}