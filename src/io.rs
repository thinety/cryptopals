use std::fmt;

pub fn from_base16(input: &str) -> Result<Vec<u8>, ()> {
    let input = input.as_bytes();

    if input.len() % 2 != 0 {
        return Err(());
    }

    let mut bytes = Vec::with_capacity(input.len() / 2);

    for cs in input.chunks(2) {
        let cs: &[u8; 2] = cs.try_into().unwrap();

        let is = cs.map(|c| match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => 10 + c - b'a',
            b'A'..=b'F' => 10 + c - b'A',
            _ => 0xff,
        });

        match is {
            [i1 @ 0x00..=0x0f, i2 @ 0x00..=0x0f] => bytes.push(i1 << 4 | i2),
            _ => return Err(()),
        }
    }

    Ok(bytes)
}

pub fn from_base64(input: &str) -> Result<Vec<u8>, ()> {
    let input = input.as_bytes();

    if input.len() % 4 != 0 {
        return Err(());
    }

    let mut bytes = Vec::with_capacity(input.len() / 4);

    for cs in input.chunks(4) {
        let cs: &[u8; 4] = cs.try_into().unwrap();

        let is = cs.map(|c| match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => 26 + c - b'a',
            b'0'..=b'9' => 52 + c - b'0',
            b'+' => 62,
            b'/' => 63,
            b'=' => 0xfe,
            _ => 0xff,
        });

        match is {
            [i1 @ 0x00..=0x3f, i2 @ 0x00..=0x3f, i3 @ 0x00..=0x3f, i4 @ 0x00..=0x3f] => {
                bytes.extend([i1 << 2 | i2 >> 4, i2 << 4 | i3 >> 2, i3 << 6 | i4])
            }
            [i1 @ 0x00..=0x3f, i2 @ 0x00..=0x3f, i3 @ 0x00..=0x3f, 0xfe] => {
                bytes.extend([i1 << 2 | i2 >> 4, i2 << 4 | i3 >> 2])
            }
            [i1 @ 0x00..=0x3f, i2 @ 0x00..=0x3f, 0xfe, 0xfe] => bytes.extend([i1 << 2 | i2 >> 4]),
            _ => return Err(()),
        }
    }

    bytes.shrink_to_fit();

    Ok(bytes)
}

pub struct ToBase16<'a>(pub &'a [u8]);

impl fmt::Display for ToBase16<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bytes = self.0;

        let iter = bytes
            .iter()
            .flat_map(|b| [(b & 0b11110000) >> 4, (b & 0b00001111)])
            .map(|i| match i {
                0..=9 => b'0' + i,
                10..=15 => b'a' + i - 10,
                _ => unreachable!(),
            });

        for c in iter {
            use fmt::Write;
            f.write_char(c as char)?;
        }

        Ok(())
    }
}

pub struct ToBase64<'a>(pub &'a [u8]);

impl fmt::Display for ToBase64<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bytes = self.0;

        let iter = bytes
            .chunks(3)
            .flat_map(|bs| match bs {
                [b1] => [(b1 & 0b11111100) >> 2, (b1 & 0b00000011) << 4, 0xff, 0xff],
                [b1, b2] => [
                    (b1 & 0b11111100) >> 2,
                    (b1 & 0b00000011) << 4 | (b2 & 0b11110000) >> 4,
                    (b2 & 0b00001111) << 2,
                    0xff,
                ],
                [b1, b2, b3] => [
                    (b1 & 0b11111100) >> 2,
                    (b1 & 0b00000011) << 4 | (b2 & 0b11110000) >> 4,
                    (b2 & 0b00001111) << 2 | (b3 & 0b11000000) >> 6,
                    (b3 & 0b00111111),
                ],
                _ => unreachable!(),
            })
            .map(|i| match i {
                0..=25 => b'A' + i,
                26..=51 => b'a' + i - 26,
                52..=61 => b'0' + i - 52,
                62 => b'+',
                63 => b'/',
                0xff => b'=',
                _ => unreachable!(),
            });

        for c in iter {
            use fmt::Write;
            f.write_char(c as char)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE16: &str = "49276d206b696c6c696e6720796f757220627261696e206c696b65206120706f69736f6e6f7573206d757368726f6f6d";
    const BASE64: &str = "SSdtIGtpbGxpbmcgeW91ciBicmFpbiBsaWtlIGEgcG9pc29ub3VzIG11c2hyb29t";
    const TEXT: &str = "I'm killing your brain like a poisonous mushroom";

    #[test]
    fn from_base16_works() {
        let bytes = from_base16(BASE16).unwrap();
        assert_eq!(bytes, TEXT.as_bytes());
    }

    #[test]
    fn from_base64_works() {
        let bytes = from_base64(BASE64).unwrap();
        assert_eq!(bytes, TEXT.as_bytes());
    }

    #[test]
    fn to_base16_works() {
        let bytes = TEXT.as_bytes();
        assert_eq!(format!("{}", ToBase16(bytes)), BASE16);
    }

    #[test]
    fn to_base64_works() {
        let bytes = TEXT.as_bytes();
        assert_eq!(format!("{}", ToBase64(bytes)), BASE64);
    }
}
