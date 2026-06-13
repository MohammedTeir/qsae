#ifndef LIBQSAE_H
#define LIBQSAE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque handles */
typedef struct QsaeConfig QsaeConfig;
typedef struct QsaeCompressor QsaeCompressor;
typedef struct QsaeDecompressor QsaeDecompressor;

/* Result structure */
typedef struct {
    int success;
    uint8_t *data;
    size_t len;
    char *error;
} QsaeResult;

/* Statistics structure */
typedef struct {
    size_t original_size;
    size_t compressed_size;
    double ratio;
    size_t block_count;
    uint64_t duration_ms;
} QsaeStats;

/* Configuration */
QsaeConfig *qsae_config_default(void);
QsaeConfig *qsae_config_new(double lambda, double delta, size_t block_size);
void qsae_config_free(QsaeConfig *config);

/* Compressor */
QsaeCompressor *qsae_compressor_new(QsaeConfig *config);
void qsae_compressor_free(QsaeCompressor *compressor);
QsaeResult qsae_compress(QsaeCompressor *compressor, const uint8_t *input, size_t input_len);
int qsae_compress_file(QsaeCompressor *compressor, const char *input_path, const char *output_path, QsaeStats *stats);

/* Decompressor */
QsaeDecompressor *qsae_decompressor_new(void);
void qsae_decompressor_free(QsaeDecompressor *decompressor);
QsaeResult qsae_decompress(QsaeDecompressor *decompressor, const uint8_t *input, size_t input_len);
int qsae_decompress_file(QsaeDecompressor *decompressor, const char *input_path, const char *output_path);

/* Memory management */
void qsae_result_free(QsaeResult result);

/* Version */
char *qsae_version(void);
void qsae_version_free(char *version);

/* Error codes */
#define QSAE_OK 0
#define QSAE_ERROR -1
#define QSAE_INVALID_MAGIC -2
#define QSAE_UNSUPPORTED_VERSION -3
#define QSAE_CHECKSUM_MISMATCH -4

#ifdef __cplusplus
}
#endif

#endif /* LIBQSAE_H */
