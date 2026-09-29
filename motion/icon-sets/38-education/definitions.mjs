// Education (38). Designs, reviews and corrections: evidence/design-review.md.
import {icons as a} from './batches/edu-a.mjs';
import {icons as b} from './batches/edu-b.mjs';
import {icons as c} from './batches/edu-c.mjs';
import {icons as d} from './batches/edu-d.mjs';
const all=new Map([...a,...b,...c,...d].map(i=>[i.name,i]));
const ORDER=['graduation-cap','chalkboard','flashcards','quiz','certificate','notebook','pencil','ruler','lab-flask','microscope','abacus','school-bell','course-progress','lecture','library-shelf','language-learning','alphabet-blocks','homework','exam-grade','geometry-compass'];
if(ORDER.length!==all.size||ORDER.some(n=>!all.has(n)))throw Error('Education order and batches disagree');
const fixed={"flashcards":["deck"],"quiz":["card-frame","q-hook","ring-1","row-0","row-1"],"microscope":["base","arm","stage"],"abacus":["frame-frame","rod-0","rod-1","rod-2","bead-0-0","bead-1-0","bead-2-0"],"course-progress":["card-frame","track"],"lecture":["screen-frame","head-frame","shoulders"],"library-shelf":["shelf","book-0-frame"],"language-learning":[],"exam-grade":[],"geometry-compass":[]};
const rigidParts={"graduation-cap":["tassel"],"school-bell":["body-frame"],"microscope":["tube-frame"],"lecture":["pointer"],"geometry-compass":["needle-leg","pencil-leg"],"library-shelf":["book-1-frame"]};
export const icons=ORDER.map(n=>all.get(n)).map((icon,index)=>({...icon,number:586+index,constraints:[...(fixed[icon.name]||[]).map(p=>({type:'fixed',part:icon.name+'-'+p})),...(rigidParts[icon.name]||[]).map(p=>({type:'rigid',part:icon.name+'-'+p}))]}));
