#!/bin/bash
# Generate test fixtures for QSAE

mkdir -p tests/fixtures

# 1. Uniform data (RLE-optimal)
dd if=/dev/zero bs=1024 count=100 of=tests/fixtures/zeros.bin 2>/dev/null

# 2. Text data (LZ4/Huffman optimal)
python3 -c "print('Hello world! ' * 10000)" > tests/fixtures/text.txt

# 3. Mixed entropy data
python3 -c "
import sys
# Low entropy section
sys.stdout.buffer.write(b'\x00' * 50000)
# High entropy section  
import random
random.seed(42)
for _ in range(50000):
    sys.stdout.buffer.write(bytes([random.randint(0, 255)]))
" > tests/fixtures/mixed.bin

# 4. JSON-like structured data
python3 -c "
import json
import sys
data = [{'id': i, 'name': f'item_{i}', 'value': i * 3.14} for i in range(1000)]
sys.stdout.write(json.dumps(data, indent=2))
" > tests/fixtures/data.json

echo "Test fixtures generated in tests/fixtures/"
ls -la tests/fixtures/
