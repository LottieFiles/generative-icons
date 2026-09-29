// Shared assembly for the Preferences set (copied from 39-food-drink helpers). No artwork.
export * from '../../library/ai-authoring.mjs';
import {make,timePair,drive,mix} from '../../library/ai-authoring.mjs';
export const TAU=Math.PI*2,cycle=t=>t/2.4,mod=x=>((x%1)+1)%1;
export function toggle(name,label,intent,parts,light,strong,initialState,finalState,reverseLight=light,reverseStrong=strong){
 return {...make(name,label,intent,parts,light,strong),stateful:true,initialState,finalState,reverseSuffix:'-reverse',forwardLabel:finalState,reverseLabel:initialState,reverseTimings:timePair(reverseLight,reverseStrong)};
}
export const curve=(t,s,l,h)=>drive(t,s,l,h);
// A 0→1 state driver. The reverse action plays its own keys backwards (1→0) so both
// directions share the exact same end drawings.
export const U=(fl,fs,rl=fl,rs=fs)=>(t,s,r)=>r?1-drive(t,s,rl,rs):drive(t,s,fl,fs);
export const clamp01=x=>Math.max(0,Math.min(1,x));
// Fixed-topology circular arc: always n cubic segments (screen angles, degrees).
export const arcN=(cx,cy,r,a,b,n=4)=>{const A=d=>d*Math.PI/180,p=t=>[cx+r*Math.cos(t),cy+r*Math.sin(t)];let s=A(a),d='M'+p(s);for(let i=1;i<=n;i++){const e=A(a+(b-a)*i/n),k=4/3*Math.tan((e-s)/4),u=p(s),v=p(e);d+=' C'+[u[0]-k*r*Math.sin(s),u[1]+k*r*Math.cos(s),v[0]+k*r*Math.sin(e),v[1]-k*r*Math.cos(e),...v];s=e;}return d;};
export const ellipse=(x,y,rx,ry)=>{const k=.552285;return `M${x} ${y-ry} C${x+rx*k} ${y-ry} ${x+rx} ${y-ry*k} ${x+rx} ${y} C${x+rx} ${y+ry*k} ${x+rx*k} ${y+ry} ${x} ${y+ry} C${x-rx*k} ${y+ry} ${x-rx} ${y+ry*k} ${x-rx} ${y} C${x-rx} ${y-ry*k} ${x-rx*k} ${y-ry} ${x} ${y-ry} Z`;};
// Linear blend of two path strings with identical command layout (same topology).
const NUM=/-?\d*\.?\d+(?:e[-+]?\d+)?/g;
export function blend(a,b,u){const na=a.match(NUM).map(Number),nb=b.match(NUM).map(Number);if(na.length!==nb.length)throw Error('blend: layouts differ');let i=0;return a.replace(NUM,()=>String(mix(na[i],nb[i++],u)));}
// Open arrow head at E for a path travelling in unit direction dir: two barbs swept back.
export const arrowHead=(E,dir,len=2.4,spread=32)=>{const a=Math.atan2(-dir[1],-dir[0]),p=d=>[E[0]+len*Math.cos(a+d*Math.PI/180),E[1]+len*Math.sin(a+d*Math.PI/180)];return `M${p(spread)} L${E} L${p(-spread)}`;};
export const L=d=>d.replace(/^M/,'L');
