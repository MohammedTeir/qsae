use crate::error::Result;
use crate::codecs::Codec;

/// BWT + MTF codec for text-heavy data.
/// 
/// Pipeline: BWT → MTF → (RLE) → output
/// 
/// BWT permutes bytes so similar contexts cluster together.
/// MTF encodes each byte as its position in a recently-seen list,
/// producing many small integers ideal for downstream compression.
/// 
/// Phase 3: Full suffix array construction + BWT + MTF.
pub struct BwtCodec;

impl BwtCodec {
    fn build_suffix_array(data: &[u8]) -> Vec<usize> {
        let n = data.len();
        if n == 0 {
            return Vec::new();
        }

        let mut doubled = Vec::with_capacity(n * 2);
        doubled.extend_from_slice(data);
        doubled.extend_from_slice(data);

        if let Ok(s) = std::str::from_utf8(&doubled) {
            let st = suffix::SuffixTable::new(s);
            st.table()
                .iter()
                .map(|&idx| idx as usize)
                .filter(|&idx| idx < n)
                .collect()
        } else {
            let mut suffix_array: Vec<usize> = (0..n).collect();
            suffix_array.sort_by(|&a, &b| doubled[a..].cmp(&doubled[b..]));
            suffix_array
        }
    }

    /// Compute BWT from suffix array.
    /// BWT[i] = data[SA[i] - 1] (with wraparound)
    fn bwt_transform(data: &[u8], suffix_array: &[usize]) -> (Vec<u8>, usize) {
        let n = data.len();
        let mut bwt = Vec::with_capacity(n);
        let mut primary_index = 0;

        for (i, &suffix_idx) in suffix_array.iter().enumerate() {
            if suffix_idx == 0 {
                bwt.push(data[n - 1]);
                primary_index = i;
            } else {
                bwt.push(data[suffix_idx - 1]);
            }
        }

        (bwt, primary_index)
    }

    /// Inverse BWT: reconstruct original from BWT and primary index.
    fn inverse_bwt(bwt: &[u8], primary_index: usize, original_len: usize) -> Vec<u8> {
        let n = bwt.len();

        // Count occurrences and build first column pointers
        let mut count = [0usize; 256];
        for &byte in bwt {
            count[byte as usize] += 1;
        }

        // Cumulative count (starting positions in first column)
        let mut cumul = [0usize; 256];
        let mut sum = 0;
        for i in 0..256 {
            cumul[i] = sum;
            sum += count[i];
        }

        // Build LF mapping: for each position in BWT (last column),
        // find corresponding position in first column
        let mut next_occurrence = [0usize; 256];
        let mut lf = vec![0usize; n];

        for (i, &byte) in bwt.iter().enumerate() {
            let byte_idx = byte as usize;
            lf[i] = cumul[byte_idx] + next_occurrence[byte_idx];
            next_occurrence[byte_idx] += 1;
        }

        // Reconstruct: start from primary index, follow LF mapping
        let mut output = vec![0u8; original_len];
        let mut idx = primary_index;

        for i in (0..original_len).rev() {
            output[i] = bwt[idx];
            idx = lf[idx];
        }

        output
    }

    /// Move-To-Front transform.
    /// Encodes each byte as its position in a recently-seen list.
    fn mtf_transform(data: &[u8]) -> Vec<u8> {
        let mut list: Vec<u8> = (0..=255).collect();
        let mut output = Vec::with_capacity(data.len());

        for &byte in data {
            let pos = list.iter().position(|&x| x == byte).unwrap();
            output.push(pos as u8);

            // Move to front
            list.remove(pos);
            list.insert(0, byte);
        }

        output
    }

    /// Inverse Move-To-Front transform.
    fn inverse_mtf(data: &[u8]) -> Vec<u8> {
        let mut list: Vec<u8> = (0..=255).collect();
        let mut output = Vec::with_capacity(data.len());

        for &pos in data {
            let byte = list[pos as usize];
            output.push(byte);

            // Move to front
            list.remove(pos as usize);
            list.insert(0, byte);
        }

        output
    }

    /// Simple RLE for MTF output (many small integers = runs of zeros).
    fn rle_encode(data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }

        let mut output = Vec::with_capacity(data.len() / 2);
        let mut current = data[0];
        let mut count = 1u16;

        for &byte in &data[1..] {
            if byte == current && count < u16::MAX {
                count += 1;
            } else {
                output.push(current);
                output.extend_from_slice(&count.to_le_bytes());
                current = byte;
                count = 1;
            }
        }

        output.push(current);
        output.extend_from_slice(&count.to_le_bytes());

        output
    }

    fn rle_decode(data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }

        let mut output = Vec::new();
        let mut pos = 0;

        while pos < data.len() {
            let byte = data[pos];
            pos += 1;

            if pos + 2 > data.len() {
                break;
            }
            let count = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
            pos += 2;

            output.extend(std::iter::repeat(byte).take(count));
        }

        output
    }
}

impl Codec for BwtCodec {
    fn id(&self) -> u8 { 0x06 }
    fn name(&self) -> &'static str { "BWT" }

    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        if data.len() < 100 {
            // Too small for BWT to be effective, just store raw
            let mut output = vec![0x00]; // Marker: raw data
            output.extend_from_slice(&(data.len() as u32).to_le_bytes());
            output.extend_from_slice(data);
            return Ok(output);
        }

        // BWT → MTF → RLE
        let suffix_array = Self::build_suffix_array(data);
        let (bwt, primary_index) = Self::bwt_transform(data, &suffix_array);
        let mtf = Self::mtf_transform(&bwt);
        let rle = Self::rle_encode(&mtf);

        // Serialize: [marker: 1][primary_index: u32][rle_data...]
        let mut output = Vec::new();
        output.push(0x01); // Marker: BWT+MTF+RLE
        output.extend_from_slice(&(primary_index as u32).to_le_bytes());
        output.extend_from_slice(&(data.len() as u32).to_le_bytes());
        output.extend_from_slice(&rle);

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

        // BWT+MTF+RLE
        let primary_index = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;
        let original_len = u32::from_le_bytes([data[5], data[6], data[7], data[8]]) as usize;
        let rle_data = &data[9..];

        let mtf = Self::rle_decode(rle_data);
        let bwt = Self::inverse_mtf(&mtf);
        let original = Self::inverse_bwt(&bwt, primary_index, original_len);

        Ok(original)
    }

    fn suitable_for(&self, entropy: f64) -> bool {
        entropy >= 3.5 && entropy < 6.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bwt_roundtrip() {
        let codec = BwtCodec;
        let data = b"Hello, world! This is BWT test data. ".repeat(100);
        let compressed = codec.compress(&data[..]).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(&data[..], &decompressed[..]);
    }

    #[test]
    fn test_bwt_repetitive_text() {
        let codec = BwtCodec;
        // Text with repeated words (BWT excels here)
        let data = b"the quick brown fox jumps over the lazy dog ".repeat(1000);
        let compressed = codec.compress(&data[..]).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(&data[..], &decompressed[..]);

        // Should compress better than raw for repetitive text
        assert!(compressed.len() < data.len(), 
            "BWT should compress repetitive text, got {} -> {}", data.len(), compressed.len());
    }

    #[test]
    fn test_bwt_small_data() {
        let codec = BwtCodec;
        // Small data should pass through
        let data = b"tiny";
        let compressed = codec.compress(&data[..]).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(&data[..], &decompressed[..]);
    }

    #[test]
    fn test_mtf_roundtrip() {
        let data = vec![0u8, 1, 2, 3, 0, 1, 2, 0, 1, 0];
        let mtf = BwtCodec::mtf_transform(&data);
        let recovered = BwtCodec::inverse_mtf(&mtf);
        assert_eq!(data, recovered);
    }

    #[test]
    fn test_rle_roundtrip() {
        let data = vec![0u8, 0, 0, 0, 1, 1, 2, 2, 2, 2];
        let rle = BwtCodec::rle_encode(&data);
        let recovered = BwtCodec::rle_decode(&rle);
        assert_eq!(data, recovered);
    }

    #[test]
    fn test_bwt_empty() {
        let codec = BwtCodec;
        let data: Vec<u8> = Vec::new();
        let compressed = codec.compress(&data).unwrap();
        assert!(compressed.is_empty());
    }

    #[test]
    fn test_bwt_dna_sequence() {
        let codec = BwtCodec;
        // DNA-like sequence (ACGT repeats)
        let data = b"ACGTACGTACGTACGTACGT".repeat(500);
        let compressed = codec.compress(&data[..]).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(&data[..], &decompressed[..]);
    }
}
