// Claude proposal-01 and revision-curves, transcribed after supervisor review.
import {route,arrowhead,annotation} from './reveal.mjs';
import {driver,phase,flex,anchor} from './drivers.mjs';
const progress=phase(.06,1.10,.04,.58,.10,.26);
const head=phase(1.10,1.34,.58,.76,0,.10);
const f=flex([[.04,0],[.50,1],[.92,0]]);
const pair=mirror=>{
 const x=v=>mirror?32-v:v,id='annotation-curve-'+(mirror?'left':'right');
 const d=(t,s,r)=>{const k=f(t,s,r);return `M${x(7)} 24 C${x(7)} ${14-2*k} ${x(13-2.5*k)} 9 ${x(20.5)} 9 L${x(26)} 9`;};
 return annotation(id,'Curved pointer '+(mirror?'left':'right'),'The curve writes from its source to a fixed target; the head completes the annotation.',[
  ...anchor('origin',x(7),24,2.1),...route('route',d,progress),...arrowhead('head',[x(26),9],[x(21.6),4.6],[x(21.6),13.4],head)
 ],1.40,1,.30);
};
const sPath=(t,s,r)=>{const k=f(t,s,r);return `M6 23 C${12+1.5*k} ${23+.5*k} ${12-1.5*k} ${9-.5*k} 18 9 L26 9`;};
export const curves=[pair(false),pair(true),annotation('annotation-s-curve','S-curve pointer','A flowing S connects two levels before the target head opens.',[
 ...anchor('origin',6,23,2.1),...route('route',sPath,progress),...arrowhead('head',[26,9],[21.6,4.6],[21.6,13.4],head)
],1.40,1,.30)];
