import {icons as a} from './calendar-tools-02.mjs';
import {icons as b} from './time-tools-02.mjs';
import {icons as c} from './alarm-clock-04.mjs';
import {icons as d} from './world-clock-04.mjs';
const fixes=new Map([...c,...d].map(i=>[i.name,i]));
export const icons=[...a,...b].map(i=>fixes.get(i.name)||i);
