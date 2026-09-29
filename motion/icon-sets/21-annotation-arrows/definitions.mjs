import {curves} from './batches/curves.mjs';
import {swoops} from './batches/swoops.mjs';
import {callouts} from './batches/callouts.mjs';
import {connectors} from './batches/connectors.mjs';
import {double} from './batches/double.mjs';
export const icons=[...curves,...swoops,...callouts,...connectors,...double].map((icon,i)=>({...icon,number:330+i}));
