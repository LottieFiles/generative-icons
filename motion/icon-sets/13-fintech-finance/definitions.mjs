import {icons as a} from './batches/01.mjs';
import {icons as b} from './batches/02.mjs';
import {icons as c} from './batches/03.mjs';
import {icons as d} from './batches/04.mjs';
export const icons=[...a,...b,...c,...d];
const names=['bank','bank-account','bank-transfer','currency-exchange','savings','deposit','withdraw','payment-terminal','contactless-payment','recurring-payment','receipt','invoice','refund','bill-payment','budget','transaction-history','investment','portfolio','calculator','vault'];
if(JSON.stringify(icons.map(i=>i.name))!==JSON.stringify(names))throw Error('Fintech scope differs from approved set');
