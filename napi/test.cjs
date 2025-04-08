//  node napi/test.cjs

async function run() {
  const { Optimizer } = require('../../qwik/dist/optimizer/index.cjs');
  const optimizer = new Optimizer();

  const result = await optimizer.transformFs({
    rootDir: '../../framework-benchmarks/frameworks/qwik/src',
  });

  console.log(result);
}

run();
