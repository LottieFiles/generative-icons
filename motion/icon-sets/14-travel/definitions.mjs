import {icons as a} from './batches/01.mjs';
import {icons as b} from './batches/02.mjs';
import {icons as c} from './batches/03.mjs';
import {icons as d} from './batches/04.mjs';
export const icons=[...a,...b,...c,...d];
const names=['airplane','passport','boarding-pass','suitcase','backpack','train','bus','car-rental','ferry','bicycle','hotel','bed','tent','beach-umbrella','map','route','globe','ticket','luggage-tag','signpost'];
if(JSON.stringify(icons.map(i=>i.name))!==JSON.stringify(names))throw Error('Travel scope differs from approved set');
