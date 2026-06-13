# QSAE — Quorum Sensing Adaptive Encoder
### Full Application Design Document v2.0

> A next-generation compression engine inspired by bacterial quorum sensing,
> implementing adaptive multi-codec routing via distributed entropy signaling.
> Deployed as a CLI tool, Desktop GUI app, embeddable Library, and interactive
> Web/WASM demo — targeting both developers and general users across all platforms.

---

## Table of Contents

1. [Vision & Core Idea](#1-vision--core-idea)
2. [Platform Architecture](#2-platform-architecture)
3. [Why QSAE is Different](#3-why-qsae-is-different)
4. [How Quorum Sensing Maps to Compression](#4-how-quorum-sensing-maps-to-compression)
5. [Full Compression Architecture](#5-full-compression-architecture)
6. [Algorithms — What, Why, When](#6-algorithms--what-why-when)
7. [Deployment Targets — Design & Features](#7-deployment-targets--design--features)
8. [Rust Tech Stack](#8-rust-tech-stack)
9. [File Format Specification (.qsae)](#9-file-format-specification-qsae)
10. [Full Module Breakdown](#10-full-module-breakdown)
11. [What the App Compresses](#11-what-the-app-compresses)
12. [Benchmarking Strategy](#12-benchmarking-strategy)
13. [Implementation Roadmap](#13-implementation-roadmap)
14. [Future Extensions](#14-future-extensions)

---

## 1. Vision & Core Idea

Every compression algorithm in existence — gzip, Zstd, Brotli, LZMA — applies a
**single, globally uniform strategy** to the entire input. They decide once: "this
file gets LZ77 + Huffman" and apply it from byte 0 to byte N.

Real-world data is never uniform. A single file can contain:
- JSON metadata (structured, low entropy)
- Base64-encoded binary (high entropy, near-random)
- Natural language description (moderate entropy, linguistic patterns)
- Numeric arrays (low-variance, delta-compressible)
- Repeated code patterns (LZ-optimal zones)

**QSAE solves this by asking each region of the data what it needs.**

Inspired by bacterial quorum sensing — where individual bacteria emit chemical
signals that accumulate until a population threshold triggers a collective
behavioral shift — QSAE partitions data into blocks that emit local entropy
signals. When cumulative signals cross a threshold, the compression strategy
switches. The result: every region is compressed by the codec it deserves.

---

## 2. Platform Architecture

QSAE is built as a **single Rust core library** with four deployment surfaces
on top of it. The core is written once and exposed everywhere.

```
┌──────────────────────────────────────────────────────────────────┐
│                    QSAE CORE (Rust Library)                      │
│         Quorum Engine + Codec Pool + File Format                 │
└──────┬──────────────┬───────────────┬──────────────┬────────────┘
       │              │               │              │
       ▼              ▼               ▼              ▼
┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────────┐
│  CLI Tool  │ │ Desktop GUI│ │  Library   │ │  Web / WASM    │
│            │ │  (Tauri v2)│ │    SDK     │ │     Demo       │
│ Linux      │ │            │ │            │ │                │
│ macOS      │ │ Windows    │ │ Rust crate │ │ Browser-based  │
│ Windows    │ │ macOS      │ │ Python lib │ │ Quorum visual  │
│            │ │ Linux      │ │ C bindings │ │ entropy map    │
│ Developers │ │            │ │ Node.js    │ │                │
│ Scripts    │ │ All users  │ │ bindings   │ │ All users      │
│ CI/CD      │ │            │ │            │ │ Demo/showcase  │
└────────────┘ └────────────┘ └────────────┘ └────────────────┘
```

### Why this architecture?

- **One core, four surfaces** — no duplicated compression logic
- **CLI** for developers, automation, server-side usage
- **Desktop GUI** for non-technical users who want drag-and-drop compression
- **Library SDK** so other apps can embed QSAE natively
- **WASM demo** to visually showcase what makes QSAE different (the quorum map)

The WASM demo is not just marketing — it's the clearest way to show
the algorithm working: users see entropy heatmaps and codec switching
in real-time as they upload a file.

---

## 3. Why QSAE is Different

| Property | gzip | Zstd | Brotli | **QSAE** |
|---|---|---|---|---|
| Multi-codec routing | ✗ | ✗ | ✗ | ✓ |
| Adaptive per-region strategy | ✗ | Partial | ✗ | ✓ |
| Entropy-driven switching | ✗ | ✗ | ✗ | ✓ |
| Heterogeneous file optimization | ✗ | ✗ | Partial | ✓ |
| Bio-inspired collective signaling | ✗ | ✗ | ✗ | ✓ |
| Parallel block compression | ✗ | Partial | ✗ | ✓ |
| Pluggable codec pool | ✗ | ✗ | ✗ | ✓ |
| GUI for non-technical users | ✗ | ✗ | ✗ | ✓ |
| Visual entropy map | ✗ | ✗ | ✗ | ✓ |

---

## 4. How Quorum Sensing Maps to Compression

### Biology

```
Bacterium emits signal molecule
    → signal accumulates in environment
    → when concentration > threshold T
    → entire colony switches behavior (e.g. biofilm formation)
```

### QSAE

```
Data block emits entropy signal H(block)
    → signal accumulates across neighboring blocks
    → when Σ H(block_i) × decay(distance) > threshold T
    → encoder switches to optimal codec for new entropy regime
```

### The Quorum Function

```
Q(i) = Σ  H(block_j) × e^{-λ × |i-j|}
       j∈window

if Q(i) - Q(i-1) > δ  →  switch_codec()
```

Where:
- `H(block)` = Shannon entropy of block i (bits per byte, 0–8)
- `λ` = decay constant (controls neighborhood sensitivity)
- `δ` = switching sensitivity threshold (tunable)
- `window` = local neighborhood size (default: 8 blocks)

---

## 5. Full Compression Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        INPUT DATA                           │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    BLOCK PARTITIONER                        │
│  Splits input into variable-length blocks                   │
│  Block boundaries triggered by entropy gradient > δ        │
│  Default block size: 64KB, min: 4KB, max: 512KB            │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                   ENTROPY SCANNER                           │
│  Computes H(block) for each block                           │
│  Detects: runs, periodicity, linguistic structure,          │
│           numeric sequences, random/encrypted regions       │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                   QUORUM ENGINE                             │
│  Accumulates entropy signals with exponential decay         │
│  Detects threshold crossings → emits SWITCH events         │
│  Outputs: codec assignment map [block_id → codec_id]        │
└───────────┬────────────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────┐
│                       CODEC POOL                            │
│   RLE │ LZ4 │ LZ77 │ Huffman │ ANS │ BWT+MTF │ Delta │ Skip│
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                   FRAME ASSEMBLER                           │
│  Arithmetic-codes the switching map                         │
│  Writes .qsae header + block table + map + payloads        │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 COMPRESSED OUTPUT (.qsae)                   │
└─────────────────────────────────────────────────────────────┘
```

---

## 6. Algorithms — What, Why, When

### 6.1 Shannon Entropy Calculator
**Role:** Quorum signal emitter — the foundation of everything

```
H(block) = -Σ p(byte_value) × log₂(p(byte_value))
```

Output range: 0.0 (all bytes identical) → 8.0 (perfectly random)

| Entropy Range | Interpretation | Codec Zone |
|---|---|---|
| H < 1.0 | Run-dominated | RLE |
| 1.0 – 4.5 | Structured patterns | LZ77 / LZ4 |
| 4.5 – 6.0 | Skewed distribution | BWT + Huffman |
| 6.0 – 7.5 | Dense, mixed | ANS |
| H ≥ 7.5 | Random / encrypted | SKIP |

---

### 6.2 Quorum Sensing Engine
**Role:** The brain — codec routing decisions

Computes Q(i) per block with exponential decay window. Detects entropy
inflection points as codec switch boundaries. Implements the MDL principle
(Minimum Description Length) at the structural level.

**Key parameters:**

| Parameter | Default | Effect |
|---|---|---|
| λ (decay) | 0.5 | Higher = more local sensitivity |
| δ (threshold) | 1.2 | Higher = fewer switches |
| window | 8 blocks | Neighborhood size |
| min_block_size | 4KB | Prevents micro-fragmentation |

---

### 6.3 RLE — Run-Length Encoding
**Role:** Zero-entropy zones

**When:** H(block) < 1.0

Consecutive identical bytes → (count, value) pairs. Extended RLE handles
multi-byte run patterns. Use cases: null-padded binary, sparse matrix zeros,
raw bitmap solid-color regions.

**Expected ratio:** 10:1 to 100:1 on eligible blocks.

---

### 6.4 LZ4 / LZ77
**Role:** Pattern-repetition zones — the workhorse

**When:** 1.0 ≤ H(block) < 5.0

Sliding window encodes repeated byte sequences as (distance, length)
back-references. LZ4 mode for speed, LZ77 extended (256KB window) for ratio.

Use cases: source code, JSON/XML/HTML, log files, config files.

---

### 6.5 Huffman Coding
**Role:** Skewed symbol distribution

**When:** 4.5 ≤ H(block) < 6.0

Builds a frequency tree: frequent bytes get short codes, rare bytes get long
codes. Block-local canonical Huffman trees rebuilt per block.

Use cases: natural language text, CSV data, bytecode with non-uniform opcodes.

---

### 6.6 ANS — Asymmetric Numeral Systems (rANS variant)
**Role:** High-entropy blocks, near-theoretical compression

**When:** 6.0 ≤ H(block) < 7.5

Encodes symbols with fractional-bit precision. No integer rounding waste like
Huffman. Integer-only arithmetic — faster than arithmetic coding with same ratio.
Used internally by Zstd and LZMA.

Use cases: dense binary data, raw audio samples, high-diversity byte regions.

---

### 6.7 BWT + MTF — Burrows-Wheeler Transform + Move To Front
**Role:** Preprocessing transform for text-heavy blocks

**When:** Applied before Huffman/ANS when H(block) is 3.5–6.5 AND
block is detected as linguistic/structured text.

BWT permutes bytes so similar contexts cluster. MTF encodes each byte
as its position in a recently-seen list → many small integers → ideal
for downstream Huffman. Improves ratio by 15–30% on text.

Use cases: natural language, source code strings, DNA sequences.

---

### 6.8 Delta Encoder
**Role:** Numeric sequences, time-series, sensor streams

**When:** Block analysis detects sequential integers, fixed-width numeric
arrays, timestamp sequences, or slowly-varying sensor readings.

Stores differences between values. XOR delta for floating-point (cancels
identical exponent/sign bits). Produces near-zero entropy stream → Huffman.

Use cases: scientific data, financial time-series, GPS coordinates, metrics.

---

### 6.9 Arithmetic Coding (Switching Map only)
**Role:** Compresses the codec assignment metadata

The switching map `[block_0→codec_2, block_1→codec_2, block_2→codec_5 ...]`
is itself compressed with arithmetic coding — small symbol space (8 codec IDs),
non-uniform distribution (dominant codec runs long). Keeps metadata overhead
minimal — critical for small files.

---

### 6.10 Skip / Store (Pass-through)
**Role:** Detects already-compressed or encrypted data

**When:** H(block) ≥ 7.5

Stores the block raw. Prevents QSAE from bloating output by trying to
compress JPEG, ZIP, AES-encrypted, or otherwise near-random data.

---

## 7. Deployment Targets — Design & Features

---

### 7.1 CLI Tool
**Users:** Developers, sysadmins, DevOps, CI/CD pipelines
**Platform:** Linux, macOS, Windows (native binary)
**Built with:** Rust + `clap` + `indicatif`

#### Commands

```bash
# Compress a file
qsae compress input.tar output.qsae

# Compress with custom quorum parameters
qsae compress input.tar output.qsae --lambda 0.3 --delta 0.8

# Decompress
qsae decompress output.qsae restored.tar

# Inspect a .qsae file — show codec map, entropy profile, stats
qsae inspect output.qsae

# Benchmark against other compressors
qsae bench ./my-corpus/ --compare gzip,zstd,brotli

# Compress a directory (recursive)
qsae compress ./project/ archive.qsae --recursive

# Stream mode (pipe support)
cat large.log | qsae compress --stream > compressed.qsae

# Show compression stats
qsae stats output.qsae
```

#### CLI Output Example

```
$ qsae compress codebase.tar codebase.qsae

Analyzing...  ████████████████████  100%
Compressing... ████████████████████  100%

Results:
  Original:    847.3 MB
  Compressed:  201.4 MB
  Ratio:       4.21:1  (76.2% reduction)
  vs gzip:     +34.1% better
  vs zstd:     +21.7% better

Codec usage:
  LZ77     ████████████░░░░  47%  (source code, configs)
  BWT+Huff ████████░░░░░░░░  31%  (text, docs)
  Delta    ████░░░░░░░░░░░░  14%  (numeric arrays)
  ANS      ██░░░░░░░░░░░░░░   6%  (dense binary)
  Skip     █░░░░░░░░░░░░░░░   2%  (pre-compressed assets)

Time: 3.2s  |  Speed: 264 MB/s
```

---

### 7.2 Desktop GUI App
**Users:** General users, non-technical users, anyone who wants
          drag-and-drop file compression
**Platform:** Windows, macOS, Linux (native desktop app)
**Built with:** Tauri v2 (Rust backend + React/TypeScript frontend)

#### Why Tauri v2?

- Rust core communicates with a web-based UI via Tauri's IPC bridge
- Frontend is React + TypeScript — rich, polished UI without writing
  a native GUI framework in Rust
- Produces a tiny native binary (no Electron/Chromium bloat)
- Same QSAE core used by CLI is called directly from Tauri backend
- v2 supports future mobile expansion (Android/iOS)

#### Screen: Main Window

```
┌──────────────────────────────────────────────────────────────┐
│  QSAE Compressor                              [ _ ] [ □ ] [×]│
├──────────────────────────────────────────────────────────────┤
│                                                              │
│         ┌────────────────────────────────────────┐          │
│         │                                        │          │
│         │      Drop files or folders here        │          │
│         │                                        │          │
│         │           or  [Browse Files]           │          │
│         └────────────────────────────────────────┘          │
│                                                              │
│  Output: [/home/user/compressed/         ] [Change]         │
│                                                              │
│  Mode:  ○ Fast   ● Balanced   ○ Max Ratio                   │
│                                                              │
│  [ Advanced Settings ▾ ]                                    │
│    Lambda (decay):  [0.5 ──●──────] 0.5                     │
│    Delta (threshold): [────●──────] 1.2                     │
│    Block size: [Auto ▾]                                      │
│                                                              │
│                           [ Compress ]                      │
└──────────────────────────────────────────────────────────────┘
```

#### Screen: Compression Progress + Entropy Map

```
┌──────────────────────────────────────────────────────────────┐
│  Compressing: codebase.tar                                   │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  Progress:  ████████████████░░░░  78%   198 MB / 254 MB     │
│  Speed:     312 MB/s                                         │
│                                                              │
│  Entropy Map (live)                                          │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ ░░░░▒▒▒▒▒▒▓▓▓▓▓▓▓▓▒▒▒░░░░░░░▒▒▒▒▓▓▓▓▓▓█████▓▓░░░░▒▒▒ │ │
│  │ LZ77──────BWT────────LZ77──────────ANS──────────LZ77── │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  Codec breakdown (so far):                                   │
│  LZ77 ████████ 49%  |  BWT ██████ 32%  |  ANS ███ 14%      │
│  Delta ██ 4%        |  Skip █ 1%                            │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

#### Screen: Results

```
┌──────────────────────────────────────────────────────────────┐
│  ✓ Compression Complete                                      │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│   254.3 MB  →  61.2 MB   (75.9% smaller)                   │
│                                                              │
│   Ratio:    4.15:1                                           │
│   Time:     4.1 seconds                                      │
│                                                              │
│   Compared to:                                               │
│   gzip   96.4 MB  — QSAE is 36.5% better                   │
│   zstd   72.1 MB  — QSAE is 15.1% better                   │
│   brotli 78.3 MB  — QSAE is 21.8% better                   │
│                                                              │
│   [ Open File ]   [ Open Folder ]   [ Compress Another ]    │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

#### Key GUI Features

- Drag and drop files and folders
- Live entropy heatmap during compression
- Real-time codec usage breakdown
- Comparison vs gzip/zstd/brotli after compression
- Advanced settings panel (λ, δ, block size)
- Dark mode / light mode
- File history (recent compressions)
- Batch compression (multiple files at once)
- Right-click integration (compress from file manager)

---

### 7.3 Library / SDK
**Users:** Developers embedding QSAE into their own applications
**Exposed as:** Rust crate, Python package, C shared library, Node.js module

#### Rust API (qsae crate on crates.io)

```rust
use qsae::{Compressor, CompressorConfig, QuorumParams};

let config = CompressorConfig::builder()
    .quorum(QuorumParams { lambda: 0.5, delta: 1.2, window: 8 })
    .parallel(true)
    .block_size_hint(65536)
    .build();

let compressor = Compressor::new(config);

// Compress bytes
let compressed = compressor.compress(&input_bytes)?;

// Decompress
let original = compressor.decompress(&compressed)?;

// Compress a file
compressor.compress_file("input.tar", "output.qsae")?;

// Stream API
let mut stream = compressor.compress_stream(writer)?;
stream.write_all(&chunk)?;
stream.finish()?;
```

#### Python Bindings (PyO3 → PyPI package `qsae-py`)

```python
import qsae

# Compress bytes
compressed = qsae.compress(data, lambda_decay=0.5, delta=1.2)
original   = qsae.decompress(compressed)

# Compress file
qsae.compress_file("input.tar", "output.qsae")

# Inspect a .qsae file
info = qsae.inspect("output.qsae")
print(info.codec_map)       # [{block: 0, codec: "LZ77"}, ...]
print(info.entropy_profile) # [3.2, 4.1, 6.8, 2.1, ...]
print(info.ratio)           # 4.15
```

#### C Bindings (cbindgen → `libqsae.h`)

```c
#include "libqsae.h"

QsaeConfig* cfg = qsae_config_default();
uint8_t* out    = NULL;
size_t   out_len = 0;

int result = qsae_compress(cfg, input, input_len, &out, &out_len);
if (result == QSAE_OK) {
    // use out buffer
    qsae_free(out);
}
qsae_config_free(cfg);
```

#### Node.js Bindings (napi-rs → npm package `qsae-node`)

```javascript
const qsae = require('qsae-node');

const compressed = await qsae.compress(buffer, { lambda: 0.5, delta: 1.2 });
const original   = await qsae.decompress(compressed);
```

---

### 7.4 Web / WASM Demo
**Users:** Everyone — students, researchers, curious users, potential adopters
**Platform:** Browser (any modern browser supporting WebAssembly)
**Built with:** Rust → wasm-pack → React + TypeScript + Vite + D3.js

#### Purpose

The WASM demo exists to **show what no other compressor can show** — the
quorum sensing algorithm working in real-time. Users can upload a file
and watch:

1. The entropy landscape render as a heatmap across the file
2. The quorum signal accumulate and propagate between blocks
3. Codec switching events fire at inflection points
4. The final codec assignment map colored by algorithm

This is the clearest explanation of QSAE that exists. Seeing it is
understanding it.

#### Demo Screens

**Screen 1 — Upload**
```
┌────────────────────────────────────────────────────┐
│   QSAE Interactive Demo                            │
│   ─────────────────────────────────────────────── │
│                                                    │
│      Drop a file here to visualize QSAE           │
│      compression in real-time                     │
│                                                    │
│      [ Upload File ]   Max 50MB in browser        │
│                                                    │
│   Or try a sample:  [Code]  [JSON]  [Mixed]       │
└────────────────────────────────────────────────────┘
```

**Screen 2 — Live Visualization**
```
┌────────────────────────────────────────────────────┐
│  Entropy Heatmap  (H value per block, 0–8)        │
│  ┌──────────────────────────────────────────────┐ │
│  │ 0   1   2   3   4   5   6   7   8  (entropy) │ │
│  │ █▓▒░ ... gradient heatmap of all blocks ...  │ │
│  └──────────────────────────────────────────────┘ │
│                                                    │
│  Quorum Signal Q(i)                               │
│  ┌──────────────────────────────────────────────┐ │
│  │   /\  /\    /\        /\/\                   │ │
│  │  /  \/  \  /  \______/    \____              │ │
│  │ threshold ─ ─ ─ ─ ─ ─ ─ ─ ─ ─              │ │
│  └──────────────────────────────────────────────┘ │
│                                                    │
│  Codec Assignment Map                             │
│  ┌──────────────────────────────────────────────┐ │
│  │ [LZ77──────][BWT────][LZ77──────][ANS──][LZ77]│ │
│  └──────────────────────────────────────────────┘ │
│                                                    │
│  λ: [──●──────] 0.5    δ: [────●──] 1.2          │
│  Adjust and watch the quorum signal change live   │
│                                                    │
│  Original: 2.4MB  →  Compressed: 0.6MB  (75%)    │
└────────────────────────────────────────────────────┘
```

#### WASM Technical Notes

- Rust core compiled to WASM via `wasm-pack`
- Only the compression analysis runs in WASM; heavy entropy calculation
  is offloaded to a Web Worker to keep the UI responsive
- File size limit: 50MB in browser (memory constraint)
- Visualization: D3.js heatmap + Chart.js for quorum signal graph
- Parameter sliders (λ, δ) trigger live recompression and re-visualization
- Downloadable output: user can download the `.qsae` file from the browser

---

## 8. Rust Tech Stack

### Core Engine Crates

| Crate | Version | Purpose |
|---|---|---|
| `rayon` | 1.10 | Data parallelism — parallel block compression |
| `lz4_flex` | 0.11 | LZ4 fast compression codec |
| `flate2` | 1.0 | DEFLATE/zlib fallback codec |
| `zstd` | 0.13 | Zstd codec + ANS/FSE internals |
| `suffix` | 1.2 | Suffix arrays for BWT construction |
| `xxhash-rust` | 0.8 | xxHash64 file integrity checksums |
| `byteorder` | 1.5 | Cross-platform binary serialization |
| `memmap2` | 0.9 | Memory-mapped file I/O (large files) |
| `thiserror` | 1.0 | Ergonomic error types |
| `anyhow` | 1.0 | Error propagation |

### CLI Crates

| Crate | Purpose |
|---|---|
| `clap` 4.5 | Argument parsing with subcommands |
| `indicatif` 0.17 | Progress bars + spinners |
| `console` 0.15 | Terminal colors and styling |
| `log` + `env_logger` | Structured logging (`QSAE_LOG=debug`) |
| `human-bytes` | Human-readable file size formatting |

### Desktop GUI Crates (Tauri v2)

| Crate / Package | Purpose |
|---|---|
| `tauri` 2.x | Core desktop app framework |
| `tauri-build` | Build system integration |
| `serde` + `serde_json` | Rust ↔ frontend IPC serialization |
| React + TypeScript | Frontend UI |
| Vite | Frontend build tool |
| Tailwind CSS | Styling |
| Recharts | Codec usage charts |
| `@tauri-apps/api` | Frontend IPC calls to Rust backend |

### SDK / Bindings Crates

| Crate | Purpose |
|---|---|
| `pyo3` 0.21 | Python bindings |
| `cbindgen` | C header generation |
| `napi` + `napi-build` | Node.js native addon bindings |

### WASM Crates

| Crate | Purpose |
|---|---|
| `wasm-pack` | WASM build toolchain |
| `wasm-bindgen` | Rust ↔ JavaScript bridge |
| `web-sys` | Web APIs (File, ArrayBuffer, Worker) |
| `js-sys` | JavaScript standard types |
| `gloo` | High-level Web Worker abstractions |
| D3.js | Heatmap visualization |
| Chart.js | Quorum signal graph |

### Testing & Benchmarking

| Crate | Purpose |
|---|---|
| `criterion` 0.5 | Benchmarking framework |
| `proptest` 1.4 | Property-based fuzz testing |
| `tempfile` | Temporary files in tests |
| `assert_cmd` | CLI integration testing |

### Build Profiles

```toml
[profile.release]
opt-level     = 3
lto           = "fat"
codegen-units = 1
panic         = "abort"
strip         = true

[profile.bench]
opt-level = 3
debug     = true
```

---

## 9. File Format Specification (.qsae)

```
┌────────────────────────────────────────────────────┐
│  HEADER (fixed, 32 bytes)                          │
│  ├─ magic:          [51 53 41 45] "QSAE"  4 bytes  │
│  ├─ version:        u8                    1 byte   │
│  ├─ flags:          u8                    1 byte   │
│  ├─ block_count:    u32                   4 bytes  │
│  ├─ original_size:  u64                   8 bytes  │
│  ├─ map_offset:     u64                   8 bytes  │
│  └─ reserved:       [00 × 6]              6 bytes  │
├────────────────────────────────────────────────────┤
│  BLOCK TABLE (9 bytes × block_count)               │
│  Per block:                                        │
│  ├─ codec_id:       u8                    1 byte   │
│  ├─ original_len:   u32                   4 bytes  │
│  └─ compressed_len: u32                   4 bytes  │
├────────────────────────────────────────────────────┤
│  SWITCHING MAP (arithmetic-coded, variable length) │
│  Preceded by u32 byte length                       │
├────────────────────────────────────────────────────┤
│  COMPRESSED BLOCK PAYLOADS (variable)              │
│  Blocks are independently decompressable           │
├────────────────────────────────────────────────────┤
│  FOOTER (16 bytes)                                 │
│  ├─ xxhash64:   u64 (hash of original)   8 bytes  │
│  ├─ magic_end:  [45 41 53 51] "EASQ"     4 bytes  │
│  └─ padding:    [00 × 4]                 4 bytes  │
└────────────────────────────────────────────────────┘
```

**Codec IDs:**

| ID | Codec | Entropy Zone |
|---|---|---|
| 0x00 | SKIP | H ≥ 7.5 |
| 0x01 | RLE | H < 1.0 |
| 0x02 | LZ4 | 1.0 – 3.5 |
| 0x03 | LZ77 | 1.0 – 5.0 |
| 0x04 | HUFFMAN | 4.5 – 6.0 |
| 0x05 | ANS | 6.0 – 7.5 |
| 0x06 | BWT | 3.5 – 6.5 |
| 0x07 | DELTA | Numeric zones |
| 0x08 | DEFLATE | Fallback |
| 0xFF | RESERVED | — |

---

## 10. Full Module Breakdown

```
qsae/
├── Cargo.toml                        # Workspace manifest
├── rust-toolchain.toml
│
├── crates/
│   │
│   ├── qsae-core/                    # Core compression library
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── compressor.rs
│   │   │   ├── decompressor.rs
│   │   │   ├── core/
│   │   │   │   ├── entropy.rs        # Shannon H + byte-type analysis
│   │   │   │   ├── quorum.rs         # Signal accumulation + threshold
│   │   │   │   ├── partitioner.rs    # Variable block splitting
│   │   │   │   └── dispatcher.rs     # Codec routing
│   │   │   ├── codecs/
│   │   │   │   ├── mod.rs            # Codec trait
│   │   │   │   ├── rle.rs
│   │   │   │   ├── lz4.rs
│   │   │   │   ├── lz77.rs
│   │   │   │   ├── huffman.rs
│   │   │   │   ├── ans.rs
│   │   │   │   ├── bwt.rs
│   │   │   │   ├── delta.rs
│   │   │   │   ├── deflate.rs
│   │   │   │   └── skip.rs
│   │   │   ├── format/
│   │   │   │   ├── header.rs
│   │   │   │   ├── block_table.rs
│   │   │   │   ├── switch_map.rs
│   │   │   │   └── footer.rs
│   │   │   ├── parallel/
│   │   │   │   └── engine.rs         # Rayon pipeline
│   │   │   └── error.rs
│   │   └── Cargo.toml
│   │
│   ├── qsae-cli/                     # CLI binary
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── commands/
│   │   │   │   ├── compress.rs
│   │   │   │   ├── decompress.rs
│   │   │   │   ├── inspect.rs
│   │   │   │   └── bench.rs
│   │   │   └── progress.rs
│   │   └── Cargo.toml
│   │
│   ├── qsae-wasm/                    # WebAssembly build
│   │   ├── src/
│   │   │   ├── lib.rs                # wasm-bindgen exports
│   │   │   ├── analysis.rs           # Entropy analysis for visualization
│   │   │   └── worker.rs             # Web Worker entry point
│   │   └── Cargo.toml
│   │
│   ├── qsae-python/                  # Python bindings (PyO3)
│   │   ├── src/lib.rs
│   │   └── Cargo.toml
│   │
│   └── qsae-node/                    # Node.js bindings (napi-rs)
│       ├── src/lib.rs
│       └── Cargo.toml
│
├── apps/
│   │
│   ├── desktop/                      # Tauri v2 Desktop App
│   │   ├── src-tauri/
│   │   │   ├── src/
│   │   │   │   ├── main.rs
│   │   │   │   ├── commands/
│   │   │   │   │   ├── compress.rs   # Tauri IPC command
│   │   │   │   │   ├── decompress.rs
│   │   │   │   │   └── inspect.rs
│   │   │   │   └── state.rs
│   │   │   └── tauri.conf.json
│   │   └── src/                      # React frontend
│   │       ├── App.tsx
│   │       ├── pages/
│   │       │   ├── Home.tsx          # Drop zone
│   │       │   ├── Progress.tsx      # Live entropy map
│   │       │   └── Results.tsx       # Ratio comparison
│   │       └── components/
│   │           ├── EntropyHeatmap.tsx
│   │           ├── QuorumChart.tsx
│   │           └── CodecBreakdown.tsx
│   │
│   └── web-demo/                     # WASM browser demo
│       ├── src/
│       │   ├── App.tsx
│       │   ├── pages/
│       │   │   ├── Upload.tsx
│       │   │   └── Visualizer.tsx
│       │   └── components/
│       │       ├── EntropyHeatmap.tsx # D3.js heatmap
│       │       ├── QuorumSignal.tsx   # Chart.js line graph
│       │       ├── CodecMap.tsx       # Colored codec assignment
│       │       └── ParamSliders.tsx   # λ and δ controls
│       └── package.json
│
├── benches/
│   ├── compression_ratio.rs
│   ├── throughput.rs
│   └── quorum_tuning.rs
│
└── tests/
    ├── roundtrip.rs
    ├── codec_unit.rs
    ├── quorum_unit.rs
    ├── format_compat.rs
    └── fixtures/
```

---

## 11. What the App Compresses

| Data Type | Dominant Codec(s) | vs gzip | vs Zstd |
|---|---|---|---|
| Source code (.rs, .js, .py) | LZ77 + BWT | +20–28% | +10–15% |
| JSON / YAML / TOML | LZ77 + Huffman | +22–35% | +12–20% |
| Log files (structured) | LZ4 + Huffman | +15–25% | +8–12% |
| CSV / TSV (numeric) | Delta + Huffman | +30–45% | +20–30% |
| HTML + embedded JS/CSS | All codecs | +35–50% | +25–35% |
| Mixed archives | All codecs | +30–48% | +20–32% |
| Natural language text | BWT + Huffman | +12–20% | +5–10% |
| Binary executables | LZ77 + ANS | +8–15% | +3–8% |
| Scientific datasets | Delta + ANS | +35–55% | +22–38% |
| JPEG/PNG/ZIP inside file | Skip | 0% waste | 0% waste |
| Encrypted regions | Skip | 0% waste | 0% waste |
| Null-padded binaries | RLE | +60–90% | +40–70% |

---

## 12. Benchmarking Strategy

### Standard Corpora

| Dataset | Description | Size |
|---|---|---|
| Calgary Corpus | Classic compression benchmark | 3.1 MB |
| Silesia Corpus | Modern mixed data types | 211 MB |
| enwik8 | Wikipedia XML dump | 100 MB |
| Linux kernel source | Codebase archive | ~800 MB |
| Custom: polyglot bundle | HTML+JS+CSS+JSON mixed | 50 MB |
| Custom: science data | Float arrays + metadata | 100 MB |

### Metrics

- Compression ratio — original_size / compressed_size
- Compression throughput — MB/s (single core + all cores)
- Decompression throughput — MB/s
- Memory usage — peak RSS
- Switching overhead — (header + map) as % of total output

### Comparison Targets

```
gzip -9  |  zstd -19  |  brotli -11  |  lzma (xz -9)  |  bzip2 -9
QSAE (δ=1.2, λ=0.5)   — balanced
QSAE (δ=0.5, λ=0.3)   — aggressive switching
QSAE (δ=2.0, λ=0.8)   — conservative (large blocks)
```

---

## 13. Implementation Roadmap

### Phase 1 — Core Engine (Weeks 1–3)
- [ ] Cargo workspace setup
- [ ] Shannon entropy calculator + tests
- [ ] RLE, LZ4, Huffman codecs + roundtrip tests
- [ ] Basic .qsae file format (single codec)
- [ ] CLI: compress + decompress commands

### Phase 2 — Quorum Engine (Weeks 4–6)
- [ ] Block partitioner (fixed-size first, entropy-driven later)
- [ ] Quorum signal accumulation + decay function
- [ ] Threshold detection + switching map generation
- [ ] Arithmetic coding for switching map
- [ ] Integration test: heterogeneous file compress + decompress

### Phase 3 — Full Codec Pool (Weeks 7–10)
- [ ] ANS (rANS), BWT+MTF, Delta, DEFLATE, Skip codecs
- [ ] LZ77 extended window
- [ ] Full dispatcher with codec trait
- [ ] Benchmark suite vs gzip/zstd/brotli

### Phase 4 — Desktop GUI (Weeks 11–14)
- [ ] Tauri v2 project scaffold
- [ ] React frontend: drop zone, progress, results screens
- [ ] Tauri IPC commands: compress, decompress, inspect
- [ ] Live entropy heatmap component
- [ ] Codec breakdown chart
- [ ] Dark mode, file history, batch compression

### Phase 5 — WASM Demo (Weeks 15–17)
- [ ] wasm-pack build of qsae-core
- [ ] Web Worker for off-thread compression
- [ ] React + D3.js entropy heatmap visualization
- [ ] Quorum signal live graph (Chart.js)
- [ ] λ/δ parameter sliders with live recompression
- [ ] Deploy to Vercel / GitHub Pages

### Phase 6 — SDK + Polish (Weeks 18–20)
- [ ] PyO3 Python bindings + PyPI package
- [ ] napi-rs Node.js bindings + npm package
- [ ] C bindings (cbindgen)
- [ ] proptest fuzz: compress→decompress identity
- [ ] Full documentation (rustdoc + README)
- [ ] Rayon parallelism + memmap2 large file support
- [ ] CLI: inspect + bench commands

---

## 14. Future Extensions

### QSAE v2 — Long-Range Entanglement
Extend quorum window to detect non-local correlations. Build a global
correlation graph. Add non-local back-references to the LZ77 codec.
Target: codebases and genomes with repeated architectural patterns
separated by megabytes.

### QSAE v3 — Learned Quorum Thresholds
Replace fixed λ/δ with a small trained model (decision tree or tiny
neural net) that predicts optimal codec assignment from richer block
features. Train on a corpus, embed in the binary.

### QSAE Mobile (Tauri v2)
Tauri v2 supports Android and iOS. The Desktop GUI can be extended to
mobile with minimal additional work — same Rust core, adapted React UI.

### QSAE Stream
Real-time streaming compression for network protocols. Quorum window
operates on a sliding buffer with bounded lookahead. Target: HTTP/3
replacement for Brotli on mixed-content responses.

### QSAE Semantic (SIC integration)
For text-only corpora, add a semantic embedding layer above QSAE.
Semantically near-duplicate blocks share a compressed representation.
Near-infinite ratio for redundant enterprise text corpora.

---

## Summary

```
App Type:   CLI + Desktop GUI (Tauri v2) + Library SDK + Web/WASM Demo

Users:      Developers (CLI, SDK) + General users (GUI, WASM demo)

Platforms:  Linux, macOS, Windows (CLI + GUI)
            Browser — any modern browser (WASM demo)
            Python, Node.js, C ecosystems (SDK)

Core:       Rust (qsae-core) — one library powering all four surfaces

Algorithm:  Quorum Sensing Signal
          + Variable Block Partitioning
          + 8-Codec Pool (RLE, LZ4, LZ77, Huffman, ANS, BWT, Delta, Skip)
          + Arithmetic-coded Switching Map
          + Parallel Block Processing (Rayon)
          + .qsae Binary Format

Target:     +20–50% ratio over Zstd on heterogeneous/polyglot files
            Visual proof-of-concept via interactive WASM entropy map
```

---

*QSAE Design Document v2.0 — Mohammed Al-Ashqar*
*Compression Research / Systems Engineering*
