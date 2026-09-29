// Shared assembly for the Search & discovery set (toggle/continuous copied from 39-food-drink helpers). No artwork.
export * from '../../library/ai-authoring.mjs';
import svgpath from 'svgpath';
import {make,timePair,drive,stroke,fill} from '../../library/ai-authoring.mjs';
export function continuous(name,label,intent,parts,duotoneFill=false){
 const i=make(name,label,intent,parts,2.4,2.4);
 return {...i,continuous:true,stateful:false,duotoneFill,timings:Object.fromEntries(['light','strong'].map(m=>[m,{active:2.4,rest:0,keyPoses:[0,.2,.4,.7,1.1,1.5,1.9,2.3]}]))};
}
export function toggle(name,label,intent,parts,light,strong,initialState,finalState,reverseLight=light,reverseStrong=strong){
 return {...make(name,label,intent,parts,light,strong),stateful:true,initialState,finalState,reverseSuffix:'-reverse',forwardLabel:finalState,reverseLabel:initialState,reverseTimings:timePair(reverseLight,reverseStrong)};
}
// State driver for true two-state icons: forward tracks run 0→1, the authored
// reverse tracks run 1→0, so both actions meet on exactly the same numbers.
export const K=(light,strong,revLight,revStrong)=>(t,s,r)=>r?drive(t,s,revLight,revStrong):drive(t,s,light,strong);
// A follower that only exists in the forward action and is exactly 0 at both ends.
export const FWD=(light,strong)=>(t,s,r)=>r?0:drive(t,s,light,strong);
// An enclosed container: tinted interior, outline stroke and a Solid ring (the
// container holds moving detail, so Solid is a ring rather than a filled disc).
export const ring=(name,d,extra={})=>[fill(name+'-interior',d,'tint',extra),stroke(name,d,extra),{name:name+'-solid',d,style:'solid',type:'stroke',width:2.6,...extra}];
const KAPPA=.552285;
export const ellipse=(cx,cy,rx,ry)=>{const x=rx*KAPPA,y=ry*KAPPA;return `M${cx} ${cy-ry} C${cx+x} ${cy-ry} ${cx+rx} ${cy-y} ${cx+rx} ${cy} C${cx+rx} ${cy+y} ${cx+x} ${cy+ry} ${cx} ${cy+ry} C${cx-x} ${cy+ry} ${cx-rx} ${cy+y} ${cx-rx} ${cy} C${cx-rx} ${cy-y} ${cx-x} ${cy-ry} ${cx} ${cy-ry} Z`;};
// Four-point sparkle with concave sides, filled in every style.
export const sparkle=(cx,cy,R,w=R*.28)=>`M${cx} ${cy-R} C${cx} ${cy-w} ${cx+w} ${cy} ${cx+R} ${cy} C${cx+w} ${cy} ${cx} ${cy+w} ${cx} ${cy+R} C${cx} ${cy+w} ${cx-w} ${cy} ${cx-R} ${cy} C${cx-w} ${cy} ${cx} ${cy-w} ${cx} ${cy-R} Z`;
export const spin=(d,angle,cx=16,cy=16)=>svgpath(d).rotate(angle,cx,cy).toString();
export const place=(d,angle,dx,dy)=>svgpath(d).rotate(angle).translate(dx,dy).toString();
export const deg=a=>a*Math.PI/180;
