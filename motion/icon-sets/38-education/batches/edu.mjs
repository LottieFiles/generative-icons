// Education (38) shared construction. No artwork of its own beyond small assembly helpers.
import {fill,stroke} from './helpers.mjs';
export * from './sky.mjs';
// A container that holds moving or inner detail: tint surface, Outline frame, and a Solid *ring*
// (a 2.6 stroke), so Solid never buries the detail inside a filled silhouette.
export const container=(name,d,extra={})=>[fill(name+'-interior',d,'tint',extra),stroke(name+'-frame',d,extra),{name:name+'-solid',d,style:'solid',type:'stroke',width:2.6,...extra}];
// Outline + Solid stroke pair for a path that may be a function of time.
export const pen=(name,d,extra={})=>[stroke(name,d,extra),{name:name+'-solid',d,style:'solid',type:'stroke',width:2.6,...extra}];
// A progressively drawn curve: fn(u) on [0,1] is revealed up to p(t,s); topology is fixed (n cubics).
import {curveFrom} from './sky.mjs';
export const reveal=(fn,p,n=4)=>(t,s)=>curveFrom(u=>fn(Math.max(.002,p(t,s))*u),n);
// A polyline revealed by progress p in [0,1]; always the same number of points.
export function drawPolyline(pts,p){
 const seg=pts.slice(1).map((b,i)=>Math.hypot(b[0]-pts[i][0],b[1]-pts[i][1])),total=seg.reduce((a,b)=>a+b,0);
 return (t,s)=>{let left=Math.max(.002,p(t,s))*total;const out=[pts[0]];
  for(let i=0;i<seg.length;i++){const a=pts[i],b=pts[i+1],k=Math.max(0,Math.min(1,left/seg[i]));out.push([a[0]+(b[0]-a[0])*k,a[1]+(b[1]-a[1])*k]);left-=seg[i];}
  return 'M'+out.map(q=>q.join(' ')).join(' L');};
}
export const lerp=(a,b,k)=>[a[0]+(b[0]-a[0])*k,a[1]+(b[1]-a[1])*k];
