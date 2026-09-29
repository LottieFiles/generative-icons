import {rect,hole,stroke,fill,tube,line,framed,rigid,circle,capsule} from './drawing.mjs';
import {motion,drive,revise} from './direction.mjs';
export function directEssentials(icons){for(const icon of icons){
 if(icon.name==='star'){
  const pose=rigid((t,s)=>({angle:drive(t,s,[[0,0],[.035,-1.5],[.27,72,'out']],[[0,0],[.06,-7,'launch'],[.30,151],[.40,142.5],[.52,144]]),pivot:[16,16.4]}));
  for(const p of icon.parts)p.pose=pose;
  revise(icon,{light:.31,strong:.52,posesStrong:[0,.06,.17,.30,.40,.52],intent:'Light advances one star point; Strong winds up, sweeps through two points in one stroke and springs into the equivalent final drawing.',cycleDegrees:72});
 }
 if(icon.name==='sliders'){
  const x=(i,t,s)=>i?drive(t,s,[[0,21],[.13,21],[.25,19.5,'out'],[.36,19.5],[.51,21]],[[0,21],[.12,21],[.29,17.4],[.40,18],[.63,21.15],[.70,21]]):drive(t,s,[[0,11],[.19,14,'out'],[.34,14],[.48,11]],[[0,11],[.05,10.6,'launch'],[.20,16.7],[.32,16],[.54,10.8],[.62,11]]);
  const parts=[];for(let i=0;i<2;i++){const y=i?21:11;parts.push(fill('sliders-tint-'+i,(t,s)=>circle(x(i,t,s),y,3),'tint'));for(const side of [0,1]){const ends=(t,s)=>side?[[x(i,t,s)+3,y],[26,y]]:[[6,y],[x(i,t,s)-3,y]];parts.push(stroke('sliders-rail-'+i+'-'+side,(t,s)=>{const[a,b]=ends(t,s);return`M${a} L${b}`;}),fill('sliders-solid-rail-'+i+'-'+side,(t,s)=>capsule(...ends(t,s),1.8)));}parts.push(stroke('sliders-handle-'+i,(t,s)=>circle(x(i,t,s),y,3)),fill('sliders-solid-handle-'+i,(t,s)=>circle(x(i,t,s),y,3.4)));}
  revise(icon,{parts,light:.55,strong:.70,posesStrong:[0,.05,.24,.38,.50,.70],intent:'The top handle over-drags and corrects; the lower handle answers with its own smaller over-drag before both return.'});
 }
 if(icon.name==='folder'){
  const back=icon.parts.find(p=>p.name==='folder-shell').d,front=rect(6,14.1,20,11.9,2.4);
  const open=(t,s)=>drive(t,s,[[0,0],[.05,-.04],[.20,.26,'out'],[.30,.26],[.48,0,'in']],[[0,0],[.06,-.06,'launch'],[.22,.56],[.34,.50],[.59,-.02],[.65,0]]);
  const hinge=(x,y,t,s)=>[x,26+(y-26)*(1-open(t,s))];
  const rise=(t,s)=>drive(t,s,[[0,0],[.17,0],[.26,.5],[.35,.5],[.44,0]],[[0,0],[.18,0],[.29,2.1],[.40,1.6],[.57,0]]);
  const sheet=(t,s)=>{const bottom=14.1+open(t,s)*11.9,top=Math.min(bottom-.02,14.1-rise(t,s)),r=Math.min(1,(bottom-top)*.4);return `M9 ${bottom} L9 ${top+r} C9 ${top+r*.448} ${9+r*.448} ${top} ${9+r} ${top} L${23-r} ${top} C${23-r*.448} ${top} 23 ${top+r*.448} 23 ${top+r} L23 ${bottom}`;};
  const visible=(t,s)=>Math.max(0,Math.min(1,open(t,s)*8));
  revise(icon,{light:.52,strong:.68,posesStrong:[0,.06,.25,.40,.52,.68],intent:'A complete front panel springs open from its bottom hinge; one sheet pops up behind it, then everything closes in one motion.',parts:[...framed('folder-back',back,hole(back,rect(7.8,11.8,16.4,12.4,1.1))),...tube('folder-sheet',sheet,{visibility:visible}),...framed('folder-front',front,front,{pose:hinge})]});
 }
}}
