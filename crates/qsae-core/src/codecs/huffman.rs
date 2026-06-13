use crate::error::Result;
use crate::codecs::Codec;
use std::collections::{BinaryHeap, HashMap};
use std::io::Cursor;
use std::io::Read;  // Added for read_exact

/// Canonical Huffman coding.
/// Builds frequency tree per block, emits canonical code table.
pub struct HuffmanCodec;

#[derive(Debug, Clone, Eq, PartialEq)]
struct HuffmanNode {
    freq: u64,
    symbol: Option<u8>,
    left: Option<Box<HuffmanNode>>,
    right: Option<Box<HuffmanNode>>,
}

impl Ord for HuffmanNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.freq.cmp(&self.freq) // min-heap via reverse
    }
}

impl PartialOrd for HuffmanNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl HuffmanCodec {
    fn build_tree(data: &[u8]) -> Option<HuffmanNode> {
        if data.is_empty() {
            return None;
        }

        let mut freqs = [0u64; 256];
        for &byte in data {
            freqs[byte as usize] += 1;
        }

        let mut heap = BinaryHeap::new();
        for (symbol, &freq) in freqs.iter().enumerate() {
            if freq > 0 {
                heap.push(HuffmanNode {
                    freq,
                    symbol: Some(symbol as u8),
                    left: None,
                    right: None,
                });
            }
        }

        if heap.len() == 1 {
            return heap.pop();
        }

        while heap.len() > 1 {
            let left = heap.pop().unwrap();
            let right = heap.pop().unwrap();
            heap.push(HuffmanNode {
                freq: left.freq + right.freq,
                symbol: None,
                left: Some(Box::new(left)),
                right: Some(Box::new(right)),
            });
        }

        heap.pop()
    }

    fn build_codes(node: &HuffmanNode, prefix: Vec<bool>, codes: &mut HashMap<u8, Vec<bool>>) {
        if let Some(symbol) = node.symbol {
            codes.insert(symbol, prefix);
        } else {
            if let Some(ref left) = node.left {
                let mut left_prefix = prefix.clone();
                left_prefix.push(false);
                Self::build_codes(left, left_prefix, codes);
            }
            if let Some(ref right) = node.right {
                let mut right_prefix = prefix.clone();
                right_prefix.push(true);
                Self::build_codes(right, right_prefix, codes);
            }
        }
    }

    fn encode(data: &[u8], codes: &HashMap<u8, Vec<bool>>) -> Vec<u8> {
        let mut bits = Vec::with_capacity(data.len() * 8);
        for &byte in data {
            bits.extend(&codes[&byte]);
        }

        // Pad to byte boundary
        let padding = (8 - (bits.len() % 8)) % 8;
        for _ in 0..padding {
            bits.push(false);
        }

        let mut bytes = Vec::with_capacity(bits.len() / 8);
        for chunk in bits.chunks(8) {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit {
                    byte |= 1 << (7 - i);
                }
            }
            bytes.push(byte);
        }

        bytes
    }

    fn decode(bytes: &[u8], tree: &HuffmanNode, original_len: usize) -> Vec<u8> {
        let mut bits = Vec::with_capacity(bytes.len() * 8);
        for &byte in bytes {
            for i in 0..8 {
                bits.push((byte & (1 << (7 - i))) != 0);
            }
        }

        let mut output = Vec::with_capacity(original_len);
        let mut node = tree;

        for &bit in &bits {
            if let Some(ref left) = node.left {
                if !bit {
                    node = left;
                } else if let Some(ref right) = node.right {
                    node = right;
                }
            }

            if let Some(symbol) = node.symbol {
                output.push(symbol);
                if output.len() >= original_len {
                    break;
                }
                node = tree;
            }
        }

        output
    }
}

impl Codec for HuffmanCodec {
    fn id(&self) -> u8 { 0x04 }
    fn name(&self) -> &'static str { "Huffman" }

    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let tree = Self::build_tree(data).unwrap();
        let mut codes = HashMap::new();
        Self::build_codes(&tree, Vec::new(), &mut codes);

        let encoded = Self::encode(data, &codes);

        // Serialize: [code_count: u16][(symbol: u8, code_len: u8, code_bits...)...][data...]
        let mut output = Vec::new();
        output.extend_from_slice(&(codes.len() as u16).to_le_bytes());

        for (symbol, code) in &codes {
            output.push(*symbol);
            output.push(code.len() as u8);
            // Pack code bits
            let mut code_byte = 0u8;
            let mut bit_count = 0;
            for &bit in code {
                if bit {
                    code_byte |= 1 << (7 - bit_count);
                }
                bit_count += 1;
                if bit_count == 8 {
                    output.push(code_byte);
                    code_byte = 0;
                    bit_count = 0;
                }
            }
            if bit_count > 0 {
                output.push(code_byte);
            }
        }

        output.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
        output.extend_from_slice(&encoded);

        Ok(output)
    }

    fn decompress(&self, data: &[u8], original_len: usize) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let mut cursor = Cursor::new(data);

        // Parse code table
        let mut code_count_buf = [0u8; 2];
        cursor.read_exact(&mut code_count_buf)?;
        let code_count = u16::from_le_bytes(code_count_buf) as usize;

        let mut codes: HashMap<u8, Vec<bool>> = HashMap::new();
        for _ in 0..code_count {
            let mut symbol_buf = [0u8; 1];
            cursor.read_exact(&mut symbol_buf)?;
            let symbol = symbol_buf[0];

            let mut len_buf = [0u8; 1];
            cursor.read_exact(&mut len_buf)?;
            let code_len = len_buf[0] as usize;

            let code_bytes = (code_len + 7) / 8;
            let mut code = Vec::with_capacity(code_len);
            for _byte_idx in 0..code_bytes {
                let mut byte_buf = [0u8; 1];
                cursor.read_exact(&mut byte_buf)?;
                let byte = byte_buf[0];
                for bit_idx in 0..8 {
                    if code.len() >= code_len { break; }
                    code.push((byte & (1 << (7 - bit_idx))) != 0);
                }
            }
            codes.insert(symbol, code);
        }

        let mut encoded_len_buf = [0u8; 4];
        cursor.read_exact(&mut encoded_len_buf)?;
        let encoded_len = u32::from_le_bytes(encoded_len_buf) as usize;

        let mut encoded = vec![0u8; encoded_len];
        cursor.read_exact(&mut encoded)?;

        // Build tree from codes
        let mut root = HuffmanNode { freq: 0, symbol: None, left: None, right: None };
        for (symbol, code) in &codes {
            let mut node = &mut root;
            for &bit in code {
                let branch = if bit { &mut node.right } else { &mut node.left };
                if branch.is_none() {
                    *branch = Some(Box::new(HuffmanNode { freq: 0, symbol: None, left: None, right: None }));
                }
                node = branch.as_mut().unwrap();
            }
            node.symbol = Some(*symbol);
        }

        Ok(Self::decode(&encoded, &root, original_len))
    }

    fn suitable_for(&self, entropy: f64) -> bool {
        entropy >= 4.5 && entropy < 6.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_huffman_roundtrip() {
        let codec = HuffmanCodec;
        let data = b"Hello, world! This is a test. ".repeat(50);
        let compressed = codec.compress(&data[..]).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(&data[..], &decompressed[..]);
    }

    #[test]
    fn test_huffman_skewed() {
        let codec = HuffmanCodec;
        // Highly skewed distribution
        let mut data = Vec::new();
        for _ in 0..1000 { data.push(0x00); }
        for _ in 0..100 { data.push(0xFF); }
        for _ in 0..10 { data.push(0x42); }

        let compressed = codec.compress(&data).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(data, decompressed);
        assert!(compressed.len() < data.len() / 2);
    }
}
