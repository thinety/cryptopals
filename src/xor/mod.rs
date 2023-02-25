use std::iter;

pub fn xor(
    bytes1: impl Iterator<Item = u8>,
    bytes2: impl Iterator<Item = u8>,
) -> impl Iterator<Item = u8> {
    bytes1.zip(bytes2).map(|(b1, b2)| b1 ^ b2)
}

const DICTIONARY: &[(u8, f64)] = &[
    (b'A', 0.0651738),
    (b'B', 0.0124248),
    (b'C', 0.0217339),
    (b'D', 0.0349835),
    (b'E', 0.1041442),
    (b'F', 0.0197881),
    (b'G', 0.0158610),
    (b'H', 0.0492888),
    (b'I', 0.0558094),
    (b'J', 0.0009033),
    (b'K', 0.0050529),
    (b'L', 0.0331490),
    (b'M', 0.0202124),
    (b'N', 0.0564513),
    (b'O', 0.0596302),
    (b'P', 0.0137645),
    (b'Q', 0.0008606),
    (b'R', 0.0497563),
    (b'S', 0.0515760),
    (b'T', 0.0729357),
    (b'U', 0.0225134),
    (b'V', 0.0082903),
    (b'W', 0.0171272),
    (b'X', 0.0013692),
    (b'Y', 0.0145984),
    (b'Z', 0.0007836),
    (b' ', 0.1918182),
];

// https://crypto.stackexchange.com/a/56477
pub fn englishness(bytes: impl Iterator<Item = u8>) -> f64 {
    let mut counts = std::collections::HashMap::<u8, usize>::new();
    let mut total = 0;

    for b in bytes {
        *counts.entry(b.to_ascii_uppercase()).or_insert(0) += 1;
        total += 1;
    }

    let mut bc = 0.0;

    for (b, f) in DICTIONARY {
        let count = *counts.get(b).unwrap_or(&0);
        bc += ((count as f64) / (total as f64) * f).sqrt();
    }

    bc
}

pub fn find_single_byte_xor_key(bytes: impl Iterator<Item = u8> + Clone) -> (u8, f64) {
    let mut best_b = 0;
    let mut best_e = 0.0;

    for b in 0x00..=0xff {
        let decoded = xor(bytes.clone(), iter::repeat(b));
        let e = englishness(decoded);

        if e > best_e {
            best_b = b;
            best_e = e;
        }
    }

    (best_b, best_e)
}

pub fn hamming_distance(
    bytes1: impl Iterator<Item = u8>,
    bytes2: impl Iterator<Item = u8>,
) -> (usize, usize) {
    let mut count = 0;

    let distance = bytes1
        .zip(bytes2)
        .inspect(|_| count += 1)
        .map(|(b1, b2)| b1 ^ b2)
        .map(|b| b.count_ones() as usize)
        .sum();

    (distance, count * 8)
}

const MAX_SEARCHED_KEY_LENGTH: usize = 40;

// https://crypto.stackexchange.com/a/66402
pub fn find_repeating_key_xor_key_length(
    bytes: impl Iterator<Item = u8> + ExactSizeIterator + Clone,
) -> (usize, f64) {
    let mut best_l = 1;
    let mut best_d = 1.0;

    let max_len = bytes.len().min(MAX_SEARCHED_KEY_LENGTH + 1);
    for l in 1..max_len {
        let (d, c) = hamming_distance(bytes.clone().skip(l), bytes.clone());
        let d = (d as f64) / (c as f64);

        if d < best_d {
            best_l = l;
            best_d = d;
        }
    }

    (best_l, best_d)
}

pub fn find_repeating_key_xor_key(
    bytes: impl Iterator<Item = u8> + ExactSizeIterator + Clone,
) -> Vec<u8> {
    let (l, _) = find_repeating_key_xor_key_length(bytes.clone());

    let mut key = vec![0; l];

    for (i, b) in key.iter_mut().enumerate() {
        let (k, _) = find_single_byte_xor_key(bytes.clone().skip(i).step_by(l));
        *b = k;
    }

    key
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::io::*;

    #[test]
    fn xor_works() {
        let bytes1 = from_base16("1c0111001f010100061a024b53535009181c").unwrap();
        let bytes2 = from_base16("686974207468652062756c6c277320657965").unwrap();
        let bytes3 = from_base16("746865206b696420646f6e277420706c6179").unwrap();

        let result: Vec<u8> = xor(bytes1.iter().copied(), bytes2.iter().copied()).collect();

        assert_eq!(bytes3, result);
    }

    #[test]
    fn can_break_single_byte_xor() {
        let message =
            from_base16("1b37373331363f78151b7f2b783431333d78397828372d363c78373e783a393b3736")
                .unwrap();

        let (key, _) = find_single_byte_xor_key(message.iter().copied());
        let message: Vec<u8> = xor(message.into_iter(), iter::repeat(key)).collect();

        assert_eq!(
            String::from_utf8_lossy(&message),
            "Cooking MC's like a pound of bacon",
        );
    }

    #[test]
    fn can_find_single_byte_xor_string() {
        let file = include_str!("single_byte_xor.txt");

        let mut message = Vec::new();
        let mut best_key = 0x00;
        let mut best_englishness = 0.0;

        for line in file.lines() {
            let bytes = from_base16(line).unwrap();

            let (key, englishness) = find_single_byte_xor_key(bytes.iter().copied());

            if englishness > best_englishness {
                message = bytes;
                best_key = key;
                best_englishness = englishness;
            }
        }

        let message: Vec<u8> = xor(message.into_iter(), iter::repeat(best_key)).collect();

        assert_eq!(
            String::from_utf8_lossy(&message),
            "Now that the party is jumping\n",
        );
    }

    #[test]
    fn repeating_key_xor_works() {
        let message = "Burning 'em, if you ain't quick and nimble\nI go crazy when I hear a cymbal"
            .as_bytes();

        let message: Vec<u8> =
            xor(message.iter().copied(), b"ICE".iter().copied().cycle()).collect();

        assert_eq!(
            format!("{}", ToBase16(&message)),
            "0b3637272a2b2e63622c2e69692a23693a2a3c6324202d623d63343c2a26226324272765272a282b2f20430a652e2c652a3124333a653e2b2027630c692b20283165286326302e27282f",
        )
    }

    #[test]
    fn hamming_distance_works() {
        let (d, _) = hamming_distance(
            "this is a test".as_bytes().iter().copied(),
            "wokka wokka!!!".as_bytes().iter().copied(),
        );

        assert_eq!(d, 37);
    }

    #[test]
    fn find_repeating_key_xor_key_works() {
        let message: String = include_str!("repeating_key_xor.txt").lines().collect();
        let message = from_base64(&message).unwrap();

        let key = find_repeating_key_xor_key(message.iter().copied());

        assert_eq!(
            String::from_utf8_lossy(&key),
            "Terminator X: Bring the noise",
        );
    }
}
