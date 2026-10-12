pub(super) fn parse(value: &str) -> Result<[u8; 3], String> {
    let value = value.trim();
    if let Some(hex) = value.strip_prefix('#') {
        return parse_hex(hex);
    }
    if let Some(rgb) = value
        .strip_prefix("rgb(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let values = rgb
            .split(',')
            .map(|part| part.trim().parse::<u8>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Tiptap color must use RGB components from 0 to 255")?;
        return values
            .as_slice()
            .try_into()
            .map_err(|_| "Tiptap RGB colors require three components".into());
    }
    match value {
        "black" => Ok([0, 0, 0]),
        "white" => Ok([255, 255, 255]),
        "red" => Ok([255, 0, 0]),
        "green" => Ok([0, 128, 0]),
        "blue" => Ok([0, 0, 255]),
        "yellow" => Ok([255, 255, 0]),
        "var(--color-content-yellow-subtle)" => Ok([0xfe, 0xf0, 0x8a]),
        "var(--color-content-green-subtle)" => Ok([0xbb, 0xf7, 0xd0]),
        "var(--color-content-blue-subtle)" => Ok([0xbf, 0xdb, 0xfe]),
        "var(--color-content-pink-subtle)" => Ok([0xfb, 0xcf, 0xe8]),
        _ => Err("Unsupported Tiptap color value".into()),
    }
}

fn parse_hex(value: &str) -> Result<[u8; 3], String> {
    let expanded;
    let value = if value.len() == 3 {
        expanded = value.chars().flat_map(|c| [c, c]).collect::<String>();
        expanded.as_str()
    } else {
        value
    };
    if value.len() != 6 {
        return Err("Tiptap colors must use three or six hexadecimal digits".into());
    }
    let number = u32::from_str_radix(value, 16).map_err(|_| "Invalid Tiptap hexadecimal color")?;
    Ok([(number >> 16) as u8, (number >> 8) as u8, number as u8])
}
