#!/usr/bin/env npx --offline zx
const allFiles = await $`find ./tests -type f`.quiet();
const allFilesArr = allFiles.stdout.split('\n').filter(s => s);
let failures = 0;
for (let file of allFilesArr) {
  let start = Date.now();
  // console.warn('start', file)
  // if (file.includes('netdata')) continue;
  try {
    await $`cmakefmt ${file}`.quiet();
  } catch (err) {
    // console.log(err.stderr);
    console.log(file);
    failures += 1;
  }
  const duration = Date.now() - start;
  if (duration > 100) {
    console.warn('slow', file, duration + 'ms');
  }
}

console.log('total', allFilesArr.length);
console.log('failures', failures);
