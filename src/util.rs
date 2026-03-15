use napi::bindgen_prelude::*;
use napi_derive::napi;

const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";
const HEX_DECODE: [u8; 256] = {
  let mut table = [0u8; 256];
  let mut i = 0;
  while i < 10 {
    table[(b'0' + i) as usize] = i;
    i += 1;
  }
  i = 0;
  while i < 6 {
    table[(b'a' + i) as usize] = 10 + i;
    table[(b'A' + i) as usize] = 10 + i;
    i += 1;
  }
  table
};

fn can_shorten(r: u8, g: u8, b: u8, a: Option<u8>) -> bool {
  r % 17 == 0 && g % 17 == 0 && b % 17 == 0 && a.map_or(true, |v| v % 17 == 0)
}

fn hex_as_u8(chars: &[u8]) -> u8 {
  (HEX_DECODE[chars[0] as usize] << 4) | HEX_DECODE[chars[1] as usize]
}

fn push_hex(s: &mut Vec<u8>, r: u8, g: u8, b: u8, a: Option<u8>, short: bool) {
  s.reserve(if short {
    1 + 3 + a.is_some() as usize
  } else {
    1 + 6 + a.is_some() as usize * 2
  });

  s.push(b'#');
  for val in [r, g, b].into_iter().chain(a) {
    if short {
      s.push(HEX_CHARS[(val & 0x0F) as usize]);
    } else {
      s.push(HEX_CHARS[(val >> 4) as usize]);
      s.push(HEX_CHARS[(val & 0x0F) as usize]);
    }
  }
}

#[napi]
pub fn rgba_to_hex(
  rgba: &[u8],
  always_include_alpha: Option<bool>,
  allow_short: Option<bool>,
) -> napi::Result<Buffer> {
  if rgba.len() % 4 != 0 {
    return Err(Error::new(
      Status::InvalidArg,
      "RGBA length must be a multiple of 4",
    ));
  }

  let alpha = always_include_alpha.unwrap_or(false);
  let mut buffer = Vec::with_capacity(rgba.len() / 4 * 9);

  for chunk in rgba.chunks_exact(4) {
    let a = if alpha || chunk[3] != 255 {
      Some(chunk[3])
    } else {
      None
    };

    push_hex(
      &mut buffer,
      chunk[0],
      chunk[1],
      chunk[2],
      a,
      allow_short.unwrap_or(false) && can_shorten(chunk[0], chunk[1], chunk[2], a),
    );
  }

  Ok(Buffer::from(buffer))
}

#[napi]
pub fn rgb_to_hex(rgb: &[u8], allow_short: Option<bool>) -> napi::Result<Buffer> {
  if rgb.len() % 3 != 0 {
    return Err(Error::new(
      Status::InvalidArg,
      "RGB length must be a multiple of 3",
    ));
  }

  let mut buffer = Vec::with_capacity(rgb.len() / 3 * 6);

  for chunk in rgb.chunks_exact(3) {
    push_hex(
      &mut buffer,
      chunk[0],
      chunk[1],
      chunk[2],
      None,
      allow_short.unwrap_or(false) && can_shorten(chunk[0], chunk[1], chunk[2], None),
    );
  }

  Ok(Buffer::from(buffer))
}

#[napi]
pub fn hex_to_rgba(hexes: Vec<String>) -> napi::Result<Buffer> {
  let mut rgba = Vec::with_capacity(hexes.len() * 4);
  for h in hexes {
    let b = h.as_bytes();
    let offset = if b.first() == Some(&b'#') { 1 } else { 0 };

    match b.len() - offset {
      8 | 6 => {
        rgba.push(hex_as_u8(&b[offset..offset + 2]));
        rgba.push(hex_as_u8(&b[offset + 2..offset + 4]));
        rgba.push(hex_as_u8(&b[offset + 4..offset + 6]));
        rgba.push(if b.len() - offset == 8 {
          hex_as_u8(&b[offset + 6..offset + 8])
        } else {
          255
        });
      }
      4 | 3 => {
        rgba.push((HEX_DECODE[b[offset] as usize] << 4) | HEX_DECODE[b[offset] as usize]);
        rgba.push((HEX_DECODE[b[offset + 1] as usize] << 4) | HEX_DECODE[b[offset + 1] as usize]);
        rgba.push((HEX_DECODE[b[offset + 2] as usize] << 4) | HEX_DECODE[b[offset + 2] as usize]);
        rgba.push(if b.len() - offset == 4 {
          (HEX_DECODE[b[offset + 3] as usize] << 4) | HEX_DECODE[b[offset + 3] as usize]
        } else {
          255
        });
      }
      _ => {
        return Err(Error::new(
          Status::InvalidArg,
          format!("Invalid hex: {}", h),
        ))
      }
    }
  }
  Ok(Buffer::from(rgba))
}

#[napi]
pub fn hex_to_rgb(hexes: Vec<String>) -> napi::Result<Buffer> {
  let mut rgb = Vec::with_capacity(hexes.len() * 3);
  for h in hexes {
    let b = h.as_bytes();
    let offset = if b.first() == Some(&b'#') { 1 } else { 0 };

    match b.len() - offset {
      6 => {
        rgb.push(hex_as_u8(&b[offset..offset + 2]));
        rgb.push(hex_as_u8(&b[offset + 2..offset + 4]));
        rgb.push(hex_as_u8(&b[offset + 4..offset + 6]));
      }
      3 => {
        rgb.push((HEX_DECODE[b[offset] as usize] << 4) | HEX_DECODE[b[offset] as usize]);
        rgb.push((HEX_DECODE[b[offset + 1] as usize] << 4) | HEX_DECODE[b[offset + 1] as usize]);
        rgb.push((HEX_DECODE[b[offset + 2] as usize] << 4) | HEX_DECODE[b[offset + 2] as usize]);
      }
      _ => {
        return Err(Error::new(
          Status::InvalidArg,
          format!("Invalid RGB hex: {}", h),
        ))
      }
    }
  }
  Ok(Buffer::from(rgb))
}

#[napi]
pub fn indexed_to_rgba(pixels: &[u8], palette: &[u8], transparent_index: Option<u8>) -> Buffer {
  let trans = transparent_index.unwrap_or(256u16 as u8);

  let mut rgba = Vec::with_capacity(pixels.len() * 4);
  for &i in pixels {
    let start = (i as usize) * 3;
    if start + 2 < palette.len() {
      rgba.push(palette[start]);
      rgba.push(palette[start + 1]);
      rgba.push(palette[start + 2]);
      rgba.push(if i == trans { 0 } else { 255 });
    } else {
      rgba.extend_from_slice(&[0, 0, 0, 255]);
    }
  }
  Buffer::from(rgba)
}

#[napi]
pub fn indexed_to_hex(
  pixels: &[u8],
  palette: &[u8],
  transparent_index: Option<u8>,
  always_include_alpha: Option<bool>,
  allow_short: Option<bool>,
) -> Buffer {
  let include_alpha = always_include_alpha.unwrap_or(false);
  let can_short = allow_short.unwrap_or(false);

  let mut cache: Vec<Vec<u8>> = Vec::with_capacity(palette.len() / 3);

  for (i, rgb) in palette.chunks_exact(3).enumerate() {
    let mut hex_buf = Vec::new();
    let is_transparent = Some(i as u8) == transparent_index;

    let a = if include_alpha || is_transparent {
      Some(if is_transparent { 0 } else { 255 })
    } else {
      None
    };

    let short = can_short && can_shorten(rgb[0], rgb[1], rgb[2], a);
    push_hex(&mut hex_buf, rgb[0], rgb[1], rgb[2], a, short);
    cache.push(hex_buf);
  }

  let mut result = Vec::with_capacity(pixels.len() * 7);

  for &pixel_index in pixels {
    if let Some(hex_str) = cache.get(pixel_index as usize) {
      result.extend_from_slice(hex_str);
    } else {
      result.extend_from_slice(b"#000000");
    }
  }

  Buffer::from(result)
}
