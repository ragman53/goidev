use lopdf::{Dictionary, Object};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct FontEncoding {
    pub map: HashMap<u8, String>,
}

impl FontEncoding {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn default_win_ansi() -> Self {
        let mut encoding = Self::new();
        populate_win_ansi(&mut encoding.map);
        encoding
    }

    pub fn decode(&self, bytes: &[u8]) -> String {
        // UTF-16BE check (BOM)
        if bytes.starts_with(&[0xFE, 0xFF]) {
            let u16_vec: Vec<u16> = bytes[2..]
                .chunks_exact(2)
                .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
                .collect();
            return String::from_utf16_lossy(&u16_vec);
        }

        let mut result = String::with_capacity(bytes.len());

        for &b in bytes {
            if let Some(s) = self.map.get(&b) {
                result.push_str(s);
            } else {
                // Fallback: Latin-1 / ASCII
                // In WinAnsiEncoding, 0x80-0xA0 are varying, but 0x20-0x7E are ASCII.
                // We keep raw byte as char if not mapped, which works for ASCII.
                // For unmapped high bytes, it produces Latin-1.
                result.push(b as char);
            }
        }

        result
    }

    pub fn apply_to_unicode(&mut self, content: &[u8]) {
        let content_str = String::from_utf8_lossy(content);
        let mut tokens = content_str.split_whitespace();

        while let Some(token) = tokens.next() {
            match token {
                "beginbfchar" => loop {
                    let t1 = match tokens.next() {
                        Some("endbfchar") | None => break,
                        Some(t) => t,
                    };
                    let t2 = match tokens.next() {
                        Some(t) => t,
                        None => break,
                    };

                    if let (Some(code), Some(uni)) = (parse_hex_string(t1), parse_hex_string(t2)) {
                        if code.len() == 1 {
                            if let Ok(s) = decode_utf16be(&uni) {
                                self.map.insert(code[0], s);
                            }
                        }
                    }
                },
                "beginbfrange" => {
                    loop {
                        let t1 = match tokens.next() {
                            Some("endbfrange") | None => break,
                            Some(t) => t,
                        };
                        let t2 = match tokens.next() {
                            Some(t) => t,
                            None => break,
                        };
                        let t3 = match tokens.next() {
                            Some(t) => t,
                            None => break,
                        };

                        if t3 == "[" {
                            if let (Some(start), Some(end)) =
                                (parse_hex_string(t1), parse_hex_string(t2))
                            {
                                if start.len() == 1 && end.len() == 1 {
                                    let mut current = start[0];
                                    let limit = end[0];
                                    loop {
                                        let t_val = match tokens.next() {
                                            Some("]") | None => break,
                                            Some(t) => t,
                                        };
                                        if t_val == "]" {
                                            break;
                                        }

                                        if let Some(uni_bytes) = parse_hex_string(t_val) {
                                            if let Ok(s) = decode_utf16be(&uni_bytes) {
                                                self.map.insert(current, s);
                                            }
                                        }
                                        if current == limit {
                                            // Consume until ]
                                            while let Some(t) = tokens.next() {
                                                if t == "]" {
                                                    break;
                                                }
                                            }
                                            break;
                                        }
                                        current = current.wrapping_add(1);
                                    }
                                }
                            }
                        } else {
                            if let (Some(start), Some(end), Some(uni_start)) = (
                                parse_hex_string(t1),
                                parse_hex_string(t2),
                                parse_hex_string(t3),
                            ) {
                                if start.len() == 1 && end.len() == 1 {
                                    let mut current = start[0];
                                    let limit = end[0];
                                    if uni_start.len() == 2 {
                                        let mut uni_val =
                                            u16::from_be_bytes([uni_start[0], uni_start[1]]);
                                        while current <= limit {
                                            if let Ok(s) = String::from_utf16(&[uni_val]) {
                                                self.map.insert(current, s);
                                            }
                                            if current == limit {
                                                break;
                                            }
                                            current = current.wrapping_add(1);
                                            uni_val = uni_val.wrapping_add(1);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn parse_hex_string(s: &str) -> Option<Vec<u8>> {
    if s.starts_with('<') && s.ends_with('>') {
        let content = &s[1..s.len() - 1];
        let mut bytes = Vec::new();
        let mut chars = content.chars();
        while let Some(c1) = chars.next() {
            let c2 = chars.next()?;
            let hex = format!("{}{}", c1, c2);
            if let Ok(b) = u8::from_str_radix(&hex, 16) {
                bytes.push(b);
            } else {
                return None;
            }
        }
        Some(bytes)
    } else {
        None
    }
}

fn decode_utf16be(bytes: &[u8]) -> Result<String, std::string::FromUtf16Error> {
    let u16_vec: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect();
    String::from_utf16(&u16_vec)
}

/// Maps common Adobe Glyph List names to Unicode strings.
pub fn glyph_name_to_unicode(name: &str) -> Option<&'static str> {
    match name {
        // Ligatures
        "ff" => Some("ff"),
        "fi" => Some("fi"),
        "fl" => Some("fl"),
        "ffi" => Some("ffi"),
        "ffl" => Some("ffl"),

        // Quotes
        "quoteleft" => Some("\u{2018}"),      // ‘
        "quoteright" => Some("\u{2019}"),     // ’
        "quotedblleft" => Some("\u{201C}"),   // “
        "quotedblright" => Some("\u{201D}"),  // ”
        "quotesinglbase" => Some("\u{201A}"), // ‚
        "quotedblbase" => Some("\u{201E}"),   // „

        // Punctuation
        "endash" => Some("\u{2013}"),   // –
        "emdash" => Some("\u{2014}"),   // —
        "ellipsis" => Some("\u{2026}"), // …
        "bullet" => Some("\u{2022}"),   // •
        "hyphen" => Some("\u{002D}"),   // -
        "space" => Some(" "),

        // Common symbols
        "copyright" => Some("\u{00A9}"),  // ©
        "registered" => Some("\u{00AE}"), // ®
        "trademark" => Some("\u{2122}"),  // ™

        // Accents / Special (Latin-1 sup)
        "Agrave" => Some("\u{00C0}"),
        "Aacute" => Some("\u{00C1}"),
        "Acircumflex" => Some("\u{00C2}"),
        "Atilde" => Some("\u{00C3}"),
        "Adieresis" => Some("\u{00C4}"),
        "Aring" => Some("\u{00C5}"),
        "AE" => Some("\u{00C6}"),
        "Ccedilla" => Some("\u{00C7}"),
        "Egrave" => Some("\u{00C8}"),
        "Eacute" => Some("\u{00C9}"),
        "Ecircumflex" => Some("\u{00CA}"),
        "Edieresis" => Some("\u{00CB}"),
        "Igrave" => Some("\u{00CC}"),
        "Iacute" => Some("\u{00CD}"),
        "Icircumflex" => Some("\u{00CE}"),
        "Idieresis" => Some("\u{00CF}"),
        "Eth" => Some("\u{00D0}"),
        "Ntilde" => Some("\u{00D1}"),
        "Ograve" => Some("\u{00D2}"),
        "Oacute" => Some("\u{00D3}"),
        "Ocircumflex" => Some("\u{00D4}"),
        "Otilde" => Some("\u{00D5}"),
        "Odieresis" => Some("\u{00D6}"),
        "Multiply" => Some("\u{00D7}"),
        "Oslash" => Some("\u{00D8}"),
        "Ugrave" => Some("\u{00D9}"),
        "Uacute" => Some("\u{00DA}"),
        "Ucircumflex" => Some("\u{00DB}"),
        "Udieresis" => Some("\u{00DC}"),
        "Yacute" => Some("\u{00DD}"),
        "Thorn" => Some("\u{00DE}"),
        "germandbls" => Some("\u{00DF}"),
        "agrave" => Some("\u{00E0}"),
        "aacute" => Some("\u{00E1}"),
        "acircumflex" => Some("\u{00E2}"),
        "atilde" => Some("\u{00E3}"),
        "adieresis" => Some("\u{00E4}"),
        "aring" => Some("\u{00E5}"),
        "ae" => Some("\u{00E6}"),
        "ccedilla" => Some("\u{00E7}"),
        "egrave" => Some("\u{00E8}"),
        "eacute" => Some("\u{00E9}"),
        "ecircumflex" => Some("\u{00EA}"),
        "edieresis" => Some("\u{00EB}"),
        "igrave" => Some("\u{00EC}"),
        "iacute" => Some("\u{00ED}"),
        "icircumflex" => Some("\u{00EE}"),
        "idieresis" => Some("\u{00EF}"),
        "eth" => Some("\u{00F0}"),
        "ntilde" => Some("\u{00F1}"),
        "ograve" => Some("\u{00F2}"),
        "oacute" => Some("\u{00F3}"),
        "ocircumflex" => Some("\u{00F4}"),
        "otilde" => Some("\u{00F5}"),
        "odieresis" => Some("\u{00F6}"),
        "divide" => Some("\u{00F7}"),
        "oslash" => Some("\u{00F8}"),
        "ugrave" => Some("\u{00F9}"),
        "uacute" => Some("\u{00FA}"),
        "ucircumflex" => Some("\u{00FB}"),
        "udieresis" => Some("\u{00FC}"),
        "yacute" => Some("\u{00FD}"),
        "thorn" => Some("\u{00FE}"),
        "ydieresis" => Some("\u{00FF}"),

        _ => None,
    }
}

/// Populate standard WinAnsiEncoding (simplified)
pub fn populate_win_ansi(map: &mut HashMap<u8, String>) {
    // WinAnsi specific overrides for 0x80-0x9F
    let overrides = [
        (0x82, "\u{201A}"), // quotesinglbase
        (0x83, "\u{0192}"), // florin
        (0x84, "\u{201E}"), // quotedblbase
        (0x85, "\u{2026}"), // ellipsis
        (0x86, "\u{2020}"), // dagger
        (0x87, "\u{2021}"), // daggerdbl
        (0x88, "\u{02C6}"), // circumflex
        (0x89, "\u{2030}"), // perthousand
        (0x8A, "\u{0160}"), // Scaron
        (0x8B, "\u{2039}"), // guilsinglleft
        (0x8C, "\u{0152}"), // OE
        (0x8E, "\u{017D}"), // Zcaron
        (0x91, "\u{2018}"), // quoteleft
        (0x92, "\u{2019}"), // quoteright
        (0x93, "\u{201C}"), // quotedblleft
        (0x94, "\u{201D}"), // quotedblright
        (0x95, "\u{2022}"), // bullet
        (0x96, "\u{2013}"), // endash
        (0x97, "\u{2014}"), // emdash
        (0x98, "\u{02DC}"), // tilde
        (0x99, "\u{2122}"), // trademark
        (0x9A, "\u{0161}"), // scaron
        (0x9B, "\u{203A}"), // guilsinglright
        (0x9C, "\u{0153}"), // oe
        (0x9E, "\u{017E}"), // zcaron
        (0x9F, "\u{0178}"), // Ydieresis
    ];

    for (code, uni) in overrides {
        map.insert(code, uni.to_string());
    }

    // 0xA0-0xFF map directly to Unicode 0x00A0-0x00FF (Latin-1)
    // We don't strictly need to fill them if our fallback is char cast,
    // but populating them ensures consistency.
    for b in 0xA0..=0xFF {
        map.insert(b, (b as char).to_string());
    }
}

pub fn parse_font_encoding(font_dict: &Dictionary) -> FontEncoding {
    let mut encoding = FontEncoding::new();
    let mut base_is_win_ansi = false;

    // Check Encoding entry
    if let Ok(enc_obj) = font_dict.get(b"Encoding") {
        match enc_obj {
            Object::Name(name) => {
                if name == b"WinAnsiEncoding" {
                    base_is_win_ansi = true;
                }
            }
            Object::Dictionary(dict) => {
                // Check BaseEncoding
                if let Ok(Object::Name(base)) = dict.get(b"BaseEncoding") {
                    if base == b"WinAnsiEncoding" {
                        base_is_win_ansi = true;
                    }
                }

                // Apply Base Encoding
                if base_is_win_ansi {
                    populate_win_ansi(&mut encoding.map);
                }

                // Apply Differences
                if let Ok(Object::Array(diffs)) = dict.get(b"Differences") {
                    let mut current_code = 0u8;
                    for item in diffs {
                        match item {
                            Object::Integer(code) => {
                                current_code = *code as u8;
                            }
                            Object::Name(name) => {
                                if let Ok(name_str) = std::str::from_utf8(name) {
                                    if let Some(uni) = glyph_name_to_unicode(name_str) {
                                        encoding.map.insert(current_code, uni.to_string());
                                    }
                                }
                                current_code = current_code.wrapping_add(1);
                            }
                            _ => {}
                        }
                    }
                }

                return encoding; // Return early as we handled dictionary
            }
            _ => {}
        }
    }

    // If just Name WinAnsiEncoding (handled in match above first arm)
    if base_is_win_ansi {
        populate_win_ansi(&mut encoding.map);
    }

    encoding
}
