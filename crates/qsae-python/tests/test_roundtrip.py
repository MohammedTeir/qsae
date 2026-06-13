import qsae
import pytest

def test_compress_decompress_roundtrip():
    data = b"Hello, world! " * 1000
    compressed = qsae.compress(data)
    original = qsae.decompress(compressed)
    assert original == data

def test_compress_ratio():
    data = b"A" * 10000
    compressed = qsae.compress(data)
    assert len(compressed) < len(data)
    assert len(compressed) < 100  # RLE should compress very well

def test_analyze_structure():
    data = b"Test data " * 500
    analysis = qsae.analyze(data)
    assert len(analysis.entropy_profile) > 0
    assert len(analysis.quorum_curve) > 0
    assert len(analysis.quorum_curve) == len(analysis.entropy_profile)

def test_file_compression(tmp_path):
    input_file = tmp_path / "test.txt"
    output_file = tmp_path / "test.qsae"
    restored_file = tmp_path / "restored.txt"

    input_file.write_bytes(b"File test data " * 1000)

    compressor = qsae.Compressor()
    stats = compressor.compress_file(str(input_file), str(output_file))

    assert stats.ratio > 1.0
    assert output_file.exists()

    decompressor = qsae.Decompressor()
    decompressor.decompress_file(str(output_file), str(restored_file))

    assert restored_file.read_bytes() == input_file.read_bytes()

def test_inspect():
    data = b"Inspect test " * 500
    compressed = qsae.compress(data)

    decompressor = qsae.Decompressor()
    info = decompressor.inspect(compressed)

    assert info.block_count > 0
    assert info.ratio > 0
    assert len(info.codec_breakdown) > 0
