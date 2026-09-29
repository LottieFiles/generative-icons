// Exact de Casteljau geometry with arc-length lookup for a native vector write-on.
// Every open path retains its complete cubic command topology throughout reveal.
import svgpath from 'svgpath';
import {make,timePair,tube,bothFill,circle,curve} from './helpers.mjs';
export {make,timePair,tube,bothFill,circle,curve};
const clamp=x=>Math.max(0,Math.min(1,x)),lerp=(a,b,u)=>a.map((v,i)=>v+(b[i]-v)*u);
export function cubicPoint(q,t){const u=1-t;return[0,1].map(i=>u*u*u*q[0][i]+3*u*u*t*q[1][i]+3*u*t*t*q[2][i]+t*t*t*q[3][i]);}
const cache=new Map();
export function compile(d){
 if(cache.has(d))return cache.get(d);
 const parsed=svgpath(d).abs().unarc().unshort();if(parsed.err)throw Error(parsed.err);
 let start,last;const segments=[];
 for(const [c,...v]of parsed.segments){
  if(c==='M'){if(start)throw Error('One MoveTo per revealed part');start=last=v;}
  else if(c==='C'){segments.push([last,v.slice(0,2),v.slice(2,4),v.slice(4,6)]);last=v.slice(4,6);}
  else if(c==='L'||c==='H'||c==='V'){const end=c==='L'?v:c==='H'?[v[0],last[1]]:[last[0],v[0]];segments.push([last,lerp(last,end,1/3),lerp(last,end,2/3),end]);last=end;}
  else throw Error('Revealed path must be open: '+c);
 }
 if(!segments.length)throw Error('Empty revealed route');
 let total=0;const spans=segments.map(q=>{let length=0,prev=q[0];const lut=[[0,0]];for(let j=1;j<=96;j++){const t=j/96,p=cubicPoint(q,t);length+=Math.hypot(p[0]-prev[0],p[1]-prev[1]);lut.push([t,length]);prev=p;}const offset=total;total+=length;return{q,lut,length,offset};});
 const result={start,end:last,spans,total};cache.set(d,result);return result;
}
export function prefix(d,progress){
 const c=compile(d),length=clamp(progress)*c.total;let pen=c.start,path='M'+pen;
 for(const {q,lut,length:span,offset}of c.spans){
  let t;
  if(length<=offset)t=0;else if(length>=offset+span)t=1;else{const target=length-offset,j=lut.findIndex(v=>v[1]>=target),a=lut[j-1],b=lut[j];t=a[0]+(b[0]-a[0])*(target-a[1])/(b[1]-a[1]);}
  if(t===1){path+=' C'+[...q[1],...q[2],...q[3]];pen=q[3];}
  else if(t===0)path+=' C'+[...pen,...pen,...pen];
  else{const a=lerp(q[0],q[1],t),b=lerp(q[1],q[2],t),cc=lerp(q[2],q[3],t),ab=lerp(a,b,t),bc=lerp(b,cc,t),end=lerp(ab,bc,t);path+=' C'+[...a,...ab,...end];pen=end;}
 }
 return path;
}
export const visible=u=>u<=.0001?0:1;
export function route(name,d,progress,extra={}){return tube(name,(t,s,r)=>prefix(typeof d==='function'?d(t,s,r):d,progress(t,s,r)),{...extra,visibility:(t,s,r)=>visible(progress(t,s,r))});}
export function arrowhead(name,tip,barbA,barbB,progress,extra={}){
 const points=(t,s,r)=>{const h=clamp(progress(t,s,r)),p=typeof tip==='function'?tip(t,s,r):tip,a=typeof barbA==='function'?barbA(t,s,r):barbA,b=typeof barbB==='function'?barbB(t,s,r):barbB;return 'M'+lerp(p,a,h)+' L'+p+' L'+lerp(p,b,h);};
 return tube(name,points,{...extra,visibility:(t,s,r)=>visible(progress(t,s,r))});
}
// The designer supplies drivers for forward and reset; this does not invent motion.
export function annotation(name,label,intent,parts,light,strong,reset=.6,duotoneFill=false){
 return {...make(name,label,intent,parts,light,strong),duotoneFill,stateful:true,initialState:'origin',finalState:'annotated',previewReset:true,reverseSuffix:'-reset',forwardLabel:'Draw annotation',reverseLabel:'Reset preview',reverseTimings:timePair(reset,reset)};
}
