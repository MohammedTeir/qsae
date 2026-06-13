#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "libqsae.h"

int main(int argc, char **argv) {
    if (argc < 3) {
        fprintf(stderr, "Usage: %s <input> <output>\n", argv[0]);
        return 1;
    }

    const char *input_path = argv[1];
    const char *output_path = argv[2];

    /* Get version */
    char *version = qsae_version();
    printf("QSAE Version: %s\n", version);
    qsae_version_free(version);

    /* Create configuration */
    QsaeConfig *config = qsae_config_new(0.5, 1.2, 65536);
    if (!config) {
        fprintf(stderr, "Failed to create config\n");
        return 1;
    }

    /* Create compressor */
    QsaeCompressor *compressor = qsae_compressor_new(config);
    if (!compressor) {
        fprintf(stderr, "Failed to create compressor\n");
        qsae_config_free(config);
        return 1;
    }

    /* Compress file */
    QsaeStats stats;
    printf("Compressing %s -> %s...\n", input_path, output_path);

    int result = qsae_compress_file(compressor, input_path, output_path, &stats);
    if (result != QSAE_OK) {
        fprintf(stderr, "Compression failed\n");
        qsae_compressor_free(compressor);
        qsae_config_free(config);
        return 1;
    }

    printf("Compression complete!\n");
    printf("  Original: %zu bytes\n", stats.original_size);
    printf("  Compressed: %zu bytes\n", stats.compressed_size);
    printf("  Ratio: %.2f:1\n", stats.ratio);
    printf("  Blocks: %zu\n", stats.block_count);
    printf("  Duration: %llu ms\n", (unsigned long long)stats.duration_ms);

    /* Create decompressor */
    QsaeDecompressor *decompressor = qsae_decompressor_new();
    if (!decompressor) {
        fprintf(stderr, "Failed to create decompressor\n");
        qsae_compressor_free(compressor);
        qsae_config_free(config);
        return 1;
    }

    /* Decompress */
    char restored_path[256];
    snprintf(restored_path, sizeof(restored_path), "%s.restored", input_path);

    printf("\nDecompressing %s -> %s...\n", output_path, restored_path);
    result = qsae_decompress_file(decompressor, output_path, restored_path);
    if (result != QSAE_OK) {
        fprintf(stderr, "Decompression failed\n");
        qsae_decompressor_free(decompressor);
        qsae_compressor_free(compressor);
        qsae_config_free(config);
        return 1;
    }

    printf("Decompression complete!\n");

    /* Cleanup */
    qsae_decompressor_free(decompressor);
    qsae_compressor_free(compressor);
    qsae_config_free(config);

    return 0;
}
