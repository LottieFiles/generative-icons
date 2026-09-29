import {spawnSync} from "node:child_process";
import {fileURLToPath} from "node:url";
const r=spawnSync(process.execPath,[fileURLToPath(new URL("../library/build-set.mjs",import.meta.url)),fileURLToPath(new URL(".",import.meta.url))],{stdio:"inherit"});process.exit(r.status??1);
