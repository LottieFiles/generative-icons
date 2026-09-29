// Shared assembly for the Gamification & rewards set (copied from 39-food-drink helpers). No artwork.
export * from '../../library/ai-authoring.mjs';
import {make,timePair,drive} from '../../library/ai-authoring.mjs';
import {rigid as nativeRigid,arc as circularArc} from '../../library/drawing.mjs';
import {track as quietTrack} from '../../library/direction.mjs';
export const TAU=Math.PI*2;
export function toggle(name,label,intent,parts,light,strong,initialState,finalState,reverseLight=light,reverseStrong=strong){
 return {...make(name,label,intent,parts,light,strong),stateful:true,initialState,finalState,reverseSuffix:'-reverse',forwardLabel:finalState,reverseLabel:initialState,reverseTimings:timePair(reverseLight,reverseStrong)};
}
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
// Fixed-topology circular arc: always n cubic segments.
export const arcN=(cx,cy,r,a,b,n=4)=>{const A=d=>d*Math.PI/180,p=t=>[cx+r*Math.cos(t),cy+r*Math.sin(t)];let s=A(a),d='M'+p(s);for(let i=1;i<=n;i++){const e=A(a+(b-a)*i/n),k=4/3*Math.tan((e-s)/4),u=p(s),v=p(e);d+=' C'+[u[0]-k*r*Math.sin(s),u[1]+k*r*Math.cos(s),v[0]+k*r*Math.sin(e),v[1]-k*r*Math.cos(e),...v];s=e;}return d;};
export const L=d=>d.replace(/^M/,'L');
export const pt=(a,r,c=[16,16])=>[c[0]+r*Math.cos(a*Math.PI/180),c[1]+r*Math.sin(a*Math.PI/180)];
// Five-point star vertices (outer radius r, inner radius r*k), first point up.
export const starPts=(cx,cy,r,k=.45,n=5)=>Array.from({length:n*2},(_,i)=>pt(-90+i*180/n,i%2?r*k:r,[cx,cy]));
// Symmetric heart: two lobes of radius s*.55 and a rounded tip; width 2*s*1.05, height ~s*1.9.
export const heartD=(cx,cy,s)=>{const w=s*1.05,top=cy-s*.85,tip=cy+s*1.05;return `M${cx} ${tip} C${cx-w*.55} ${cy+s*.35} ${cx-w} ${cy} ${cx-w} ${top+s*.45} C${cx-w} ${top+s*.02} ${cx-w*.72} ${top-s*.15} ${cx-w*.45} ${top-s*.1} C${cx-w*.2} ${top-s*.05} ${cx-w*.05} ${top+s*.15} ${cx} ${top+s*.4} C${cx+w*.05} ${top+s*.15} ${cx+w*.2} ${top-s*.05} ${cx+w*.45} ${top-s*.1} C${cx+w*.72} ${top-s*.15} ${cx+w} ${top+s*.02} ${cx+w} ${top+s*.45} C${cx+w} ${cy} ${cx+w*.55} ${cy+s*.35} ${cx} ${tip} Z`;};
// Sector-shaped counter between two radii for Solid wheels.
export const wedge=(cx,cy,r0,r1,a,b,inset=6)=>`M${pt(a+inset,r0,[cx,cy])} L${pt(a+inset*.6,r1,[cx,cy])} ${L(arcN(cx,cy,r1,a+inset*.6,b-inset*.6,1))} L${pt(b-inset,r0,[cx,cy])} ${L(arcN(cx,cy,r0,b-inset,a+inset,1))} Z`;
export const scalePose=(cx,cy,k)=>(x,y)=>[cx+(x-cx)*k,cy+(y-cy)*k];
