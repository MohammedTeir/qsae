import test from 'ava';
import { compress, decompress, analyze, inspect } from '../index.js';

test('compress and decompress roundtrip', async t => {
  const data = Buffer.from('Hello, world! '.repeat(1000));
  const result = await compress(data);
  const original = await decompress(result.data);
  t.deepEqual(original, data);
});

test('compression ratio', async t => {
  const data = Buffer.from('A'.repeat(10000));
  const result = await compress(data);
  t.true(result.compressed_size < result.original_size);
  t.true(result.ratio > 1.0);
});

test('analyze returns structure', async t => {
  const data = Buffer.from('Test data '.repeat(500));
  const analysis = await analyze(data);
  t.true(analysis.block_count > 0);
  t.true(analysis.entropy_profile.length > 0);
  t.is(analysis.entropy_profile.length, analysis.quorum_curve.length);
});

test('inspect qsae file', async t => {
  const data = Buffer.from('Inspect test '.repeat(500));
  const compressed = (await compress(data)).data;
  const info = await inspect(compressed);
  t.true(info.block_count > 0);
  t.true(info.ratio > 0);
  t.true(info.codec_breakdown.length > 0);
});
