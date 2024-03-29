// Copyright 2019 Peter Williams <pwil3058@gmail.com>

use std::collections::HashMap;

use crate::lines::Line;
use crate::text_diff::{DiffParseError, DiffParseResult};
use crate::DiffFormat;

const ENCODE: &[u8; 85] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&()*+-;<=>?@^_`{|}~";
const MAX_VAL: u64 = 0xFFFF_FFFF;

lazy_static! {
    static ref DECODE: HashMap<u8, u64> = {
        let mut decode_map = HashMap::new();
        for (index, chr) in ENCODE.iter().enumerate() {
            decode_map.insert(*chr, index as u64);
        }
        decode_map
    };
}

pub struct Encoding {
    string: Vec<u8>,
    size: usize,
}

pub fn encode(data: &[u8]) -> Encoding {
    let mut string: Vec<u8> = Vec::new();
    let mut index = 0;
    while index < data.len() {
        let mut acc: u64 = 0;
        for cnt in [24, 16, 8, 0].iter() {
            acc |= (data[index] as u64) << cnt;
            index += 1;
            if index == data.len() {
                break;
            }
        }
        let mut snippet: Vec<u8> = Vec::new();
        for _ in 0..5 {
            let val = acc % 85;
            acc /= 85;
            snippet.insert(0, ENCODE[val as usize]);
        }
        string.append(&mut snippet);
    }
    Encoding {
        string,
        size: data.len(),
    }
}

fn decode(encoding: &Encoding) -> DiffParseResult<Vec<u8>> {
    let mut data = vec![0u8; encoding.size];
    let mut d_index: usize = 0;
    let mut s_index: usize = 0;
    while d_index < encoding.size {
        let mut acc: u64 = 0;
        for _ in 0..5 {
            if s_index == encoding.string.len() {
                break;
            }
            if let Some(ch) = encoding.string.get(s_index) {
                if let Some(d) = DECODE.get(ch) {
                    acc = acc * 85 + d;
                } else {
                    return Err(DiffParseError::Base85Error(
                        "Illegal git base 85 character".to_string(),
                    ));
                }
                s_index += 1;
            } else {
                return Err(DiffParseError::Base85Error(format!(
                    "{s_index}: base85 source access out of range."
                )));
            }
        }
        if acc > MAX_VAL {
            return Err(DiffParseError::Base85Error(format!(
                "{acc}: base85 accumulator overflow."
            )));
        }
        for _ in 0..4 {
            if d_index == encoding.size {
                break;
            }
            acc = (acc << 8) | (acc >> 24);
            data[d_index] = (acc % 256) as u8;
            d_index += 1;
        }
    }
    Ok(data)
}

pub fn decode_size(ch: u8) -> DiffParseResult<usize> {
    if (b'A'..=b'Z').contains(&ch) {
        Ok((ch - b'A') as usize)
    } else if (b'a'..=b'z').contains(&ch) {
        Ok((ch - b'a' + 27) as usize)
    } else {
        Err(DiffParseError::UnexpectedInput(
            DiffFormat::GitBinary,
            format!("{}: expected char in range [azAZ]", ch as char),
        ))
    }
}

pub fn decode_line(line: &Line) -> DiffParseResult<Vec<u8>> {
    let string = line.trim_end().as_bytes();
    let size = decode_size(string[0])?;
    let encoding = Encoding {
        string: string[1..].to_vec(),
        size,
    };
    decode(&encoding)
}

pub fn decode_lines(lines: &[Line]) -> DiffParseResult<Vec<u8>> {
    let mut data: Vec<u8> = Vec::new();
    for line in lines.iter() {
        data.append(&mut decode_line(line)?);
    }
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    // test over a range of data sizes
    const TEST_DATA: &[u8] = b"uioyf2oyqo;3nhi8uydjauyo98ua 54\000jhkh\034hh;kjjh";

    #[test]
    fn git_base85_encode_decode_work() {
        for i in 0..10 {
            let encoding = encode(&TEST_DATA[i..]);
            let decoding = decode(&encoding).unwrap();
            assert_eq!(decoding, TEST_DATA[i..].to_vec());
        }
    }
}
