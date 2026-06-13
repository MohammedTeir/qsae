# QSAE C Bindings

C bindings for the QSAE compression engine using cbindgen.

## Building

```bash
cd crates/qsae-c

# Build the library
cargo build --release

# Generate header (optional, header is provided)
cbindgen --config cbindgen.toml --crate qsae-c --output include/libqsae.h

# Build example
cd examples
make
```

## Usage

```c
#include <stdio.h>
#include "libqsae.h"

int main() {
    /* Create configuration */
    QsaeConfig *config = qsae_config_new(0.5, 1.2, 65536);

    /* Create compressor */
    QsaeCompressor *compressor = qsae_compressor_new(config);

    /* Compress file */
    QsaeStats stats;
    int result = qsae_compress_file(compressor, "input.txt", "output.qsae", &stats);

    if (result == QSAE_OK) {
        printf("Ratio: %.2f:1\n", stats.ratio);
    }

    /* Cleanup */
    qsae_compressor_free(compressor);
    qsae_config_free(config);

    return 0;
}
```

## API Reference

### Configuration
- `QsaeConfig *qsae_config_default(void)` — Create default config
- `QsaeConfig *qsae_config_new(double lambda, double delta, size_t block_size)` — Create custom config
- `void qsae_config_free(QsaeConfig *config)` — Free config

### Compressor
- `QsaeCompressor *qsae_compressor_new(QsaeConfig *config)` — Create compressor
- `QsaeResult qsae_compress(QsaeCompressor *compressor, const uint8_t *input, size_t input_len)` — Compress data
- `int qsae_compress_file(QsaeCompressor *compressor, const char *input_path, const char *output_path, QsaeStats *stats)` — Compress file

### Decompressor
- `QsaeDecompressor *qsae_decompressor_new(void)` — Create decompressor
- `QsaeResult qsae_decompress(QsaeDecompressor *decompressor, const uint8_t *input, size_t input_len)` — Decompress data
- `int qsae_decompress_file(QsaeDecompressor *decompressor, const char *input_path, const char *output_path)` — Decompress file

### Memory Management
- `void qsae_result_free(QsaeResult result)` — Free result data
- `char *qsae_version(void)` — Get version string
- `void qsae_version_free(char *version)` — Free version string

### Error Codes
- `QSAE_OK` (0) — Success
- `QSAE_ERROR` (-1) — Generic error
