// Claude Code first design; Codex geometry, motion, and native export supervision.
export * from '../../library/ai-authoring.mjs';
import {framed,circle,hole,rect,tube,fill,mix,arc} from '../../library/ai-authoring.mjs';
export const ink=tube;
export const policy=(icon,duotoneFill=true)=>Object.assign(icon,{duotoneFill,revision:'finance-v1-claude-supervised'});
export const ring=(name,x,y,r)=>framed(name,circle(x,y,r),hole(circle(x,y,r+1.3),circle(x,y,r-1.3)));
// A real, transparent content window. The fixed Solid housing never fades when a result resets.
export const panel=(name,x,y,w,h,r=2.5)=>framed(name,rect(x,y,w,h,r),hole(rect(x-1.3,y-1.3,w+2.6,h+2.6,r+1.3),rect(x+1.3,y+1.3,w-2.6,h-2.6,Math.max(.6,r-1.3))));
export const disc=(name,x,y,r,extra={})=>framed(name,circle(x,y,r),circle(x,y,r+1.15),extra);
export const tint=(name,d)=>fill(name,d,'tint');
export const reveal=(name,a,b,p)=>ink(name,(t,s)=>`M${a} L${mix(a[0],b[0],p(t,s))} ${mix(a[1],b[1],p(t,s))}`,{visibility:(t,s)=>p(t,s)>1e-5?1:0});
export function sweep(cx,cy,r,a,b){const at=v=>[cx+r*Math.cos(v),cy+r*Math.sin(v)];let start=a*Math.PI/180,d='M'+at(start);for(let i=1;i<=4;i++){const end=(a+(b-a)*i/4)*Math.PI/180,k=4/3*Math.tan((end-start)/4),u=at(start),v=at(end);d+=' C'+[u[0]-k*r*Math.sin(start),u[1]+k*r*Math.cos(start),v[0]+k*r*Math.sin(end),v[1]-k*r*Math.cos(end),...v];start=end;}return d;}
export function arrowArc(name,cx,cy,r,a,b){const angle=b*Math.PI/180,tip=[cx+r*Math.cos(angle),cy+r*Math.sin(angle)],tangent=[-Math.sin(angle),Math.cos(angle)],normal=[Math.cos(angle),Math.sin(angle)],arm=(side)=>[tip[0]-2.2*tangent[0]+side*1.9*normal[0],tip[1]-2.2*tangent[1]+side*1.9*normal[1]];return [...ink(name+'-arc',arc(cx,cy,r,a,b)),...ink(name+'-head',`M${arm(-1)} L${tip} L${arm(1)}`)];}
export function sector(cx,cy,r,a,b){const p=(radius,angle)=>[cx+radius*Math.cos(angle*Math.PI/180),cy+radius*Math.sin(angle*Math.PI/180)];const v=.75,delta=5,start=p(r,a+delta),end=p(r,b-delta),ra=p(r-v,a),rb=p(r-v,b),ca=p(v,a),cb=p(v,b);return `M${ca} L${ra} C${p(r,a)} ${start} ${start} `+arc(cx,cy,r,a+delta,b-delta).replace(/^M[^C]+/,'')+` C${end} ${p(r,b)} ${rb} L${cb} C${cx} ${cy} ${cx} ${cy} ${ca} Z`;}
