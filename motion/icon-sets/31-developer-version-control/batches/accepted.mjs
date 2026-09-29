import {icons as old} from './version-control-01.mjs';
import {icons as revised} from './version-control-02.mjs';
import {icons as debug} from './debug-events-02.mjs';
const replacements=new Map(revised.map(i=>[i.name,i]));
export const icons=[...old.map(i=>replacements.get(i.name)||i),...debug];
icons.find(i=>i.name==='repository').duotoneFill=true;
