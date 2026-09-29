// Build or verify the complete four-set AI scope. No partial catalog writes.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
const here=path.dirname(fileURLToPath(import.meta.url));
const scope=JSON.parse(fs.readFileSync(path.join(here,'design-reviews/ai-category/scope.json')));
const action=process.argv[2];
if(!['build','verify'].includes(action))throw Error('Use build or verify');
for(const set of scope.sets){
 const root=path.join(here,'..',set.id);
 const {icons}=await import(path.join(root,'definitions.mjs'));
 if(JSON.stringify(icons.map(i=>i.name))!==JSON.stringify(set.icons))throw Error('Incomplete or misordered AI scope: '+set.id);
 const r=spawnSync(process.execPath,[path.join(root,action+'.mjs')],{stdio:'inherit'});
 if(r.status!==0)process.exit(r.status??1);
}
