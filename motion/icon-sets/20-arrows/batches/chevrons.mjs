import{make,tube,rigid,curve}from'./helpers.mjs';
export const chevronKeys={light:[[0,0],[.22,2.2,'out'],[.28,2.2],[.62,0]],strong:[[0,0],[.1,-1],[.26,4.6],[.4,-.5],[.56,.15],[.68,0]]};
const splay=(t,s)=>curve(t,s,[[0,0],[.26,.4,'out'],[.32,.4],[.62,0]],[[0,0],[.12,-.25],[.3,1],[.46,-.2],[.6,.05],[.68,0]]);
export const chevrons=[['chevron-right','Chevron right',0],['chevron-left','Chevron left',180],['chevron-up','Chevron up',270],['chevron-down','Chevron down',90]].map(([name,label,angle])=>{
 const icon=make(name,label,'The apex leads a directional nudge while the legs flex behind it. Strong prepares, drives farther, then gives a short damped return.',tube('vee',(t,s)=>{const x=curve(t,s,chevronKeys.light,chevronKeys.strong),v=splay(t,s);return`M${12+x} ${8-v} L${20+x} 16 L${12+x} ${24+v}`;},{pose:rigid(()=>({angle,pivot:[16,16]}))}),.62,.68);icon.duotoneFill=false;icon.directionAngle=angle;return icon;
});
