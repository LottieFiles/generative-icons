// Drawing & design tools (51). Designs, reviews and corrections: evidence/design-review.md.
import {icons as a} from './batches/tools-a.mjs';
import {icons as b} from './batches/tools-b.mjs';
import {icons as c} from './batches/tools-c.mjs';
import {icons as d} from './batches/tools-d.mjs';
const all=new Map([...a,...b,...c,...d].map(i=>[i.name,i]));
const ORDER=['pen-tool','brush','eraser','corner-radius','line-tool','rectangle-tool','ellipse-tool','polygon-tool','fill-bucket','eyedropper','group-objects','align-objects','magic-wand','lasso-select','transform-tool','rotate-tool','mirror-tool','snap-to-grid','ruler-guides','scissors'];
if(ORDER.length!==all.size||ORDER.some(n=>!all.has(n)))throw Error('Drawing & design tools order and batches disagree');
const fixed={"pen-tool":["path","start","anchor"],"eraser":[],"rectangle-tool":["origin"],"ellipse-tool":["anchor"],"eyedropper":[],"align-objects":["guide"],"lasso-select":["object"],"rotate-tool":["arc","head"],"mirror-tool":["axis-0","axis-1","axis-2","left-frame"],"snap-to-grid":["dot-7-7","dot-25-25"],"ruler-guides":["ruler-frame","tick-t13","tick-l13"],"scissors":["hub"]};
const rigidParts={"pen-tool":["nib","barrel"],"brush":["bristles-frame","ferrule","handle"],"eraser":["block","band"],"fill-bucket":["body-frame","handle"],"eyedropper":["cone","tube","collar"],"group-objects":["square","circle"],"align-objects":["block-0","block-1","block-2"],"magic-wand":["stick"],"rotate-tool":["square-frame"],"snap-to-grid":["object-frame"],"ruler-guides":["guide-v","guide-h"],"scissors":["blade-a-shaft","blade-a-ring","blade-b-shaft","blade-b-ring"]};
export const icons=ORDER.map(n=>all.get(n)).map((icon,index)=>({...icon,number:846+index,constraints:[...(fixed[icon.name]||[]).map(p=>({type:'fixed',part:icon.name+'-'+p})),...(rigidParts[icon.name]||[]).map(p=>({type:'rigid',part:icon.name+'-'+p}))]}));
