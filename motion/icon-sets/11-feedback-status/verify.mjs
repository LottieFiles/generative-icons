import {spawnSync} from "node:child_process";
import {fileURLToPath} from "node:url";
for(const args of [
 [fileURLToPath(new URL("../library/verify-directed.mjs",import.meta.url)),fileURLToPath(new URL(".",import.meta.url))],
 [fileURLToPath(new URL("verify-attachments.mjs",import.meta.url))]
]){const r=spawnSync(process.execPath,args,{stdio:"inherit"});if(r.status!==0)process.exit(r.status??1);}
