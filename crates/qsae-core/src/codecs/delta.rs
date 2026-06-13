use crate::error::Result;
use crate::codecs::Codec;

/// Delta encoder for numeric sequences and time-series data.
/// 
/// Detects and encodes:
/// - Sequential integers (1, 2, 3, ... → differences of 1)
/// - Fixed-width numeric arrays (u32, u64, f32, f64)
/// - Timestamps (slowly varying, good for delta)
/// - Sensor readings (small changes between samples)
/// 
/// Format: [detected_type: u8][width: u8][count: u32][deltas...]
/// 
/// Phase 3: Full delta + auto-detection + Huffman/RLE on residuals.
pub struct DeltaCodec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumericType {
    U8 = 0x01,
    U16 = 0x02,
    U32 = 0x03,
    U64 = 0x04,
    I8 = 0x05,
    I16 = 0x06,
    I32 = 0x07,
    I64 = 0x08,
    F32 = 0x09,
    F64 = 0x0A,
    Unknown = 0x00,
}

impl NumericType {
    fn width(&self) -> usize {
        match self {
            NumericType::U8 | NumericType::I8 => 1,
            NumericType::U16 | NumericType::I16 => 2,
            NumericType::U32 | NumericType::I32 | NumericType::F32 => 4,
            NumericType::U64 | NumericType::I64 | NumericType::F64 => 8,
            NumericType::Unknown => 1,
        }
    }

    fn from_u8(val: u8) -> Self {
        match val {
            0x01 => NumericType::U8,
            0x02 => NumericType::U16,
            0x03 => NumericType::U32,
            0x04 => NumericType::U64,
            0x05 => NumericType::I8,
            0x06 => NumericType::I16,
            0x07 => NumericType::I32,
            0x08 => NumericType::I64,
            0x09 => NumericType::F32,
            0x0A => NumericType::F64,
            _ => NumericType::Unknown,
        }
    }
}

impl DeltaCodec {
    /// Auto-detect numeric type based on data alignment and patterns.
    /// Returns `true` if the data appears to be a numeric sequence suitable
    /// for delta encoding (sequential integers, timestamps, sensor readings, etc.).
    pub fn is_numeric(data: &[u8]) -> bool {
        Self::detect_type(data) != NumericType::Unknown
    }

    /// Auto-detect numeric type based on data alignment and patterns.
    fn detect_type(data: &[u8]) -> NumericType {
        if data.len() < 16 {
            return NumericType::Unknown;
        }

        // Try different widths and see which produces smallest deltas
        let mut best_type = NumericType::Unknown;
        let mut best_score = f64::MAX;

        for candidate in [
            NumericType::U8, NumericType::U16, NumericType::U32, NumericType::U64,
            NumericType::I8, NumericType::I16, NumericType::I32, NumericType::I64,
        ] {
            let width = candidate.width();
            if data.len() % width != 0 {
                continue;
            }

            let count = data.len() / width;
            if count < 4 {
                continue;
            }

            // Calculate average delta magnitude
            let mut total_delta = 0.0f64;
            let mut prev = 0u64;

            for i in 0..count {
                let val = Self::read_uint(data, i * width, width);
                if i > 0 {
                    let delta = if val > prev { val - prev } else { prev - val };
                    total_delta += delta as f64;
                }
                prev = val;
            }

            if total_delta == 0.0 {
                continue; // Skip uniform data, RLE is better
            }
            let avg_delta = total_delta / (count - 1) as f64;
            let max_val = match candidate {
                NumericType::U8 | NumericType::I8 => 255.0,
                NumericType::U16 | NumericType::I16 => 65535.0,
                NumericType::U32 | NumericType::I32 => u32::MAX as f64,
                NumericType::U64 | NumericType::I64 => u64::MAX as f64,
                _ => 1.0,
            };
            let score = avg_delta / max_val; // Normalize

            if score < best_score {
                best_score = score;
                best_type = candidate;
            }
        }

        // If deltas are very small, it's likely numeric
        if best_score < 0.01 {
            best_type
        } else {
            NumericType::Unknown
        }
    }

    fn read_uint(data: &[u8], offset: usize, width: usize) -> u64 {
        match width {
            1 => data[offset] as u64,
            2 => u16::from_le_bytes([data[offset], data[offset + 1]]) as u64,
            4 => u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]) as u64,
            8 => u64::from_le_bytes([
                data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
                data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
            ]),
            _ => 0,
        }
    }

    fn write_uint(val: u64, width: usize) -> Vec<u8> {
        match width {
            1 => vec![val as u8],
            2 => (val as u16).to_le_bytes().to_vec(),
            4 => (val as u32).to_le_bytes().to_vec(),
            8 => val.to_le_bytes().to_vec(),
            _ => vec![val as u8],
        }
    }

    fn zigzag_encode(val: i64) -> u64 {
        ((val << 1) ^ (val >> 63)) as u64
    }

    fn zigzag_decode(val: u64) -> i64 {
        ((val >> 1) as i64) ^ (-((val & 1) as i64))
    }

    /// Encode deltas using variable-length encoding for small values (LEB128).
    fn encode_deltas(deltas: &[i64]) -> Vec<u8> {
        let mut output = Vec::with_capacity(deltas.len() * 2);
        for &delta in deltas {
            let mut val = Self::zigzag_encode(delta);
            loop {
                let mut byte = (val & 0x7F) as u8;
                val >>= 7;
                if val != 0 {
                    byte |= 0x80;
                    output.push(byte);
                } else {
                    output.push(byte);
                    break;
                }
            }
        }
        output
    }

    fn decode_deltas(data: &[u8], count: usize) -> Result<Vec<i64>> {
        let mut deltas = Vec::with_capacity(count);
        let mut pos = 0;

        for _ in 0..count {
            let mut result = 0u64;
            let mut shift = 0;
            loop {
                if pos >= data.len() {
                    return Err(crate::error::QsaeError::Decompression(
                        "Delta decode: insufficient data".to_string()
                    ));
                }
                let byte = data[pos];
                pos += 1;
                result |= ((byte & 0x7F) as u64) << shift;
                if byte & 0x80 == 0 {
                    break;
                }
                shift += 7;
                if shift >= 64 {
                    return Err(crate::error::QsaeError::Decompression(
                        "Delta decode: overflow".to_string()
                    ));
                }
            }
            deltas.push(Self::zigzag_decode(result));
        }

        Ok(deltas)
    }
}

impl Codec for DeltaCodec {
    fn id(&self) -> u8 { 0x07 }
    fn name(&self) -> &'static str { "Delta" }

    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        if data.len() < 16 {
            // Too small for delta to be effective
            let mut output = vec![0x00]; // Marker: raw data
            output.extend_from_slice(&(data.len() as u32).to_le_bytes());
            output.extend_from_slice(data);
            return Ok(output);
        }

        // Auto-detect numeric type
        let num_type = Self::detect_type(data);

        if num_type == NumericType::Unknown {
            // Not numeric, store raw
            let mut output = vec![0x00]; // Marker: raw data
            output.extend_from_slice(&(data.len() as u32).to_le_bytes());
            output.extend_from_slice(data);
            return Ok(output);
        }

        let width = num_type.width();
        let count = data.len() / width;

        // Calculate deltas
        let mut deltas = Vec::with_capacity(count);
        let mut prev = 0u64;

        for i in 0..count {
            let val = Self::read_uint(data, i * width, width);
            if i == 0 {
                deltas.push(val as i64); // First value is stored as-is
            } else {
                deltas.push(val.wrapping_sub(prev) as i64);
            }
            prev = val;
        }

        // Encode deltas with variable-length encoding
        let encoded_deltas = Self::encode_deltas(&deltas[1..]); // Skip first (stored separately)

        // Serialize: [marker: 1][type: 1][width: 1][count: u32][first_value: width][encoded_deltas...]
        let mut output = Vec::new();
        output.push(0x01); // Marker: Delta compressed
        output.push(num_type as u8);
        output.push(width as u8);
        output.extend_from_slice(&(count as u32).to_le_bytes());
        output.extend_from_slice(&Self::write_uint(deltas[0] as u64, width));
        output.extend_from_slice(&(encoded_deltas.len() as u32).to_le_bytes());
        output.extend_from_slice(&encoded_deltas);

        Ok(output)
    }

    fn decompress(&self, data: &[u8], _original_len: usize) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let marker = data[0];

        if marker == 0x00 {
            // Raw data
            let len = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;
            return Ok(data[5..5 + len].to_vec());
        }

        // Delta compressed
        let _num_type = NumericType::from_u8(data[1]);
        let width = data[2] as usize;
        let count = u32::from_le_bytes([data[3], data[4], data[5], data[6]]) as usize;
        let mut pos = 7;

        // Read first value
        let first_val = Self::read_uint(data, pos, width);
        pos += width;

        // Read encoded deltas
        let delta_len = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;
        let delta_data = &data[pos..pos + delta_len];

        let deltas = Self::decode_deltas(delta_data, count - 1)?;

        // Reconstruct original values
        let mut output = Vec::with_capacity(count * width);
        let mut current = first_val;

        // Write first value
        output.extend_from_slice(&Self::write_uint(current, width));

        // Apply deltas
        for delta in deltas {
            current = current.wrapping_add(delta as u64);
            output.extend_from_slice(&Self::write_uint(current, width));
        }

        Ok(output)
    }

    fn suitable_for(&self, entropy: f64) -> bool {
        entropy >= 1.0 && entropy < 5.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_sequential_integers() {
        let codec = DeltaCodec;
        // Sequential u32: 1, 2, 3, 4, ...
        let mut data = Vec::new();
        for i in 1..=1000u32 {
            data.extend_from_slice(&i.to_le_bytes());
        }

        let compressed = codec.compress(&data).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(data, decompressed);

        // Should compress very well (all deltas = 1)
        assert!(compressed.len() < 1100,
            "Sequential integers should compress well, got {} -> {}", data.len(), compressed.len());
    }

    #[test]
    fn test_delta_sensor_data() {
        let codec = DeltaCodec;
        // Sensor readings with small variations
        let mut data = Vec::new();
        let mut val = 100u16;
        for i in 0..1000 {
            data.extend_from_slice(&val.to_le_bytes());
            val = val.wrapping_add((i % 3) as u16); // Small changes
        }

        let compressed = codec.compress(&data).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_delta_non_numeric() {
        let codec = DeltaCodec;
        // Text data (not numeric, should pass through)
        let data = b"Hello, world! This is text data. ".repeat(100);
        let compressed = codec.compress(&data[..]).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(&data[..], &decompressed[..]);
    }

    #[test]
    fn test_delta_small_data() {
        let codec = DeltaCodec;
        let data = b"tiny";
        let compressed = codec.compress(&data[..]).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(&data[..], &decompressed[..]);
    }

    #[test]
    fn test_delta_timestamps() {
        let codec = DeltaCodec;
        // Timestamps (u64, slowly increasing)
        let mut data = Vec::new();
        let mut ts = 1609459200u64; // 2021-01-01
        for _ in 0..500 {
            data.extend_from_slice(&ts.to_le_bytes());
            ts += 60; // +1 minute
        }

        let compressed = codec.compress(&data).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(data, decompressed);

        // Should compress well (constant delta = 60)
        assert!(compressed.len() < data.len() / 4,
            "Timestamps should compress well, got {} -> {}", data.len(), compressed.len());
    }

    #[test]
    fn test_delta_empty() {
        let codec = DeltaCodec;
        let data: Vec<u8> = Vec::new();
        let compressed = codec.compress(&data).unwrap();
        assert!(compressed.is_empty());
    }

    #[test]
    fn test_delta_variable_length_encoding() {
        let deltas = vec![0i64, 1, -1, 100, -100, 10000, -10000, 100000];
        let encoded = DeltaCodec::encode_deltas(&deltas);
        let decoded = DeltaCodec::decode_deltas(&encoded, deltas.len()).unwrap();
        assert_eq!(deltas, decoded);
    }
}
