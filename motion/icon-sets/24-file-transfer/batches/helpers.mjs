// Common assembly for the reviewed Claude motion-editor drawings. No shared artwork edits.
export * from '../../library/ai-authoring.mjs';
import {make,timePair,drive} from '../../library/ai-authoring.mjs';
export const TAU=Math.PI*2,cycle=t=>t/2.4,mod=x=>((x%1)+1)%1;
export function continuous(name,label,intent,parts,duotoneFill=false){
 const i=make(name,label,intent,parts,2.4,2.4);
 return {...i,continuous:true,stateful:false,duotoneFill,timings:Object.fromEntries(['light','strong'].map(m=>[m,{active:2.4,rest:0,keyPoses:[0,.2,.4,.7,1.1,1.5,1.9,2.3]}]))};
}
export function toggle(name,label,intent,parts,light,strong,initialState,finalState,reverseLight=light,reverseStrong=strong){
 return {...make(name,label,intent,parts,light,strong),stateful:true,initialState,finalState,reverseSuffix:'-reverse',forwardLabel:finalState,reverseLabel:initialState,reverseTimings:timePair(reverseLight,reverseStrong)};
}
export const curve=(t,s,l,h)=>drive(t,s,l,h);
export const field=(x,baseline,height)=>`M${x} ${baseline} L${x} ${baseline-height}`;
// Invert the time coordinate of an explicitly directed cubic timing curve.
export function cubicEase(u,x1,y1,x2,y2){u=Math.max(0,Math.min(1,u));let lo=0,hi=1;const b=(t,a,c)=>3*(1-t)*(1-t)*t*a+3*(1-t)*t*t*c+t*t*t;for(let i=0;i<30;i++){const t=(lo+hi)/2;if(b(t,x1,x2)<u)lo=t;else hi=t;}return u===0?0:u===1?1:b((lo+hi)/2,y1,y2);}
// A true closed envelope around a circular arc, including round end caps.
// Used for transparent counters in filled surfaces, never background patches.
import {arc as circularArc} from '../../library/drawing.mjs';
import {rigid as nativeRigid} from '../../library/drawing.mjs';
import {track as quietTrack} from '../../library/direction.mjs';
// Preserve the designed forward action. Retract connected output as opaque
// geometry during the gallery-only reset so joined strokes never fade twice.
// Parts that have the same initial/final state stay still during the reset.
export function geometricRearm(icon){
 const differs=part=>[false,true].some(s=>{
  const snap=t=>JSON.stringify({d:typeof part.d==='function'?part.d(t,s,false,2):part.d,pose:part.pose?.transform?part.pose.transform(t,s,false):part.pose?[part.pose(0,0,t,s,false),part.pose(32,32,t,s,false)]:null,v:part.visibility?.(t,s,false)??1});
  return snap(0)!==snap(2.4);
 });
 for(const part of icon.parts){
  const changing=differs(part);
  const clock=(t,s,r)=>r?(changing?quietTrack([[0,icon.timings[s?'strong':'light'].active],[.3,0,'smooth']],t):0):t;
  if(typeof part.d==='function'){const d=part.d;part.d=(t,s,r,w)=>d(clock(t,s,r),s,false,w);}
  if(part.pose){const pose=part.pose;part.pose=pose.transform?nativeRigid((t,s,r)=>pose.transform(clock(t,s,r),s,false)):(x,y,t,s,r)=>pose(x,y,clock(t,s,r),s,false);}
  if(part.visibility){const v=part.visibility;part.visibility=(t,s,r)=>v(clock(t,s,r),s,false);}
 }
 return {...icon,stateful:true,initialState:icon.initialState||'prepared',finalState:icon.finalState||'result',previewReset:true,reverseSuffix:'-reset',forwardLabel:'Play action',reverseLabel:'Reset preview',reverseTimings:timePair(.3,.3)};
}
export function arcTube(cx,cy,r,a,b,width){
 const h=width/2,sign=Math.sign(b-a)||1,p=deg=>[cx+r*Math.cos(deg*Math.PI/180),cy+r*Math.sin(deg*Math.PI/180)],start=p(a),end=p(b);
 return circularArc(cx,cy,r+h,a,b)+circularArc(...end,h,b,b+sign*180).replace(/^M/,'L')+circularArc(cx,cy,r-h,b,a).replace(/^M/,'L')+circularArc(...start,h,a+sign*180,a+sign*360).replace(/^M/,'L')+' Z';
}
