const qsae = require('qsae-node');

async function main() {
  // Basic compression
  const data = Buffer.from('Hello, world! '.repeat(1000));

  console.log('Compressing...');
  const result = await qsae.compress(data);

  console.log(`Original: ${result.original_size} bytes`);
  console.log(`Compressed: ${result.compressed_size} bytes`);
  console.log(`Ratio: ${result.ratio.toFixed(2)}:1`);
  console.log(`Duration: ${result.duration_ms}ms`);

  // Decompress
  const original = await qsae.decompress(result.data);
  console.log(`Decompressed: ${original.length} bytes`);

  // Verify roundtrip
  if (Buffer.compare(data, original) === 0) {
    console.log('✓ Roundtrip successful!');
  }

  // Analyze
  const analysis = await qsae.analyze(data);
  console.log(`
Analysis:`);
  console.log(`  Blocks: ${analysis.block_count}`);
  console.log(`  Switch points: ${analysis.switch_points.length}`);
  console.log(`  Avg entropy: ${(analysis.entropy_profile.reduce((a, b) => a + b, 0) / analysis.entropy_profile.length).toFixed(2)}`);
}

main().catch(console.error);
