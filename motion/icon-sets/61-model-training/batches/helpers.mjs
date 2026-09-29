// Set-owned assembly. Geometry stays on the family 32-unit construction grid.
export * from '../../library/ai-authoring.mjs';
import {make,drive,timePair,framed,rect,holes,rigid,polygon} from '../../library/ai-authoring.mjs';

export const stateTrack=(light,strong,backLight,backStrong)=>(t,s,r)=>drive(t,s,r?backLight:light,r?backStrong:strong);
export function state(name,label,intent,parts,l,s,from,to,rl,rs){
 return {...make(name,label,intent,parts,l,s),stateful:true,initialState:from,finalState:to,
  reverseSuffix:'-reverse',forwardLabel:to,reverseLabel:from,reverseTimings:timePair(rl,rs)};
}
export const panel=(name,x,y,w,h,r=1.6)=>framed(name,rect(x,y,w,h,r),
 holes(rect(x-1.3,y-1.3,w+2.6,h+2.6,r+1.3),rect(x+1.3,y+1.3,w-2.6,h-2.6,Math.max(.3,r-1.3))));
export const anchor=(parts,pivot,fn=()=>({}))=>parts.map(p=>({...p,pivot,pose:rigid((t,s,r)=>({pivot,...fn(t,s,r)}))}));
export const star=(cx,cy,r,inner)=>polygon(Array.from({length:10},(_,i)=>{
 const a=(-90+i*36)*Math.PI/180,q=i%2?inner:r;return[cx+q*Math.cos(a),cy+q*Math.sin(a)];}),.65);
// A rounded open polyline with fixed topology, including when all points are collinear.
export function roundedLine(points,r=.65){
 let d='M'+points[0];
 for(let i=1;i<points.length-1;i++){
  const a=points[i-1],v=points[i],b=points[i+1],step=p=>{const l=Math.hypot(p[0]-v[0],p[1]-v[1]);const q=l?Math.min(r,l*.4)/l:0;return [v[0]+(p[0]-v[0])*q,v[1]+(p[1]-v[1])*q];};
  const u=step(a),w=step(b);d+=' L'+u+' C'+[u[0]+(v[0]-u[0])*2/3,u[1]+(v[1]-u[1])*2/3,w[0]+(v[0]-w[0])*2/3,w[1]+(v[1]-w[1])*2/3,...w];
 }
 return d+' L'+points.at(-1);
}
export function finish(icon){
 icon.duotoneFill=true;
 for(const part of icon.parts){part.pivot??=[16,16];}
 icon.pivots=Object.fromEntries(icon.parts.map(p=>[p.name,p.pivot]));
 return icon;
}
