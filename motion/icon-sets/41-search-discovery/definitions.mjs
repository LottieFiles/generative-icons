// Search & discovery (41). Designs, reviews and corrections: evidence/design-review.md.
import {icons as a} from './batches/lens.mjs';
import {icons as b} from './batches/fields.mjs';
import {icons as c} from './batches/discovery.mjs';
const all=new Map([...a,...b,...c].map(i=>[i.name,i]));
// Sample the clock hand into native path coordinates. Exporting its full turn
// as a rotation leaves a small raster difference between -360 degrees and 0.
// Keep the authored motion while making both profiles' rest geometry exact.
for(const part of all.get('recent-searches').parts.filter(p=>/^recent-searches-minute(?:-solid)?$/.test(p.name))){
 const pose=part.pose;
 part.pose=(...args)=>pose(...args);
}
const ORDER=['search-suggest','recent-searches','search-clear','search-voice','search-image','search-filter','sort-ascending','sort-descending','hashtag','tag-add','tag-remove','trending-up','trending-down','telescope','related-items','saved-search','web-search','search-match','search-scope','searching'];
if(ORDER.length!==all.size||ORDER.some(n=>!all.has(n)))throw Error('Search & discovery order and batches disagree');
const fixed={"recent-searches":["lens","handle"],"search-voice":["lens","handle"],"saved-search":["lens","handle"],"searching":["lens","handle"],"search-scope":["lens","handle"],"search-image":["frame","mountain"],"search-suggest":["field"],"telescope":["post","base"],"related-items":["hub"],"web-search":["globe","equator"]};
const rigidParts={"recent-searches":["hour","minute"],"search-filter":["lens","handle"],"saved-search":["ribbon"],"search-match":["lens","handle"],"searching":["sweep"],"search-image":["lens","handle"],"search-suggest":["item-0","item-1","item-2"],"search-clear":["field"],"sort-ascending":["shaft","head"],"sort-descending":["shaft","head"],"hashtag":["post-0","post-1","bar-0","bar-1"],"tag-add":["tag","eyelet"],"tag-remove":["tag","eyelet"],"telescope":["tube"],"related-items":["item-0","item-1","item-2"],"web-search":["lens","handle"]};
export const icons=ORDER.map(n=>all.get(n)).map((icon,index)=>({...icon,number:646+index,constraints:[...(fixed[icon.name]||[]).map(p=>({type:'fixed',part:icon.name+'-'+p})),...(rigidParts[icon.name]||[]).map(p=>({type:'rigid',part:icon.name+'-'+p}))]}));
