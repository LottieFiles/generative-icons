// Smart Home (37). Designs, reviews and corrections: evidence/design-review.md.
import {icons as a} from './batches/home-a.mjs';
import {icons as b} from './batches/home-b.mjs';
import {icons as c} from './batches/home-c.mjs';
import {icons as d} from './batches/home-d.mjs';
const all=new Map([...a,...b,...c,...d].map(i=>[i.name,i]));
const ORDER=["smart-bulb", "thermostat", "smart-lock", "door-sensor", "garage-door", "ceiling-fan", "air-conditioner", "smart-speaker", "video-doorbell", "security-camera", "window-blinds", "robot-vacuum", "washing-machine", "refrigerator", "oven", "smoke-alarm", "water-leak", "power-outlet", "energy-usage", "motion-sensor"];
if(ORDER.length!==all.size||ORDER.some(n=>!all.has(n)))throw Error('Smart home order and batches disagree');
const fixed={"smart-bulb":["thread-0","thread-1"],"thermostat":["ring-frame"],"door-sensor":["sensor-frame"],"garage-door":["house","opening"],"air-conditioner":["unit-frame"],"security-camera":["plate","arm","joint"],"window-blinds":["rail"],"refrigerator":["cabinet","divider","handle-top"],"oven":["body-frame","window-frame"],"smoke-alarm":["ceiling"],"water-leak":["pipe-l","pipe-r","coupling-frame"],"smart-lock":["plate-frame","rose-frame"],"ceiling-fan":["ceiling"],"power-outlet":["plate-frame","socket-frame","slot-l","slot-r"],"motion-sensor":["wall","dome-frame"]};
const rigidParts={"smart-lock":["lever"],"security-camera":["body-frame"],"washing-machine":["swirl","body-frame"],"robot-vacuum":["bumper","lidar"],"power-outlet":["button"]};
export const icons=ORDER.map(n=>all.get(n)).map((icon,index)=>({...icon,number:566+index,constraints:[...(fixed[icon.name]||[]).map(p=>({type:'fixed',part:icon.name+'-'+p})),...(rigidParts[icon.name]||[]).map(p=>({type:'rigid',part:icon.name+'-'+p}))]}));
