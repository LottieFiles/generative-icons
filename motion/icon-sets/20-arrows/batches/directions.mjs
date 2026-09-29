import {make,stroke,fill,capsule,polygon,rigid,curve} from './helpers.mjs';
export const axialKeys={light:[[0,0],[.18,2,'out'],[.5,0]],strong:[[0,0],[.1,-1.5],[.28,4.5],[.42,-.6],[.58,.2],[.72,0]]};
const local=(angle,driver)=>rigid((t,s)=>{const u=driver(t,s),a=angle*Math.PI/180;return{angle,pivot:[16,16],dx:u*Math.cos(a),dy:u*Math.sin(a)};});
export const directions=[['arrow-right','Arrow right',0],['arrow-left','Arrow left',180],['arrow-up','Arrow up',-90],['arrow-down','Arrow down',90],['arrow-up-right','Arrow up right',-45],['arrow-up-left','Arrow up left',-135],['arrow-down-right','Arrow down right',45],['arrow-down-left','Arrow down left',135]].map(([name,label,angle])=>{
 const pose=local(angle,(t,s)=>curve(t,s,axialKeys.light,axialKeys.strong));
 const icon=make(name,label,'The arrow advances along its pointing axis, then settles home. Strong pulls back, drives farther and faster, and gives one damped return.',[
 stroke('shaft','M7 16 L24 16',{pose}),stroke('head','M17.5 9.5 L24 16 L17.5 22.5',{pose}),
 fill('shaft-solid',capsule([7,16],[19,16],2.6),'solid',{pose}),fill('head-solid',polygon([[25.8,16],[16.4,7.4],[16.4,24.6]],[1.2,.8,.8]),'solid',{pose})
 ],.5,.72);
 icon.duotoneFill=false;icon.directionAngle=angle;icon.constraints=[{type:'rigid',part:name+'-head'},{type:'rigid',part:name+'-shaft'},{type:'opaque',part:name+'-head'}];return icon;
});
