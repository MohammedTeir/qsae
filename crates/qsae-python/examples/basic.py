import qsae

# Basic compression
data = b"Hello, world! This is QSAE Python test. " * 1000

# Compress with default parameters
compressed = qsae.compress(data)
print(f"Original: {len(data)} bytes")
print(f"Compressed: {len(compressed)} bytes")
print(f"Ratio: {len(data) / len(compressed):.2f}:1")

# Decompress
original = qsae.decompress(compressed)
assert original == data
print("Roundtrip successful!")

# Analyze
analysis = qsae.analyze(data)
print(f"Blocks: {len(analysis.entropy_profile)}")
print(f"Switch points: {len(analysis.switch_points)}")
print(f"Avg entropy: {sum(analysis.entropy_profile) / len(analysis.entropy_profile):.2f}")
