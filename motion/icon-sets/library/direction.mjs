import { mix } from './drawing.mjs';
export const clamp=x=>Math.max(0,Math.min(1,x));
// The destination key chooses the interval's acceleration. Linear is reserved
// for transport/phase; smooth contacts and arrivals use explicitly chosen curves.
export function track(keys,t){if(t<=keys[0][0])return keys[0][1];if(t>=keys.at(-1)[0])return keys.at(-1)[1];const i=keys.findIndex(k=>k[0]>t),a=keys[i-1],b=keys[i],u=(t-a[0])/(b[0]-a[0]),e=b[2]||'io';const p=e==='out'?1-(1-u)**3:e==='in'?u**3:e==='linear'?u:e==='smooth'?u*u*(3-2*u):(1-Math.cos(Math.PI*u))/2;return mix(a[1],b[1],p);}
export const motion=(t,s,light,strong)=>track(s?strong:light,t);
// Bounded cubic motion. Ordinary same-direction keys share a tangent, so a
// waypoint does not become a stop. Extrema, explicit holds and end states stop.
// Spring extrema are authored in the keys; this never invents extra overshoot.
// Contact tags deliberately break velocity: hit (impact), launch (release).
export function flow(keys,t){
 const n=keys.length;if(!n)return 0;
 if(t<=keys[0][0])return keys[0][1];if(t>=keys[n-1][0])return keys[n-1][1];
 let i=0;while(keys[i+1][0]<=t)i++;
 const span=k=>keys[k+1][0]-keys[k][0];
 const sec=k=>(keys[k+1][1]-keys[k][1])/span(k);
 const tangent=(k,side)=>{
  if(k===0||k===n-1||keys[k][2]==='hold')return 0;
  const a=sec(k-1),b=sec(k),tag=keys[k][2];
  if(!a||!b)return 0;
  if(tag==='hit'||tag==='launch'){
   const gain=tag==='hit'?(side==='in'?1.6:.7):(side==='in'?.7:1.6);
   return (side==='in'?a:b)*gain;
  }
  if(a*b<=0)return 0;
  const h0=span(k-1),h1=span(k),w0=2*h1+h0,w1=h1+2*h0;
  const slope=(w0+w1)/(w0/a+w1/b)*(tag==='pass'?1.35:1);
  return Math.sign(a)*Math.min(Math.abs(slope),3*Math.min(Math.abs(a),Math.abs(b)));
 };
 const d=sec(i);if(!d)return keys[i][1];
 const h=span(i),u=(t-keys[i][0])/h,u2=u*u,u3=u2*u;
 const m0=tangent(i,'out')*h,m1=tangent(i+1,'in')*h;
 return keys[i][1]*(2*u3-3*u2+1)+m0*(u3-2*u2+u)+keys[i+1][1]*(-2*u3+3*u2)+m1*(u3-u2);
}
export const drive=(t,s,light,strong)=>s?flow(strong,t):track(light,t);
export const timePair=(light,strong,lp,sp)=>({light:{active:light,rest:light+.04,keyPoses:lp||[0,light*.2,light*.4,light*.6,light*.8,light]},strong:{active:strong,rest:strong+.04,keyPoses:sp||[0,strong*.2,strong*.4,strong*.6,strong*.8,strong]}});
export function revise(icon,{parts=icon.parts,light,strong,posesLight,posesStrong,intent, ...extra}){Object.assign(icon,{parts,timings:timePair(light,strong,posesLight,posesStrong),intent,description:intent,revision:'directed-v2',...extra});return icon;}
// A result is not played backwards to rearm the gallery. Only changing parts
// fade, reset while invisible, then return. The stationary shell remains visible.
export const rearmTime=(t,s,r,end)=>r?(t<.12?end:0):t;
export const rearmOpacity=(t,s,r)=>r?track([[0,1],[.08,0,'out'],[.15,0],[.25,1,'out']],t):1;
export function resultAction(icon,partsToRearm){
 Object.assign(icon,{stateful:true,initialState:'prepared',finalState:'result',reverseSuffix:'-reset',forwardLabel:'Play action',reverseLabel:'Reset preview',reverseTimings:timePair(.28,.30,[0,.06,.1,.14,.2,.28],[0,.06,.1,.14,.22,.30]),previewReset:true});
 for(const part of icon.parts.filter(partsToRearm)){
  const d=part.d,pose=part.pose,visibility=part.visibility;
  if(typeof d==='function')part.d=(t,s,r)=>d(rearmTime(t,s,r,2.4),s,false);
  if(pose){part.pose=(x,y,t,s,r)=>pose(x,y,rearmTime(t,s,r,2.4),s,false);if(pose.transform)part.pose.transform=(t,s,r)=>pose.transform(rearmTime(t,s,r,2.4),s,false);}
  part.visibility=(t,s,r)=>rearmOpacity(t,s,r)*(visibility?visibility(rearmTime(t,s,r,2.4),s,false):1);
 }
 return icon;
}
export const has=(part,...keys)=>keys.some(k=>part.name.includes(k));
export function rePose(icon,predicate,fn){for(const part of icon.parts.filter(predicate))part.pose=fn;}
