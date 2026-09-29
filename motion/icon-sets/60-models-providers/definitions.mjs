// Models & providers. Generic app integration concepts; no brands or logos.
// Geometry, pivots, motion and per-icon briefs are in the four batch modules.
import {icons as access} from './batches/access.mjs';
import {icons as models} from './batches/models.mjs';
import {icons as parameters} from './batches/parameters.mjs';
import {icons as operations} from './batches/operations.mjs';
const all=[...access,...models,...parameters,...operations];
if(all.length!==20 || new Set(all.map(i=>i.name)).size!==20)throw Error('Set 60 must contain exactly 20 unique icons');
export const icons=all.map((icon,i)=>({...icon,number:1026+i}));
