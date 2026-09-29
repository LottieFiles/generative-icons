import {icons as a} from './batches/rewards-a.mjs';
import {icons as b} from './batches/rewards-b.mjs';
import {icons as c} from './batches/rewards-c.mjs';
import {icons as d} from './batches/rewards-d.mjs';
const all=[...a,...b,...c,...d];
if(all.length!==20||new Set(all.map(i=>i.name)).size!==20)throw Error('Expected exactly 20 distinct reward icons');
export const icons=all.map((icon,index)=>({...icon,number:906+index}));
