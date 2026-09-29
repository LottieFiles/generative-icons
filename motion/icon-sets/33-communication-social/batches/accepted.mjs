import {icons as a} from './mail-call-02.mjs';
import {icons as b} from './social-broadcast-01.mjs';
import {icons as e} from './emoji-02.mjs';
import {icons as m} from './mail-call-04.mjs';
import {icons as t} from './transfer-call-04.mjs';
const fixes=new Map([...m,...t].map(i=>[i.name,i]));
export const icons=[...a,...b.map(i=>e.find(x=>x.name===i.name)||i)].map(i=>fixes.get(i.name)||i);
