// Set-local chat construction. Shared helpers provide paint and native rigid transforms.
export * from '../../library/ai-authoring.mjs';
import {make,tube,fill,rigid,posed,F,track,timePair} from '../../library/ai-authoring.mjs';
export const bubble=(x=5,y=5,w=22,h=14)=>`M${x+3} ${y} L${x+w-3} ${y} C${x+w-1.3} ${y} ${x+w} ${y+1.3} ${x+w} ${y+3} L${x+w} ${y+h-3} C${x+w} ${y+h-1.3} ${x+w-1.3} ${y+h} ${x+w-3} ${y+h} L${x+7} ${y+h} L${x+2.6} ${y+h+3} C${x+2} ${y+h+3.4} ${x+1.5} ${y+h+3} ${x+1.7} ${y+h+2.3} L${x+2.2} ${y+h} C${x+.9} ${y+h-.3} ${x} ${y+h-1.5} ${x} ${y+h-3} L${x} ${y+3} C${x} ${y+1.3} ${x+1.3} ${y} ${x+3} ${y} Z`;
// A solid border keeps interior marks visible; tint is the real enclosed chat surface.
export const surface=(n,d,extra={})=>[fill(n+'-interior',d,'tint',extra),...tube(n,d,extra)];
export const fixed=parts=>posed(parts,rigid(()=>({pivot:[16,16]})));
export function icon(name,label,intent,parts,light=.8,strong=.72,states){
 const i=make(name,label,intent,parts.map(p=>p.pose?p:{...p,pose:rigid(()=>({pivot:[16,16]}))}),light,strong);
 i.duotoneFill=parts.some(p=>p.style==='tint');i.revision='chat-v1';
 if(states)Object.assign(i,{stateful:true,initialState:states[0],finalState:states[1],reverseSuffix:'-reverse',forwardLabel:states[1],reverseLabel:states[0],reverseTimings:timePair(light,strong)});
 return i;
}
// True reverse with its own short hesitation and directed return. Endpoints are profile-independent.
export const state=(t,s,r,delay=0)=>r?track(s?[[0,1],[.06+delay,1],[.44+delay,0,'smooth']]:[[0,1],[.04+delay,1],[.64+delay,0,'smooth']],t):track(s?[[0,0],[.1+delay,0],[.4+delay,1,'out']]:[[0,0],[.04+delay,0],[.68+delay,1,'smooth']],t);
export const appear=(parts,dx=0,dy=0,pivot=[16,16])=>parts.map(p=>({...p,pose:rigid((t,s,r)=>{const u=state(t,s,r);return {dx:dx*(1-u)*(s?1.5:1),dy:dy*(1-u)*(s?1.5:1),pivot};}),visibility:(t,s,r)=>state(t,s,r)}));
export const gesture=(l,h)=>F([[0,0],[.2,l],[.65,0,'smooth']],[[0,0],[.08,-h*.16],[.28,h],[.47,-h*.16],[.72,0]]);
