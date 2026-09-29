// Vector editing & design: 20 generic operations, 10 true two-state actions.
import {icons as boolean} from './batches/boolean.mjs';
import {icons as paths} from './batches/paths.mjs';
import {icons as layout} from './batches/layout.mjs';
import {icons as design} from './batches/design.mjs';
export const icons=[...boolean,...paths,...layout,...design].map((icon,index)=>({...icon,number:1066+index}));
if(icons.length!==20 || icons.filter(i=>i.stateful).length!==10)throw Error('Set 62 requires exactly 20 icons and 10 authored reverses');
