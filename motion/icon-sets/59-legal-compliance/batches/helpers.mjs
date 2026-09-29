// Shared assembly for the Legal & compliance set (copied from 39-food-drink helpers). No artwork.
export * from '../../library/ai-authoring.mjs';
import {make,timePair,drive} from '../../library/ai-authoring.mjs';
export function toggle(name,label,intent,parts,light,strong,initialState,finalState,reverseLight=light,reverseStrong=strong,labels={}){
 return {...make(name,label,intent,parts,light,strong),stateful:true,initialState,finalState,reverseSuffix:'-reverse',forwardLabel:labels.forward||finalState,reverseLabel:labels.reverse||initialState,reverseTimings:timePair(reverseLight,reverseStrong)};
}
// A stateful driver pair: forward keys run initial → final, reverse keys final → initial.
export const dir=(forward,reverse)=>(t,s,r)=>r?reverse(t,s):forward(t,s);
export const clamp01=x=>Math.max(0,Math.min(1,x));
// Fixed-topology circular arc: always n cubic segments.
export const arcN=(cx,cy,r,a,b,n=4)=>{const A=d=>d*Math.PI/180,p=t=>[cx+r*Math.cos(t),cy+r*Math.sin(t)];let s=A(a),d='M'+p(s);for(let i=1;i<=n;i++){const e=A(a+(b-a)*i/n),k=4/3*Math.tan((e-s)/4),u=p(s),v=p(e);d+=' C'+[u[0]-k*r*Math.sin(s),u[1]+k*r*Math.cos(s),v[0]+k*r*Math.sin(e),v[1]-k*r*Math.cos(e),...v];s=e;}return d;};
// Fixed-topology elliptical arc (n cubic segments), angles in degrees.
export const ellArc=(cx,cy,rx,ry,a,b,n=4)=>{const A=d=>d*Math.PI/180,p=t=>[cx+rx*Math.cos(t),cy+ry*Math.sin(t)];let s=A(a),d='M'+p(s);for(let i=1;i<=n;i++){const e=A(a+(b-a)*i/n),k=4/3*Math.tan((e-s)/4),u=p(s),v=p(e);d+=' C'+[u[0]-k*rx*Math.sin(s),u[1]+k*ry*Math.cos(s),v[0]+k*rx*Math.sin(e),v[1]-k*ry*Math.cos(e),...v];s=e;}return d;};
export const ellipse=(cx,cy,rx,ry)=>ellArc(cx,cy,rx,ry,0,360,4)+' Z';
// Hermite → cubic for a sampled curve fn(u) on [0,1]; n segments.
export function curveFrom(fn,n=4){const h=1e-4,dv=u=>{const a=fn(Math.max(0,u-h)),b=fn(Math.min(1,u+h));const w=Math.min(1,u+h)-Math.max(0,u-h);return[(b[0]-a[0])/w,(b[1]-a[1])/w];};let d='M'+fn(0);for(let i=0;i<n;i++){const u0=i/n,u1=(i+1)/n,p0=fn(u0),p1=fn(u1),d0=dv(u0),d1=dv(u1),k=(u1-u0)/3;d+=' C'+[p0[0]+d0[0]*k,p0[1]+d0[1]*k,p1[0]-d1[0]*k,p1[1]-d1[1]*k,...p1];}return d;}
export const L=d=>d.replace(/^M/,'L');
// A true closed envelope around a circular arc, including round end caps.
export function arcTube(cx,cy,r,a,b,width){
 const h=width/2,sign=Math.sign(b-a)||1,p=deg=>[cx+r*Math.cos(deg*Math.PI/180),cy+r*Math.sin(deg*Math.PI/180)],start=p(a),end=p(b);
 return arcN(cx,cy,r+h,a,b)+L(arcN(...end,h,b,b+sign*180,2))+L(arcN(cx,cy,r-h,b,a))+L(arcN(...start,h,a+sign*180,a+sign*360,2))+' Z';
}
// A polyline revealed from its start: segments past the pen tip collapse onto it.
// Callers hide the part while the revealed length is near zero (round-cap dot).
export function reveal(points,p){
 const lens=points.slice(1).map((q,i)=>Math.hypot(q[0]-points[i][0],q[1]-points[i][1])),total=lens.reduce((a,b)=>a+b,0);
 let left=Math.max(0,Math.min(1,p))*total,tip=points[0];const out=[points[0]];
 for(let i=0;i<lens.length;i++){if(left>=lens[i]){out.push(points[i+1]);tip=points[i+1];left-=lens[i];}else{const u=lens[i]?left/lens[i]:0;tip=[points[i][0]+(points[i+1][0]-points[i][0])*u,points[i][1]+(points[i+1][1]-points[i][1])*u];for(let j=i;j<lens.length;j++)out.push(tip);break;}}
 return 'M'+out[0]+out.slice(1).map(q=>' L'+q).join('');
}
