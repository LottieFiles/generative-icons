// Food & Drink (39). Designs, reviews and corrections: evidence/design-review.md.
import {icons as a} from './batches/food-a.mjs';
import {icons as b} from './batches/food-b.mjs';
import {icons as c} from './batches/food-c.mjs';
import {icons as d} from './batches/food-d.mjs';
const all=new Map([...a,...b,...c,...d].map(i=>[i.name,i]));
const ORDER=['coffee-cup','tea-cup','wine-glass','beer-mug','cocktail','cooking-pot','frying-pan','kettle','toaster','blender','chef-hat','cutlery','pizza','burger','cake','ice-cream','noodle-bowl','bread','recipe-book','restaurant-menu'];
if(ORDER.length!==all.size||ORDER.some(n=>!all.has(n)))throw Error('Food & Drink order and batches disagree');
const fixed={"coffee-cup":["body-frame","handle"],"tea-cup":["saucer"],"toaster":[],"cake":["cake-frame","candle"],"ice-cream":["cone-frame"],"noodle-bowl":["bowl-frame"],"restaurant-menu":["page-frame"],"burger":["bottom-bun-frame"]};
const rigidParts={"cooking-pot":["lid-frame","knob"],"frying-pan":["pan","handle"],"kettle":["body-frame","spout"],"cutlery":["fork","blade-frame"],"burger":["top-bun-frame"],"recipe-book":["cover-frame","ribbon"],"noodle-bowl":["stick-0","stick-1"]};
export const icons=ORDER.map(n=>all.get(n)).map((icon,index)=>({...icon,number:606+index,constraints:[...(fixed[icon.name]||[]).map(p=>({type:'fixed',part:icon.name+'-'+p})),...(rigidParts[icon.name]||[]).map(p=>({type:'rigid',part:icon.name+'-'+p}))]}));
