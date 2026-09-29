// Health & Fitness (36). Designs, reviews and corrections: evidence/design-review.md.
import {icons as a} from './batches/health-a.mjs';
import {icons as b} from './batches/health-b.mjs';
import {icons as c} from './batches/health-c.mjs';
import {icons as d} from './batches/health-d.mjs';
const all=new Map([...a,...b,...c,...d].map(i=>[i.name,i]));
const ORDER=['heart-rate','step-counter','dumbbell','running','water-intake','sleep-tracking','calories','stethoscope','pill','syringe','first-aid-kit','bandage','weight-scale','blood-pressure','yoga','meditation','nutrition','jump-rope','hospital','dna'];
if(ORDER.length!==all.size||ORDER.some(n=>!all.has(n)))throw Error('Health order and batches disagree');
const fixed={"water-intake":["glass"],"stethoscope":["binaural"],"syringe":["barrel","needle","flange"],"blood-pressure":["dial-frame","hub"],"yoga":[],"hospital":["building","doorway"]};
const rigidParts={"dumbbell":["bar","plate-li"],"first-aid-kit":["case","handle"],"pill":["half-a","half-b"],"syringe":["rod","thumb"],"stethoscope":["chest-frame"],"jump-rope":["handle-l-frame","handle-r-frame"],"sleep-tracking":["moon-frame"],"nutrition":["apple-frame"]};
export const icons=ORDER.map(n=>all.get(n)).map((icon,index)=>({...icon,number:546+index,constraints:[...(fixed[icon.name]||[]).map(p=>({type:'fixed',part:icon.name+'-'+p})),...(rigidParts[icon.name]||[]).map(p=>({type:'rigid',part:icon.name+'-'+p}))]}));
