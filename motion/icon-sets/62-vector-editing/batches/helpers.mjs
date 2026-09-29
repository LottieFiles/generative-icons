// Set-owned assembly. Geometry and action plans live in the four drawing batches.
export * from '../../library/ai-authoring.mjs';
import {make,F,timePair,rigid,posed,framed,rect,holes} from '../../library/ai-authoring.mjs';
export const still=(parts,pivot=[16,16])=>posed(parts,rigid(()=>({pivot})));
export const ringRect=(x,y,w,h,r=1.6,b=2.6)=>holes(rect(x-b/2,y-b/2,w+b,h+b,r+b/2),rect(x+b/2,y+b/2,w-b,h-b,Math.max(.2,r-b/2)));
export const box=(name,x,y,w,h,r=1.5)=>framed(name,rect(x,y,w,h,r),ringRect(x,y,w,h,r));
export const keyed=(light,strong,revLight=light,revStrong=strong)=>{
 const f=F(light,strong),back=F(revLight,revStrong);
 return (t,s,r=false)=>r?1-back(t,s):f(t,s);
};
export const progress=keyed([[0,0],[.06,0],[.74,1,'smooth']],[[0,0],[.1,0,'hold'],[.4,1],[.64,1]],[[0,0],[.06,0],[.66,1,'smooth']],[[0,0],[.08,0],[.42,1],[.58,1]]);
export const kick=F([[0,0],[.13,.35],[.45,0],[.74,0]],[[0,0],[.1,1],[.32,0],[.64,0]]);
export function icon(name,label,intent,parts,{state,light=.9,strong=.9,fixed=[],rigidParts=[]}={}){
 for(const p of parts){p.pivot=p.pivot||[16,16];if(!p.pose)p.pose=rigid(()=>({pivot:p.pivot}));}
 const i=make(name,label,intent,parts,light,strong);
 i.duotoneFill=parts.some(p=>p.style==='tint');
 i.constraints=[...fixed.map(n=>({type:'fixed',part:name+'-'+n})),...rigidParts.map(n=>({type:'rigid',part:name+'-'+n}))];
 if(state)Object.assign(i,{stateful:true,initialState:state[0],finalState:state[1],reverseSuffix:'-reverse',forwardLabel:state[1],reverseLabel:state[0],reverseTimings:timePair(.9,.9)});
 return i;
}
export const diamond=(x,y,r)=>`M${x} ${y-r} L${x+r} ${y} L${x} ${y+r} L${x-r} ${y} Z`;
export const node=(x,y,r=1.4)=>rect(x-r,y-r,2*r,2*r,.45);
