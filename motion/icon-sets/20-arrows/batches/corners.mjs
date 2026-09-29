import{make,stroke,fill,capsule,polygon,rigid,curve}from'./helpers.mjs';
export const expandKeys={light:[[0,0],[.22,1],[.66,0,'out']],strong:[[0,0],[.14,-.85],[.4,2.5],[.72,0]]};
export const collapseKeys={light:[[0,0],[.22,.8],[.66,0,'out']],strong:[[0,0],[.14,-.9],[.38,1.8],[.7,0]]};
const solidHead=polygon([[-1.3,-1.3],[6.3,-1.3],[6.3,1.3],[1.3,1.3],[1.3,6.3],[-1.3,6.3]],[1.3,1.3,1.3,.9,1.3,1.3]);
export const cornerIcons=[false,true].map(collapse=>{
 const name=collapse?'arrows-collapse':'arrows-expand',keys=collapse?collapseKeys:expandKeys,parts=[];
 for(let i=0;i<4;i++){
  const angle=i*90,a=angle*Math.PI/180,center=collapse?12:8,tip=[16+(center-16)*(Math.cos(a)-Math.sin(a)),16+(center-16)*(Math.sin(a)+Math.cos(a))];
  const pose=rigid((t,s)=>{const d=curve(t,s,keys.light,keys.strong)*(collapse?1:-1);return{angle:angle+(collapse?180:0),pivot:[0,0],dx:tip[0]+d*(Math.cos(a)-Math.sin(a)),dy:tip[1]+d*(Math.sin(a)+Math.cos(a))};});
  parts.push(stroke('corner-'+i+'-head','M5 0 L0 0 L0 5',{pose}),stroke('corner-'+i+'-shaft','M0 0 L5 5',{pose}),fill('corner-'+i+'-head-solid',solidHead,'solid',{pose}),fill('corner-'+i+'-shaft-solid',capsule([.8,.8],[5,5],2.6),'solid',{pose}));
 }
 const icon=make(name,collapse?'Collapse arrows':'Expand arrows',collapse?'Four inward heads pull toward the center. Strong prepares outward and makes a quick larger contraction before returning.':'Four corner arrows open outward. Strong pulls inward first, drives farther along the diagonals, then settles home.',parts,.66,collapse?.7:.72);icon.duotoneFill=false;return icon;
});
