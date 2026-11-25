import { createRequire } from "module";
const require = createRequire(import.meta.url);
const { ignore } = require("./bc");


const matcher = new ignore.IgnoreMatcher([
    'node_modules',
    '*.log',
    '!important.log',
    'dist/*',
    '!dist/keep.js',
]);

console.log(matcher.isIgnored('node_modules/pkg/index.js')); // true
console.log(matcher.isIgnored('logs\\important.log'));       // false
console.log(matcher.isIgnored('dist/keep.js'));              // false

const allowed = matcher.filterIgnored([
    'src/index.ts',
    'node_modules/react/index.js',
    'error.log',
    'important.log',
]);
// ['src/index.ts', 'important.log']
