// Assembly only. All silhouettes and directed motion belong to this set.
export * from '../../library/ai-authoring.mjs';
import {make,rigid,timePair,framed,rect,holes,drive} from '../../library/ai-authoring.mjs';
export const well=(name,x,y,w,h,r=2)=>framed(name,rect(x,y,w,h,r),holes(rect(x-1.3,y-1.3,w+2.6,h+2.6,r+1.3),rect(x+1.3,y+1.3,w-2.6,h-2.6,Math.max(.3,r-1.3))));
export function finish(name,label,intent,parts,light,strong,states){
 const icon=make(name,label,intent,parts,light,strong);
 icon.duotoneFill=true;
 icon.revision='models-providers-v1';
 // Explicit pivot for every named part; moving rigid parts supply their own.
 for(const p of icon.parts)if(!p.pose)p.pose=rigid(()=>({pivot:[16,16]}));
 icon.constraints=icon.parts.filter(p=>p.style==='outline'&&typeof p.d==='string'&&p.pose.transform).map(p=>({type:'rigid',part:p.name}));
 if(states)Object.assign(icon,{stateful:true,initialState:states[0],finalState:states[1],reverseSuffix:'-reverse',forwardLabel:states[1],reverseLabel:states[0],reverseTimings:timePair(light,strong)});
 return icon;
}
// A true reverse has its own drive, with exact common endpoints.
export const setting=(light,strong,backLight,backStrong)=>(t,s,r)=>drive(t,s,r?backLight:light,r?backStrong:strong);
export const progress=(l=.64,h=.42)=>(t,s,r)=>{
 const p=drive(t,s,[[0,0],[.04,0],[l,1,'smooth']],[[0,0],[.08,0,'hold'],[h,1]]);
 return r?1-p:p;
};
