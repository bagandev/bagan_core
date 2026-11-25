import { createRequire } from "module";
import fs from "fs";
const require = createRequire(import.meta.url);
const { collect } = require("./loc/bc");

const exts = collect.defaultFiles();

const files = collect.getFilePaths(".", exts, ["loc"]);
const fileObj = collect.initPageFiles(files, exts, "dist");

fs.writeFileSync("bb.json", JSON.stringify(fileObj, null, 2));