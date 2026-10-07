// Legacy export entry point. The legacy builders called it to run a local Engine
// exporter binary over stdio; that path is retired. Native export runs on LottieFiles'
// hosted engine through the generative-icons CLI, which reproduces these builders'
// outputs byte for byte.
// node src/export.mjs scene.engine.js output.json scene.scene.json
const set = process.argv[2]?.match(/icon-sets\/([^/]+)\//)?.[1] ?? 'SET-ID';
console.error(
  `Local native export is retired. Build the set with the hosted engine instead:\n` +
    `  generative-icons library build ${set}\n` +
    `Sign in once with generative-icons login. See docs/development.md.`,
);
process.exit(1);
