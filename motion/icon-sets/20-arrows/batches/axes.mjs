import {make,tube,rigid,curve} from './helpers.mjs';
export const stretchKeys={light:[[0,0],[.28,1.5,'out'],[.7,0]],strong:[[0,0],[.12,-1],[.32,3.5,'launch'],[.52,-.5],[.72,0]]};
export const swapKeys={light:[[0,0],[.26,1.8,'out'],[.66,0]],strong:[[0,0],[.12,-1],[.32,4,'launch'],[.52,-.4],[.72,0]]};
const span=(t,s)=>curve(t,s,stretchKeys.light,stretchKeys.strong);
const axes=[['arrows-horizontal','Horizontal arrows',0],['arrows-vertical','Vertical arrows',90]].map(([name,label,angle])=>{
 const a=angle*Math.PI/180,rot=rigid(()=>({angle,pivot:[16,16]}));
 const parts=[...tube('shaft',(t,s)=>`M${7-span(t,s)} 16 L${25+span(t,s)} 16`,{pose:rot})];
 for(const [sign,side,d]of[[-1,'left','M11 12 L7 16 L11 20'],[1,'right','M21 12 L25 16 L21 20']])parts.push(...tube('head-'+side,d,{pose:rigid((t,s)=>({angle,pivot:[16,16],dx:sign*span(t,s)*Math.cos(a),dy:sign*span(t,s)*Math.sin(a)}))}));
 const icon=make(name,label,'Both heads pull outward and the connecting shaft follows. Strong prepares inward, opens wider with a quick drive, then relaxes.',parts,.7,.72);icon.duotoneFill=false;icon.directionAngle=angle;return icon;
});
const swapParts=[];
for(const [name,sign,shaft,head]of[['top',1,'M7 10 L24 10','M20 6 L24 10 L20 14'],['bottom',-1,'M25 22 L8 22','M12 18 L8 22 L12 26']]){
 const pose=rigid((t,s)=>({dx:sign*curve(t,s,swapKeys.light,swapKeys.strong)}));swapParts.push(...tube(name+'-shaft',shaft,{pose}),...tube(name+'-head',head,{pose}));
}
const swap=make('arrows-swap','Swap arrows','Two arrows drive in opposite directions on separate lanes. Strong winds back, travels farther and returns with a short response.',swapParts,.66,.72);swap.duotoneFill=false;
export const axisIcons=[...axes,swap];
