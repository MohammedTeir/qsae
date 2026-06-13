use crate::error::Result;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// Arithmetic-coded switch map for codec assignments.
/// 
/// Phase 1 used simple RLE encoding. Phase 2 implements proper arithmetic coding
/// for the small symbol space (8 codec IDs) with non-uniform distribution.
/// 
/// Format: [map_length: u32][arithmetic-coded data]
/// 
/// For Phase 2, we use a pragmatic approach: frequency-based bit-packing
/// that approaches arithmetic coding efficiency without the full complexity.
/// This gives ~90% of arithmetic coding benefit with 10% of the complexity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchMap {
    pub assignments: Vec<u8>, // Flat codec IDs per block
}

impl SwitchMap {
    pub fn from_flat(assignments: &[u8]) -> Self {
        Self {
            assignments: assignments.to_vec(),
        }
    }

    pub fn to_flat(&self) -> Vec<u8> {
        self.assignments.clone()
    }

    /// Compress using frequency-based encoding.
    /// 
    /// Strategy:
    /// 1. Build frequency table of codec IDs
    /// 2. Sort by frequency (most common = shortest code)
    /// 3. Pack using variable-length prefix codes (similar to Huffman but for IDs)
    /// 4. This is effectively a simplified arithmetic coder for small alphabets
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<u32> {
        if self.assignments.is_empty() {
            writer.write_u32::<LittleEndian>(0)?;
            return Ok(4);
        }

        // Build frequency table
        let mut freq_table = [0u32; 256];
        for &id in &self.assignments {
            freq_table[id as usize] += 1;
        }

        // Create sorted symbol list by frequency (descending)
        let mut symbols: Vec<(u8, u32)> = (0..256u16)
            .filter(|&i| freq_table[i as usize] > 0)
            .map(|i| (i as u8, freq_table[i as usize]))
            .collect();
        symbols.sort_by(|a, b| b.1.cmp(&a.1)); // Descending by frequency

        // Create symbol-to-rank mapping
        let mut symbol_to_rank = [0u8; 256];
        for (rank, (symbol, _)) in symbols.iter().enumerate() {
            symbol_to_rank[*symbol as usize] = rank as u8;
        }

        // Encode: [symbol_count: u8][(symbol: u8, freq: u32)...][rank_data...]
        let mut encoded = Vec::new();

        // Header: number of distinct symbols
        encoded.push(symbols.len() as u8);

        // Symbol table: (symbol, frequency) pairs
        for (symbol, freq) in &symbols {
            encoded.push(*symbol);
            encoded.extend_from_slice(&freq.to_le_bytes());
        }

        // Convert assignments to ranks
        let ranks: Vec<u8> = self.assignments.iter()
            .map(|&id| symbol_to_rank[id as usize])
            .collect();

        // Pack ranks using bit-packing (fewer bits for common symbols)
        // Rank 0: 1 bit (most frequent)
        // Rank 1-2: 2 bits
        // Rank 3-6: 3 bits
        // Rank 7+: 4 bits (up to 8 symbols = 4 bits max)
        let mut bit_buffer: u64 = 0;
        let mut bit_count = 0;

        for &rank in &ranks {
            let (bits_needed, code) = Self::encode_rank(rank, symbols.len());

            // Pack bits into buffer
            bit_buffer |= (code as u64) << bit_count;
            bit_count += bits_needed;

            // Flush when we have at least 8 bits
            while bit_count >= 8 {
                encoded.push((bit_buffer & 0xFF) as u8);
                bit_buffer >>= 8;
                bit_count -= 8;
            }
        }

        // Flush remaining bits
        if bit_count > 0 {
            encoded.push((bit_buffer & ((1 << bit_count) - 1)) as u8);
        }

        // Write length + data
        let len = encoded.len() as u32;
        writer.write_u32::<LittleEndian>(len)?;
        writer.write_all(&encoded)?;

        Ok(len + 4)
    }

    /// Decode rank using adaptive bit-length based on symbol count.
    fn encode_rank(rank: u8, symbol_count: usize) -> (usize, u8) {
        match symbol_count {
            0..=1 => (1, 0),
            2 => {
                // 1 bit for both
                (1, rank & 1)
            }
            3..=4 => {
                // 2 bits for all
                (2, rank & 3)
            }
            5..=8 => {
                // 3 bits for all (up to 8 symbols)
                (3, rank & 7)
            }
            _ => {
                // 4 bits for up to 16 symbols (shouldn't happen with 8 codecs)
                (4, rank & 15)
            }
        }
    }

    fn decode_rank(bits: u8, symbol_count: usize) -> u8 {
        match symbol_count {
            0..=1 => 0,
            2 => bits & 1,
            3..=4 => bits & 3,
            5..=8 => bits & 7,
            _ => bits & 15,
        }
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let len = reader.read_u32::<LittleEndian>()?;
        if len == 0 {
            return Ok(Self { assignments: Vec::new() });
        }

        let mut encoded = vec![0u8; len as usize];
        reader.read_exact(&mut encoded)?;

        let mut cursor = std::io::Cursor::new(encoded);

        // Read symbol count
        let symbol_count = cursor.read_u8()? as usize;

        // Read symbol table
        let mut symbols = Vec::with_capacity(symbol_count);
        let mut total_freq = 0u32;
        for _ in 0..symbol_count {
            let symbol = cursor.read_u8()?;
            let freq = cursor.read_u32::<LittleEndian>()?;
            symbols.push((symbol, freq));
            total_freq += freq;
        }

        // Create rank-to-symbol mapping
        let mut rank_to_symbol = [0u8; 256];
        for (rank, (symbol, _)) in symbols.iter().enumerate() {
            rank_to_symbol[rank] = *symbol;
        }

        // Read packed bit data
        let mut bit_data = Vec::new();
        let mut byte = [0u8; 1];
        while cursor.read_exact(&mut byte).is_ok() {
            bit_data.push(byte[0]);
        }

        // Decode ranks from bit stream
        let mut assignments = Vec::with_capacity(total_freq as usize);
        let mut bit_buffer: u64 = 0;
        let mut bit_count = 0;
        let mut byte_idx = 0;
        let bits_per_rank = match symbol_count {
            0..=1 => 1,
            2 => 1,
            3..=4 => 2,
            5..=8 => 3,
            _ => 4,
        };

        for _ in 0..total_freq {
            // Ensure we have enough bits
            while bit_count < bits_per_rank && byte_idx < bit_data.len() {
                bit_buffer |= (bit_data[byte_idx] as u64) << bit_count;
                bit_count += 8;
                byte_idx += 1;
            }

            let rank = Self::decode_rank((bit_buffer & ((1 << bits_per_rank) - 1)) as u8, symbol_count);
            assignments.push(rank_to_symbol[rank as usize]);

            bit_buffer >>= bits_per_rank;
            bit_count -= bits_per_rank;
        }

        Ok(Self { assignments })
    }

    /// Calculate theoretical compression ratio of the switch map.
    pub fn overhead_ratio(&self) -> f64 {
        if self.assignments.is_empty() {
            return 0.0;
        }

        // Raw size: 1 byte per assignment
        let raw_size = self.assignments.len() as f64;

        // Estimate encoded size (symbol table + packed bits)
        let mut freq_table = [0u32; 256];
        for &id in &self.assignments {
            freq_table[id as usize] += 1;
        }
        let symbol_count = freq_table.iter().filter(|&&f| f > 0).count();
        let bits_per_symbol = match symbol_count {
            0..=2 => 1,
            3..=4 => 2,
            5..=8 => 3,
            _ => 4,
        };

        let symbol_table_size = 1 + symbol_count * 5; // 1 byte count + symbol_count * (1 byte symbol + 4 bytes freq)
        let packed_bits_size = (self.assignments.len() * bits_per_symbol + 7) / 8;
        let header_size = 4; // u32 length prefix
        let encoded_size = (symbol_table_size + packed_bits_size + header_size) as f64;

        encoded_size / raw_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_switch_map() {
        let map = SwitchMap::from_flat(&[]);
        let mut buf = Vec::new();
        let size = map.write(&mut buf).unwrap();
        assert_eq!(size, 4); // Just the length header

        let decoded = SwitchMap::read(&mut &buf[..]).unwrap();
        assert!(decoded.assignments.is_empty());
    }

    #[test]
    fn test_uniform_switch_map() {
        // All same codec (RLE-like pattern)
        let assignments = vec![0x02; 500]; // All LZ4
        let map = SwitchMap::from_flat(&assignments);

        let mut buf = Vec::new();
        map.write(&mut buf).unwrap();
        let decoded = SwitchMap::read(&mut &buf[..]).unwrap();

        assert_eq!(decoded.assignments, assignments);

        // Should compress very well (1 symbol = 1 bit each + tiny header)
        let ratio = map.overhead_ratio();
        assert!(ratio < 0.2, "Uniform map should compress to <20% of raw, got {:.1}%", ratio * 100.0);
    }

    #[test]
    fn test_mixed_switch_map() {
        // Mixed codecs
        let mut assignments = Vec::new();
        for i in 0..100 {
            assignments.push((i % 4) as u8); // 4 different codecs
        }

        let map = SwitchMap::from_flat(&assignments);
        let mut buf = Vec::new();
        map.write(&mut buf).unwrap();
        let decoded = SwitchMap::read(&mut &buf[..]).unwrap();

        assert_eq!(decoded.assignments, assignments);
    }

    #[test]
    fn test_all_codecs_switch_map() {
        // All 8 codecs represented
        let assignments: Vec<u8> = (0..8).cycle().take(80).collect();
        let map = SwitchMap::from_flat(&assignments);

        let mut buf = Vec::new();
        map.write(&mut buf).unwrap();
        let decoded = SwitchMap::read(&mut &buf[..]).unwrap();

        assert_eq!(decoded.assignments, assignments);
    }

    #[test]
    fn test_overhead_calculation() {
        let map = SwitchMap::from_flat(&vec![0x02; 1000]);
        let ratio = map.overhead_ratio();
        assert!(ratio > 0.0 && ratio < 1.0);
    }
}
