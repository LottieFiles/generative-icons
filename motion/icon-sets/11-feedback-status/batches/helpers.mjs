// Codex design, authorized after Claude Code hit its session limit.
export * from '../../library/ai-authoring.mjs';
import {framed,circle,hole,rect,fill,stroke,tube,mix,track} from '../../library/ai-authoring.mjs';
export const ring=(name,x=16,y=16,r=10.5)=>framed(name,circle(x,y,r),hole(circle(x,y,r+1.3),circle(x,y,r-1.3)));
export const frame=(name,x,y,w,h,r=2)=>framed(name,rect(x,y,w,h,r),hole(rect(x-1.3,y-1.3,w+2.6,h+2.6,r+1.3),rect(x+1.3,y+1.3,w-2.6,h-2.6,Math.max(.5,r-1.3))));
export const ink=(name,d,extra={})=>tube(name,d,extra);
export const dynamic=(parts,fn)=>parts.map(p=>({...p,d:(t,s,r)=>fn(t,s,r,p.style)}));
export const policy=(icon,duotoneFill)=>Object.assign(icon,{duotoneFill,revision:'feedback-v1-codex'});
// Fixed four-cubic topology, including when the revealed sweep is less than 90°.
export function sweep(cx,cy,r,a,b){const point=v=>[cx+r*Math.cos(v),cy+r*Math.sin(v)];let start=a*Math.PI/180,d='M'+point(start);for(let i=1;i<=4;i++){const end=(a+(b-a)*i/4)*Math.PI/180,k=4/3*Math.tan((end-start)/4),u=point(start),v=point(end);d+=' C'+[u[0]-k*r*Math.sin(start),u[1]+k*r*Math.cos(start),v[0]+k*r*Math.sin(end),v[1]-k*r*Math.cos(end),...v];start=end;}return d;}
export const revealLine=(name,a,b,progress)=>ink(name,(t,s)=>{const p=progress(t,s);return `M${a} L${mix(a[0],b[0],p)} ${mix(a[1],b[1],p)}`;},{visibility:(t,s)=>progress(t,s)>.0001?1:0});
export const tint=(name,d)=>fill(name,d,'tint');
