const qsae = require('qsae-node');
const fs = require('fs');
const { Transform } = require('stream');

// Create a QSAE compression transform stream
function createCompressStream(options = {}) {
  return new Transform({
    transform(chunk, encoding, callback) {
      qsae.compress(chunk, options)
        .then(result => callback(null, result.data))
        .catch(err => callback(err));
    },
  });
}

// Compress a file using streams
async function compressFileStream(inputPath, outputPath) {
  return new Promise((resolve, reject) => {
    fs.createReadStream(inputPath)
      .pipe(createCompressStream())
      .pipe(fs.createWriteStream(outputPath))
      .on('finish', resolve)
      .on('error', reject);
  });
}

// Example usage
compressFileStream('input.txt', 'output.qsae')
  .then(() => console.log('Stream compression complete'))
  .catch(console.error);
