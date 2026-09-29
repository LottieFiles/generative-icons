// Claude Code first design; Codex geometry, motion, and native export supervision.
export * from '../../library/ai-authoring.mjs';
import {framed,circle,hole,rect,tube,fill,mix,arc} from '../../library/ai-authoring.mjs';
export const ink=tube;
export const policy=(icon,duotoneFill=true)=>Object.assign(icon,{duotoneFill,revision:'commerce-v1-claude-supervised'});
export const ring=(name,x,y,r)=>framed(name,circle(x,y,r),hole(circle(x,y,r+1.3),circle(x,y,r-1.3)));
// A real, transparent content window. The fixed Solid housing never fades when a result resets.
export const panel=(name,x,y,w,h,r=2.5)=>framed(name,rect(x,y,w,h,r),hole(rect(x-1.3,y-1.3,w+2.6,h+2.6,r+1.3),rect(x+1.3,y+1.3,w-2.6,h-2.6,Math.max(.6,r-1.3))));
export const disc=(name,x,y,r,extra={})=>framed(name,circle(x,y,r),circle(x,y,r+1.15),extra);
export const tint=(name,d)=>fill(name,d,'tint');
export const reveal=(name,a,b,p)=>ink(name,(t,s)=>`M${a} L${mix(a[0],b[0],p(t,s))} ${mix(a[1],b[1],p(t,s))}`,{visibility:(t,s)=>p(t,s)>1e-5?1:0});
export function sweep(cx,cy,r,a,b){const at=v=>[cx+r*Math.cos(v),cy+r*Math.sin(v)];let start=a*Math.PI/180,d='M'+at(start);for(let i=1;i<=4;i++){const end=(a+(b-a)*i/4)*Math.PI/180,k=4/3*Math.tan((end-start)/4),u=at(start),v=at(end);d+=' C'+[u[0]-k*r*Math.sin(start),u[1]+k*r*Math.cos(start),v[0]+k*r*Math.sin(end),v[1]-k*r*Math.cos(end),...v];start=end;}return d;}
export function arrowArc(name,cx,cy,r,a,b){const angle=b*Math.PI/180,tip=[cx+r*Math.cos(angle),cy+r*Math.sin(angle)],tangent=[-Math.sin(angle),Math.cos(angle)],normal=[Math.cos(angle),Math.sin(angle)],arm=(side)=>[tip[0]-2.2*tangent[0]+side*1.9*normal[0],tip[1]-2.2*tangent[1]+side*1.9*normal[1]];return [...ink(name+'-arc',arc(cx,cy,r,a,b)),...ink(name+'-head',`M${arm(-1)} L${tip} L${arm(1)}`)];}
export function sector(cx,cy,r,a,b){const p=(radius,angle)=>[cx+radius*Math.cos(angle*Math.PI/180),cy+radius*Math.sin(angle*Math.PI/180)];const v=.75,delta=5,start=p(r,a+delta),end=p(r,b-delta),ra=p(r-v,a),rb=p(r-v,b),ca=p(v,a),cb=p(v,b);return `M${ca} L${ra} C${p(r,a)} ${start} ${start} `+arc(cx,cy,r,a+delta,b-delta).replace(/^M[^C]+/,'')+` C${end} ${p(r,b)} ${rb} L${cb} C${cx} ${cy} ${cx} ${cy} ${ca} Z`;}
import {stroke,holes,annulus} from '../../library/ai-authoring.mjs';
const at=(v,t,s,r)=>typeof v==='function'?v(t,s,r):v;
export const surface=(name,d,cuts=[],extra={})=>[fill(name+'-interior',d,'tint',extra),stroke(name,d,extra),fill(name+'-solid',cuts.length?(typeof d==='string'&&cuts.every(c=>typeof c==='string')?holes(d,...cuts):(t,s,r)=>holes(at(d,t,s,r),...cuts.map(c=>at(c,t,s,r)))):d,'solid',extra),{name:name+'-solid-edge',d,type:'stroke',width:2.6,style:'solid',...extra}];
export const plainRing=(name,x,y,r,extra={})=>[stroke(name,circle(x,y,r),extra),fill(name+'-solid',annulus(x,y,r,2.6),'solid',extra)];
export const arch=(left,right,base,top,r=1.5)=>`M${left} ${base} L${left} ${top+r} C${left} ${top+r*.448} ${left+r*.448} ${top} ${left+r} ${top} L${right-r} ${top} C${right-r*.448} ${top} ${right} ${top+r*.448} ${right} ${top+r} L${right} ${base}`;

// A physical reverse returns only the driven mechanism; fixed geometry stays opaque.
import {timePair,drive as motion} from '../../library/ai-authoring.mjs';
export const bidir=(light,strong,reverseLight,reverseStrong)=>(t,s,r)=>motion(t,s,r?reverseLight:light,r?reverseStrong:strong);
export const held=(icon,initialState,finalState,reverseLight=.65,reverseStrong=.8)=>Object.assign(icon,{stateful:true,initialState,finalState,reverseSuffix:'-reverse',forwardLabel:'Play action',reverseLabel:'Return',reverseTimings:timePair(reverseLight,reverseStrong)});

// One canonical path head for reveals and attached travellers; no independent motion path.
export function trace(points,amount){const lengths=points.slice(1).map((p,i)=>Math.hypot(p[0]-points[i][0],p[1]-points[i][1])),total=lengths.reduce((a,b)=>a+b,0);let covered=Math.max(0,Math.min(1,amount))*total,head=points[0],out=[points[0]];for(let i=0;i<lengths.length;i++){const u=Math.max(0,Math.min(1,covered/(lengths[i]||1)));if(covered>0)head=[mix(points[i][0],points[i+1][0],u),mix(points[i][1],points[i+1][1],u)];out.push(head);covered-=lengths[i];}return {d:'M'+out[0]+out.slice(1).map(p=>' L'+p).join(''),head};}
export const tracedInk=(name,points,progress)=>ink(name,(t,s,r)=>trace(points,progress(t,s,r)).d,{visibility:(t,s,r)=>progress(t,s,r)>1e-5?1:0});
