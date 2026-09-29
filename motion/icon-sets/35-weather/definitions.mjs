// Weather (35). Designs, reviews and corrections: evidence/design-review.md.
import {icons as a} from './batches/weather-a.mjs';
import {icons as b} from './batches/weather-b.mjs';
import {icons as c} from './batches/weather-c.mjs';
import {icons as d} from './batches/weather-d.mjs';
import {icons as e} from './batches/weather-e.mjs';
const all=new Map([...a,...b,...c,...d,...e].map(i=>[i.name,i]));
const ORDER=['partly-cloudy','rain','snow','hail','thunderstorm','snowflake','wind','fog','tornado','hurricane','rainbow','sunrise','sunset','moon-phase','temperature','humidity','uv-index','umbrella','windsock','weather-radar'];
if(ORDER.length!==all.size||ORDER.some(n=>!all.has(n)))throw Error('Weather order and batches disagree');
const fixed={"snow":["cloud-frame"],"hail":["cloud-frame","ground"],"fog":["cloud-frame"],"rainbow":["horizon"],"sunrise":["horizon"],"sunset":["horizon"],"temperature":["body-frame","tick-0","tick-1","tick-2"],"humidity":["drop"],"weather-radar":["scope-frame","range"],"windsock":["pole","foot"]};
const rigidParts={"snowflake":["arm-0","arm-1","arm-2"],"hurricane":["arm-0","arm-1","eye-frame"],"thunderstorm":["bolt"],"weather-radar":["arm"],"wind":["gust-0","gust-1","gust-2"],"partly-cloudy":["cloud-frame"]};
export const icons=ORDER.map(n=>all.get(n)).map((icon,index)=>({...icon,number:526+index,constraints:[...(fixed[icon.name]||[]).map(p=>({type:'fixed',part:icon.name+'-'+p})),...(rigidParts[icon.name]||[]).map(p=>({type:'rigid',part:icon.name+'-'+p}))]}));
