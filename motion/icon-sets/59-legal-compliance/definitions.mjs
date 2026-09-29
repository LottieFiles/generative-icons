// Legal & compliance (59). Designs, reviews and corrections: evidence/design-review.md.
import {icons as a} from './batches/legal-a.mjs';
import {icons as b} from './batches/legal-b.mjs';
import {icons as c} from './batches/legal-c.mjs';
import {icons as d} from './batches/legal-d.mjs';
const all=new Map([...a,...b,...c,...d].map(i=>[i.name,i]));
const ORDER=['gavel','justice-scale','contract','rubber-stamp','official-seal','courthouse','gdpr-shield','audit','policy','terms-of-service','compliance-check','redact','consent','cookie-banner','data-retention','disclaimer','copyright','whistleblower','evidence-bag','legal-hold'];
if(ORDER.length!==all.size||ORDER.some(n=>!all.has(n)))throw Error('Legal & compliance order and batches disagree');
const fixed={"justice-scale":["post","base"],"courthouse":["base"],"audit":["sheet-frame","entry-0","entry-1","entry-2","entry-3"],"policy":["roll-top-frame","clause-0"],"terms-of-service":["page-frame","track","clause-0","clause-4"],"consent":["statement-0","statement-1"],"cookie-banner":["screen-frame"],"data-retention":["top","side-l","side-r","band","bottom"],"disclaimer":["print-0","print-1","print-2"],"legal-hold":["doc-frame","line-0","line-1","line-2"]};
const rigidParts={"gavel":["head-frame","handle","block"],"justice-scale":["beam","pan-l-frame","pan-r-frame"],"rubber-stamp":["knob-frame","base-frame","neck"],"courthouse":["pediment-frame"],"audit":["lens-frame","grip"],"terms-of-service":["thumb"],"whistleblower":["whistle-frame"],"evidence-bag":["bag-frame","band"],"legal-hold":["body-frame"],"data-retention":["clock-frame","hand"]};
export const icons=ORDER.map(n=>all.get(n)).map((icon,index)=>({...icon,number:1006+index,constraints:[...(fixed[icon.name]||[]).map(p=>({type:'fixed',part:icon.name+'-'+p})),...(rigidParts[icon.name]||[]).map(p=>({type:'rigid',part:icon.name+'-'+p}))]}));
