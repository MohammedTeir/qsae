# QSAE Node.js Bindings

Node.js bindings for the QSAE compression engine using napi-rs.

## Installation

```bash
npm install qsae-node
```

## Quick Start

```javascript
const qsae = require('qsae-node');

async function main() {
  // Compress data
  const data = Buffer.from('Hello, world! '.repeat(1000));
  const result = await qsae.compress(data);

  console.log(`Original: ${result.original_size} bytes`);
  console.log(`Compressed: ${result.compressed_size} bytes`);
  console.log(`Ratio: ${result.ratio.toFixed(2)}:1`);
  console.log(`Time: ${result.duration_ms}ms`);

  // Decompress
  const original = await qsae.decompress(result.data);
  console.log(`Decompressed: ${original.length} bytes`);

  // Analyze without compressing
  const analysis = await qsae.analyze(data);
  console.log(`Blocks: ${analysis.block_count}`);
  console.log(`Switch points: ${analysis.switch_points.length}`);

  // Compress file
  const fileResult = await qsae.compressFile('input.txt', 'output.qsae', {
    lambda: 0.3,
    delta: 0.8,
    blockSize: 32768,
  });

  // Decompress file
  await qsae.decompressFile('output.qsae', 'restored.txt');

  // Inspect .qsae file
  const fs = require('fs').promises;
  const qsaeData = await fs.readFile('output.qsae');
  const info = await qsae.inspect(qsaeData);
  console.log(`Blocks: ${info.block_count}, Ratio: ${info.ratio.toFixed(2)}`);
  console.log('Codec breakdown:', info.codec_breakdown);
}

main().catch(console.error);
```

## API Reference

### `compress(data, options?)`
Compress a Buffer and return a `CompressionResult`.

### `decompress(data)`
Decompress a Buffer and return the original data.

### `analyze(data, options?)`
Analyze data without compressing. Returns `AnalysisResult`.

### `inspect(data)`
Inspect a .qsae file without decompressing. Returns `FileInfo`.

### `compressFile(inputPath, outputPath, options?)`
Compress a file to a .qsae file.

### `decompressFile(inputPath, outputPath)`
Decompress a .qsae file.

### Options
- `lambda` — Decay constant (default: 0.5)
- `delta` — Switch threshold (default: 1.2)
- `blockSize` — Block size hint (default: 65536)
- `useQuorum` — Enable quorum sensing (default: true)
