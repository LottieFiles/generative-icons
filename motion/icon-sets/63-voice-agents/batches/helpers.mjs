// Set-owned assembly. Geometry stays on the shared 32-unit voice grid.
export * from '../../library/ai-authoring.mjs';
import {make, timePair, drive, tube, fill, framed, rect, circle, hole, rigid} from '../../library/ai-authoring.mjs';

export const motion=(t,s,light,strong)=>drive(t,s,light,strong);
export const state=(t,s,r,light=.64,strong=.42)=>r
  ? motion(t,s,[[0,1],[light,0,'smooth']],[[0,1],[.06,1],[strong,0,'hit']])
  : motion(t,s,[[0,0],[light,1,'smooth']],[[0,0],[.06,0],[strong,1,'hit']]);
export const gesture=(t,s,amount=1)=>motion(t,s,
  [[0,0],[.32,amount],[.48,amount],[.9,0]],
  [[0,0],[.07,-amount*.12],[.25,amount*1.6,'hit'],[.38,amount*1.6],[.68,0,'hit'],[.79,amount*.08],[.92,0]]);
export const ring=(name,d,extra={})=>[fill(name+'-tint',d,'tint',extra),...tube(name,d,extra)];
export const disk=(name,x,y,r,extra={})=>framed(name,circle(x,y,r),circle(x,y,r),extra);
export const panel=(name,x,y,w,h,r=2.5,extra={})=>ring(name,rect(x,y,w,h,r),extra);
export function bubbleD(x,y,w,h,tail=2.6){
  const r=Math.min(2.8,h/3);
  return `M${x+r} ${y} L${x+w-r} ${y} C${x+w-.45*r} ${y} ${x+w} ${y+.45*r} ${x+w} ${y+r} L${x+w} ${y+h-r} C${x+w} ${y+h-.45*r} ${x+w-.45*r} ${y+h} ${x+w-r} ${y+h} L${x+5} ${y+h} L${x+2.5} ${y+h+tail} C${x+2.1} ${y+h+tail+.3} ${x+1.6} ${y+h+tail} ${x+1.6} ${y+h+tail-.5} L${x+1.6} ${y+h-.3} C${x+.6} ${y+h-.7} ${x} ${y+h-1.5} ${x} ${y+h-r} L${x} ${y+r} C${x} ${y+.45*r} ${x+.45*r} ${y} ${x+r} ${y} Z`;
}
export const bubble=(name,x,y,w,h,tail=2.6,extra={})=>ring(name,bubbleD(x,y,w,h,tail),extra);
export const dot=(name,x,y,r=1.2,extra={})=>[
  fill(name,circle(x,y,r),'outline',extra),fill(name+'-solid',circle(x,y,r+.15),'solid',extra)
];
export function toggle(name,label,intent,parts,initialState,finalState,light=.64,strong=.42){
  return {...make(name,label,intent,parts,light,strong),stateful:true,initialState,finalState,
    reverseSuffix:'-reverse',forwardLabel:finalState,reverseLabel:initialState,
    reverseTimings:timePair(light,strong)};
}
export function checkD(x,y,w,h,p){
  const a=Math.min(1,p/.35),b=Math.max(0,(p-.35)/.65);
  return `M${x} ${y} L${x+w*.34*a} ${y+h*.4*a} L${x+w*.34+(w*.66)*b} ${y+h*.4-h*b}`;
}
// Independent strokes meet at one vertex once the first check leg is complete.
// Before contact, the unused leg collapses exactly onto the moving pen tip.
export function checkPath(x,y,w,h,p){
  const a=Math.min(1,p/.35),b=Math.max(0,(p-.35)/.65);
  const elbow=[x+w*.34*a,y+h*.4*a];
  return `M${x} ${y} L${elbow} L${elbow[0]+w*.66*b} ${elbow[1]-h*b}`;
}
export const receiver='M6.5 21 C8.8 18.3 12.1 17 16 17 C19.9 17 23.2 18.3 25.5 21 C26.1 21.7 26.1 22.6 25.6 23.3 L24.4 25 C24 25.6 23.2 25.8 22.6 25.4 L19.8 23.7 C19.3 23.4 19.1 22.8 19.2 22.2 L19.5 20.9 C17.2 20.1 14.8 20.1 12.5 20.9 L12.8 22.2 C12.9 22.8 12.7 23.4 12.2 23.7 L9.4 25.4 C8.8 25.8 8 25.6 7.6 25 L6.4 23.3 C5.9 22.6 5.9 21.7 6.5 21 Z';
export function finalize(icon){
  icon.duotoneFill=icon.parts.some(p=>p.style==='tint');
  icon.revision='voice-agents-v1';
  for(const p of icon.parts){p.pivot=p.pose?.transform?.(0,false,false)?.pivot||[16,16]; if(!p.pose)p.pose=rigid(()=>({pivot:p.pivot}));}
  return icon;
}
