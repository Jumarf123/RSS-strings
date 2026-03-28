#[derive(Clone, Debug)]
pub struct ExtractedString {
    pub offset: usize,
    pub text: String,
}

pub fn extract_ascii_strings(data: &[u8], min_len: usize) -> Vec<ExtractedString> {
    let mut out = Vec::new();
    let mut start = None;
    for (idx, byte) in data.iter().enumerate() {
        if is_ascii_printable(*byte) {
            if start.is_none() {
                start = Some(idx);
            }
        } else if let Some(s) = start.take() {
            if idx - s >= min_len {
                let slice = &data[s..idx];
                out.push(ExtractedString {
                    offset: s,
                    text: String::from_utf8_lossy(slice).to_string(),
                });
            }
        }
    }

    if let Some(s) = start {
        if data.len() - s >= min_len {
            out.push(ExtractedString {
                offset: s,
                text: String::from_utf8_lossy(&data[s..]).to_string(),
            });
        }
    }

    out
}

pub fn extract_utf16_strings(data: &[u8], min_len: usize, extended: bool) -> Vec<ExtractedString> {
    let mut out = Vec::new();
    let mut start_byte = None;
    let mut current = Vec::new();

    for (i, chunk) in data.chunks_exact(2).enumerate() {
        let word = u16::from_le_bytes([chunk[0], chunk[1]]);
        if word == 0 {
            flush_utf16(
                i * 2,
                &mut start_byte,
                &mut current,
                min_len,
                extended,
                &mut out,
            );
            continue;
        }
        if let Some(ch) = char::from_u32(word as u32) {
            if unicode_allowed(ch, extended) {
                if start_byte.is_none() {
                    start_byte = Some(i * 2);
                }
                current.push(word);
                continue;
            }
        }
        flush_utf16(
            i * 2,
            &mut start_byte,
            &mut current,
            min_len,
            extended,
            &mut out,
        );
    }

    if !current.is_empty() {
        flush_utf16(
            data.len(),
            &mut start_byte,
            &mut current,
            min_len,
            extended,
            &mut out,
        );
    }

    out
}

fn flush_utf16(
    _end_byte: usize,
    start_byte: &mut Option<usize>,
    buf: &mut Vec<u16>,
    min_len: usize,
    extended: bool,
    out: &mut Vec<ExtractedString>,
) {
    if let Some(start) = start_byte.take() {
        if buf.len() >= min_len {
            if let Ok(text) = String::from_utf16(buf) {
                if text.chars().all(|c| unicode_allowed(c, extended)) {
                    out.push(ExtractedString {
                        offset: start,
                        text,
                    });
                }
            }
        }
    }
    buf.clear();
}

fn is_ascii_printable(b: u8) -> bool {
    matches!(b, 0x20..=0x7E)
}

fn unicode_allowed(ch: char, extended: bool) -> bool {
    if ch.is_ascii() {
        return is_ascii_printable(ch as u8);
    }
    if !extended {
        return false;
    }
    !ch.is_control() && !is_cjk(ch)
}

fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0x20000..=0x2A6DF | 0x2A700..=0x2B73F
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_ascii() {
        let data = b"hello world\x01yy";
        let result = extract_ascii_strings(data, 3);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, "hello world");
    }

    #[test]
    fn extracts_utf16() {
        let data: Vec<u8> = "test"
            .encode_utf16()
            .flat_map(|w| w.to_le_bytes())
            .collect();
        let result = extract_utf16_strings(&data, 4, false);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, "test");
        assert_eq!(result[0].offset, 0);
    }
}
