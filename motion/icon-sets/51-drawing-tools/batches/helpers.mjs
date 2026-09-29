// Shared assembly for the Drawing & design tools set (copied from 39-food-drink helpers). No artwork.
export * from '../../library/ai-authoring.mjs';
import {make,timePair,drive} from '../../library/ai-authoring.mjs';
import {rigid as nativeRigid} from '../../library/drawing.mjs';
import {track as quietTrack} from '../../library/direction.mjs';
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
// A forward/reverse driver pair for true two-state toggles: r selects the reverse tracks.
export const FR=(lf,sf,lr,sr)=>{const f=(t,s)=>drive(t,s,lf,sf),b=(t,s)=>drive(t,s,lr,sr);return (t,s,r)=>r?b(t,s):f(t,s);};
// Fixed-topology circular arc: always n cubic segments (adapted from 39-food-drink/batches/kitchen.mjs).
export const arcN=(cx,cy,r,a,b,n=4)=>{const A=d=>d*Math.PI/180,p=t=>[cx+r*Math.cos(t),cy+r*Math.sin(t)];let s=A(a),d='M'+p(s);for(let i=1;i<=n;i++){const e=A(a+(b-a)*i/n),k=4/3*Math.tan((e-s)/4),u=p(s),v=p(e);d+=' C'+[u[0]-k*r*Math.sin(s),u[1]+k*r*Math.cos(s),v[0]+k*r*Math.sin(e),v[1]-k*r*Math.cos(e),...v];s=e;}return d;};
// Hermite → cubic for a sampled curve fn(u) on [0,1]; n segments. Sampling fn(u*p) reveals the
// curve up to progress p with constant topology; the unrevealed remainder collapses at the tip.
export function curveFrom(fn,n=4){const h=1e-4,dv=u=>{const a=fn(Math.max(0,u-h)),b=fn(Math.min(1,u+h));const w=Math.min(1,u+h)-Math.max(0,u-h);return[(b[0]-a[0])/w,(b[1]-a[1])/w];};let d='M'+fn(0);for(let i=0;i<n;i++){const u0=i/n,u1=(i+1)/n,p0=fn(u0),p1=fn(u1),d0=dv(u0),d1=dv(u1),k=(u1-u0)/3;d+=' C'+[p0[0]+d0[0]*k,p0[1]+d0[1]*k,p1[0]-d1[0]*k,p1[1]-d1[1]*k,...p1];}return d;}
export const reveal=(fn,n=4)=>p=>curveFrom(u=>fn(u*Math.max(0,Math.min(1,p))),n);
// Cubic Bézier point for authored swash curves.
export const bez=(a,b,c,d)=>u=>{const v=1-u;return [0,1].map(i=>v*v*v*a[i]+3*v*v*u*b[i]+3*v*u*u*c[i]+u*u*u*d[i]);};
export const L=d=>d.replace(/^M/,'L');
export const pt=(a,r,c=[16,16])=>[c[0]+r*Math.cos(a*Math.PI/180),c[1]+r*Math.sin(a*Math.PI/180)];
export const clamp01=x=>Math.max(0,Math.min(1,x));
