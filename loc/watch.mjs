import { createRequire } from "module";
import fs from 'node:fs/promises';
import path from 'node:path';
const require = createRequire(import.meta.url);
const { watch } = require("./bc");

/** @type {Record<string, { content: string; updatedAt: number }>} */
const siteCache = {};

const watcher = new watch.FsWatcher(async (err, event) => {
    if (err) return console.error(err);

    console.log(`Event : ${event.kind}, Path : ${event.path}`)
},
    {
        debounceMs: 100,
        ignore: ['**/node_modules/**', '**/.git/**', '**/*.tmp'],
    });


watcher.watch(path.resolve(process.cwd(), 'content'));

// Clean up on exit so Node's event loop isn't kept alive
process.on('SIGINT', () => {
    watcher.close();
    process.exit(0);
});