import {icons as b0} from './batches/people.mjs';
import {icons as b1} from './batches/coordination.mjs';
import {icons as b2} from './batches/planning.mjs';
import {icons as b3} from './batches/editing.mjs';
import {icons as b4} from './batches/discussion.mjs';
import {icons as b5} from './batches/review.mjs';
import {icons as b6} from './batches/access.mjs';
import {icons as b7} from './batches/rituals.mjs';
export const icons=[...b0,...b1,...b2,...b3,...b4,...b5,...b6,...b7].map((icon,i)=>({...icon,number:342+i}));
