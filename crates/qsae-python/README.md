# QSAE Python Bindings

Python bindings for the QSAE compression engine using PyO3.

## Installation

```bash
pip install qsae
```

Or build from source:

```bash
cd crates/qsae-python
maturin develop
```

## Quick Start

```python
import qsae

# Compress bytes
data = b"Hello, world! " * 1000
compressed = qsae.compress(data, lambda=0.5, delta=1.2)

# Decompress
original = qsae.decompress(compressed)

# Analyze without compressing
analysis = qsae.analyze(data)
print(f"Switch points: {analysis.switch_points}")
print(f"Entropy profile: {analysis.entropy_profile[:10]}")

# Compress file with stats
compressor = qsae.Compressor(lambda=0.3, delta=0.8, block_size=32768)
stats = compressor.compress_file("input.txt", "output.qsae")
print(f"Ratio: {stats.ratio:.2f}:1")
print(f"Codecs used: {stats.codec_usage}")

# Decompress file
decompressor = qsae.Decompressor()
decompressor.decompress_file("output.qsae", "restored.txt")

# Inspect .qsae file
info = decompressor.inspect(open("output.qsae", "rb").read())
print(f"Blocks: {info.block_count}, Ratio: {info.ratio:.2f}")
```

## API Reference

### `qsae.compress(data, lambda=0.5, delta=1.2, block_size=65536)`
Compress bytes and return compressed data.

### `qsae.decompress(data)`
Decompress .qsae data and return original bytes.

### `qsae.analyze(data, lambda=0.5, delta=1.2, block_size=65536)`
Analyze data without compressing. Returns `QuorumAnalysis` with entropy profile, quorum curve, and switch points.

### `qsae.Compressor(lambda, delta, block_size, use_quorum)`
Compressor instance with configurable parameters.
- `compress(data)` — compress bytes
- `compress_file(input_path, output_path)` — compress file, returns `CompressionStats`
- `analyze(data)` — analyze without compressing

### `qsae.Decompressor()`
Decompressor instance.
- `decompress(data)` — decompress bytes
- `decompress_file(input_path, output_path)` — decompress file
- `inspect(data)` — inspect .qsae file metadata

### `CompressionStats`
- `original_size` — original size in bytes
- `compressed_size` — compressed size in bytes
- `ratio` — compression ratio
- `block_count` — number of blocks
- `duration_ms` — compression time in milliseconds
- `codec_usage` — list of (codec_name, count, percentage)

### `QuorumAnalysis`
- `entropy_profile` — list of entropy values per block
- `quorum_curve` — list of quorum signal values
- `switch_points` — indices where codec switches occur
- `codec_assignments` — codec ID for each block
