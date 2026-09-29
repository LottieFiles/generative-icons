import {finish,well,F,setting,framed,tube,annulus,circle,rect,rigid,posed,bothFill,holes,fill,stroke} from './helpers.mjs';
const priceLift=F([[0,0],[.22,-1.4],[.6,0,'smooth']],[[0,0],[.09,.6],[.27,-4],[.43,.5,'hit'],[.61,0]]);
const costEstimate=finish('cost-estimate','Cost estimate','Estimate request cost: a generic value coin lifts beside a fixed estimate slip. Light lifts 1.4 units; Strong dips, lifts 4 and lands with one catch. The approximate-value lines remain on the receipt.',[
 ...well('estimate-slip',5,6,11,21,1.8),...tube('estimate-a','M8.5 12 C10 10.5 11 13.5 12.5 12'),...tube('estimate-b','M8.5 18 C10 16.5 11 19.5 12.5 18'),
 ...posed([...framed('coin',circle(24,17,4.2),annulus(24,17,4.2,2.6)),...bothFill('value',circle(24,17,1.1))],rigid((t,s)=>({dy:priceLift(t,s),pivot:[24,17]})))
],.6,.61);

const temp=setting([[0,-60],[.06,-60],[.72,60,'smooth']],[[0,-60],[.1,-73],[.36,67],[.55,60]],[[0,60],[.06,60],[.72,-60,'smooth']],[[0,60],[.1,73],[.36,-67],[.55,-60]]);
const temperatureDial=finish('temperature-dial','Temperature dial','Set sampling temperature with a rotary knob. Light turns from minus 60 to plus 60 degrees; Strong prepares 13 degrees, turns decisively and settles 7 degrees back. Reverse returns to the low setting. Outer setting marks remain fixed.',[
 ...framed('knob',circle(16,17,8),annulus(16,17,8,2.6)),
 ...posed(tube('indicator','M16 17 L16 12.8'),rigid((t,s,r)=>({angle:temp(t,s,r),pivot:[16,17]}))),
 ...tube('low-mark','M5 22 L6 21.5'),...tube('mid-mark','M16 4 L16 5'),...tube('high-mark','M26 10 L27 9.5')
],.72,.55,['low temperature','high temperature']);

// Probability density has fixed cubic topology. The cutoff's top is evaluated
// on that same boundary, so changing probability cannot detach the marker.
const topCurve='M5 25 C9 25 10 7 16 7 C22 7 23 25 27 25 L5 25 Z';
const quantile=F([[0,.7],[.3,.48],[.72,.7,'smooth']],[[0,.7],[.07,.73],[.28,.24],[.5,.28],[.83,.7]]);
const cubic=(a,b,c,d,u)=>a*(1-u)**3+3*b*(1-u)**2*u+3*c*(1-u)*u*u+d*u**3;
const cutoff=(t,s)=>{const q=quantile(t,s),left=q<=.5,u=left?q*2:(q-.5)*2;return [cubic(...(left?[5,9,10,16]:[16,22,23,27]),u),cubic(...(left?[25,25,7,7]:[7,7,25,25]),u)];};
const topP=finish('top-p','Top P','Inspect the nucleus sampling cutoff on a probability distribution. Light moves the boundary from the 70 percent position to 48 percent; Strong prepares then sweeps to 24 percent and returns. The marker stays attached to the density curve and baseline.',[
 // Solid is a complete contour ring, leaving the cutoff legible inside.
 fill('density-tint',topCurve,'tint'),stroke('density',topCurve),{name:'density-solid',d:topCurve,style:'solid',type:'stroke',width:2.6},
 ...tube('cutoff',(t,s)=>{const [x,y]=cutoff(t,s);return `M${x} ${y} L${x} 25`;})
],.72,.83);

const limit=setting([[0,0],[.06,0],[.68,-5,'smooth']],[[0,0],[.08,.5],[.34,-5.5],[.5,-5]],[[0,-5],[.06,-5],[.68,0,'smooth']],[[0,-5],[.08,-5.5],[.34,.5],[.5,0]]);
const maxTokens=finish('max-tokens','Max tokens','Set a hard output length: the end bracket moves 5 units toward the fixed output rows. Light slides directly; Strong prepares half a unit and catches half a unit beyond. Reverse expands the output allowance.',[
 ...framed('first-token',rect(5,8,10,4,2)),...tube('output-second','M5 18 L16.5 18'),...tube('output-third','M5 24 L12 24'),
 ...posed(tube('limit','M23 5 L26 5 L26 27 L23 27'),rigid((t,s,r)=>({dx:limit(t,s,r),pivot:[26,16]})))
],.68,.5,['large maximum','small maximum']);

const lock=setting([[0,30],[.06,30],[.68,0,'smooth']],[[0,30],[.1,37],[.34,0,'hit'],[.51,0]],[[0,0],[.06,0],[.68,30,'smooth']],[[0,0],[.1,-4],[.33,35],[.51,30]]);
const die=rect(6,15,20,12,2.6),pips=[[11,19],[16,21],[21,23]];
const seedLock=finish('seed-lock','Seed lock','Lock a repeatable random seed: a shackle closes onto a die with three permanent pips. Light closes 30 degrees; Strong opens seven degrees farther before a firm latch. Reverse unlocks with one bounded catch.',[
 ...framed('die',die,holes(die,...pips.map(p=>circle(...p,1.4)))),...pips.map((p,i)=>fill('pip-'+i,circle(...p,1.2),'outline')),
 ...posed(tube('shackle','M11 15 L11 11.2 C11 8.9 13.2 7 16 7 C18.8 7 21 8.9 21 11.2 L21 15'),rigid((t,s,r)=>({angle:lock(t,s,r),pivot:[21,15]})))
],.68,.51,['variable seed','locked seed']);
export const icons=[costEstimate,temperatureDial,topP,maxTokens,seedLock];
