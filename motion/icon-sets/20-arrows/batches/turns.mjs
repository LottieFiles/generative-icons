import{make,tube,stroke,fill,polygon,rigid,curve}from'./helpers.mjs';
const head='M-4.5 4.5 L0 0 L4.5 4.5';
const solidHead=polygon([[0,0],[5.4,5.4],[3.56,7.24],[0,3.68],[-3.56,7.24],[-5.4,5.4]],[1,1.3,1.3,1,1.3,1.3]);
export const turnKeys={light:[[0,25],[.18,23.5],[.58,25,'out']],strong:[[0,25],[.16,21.5],[.4,27],[.68,25]]};
export const uturnKeys={light:[[0,23],[.18,21.6],[.6,23,'out']],strong:[[0,23],[.18,20.5],[.44,25.2],[.74,23]]};
const turns=[['arrow-turn-right','Turn right',1],['arrow-turn-left','Turn left',-1]].map(([name,label,sign])=>{
 const x=(t,s)=>{const v=curve(t,s,turnKeys.light,turnKeys.strong);return sign===1?v:32-v;};
 const pose=rigid((t,s)=>({angle:sign*90,pivot:[0,0],dx:x(t,s),dy:8}));
 const icon=make(name,label,'The stem stays planted while the outlet prepares and completes the turn. Strong pulls back farther, drives beyond rest and settles once.',[
 ...tube('stem-arc',sign===1?'M7 26 L7 15 C7 11.134 10.134 8 14 8':'M25 26 L25 15 C25 11.134 21.866 8 18 8'),
 stroke('outlet',(t,s)=>`M${sign===1?14:18} 8 L${x(t,s)} 8`),{name:'outlet-solid',d:(t,s)=>`M${sign===1?14:18} 8 L${x(t,s)-2*sign} 8`,style:'solid',type:'stroke',width:2.6},
 stroke('head',head,{pose}),fill('head-solid',solidHead,'solid',{pose})
 ],.58,.68);icon.duotoneFill=false;icon.constraints=[{type:'fixed',part:name+'-stem-arc'},{type:'rigid',part:name+'-head'},{type:'opaque',part:name+'-head'}];return icon;
});
const y=(t,s)=>curve(t,s,uturnKeys.light,uturnKeys.strong),pose=rigid((t,s)=>({angle:180,pivot:[0,0],dx:10,dy:y(t,s)}));
const uturn=make('arrow-u-turn','U-turn','A planted U bend feeds a downward outlet. Strong draws the head back, completes a longer drop, then settles at the turn endpoint.',[
 ...tube('stem-arc','M22 25 L22 13 C22 9.686 19.314 7 16 7 C12.686 7 10 9.686 10 13'),stroke('outlet',(t,s)=>`M10 13 L10 ${y(t,s)}`),{name:'outlet-solid',d:(t,s)=>`M10 13 L10 ${y(t,s)-2}`,style:'solid',type:'stroke',width:2.6},stroke('head',head,{pose}),fill('head-solid',solidHead,'solid',{pose})
],.6,.74);uturn.duotoneFill=false;uturn.constraints=[{type:'fixed',part:'arrow-u-turn-stem-arc'},{type:'rigid',part:'arrow-u-turn-head'}];
export const turnIcons=[...turns,uturn];
