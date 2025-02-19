use napi::bindgen_prelude::*;
use napi_derive::napi;

fn can_short(hex: &str) -> bool {
    hex.len() % 2 == 0 && hex.as_bytes().chunks(2)
        .all(|pair| pair[0] == pair[1])
}

/// Convert RGBA to hex color. (e.g. `rgbaToHex([255, 0, 0, 255, 255, 128, 0, 128])` -> `["#FF0000", "#FF800080"]`)
/// @param rgba - An array of RGBA values.
/// @param alwaysIncludeAlpha - Whether to always include the alpha channel in the output.
/// @param allowShort - Whether to allow short hex output.
/// @returns An array of hex color strings. (e.g. `"#FF0000"`)
#[napi]
pub fn rgba_to_hex(
    rgba: &[u8],
    always_include_alpha: Option<bool>,
    allow_short: Option<bool>
) -> napi::Result<Vec<String>> {
    if rgba.len() % 4 != 0 {
        return Err(Error::new(
            Status::InvalidArg, "RGBA length must be a multiple of 4",
        ));
    }

    Ok(rgba.chunks(4)
        .map(|chunk| {
            let include_alpha = always_include_alpha.unwrap_or(false);
            let hex = if include_alpha {
                format!("#{:02X}{:02X}{:02X}{:02X}", chunk[0], chunk[1], chunk[2], chunk[3])
            } else if chunk[3] == 255 {
                format!("#{:02X}{:02X}{:02X}", chunk[0], chunk[1], chunk[2])
            } else {
                format!("#{:02X}{:02X}{:02X}{:02X}", chunk[0], chunk[1], chunk[2], chunk[3])
            };

            if allow_short.unwrap_or(false) && can_short(&hex[1..]) {
                match hex.len() {
                    9 => format!("#{}{}{}{}", &hex[1..2], &hex[3..4], &hex[5..6], &hex[7..8]),
                    7 => format!("#{}{}{}", &hex[1..2], &hex[3..4], &hex[5..6]),
                    _ => hex,
                }
            } else { hex }
        }).collect()
    )
}

/// Convert RGB to hex color. (e.g. `rgbToHex([255, 0, 0, 255, 128, 0])` -> `["#FF0000", "#FF8000"]`)
/// @param rgb - An array of RGB values.
/// @param allowShort - Whether to allow short hex output.
/// @returns An array of hex color strings. (e.g. `"#FF0000"`)
#[napi]
pub fn rgb_to_hex(
    rgb: &[u8],
    allow_short: Option<bool>
) -> napi::Result<Vec<String>> {
    if rgb.len() % 3 != 0 {
        return Err(Error::new(
            Status::InvalidArg, "RGB length must be a multiple of 3",
        ));
    }

    Ok(rgb.chunks(3)
        .map(|chunk| {
            let hex = format!("#{:02X}{:02X}{:02X}", chunk[0], chunk[1], chunk[2]);

            if allow_short.unwrap_or(false) && can_short(&hex[1..]) {
                format!("#{}{}{}", &hex[1..2], &hex[3..4], &hex[5..6])
            } else { hex }
        }).collect()
    )
}

/// Convert hex color to RGBA. (e.g. `hexToRgba(["#FF0000FF", "#800080FF"])` -> `[255, 0, 0, 255, 128, 0, 128, 255]`)
/// @param hex - An array of hex color strings.
/// @returns A flattened array of RGBA values.
#[napi]
pub fn hex_to_rgba(hex: Vec<String>) -> napi::Result<Buffer> {
    let mut rgba = Vec::with_capacity(hex.len() * 4);

    for h in hex {
        let h = h.trim_start_matches('#');
        let full = match h.len() {
            8 => h.to_string(),
            6 => format!("{}FF", h),
            4 => {
                let r = &h[0..1];
                let g = &h[1..2];
                let b = &h[2..3];
                let a = &h[3..4];
                format!("{r}{r}{g}{g}{b}{b}{a}{a}")
            }
            3 => {
                let r = &h[0..1];
                let g = &h[1..2];
                let b = &h[2..3];
                format!("{r}{r}{g}{g}{b}{b}FF")
            }
            _ => return Err(Error::new(Status::InvalidArg, &format!("Invalid hex: {}", h)))
        };

        rgba.extend_from_slice(
            [&full[0..2], &full[2..4], &full[4..6], &full[6..8]]
                .iter().map(|s| {
                    u8::from_str_radix(s, 16).map_err(|e| Error::new(
                        Status::GenericFailure, &format!("Failed to parse hex to RGBA: {}", e)
                    ))
                }).collect::<napi::Result<Vec<u8>>>()?
                .as_slice(),
        );
    }

    Ok(Buffer::from(rgba))
}

/// Convert hex color to RGB. (e.g. `hexToRgb(["#FF0000", "#FF8000"])` -> `[255, 0, 0, 255, 128, 0]`)
/// @param hex - An array of hex color strings.
/// @returns A flattened array of RGB values.
#[napi]
pub fn hex_to_rgb(hex: Vec<String>) -> napi::Result<Buffer> {
    let mut rgb = Vec::with_capacity(hex.len() * 3);

    for h in hex {
        let h = h.trim_start_matches('#');
        let full = match h.len() {
            6 => h.to_string(),
            3 => {
                let r = &h[0..1];
                let g = &h[1..2];
                let b = &h[2..3];
                format!("{r}{r}{g}{g}{b}{b}")
            }
            _ => return Err(Error::new(Status::InvalidArg, &format!("Invalid hex: {}", h)))
        };

        rgb.extend_from_slice([&full[0..2], &full[2..4], &full[4..6]]
            .iter().map(|s|
                u8::from_str_radix(s, 16).map_err(|e| Error::new(
                    Status::GenericFailure, &format!("Failed to convert hex to rgba: {}", e))
                )
            ).collect::<napi::Result<Vec<u8>>>()?
            .as_slice()
        );
    }
    Ok(Buffer::from(rgb))
}

#[napi]
pub fn indexed_to_rgba(
    pixels: &[u8], palette: &[u8],
    transparent: Option<u8>
) -> Buffer {
    let mut rgba = Vec::with_capacity(pixels.len() * 4);
    for &i in pixels {
        let start = i as usize * 3;
        rgba.extend_from_slice(&[
            palette[start], palette[start + 1],
            palette[start + 2],
            if transparent.is_some() { 255 } else { 0 }
        ]);
    }
    Buffer::from(rgba)
}

#[napi]
pub fn indexed_to_hex(
    pixels: &[u8], palette: &[u8],
    transparent: Option<u8>,
    always_include_alpha: Option<bool>,
    allow_short: Option<bool>,
) -> Vec<String> {
    let mut res = Vec::with_capacity(pixels.len());
    for &i in pixels {
        let start = i as usize * 3;
        let r = palette[start];
        let g = palette[start + 1];
        let b = palette[start + 2];
        let a = if transparent.is_some() { 255 } else { 0 };

        let hex = if always_include_alpha.unwrap_or(false)
            { format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a) }
        else if a == 255 { format!("#{:02X}{:02X}{:02X}", r, g, b) }
        else { format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a) };

        res.push(if allow_short.unwrap_or(false) && can_short(&hex[1..]) {
            match hex.len() {
                9 => format!("#{}{}{}{}", &hex[1..2], &hex[3..4], &hex[5..6], &hex[7..8]),
                7 => format!("#{}{}{}", &hex[1..2], &hex[3..4], &hex[5..6]),
                _ => hex,
            }
        } else { hex });
    }
    res
}