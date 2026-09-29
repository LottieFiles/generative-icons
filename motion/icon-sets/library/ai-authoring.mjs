// Compact authoring vocabulary for AI sets. It supplies assembly, not silhouettes.
// Claude's per-icon drawings and motion tracks live in each set's batch modules.
import {rigid} from './drawing.mjs';
import {motion,drive,timePair,resultAction} from './direction.mjs';
export * from './drawing.mjs';
export * from './direction.mjs';
export const P=(light,strong)=>(t,s)=>motion(t,s,light,strong);
export const F=(light,strong)=>(t,s)=>drive(t,s,light,strong);
export const posed=(parts,pose)=>parts.map(p=>({...p,pose}));
export const shown=(parts,visibility)=>parts.map(p=>({...p,visibility}));
export const segment=(a,b)=>`M${a} L${b}`;
export function make(name,label,intent,parts,light,strong,rearm=[]){
 for(const p of parts)p.name=name+'-'+p.name;
 const changing=p=>rearm.some(n=>p.name===name+'-'+n||p.name.startsWith(name+'-'+n+'-'));
 // A follower that returns to rest must not replay inside the reset hold.
 if(rearm.length)for(const p of parts.filter(p=>!changing(p))){
  if(typeof p.d==='function'){const fn=p.d;p.d=(t,s,r,w)=>fn(r?0:t,s,false,w);}
  if(p.pose){const fn=p.pose;if(fn.transform){const tr=fn.transform;p.pose=rigid((t,s,r)=>tr(r?0:t,s,false));}else p.pose=(x,y,t,s,r)=>fn(x,y,r?0:t,s,false);}
  if(p.visibility){const fn=p.visibility;p.visibility=(t,s,r)=>fn(r?0:t,s,false);}
 }
 const icon={name,label,heading:label,description:intent,intent,brief:intent,parts,timings:timePair(light,strong)};
 return rearm.length?resultAction(icon,changing):icon;
}
