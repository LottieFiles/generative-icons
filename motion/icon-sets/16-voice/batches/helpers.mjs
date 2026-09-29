// Common assembly for the reviewed Claude voice drawings. No shared artwork edits.
export * from '../../library/ai-authoring.mjs';
import {make,timePair,drive} from '../../library/ai-authoring.mjs';
export const TAU=Math.PI*2,cycle=t=>t/2.4,mod=x=>((x%1)+1)%1;
export function continuous(name,label,intent,parts,duotoneFill=false){
 const i=make(name,label,intent,parts,2.4,2.4);
 return {...i,continuous:true,stateful:false,duotoneFill,timings:Object.fromEntries(['light','strong'].map(m=>[m,{active:2.4,rest:0,keyPoses:[0,.2,.4,.7,1.1,1.5,1.9,2.3]}]))};
}
export function toggle(name,label,intent,parts,light,strong,initialState,finalState,reverseLight=light,reverseStrong=strong){
 return {...make(name,label,intent,parts,light,strong),stateful:true,initialState,finalState,reverseSuffix:'-reverse',forwardLabel:finalState,reverseLabel:initialState,reverseTimings:timePair(reverseLight,reverseStrong)};
}
export const curve=(t,s,l,h)=>drive(t,s,l,h);
export const field=(x,baseline,height)=>`M${x} ${baseline} L${x} ${baseline-height}`;
