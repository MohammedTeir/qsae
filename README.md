# QSAE — Quorum Sensing Adaptive Encoder

> A bio-inspired compression engine that adaptively routes data regions to optimal codecs based on local entropy signals.

## Overview

QSAE is a next-generation compression engine inspired by bacterial quorum sensing. Unlike traditional compressors (gzip, Zstd, Brotli) that apply a single global strategy, QSAE partitions data into blocks, measures local entropy, accumulates quorum signals across neighborhoods, and routes each region to its optimal codec via contextual threshold detection.

**Key Innovation:** Every region of data gets the codec it deserves, based on neighborhood entropy trends.

## Four Deployment Surfaces (All Complete)

| Surface | Technology | Status | Package |
|---------|-----------|--------|---------|
| **CLI Tool** | Rust + clap | ✅ Phase 1 | `cargo install qsae-cli` |
| **Desktop GUI** | Tauri v2 + React | ✅ Phase 4 | Native binary |
| **Library SDK** | Rust crate | ✅ Phase 1 | `qsae-core` |
| **Web/WASM Demo** | wasm-pack + React | ✅ Phase 5 | Browser demo |

## SDK Bindings (Phase 6 — Complete)

| Language | Technology | Package | Status |
|----------|-----------|---------|--------|
| **Python** | PyO3 | `pip install qsae` | ✅ |
| **Node.js** | napi-rs | `npm install qsae-node` | ✅ |
| **C** | cbindgen | `libqsae.h` + `.so`/`.dll`/`.dylib` | ✅ |

## Architecture

```
Input Data
    ↓
Block Partitioner (entropy-driven boundaries)
    ↓
Entropy Scanner (Shannon H per block)
    ↓
Quorum Engine (Q(i) = Σ H(j) × e^{-λ|i-j|})
    ↓
Dispatcher (contextual codec routing)
    ↓
Parallel Codec Pool (9 codecs via Rayon)
    ↓
Frame Assembler (.qsae format)
    ↓
Compressed Output
```

## Project Structure

```
qsae/
├── Cargo.toml                    # Workspace manifest (7 crates)
├── crates/
│   ├── qsae-core/                # Core compression library
│   │   ├── src/
│   │   │   ├── lib.rs            # Public API
│   │   │   ├── compressor.rs     # Parallel compression
│   │   │   ├── decompressor.rs   # Decompression + inspect
│   │   │   ├── entropy.rs        # Shannon entropy
│   │   │   ├── quorum.rs         # Quorum sensing engine
│   │   │   ├── partitioner.rs    # Entropy-driven boundaries
│   │   │   ├── dispatcher.rs     # Contextual codec router
│   │   │   ├── bench.rs          # Benchmark suite
│   │   │   ├── codecs/           # 9 codec implementations
│   │   │   │   ├── rle.rs, lz4.rs, lz77.rs, huffman.rs
│   │   │   │   ├── ans.rs, bwt.rs, delta.rs, skip.rs, deflate.rs
│   │   │   └── format/           # .qsae file format
│   │   └── Cargo.toml
│   ├── qsae-cli/                 # Command-line tool
│   ├── qsae-wasm/                # WebAssembly build
│   ├── qsae-python/              # Python bindings (PyO3)
│   │   ├── src/lib.rs            # Python module exports
│   │   ├── examples/             # basic.py, batch_compress.py
│   │   └── tests/                # pytest tests
│   ├── qsae-node/                # Node.js bindings (napi-rs)
│   │   ├── src/lib.rs            # Async Node.js API
│   │   ├── examples/             # basic.js, stream.js
│   │   └── __test__/             # ava tests
│   └── qsae-c/                   # C bindings (cbindgen)
│       ├── src/lib.rs            # C-compatible API
│       ├── include/libqsae.h     # C header
│       └── examples/             # example.c + Makefile
├── apps/
│   ├── desktop/                  # Tauri v2 Desktop GUI
│   │   ├── src-tauri/            # Rust backend + IPC
│   │   └── src/                  # React frontend
│   │       ├── pages/            # Home, Progress, Results
│   │       └── components/       # DropZone, Heatmap, Charts
│   └── web-demo/                 # Web/WASM Demo
│       ├── src/                  # React frontend
│       │   ├── pages/            # Upload, Visualizer
│       │   └── components/       # EntropyHeatmap, QuorumSignal, CodecMap
│       └── src/wasm/             # wasm-pack output
└── tests/
    └── roundtrip.rs              # Integration tests
```

## Quick Start

### Prerequisites

- Rust 1.70+ ([rustup](https://rustup.rs))
- Node.js 18+ (for GUI and web)
- Python 3.8+ (for Python bindings)
- wasm-pack (for WASM)

### Install Tools

```bash
# wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# maturin (for Python)
pip install maturin

# napi-rs (for Node.js)
npm install -g @napi-rs/cli
```

### Build All

```bash
cd qsae

# Core library + CLI
cargo build --release

# WASM
cd crates/qsae-wasm
wasm-pack build --target web

# Python bindings
cd ../qsae-python
maturin develop

# Node.js bindings
cd ../qsae-node
napi build --platform

# C bindings
cd ../qsae-c
cargo build --release

# Desktop GUI
cd ../../apps/desktop
npm install
npm run tauri build

# Web Demo
cd ../web-demo
npm run wasm
npm install
npm run build
```

## Usage Examples

### CLI

```bash
qsae compress input.txt output.qsae
qsae decompress output.qsae restored.txt
qsae inspect output.qsae
qsae bench input.txt
```

### Python

```python
import qsae

# Compress
data = b"Hello, world! " * 1000
compressed = qsae.compress(data)

# Analyze
analysis = qsae.analyze(data)
print(f"Switch points: {analysis.switch_points}")

# Compress file
compressor = qsae.Compressor(lambda=0.3, delta=0.8)
stats = compressor.compress_file("input.txt", "output.qsae")
print(f"Ratio: {stats.ratio:.2f}:1")
```

### Node.js

```javascript
const qsae = require('qsae-node');

async function main() {
  const data = Buffer.from('Hello, world! '.repeat(1000));
  const result = await qsae.compress(data);
  console.log(`Ratio: ${result.ratio.toFixed(2)}:1`);

  const analysis = await qsae.analyze(data);
  console.log(`Blocks: ${analysis.block_count}`);
}
```

### C

```c
#include "libqsae.h"

QsaeConfig *config = qsae_config_new(0.5, 1.2, 65536);
QsaeCompressor *comp = qsae_compressor_new(config);

QsaeStats stats;
qsae_compress_file(comp, "input.txt", "output.qsae", &stats);
printf("Ratio: %.2f:1\n", stats.ratio);

qsae_compressor_free(comp);
qsae_config_free(config);
```

### Rust

```rust
use qsae::{Compressor, CompressorConfig, QuorumParams};

let config = CompressorConfig::builder()
    .quorum(QuorumParams::new().with_lambda(0.5).with_delta(1.2))
    .build();
let compressor = Compressor::new(config);
let compressed = compressor.compress(&data)?;
```

## Codecs (9 Total)

| ID | Codec | Entropy Zone | Best For |
|----|-------|-------------|----------|
| 0x00 | Skip | H ≥ 7.5 | Already-compressed |
| 0x01 | RLE | H < 1.0 | Uniform runs |
| 0x02 | LZ4 | 1.0 – 3.5 | Structured text |
| 0x03 | LZ77 | 1.0 – 5.0 | Repetitive patterns |
| 0x04 | Huffman | 4.5 – 6.0 | Skewed distributions |
| 0x05 | ANS | 6.0 – 7.5 | Dense binary |
| 0x06 | BWT | 3.5 – 6.5 | Text-heavy data |
| 0x07 | Delta | Numeric | Time-series |
| 0x08 | DEFLATE | Fallback | General-purpose |

## Implementation Roadmap (Complete)

### Phase 1 — Core Foundation ✅
- File format, entropy calculator, 3 codecs, CLI

### Phase 2 — Quorum Engine ✅
- Entropy-driven boundaries, quorum signals, contextual routing, arithmetic-coded switch map

### Phase 3 — Full Codec Pool ✅
- Custom LZ77, ANS, BWT+MTF, Delta, Rayon parallelism, benchmark suite

### Phase 4 — Desktop GUI ✅
- Tauri v2 + React, drag-drop, live visualization, mode presets

### Phase 5 — Web/WASM Demo ✅
- Browser-based WASM compression, interactive visualization

### Phase 6 — SDK Bindings ✅ (Current)
- Python (PyO3) → `pip install qsae`
- Node.js (napi-rs) → `npm install qsae-node`
- C (cbindgen) → `libqsae.h` + shared library

## Testing

```bash
# Rust tests
cargo test

# Python tests
cd crates/qsae-python
pytest

# Node.js tests
cd crates/qsae-node
npm test

# Integration tests
cargo test --test roundtrip
```

## Performance Notes

- **Parallel scaling:** Automatic via Rayon (Rust), scales with CPU cores
- **WASM:** Single-threaded, ~3-5× slower than native (acceptable for demo)
- **Python/Node.js/C:** Zero-copy where possible, minimal overhead over Rust core
- **Block size:** Default 64KB balances parallelism and granularity

## License

MIT OR Apache-2.0
