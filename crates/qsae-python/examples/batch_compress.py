import qsae
import os
import glob

def batch_compress(directory, output_dir, lambda_param=0.5, delta=1.2):
    """Compress all files in a directory."""
    os.makedirs(output_dir, exist_ok=True)

    compressor = qsae.Compressor(lambda=lambda_param, delta=delta)

    for filepath in glob.glob(os.path.join(directory, "*")):
        if os.path.isfile(filepath):
            filename = os.path.basename(filepath)
            output_path = os.path.join(output_dir, filename + ".qsae")

            try:
                stats = compressor.compress_file(filepath, output_path)
                print(f"{filename}: {stats.original_size} → {stats.compressed_size} "
                      f"({stats.ratio:.2f}:1, {stats.duration_ms}ms)")
            except Exception as e:
                print(f"Failed to compress {filename}: {e}")

if __name__ == "__main__":
    batch_compress("./input", "./output", lambda_param=0.3, delta=0.8)
