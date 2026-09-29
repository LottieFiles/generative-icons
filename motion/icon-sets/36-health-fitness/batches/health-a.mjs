// Health & Fitness batch A: heart-rate, step-counter, dumbbell, running, water-intake.
import {make,F,P,tube,framed,rigid,posed,shown,fill,stroke,circle,bothFill,rotate,rect} from './helpers.mjs';
import svgpath from 'svgpath';
import {trace,lerp,ellipse} from './sky.mjs';
const L=d=>d.replace(/^M/,'L');

// ---------- HEART RATE ----------
// A heart above a pulse trace. The trace is revealed by arc length (constant topology); when the pen
// crosses the spike the heart beats about its bottom tip, so its top never nears the edge and its tip
// never nears the spike (Bold gap 1.1).
const HEART='M16 18.6 C11 15.2 8.5 12.7 8.5 9.8 C8.5 7.5 10.2 5.8 12.4 5.8 C14 5.8 15.3 6.8 16 8.1 C16.7 6.8 18 5.8 19.6 5.8 C21.8 5.8 23.5 7.5 23.5 9.8 C23.5 12.7 21 15.2 16 18.6 Z';
const ECG=[[4.5,25],[12.4,25],[14.4,22.2],[16.8,28],[18.8,25],[27.5,25]];
const pen=F([[0,0],[.06,0],[.7,1,'smooth']],[[0,0],[.06,0,'hold'],[.4,1]]);
const beat=F([[0,1],[.28,1],[.38,1.07],[.52,1,'smooth']],[[0,1],[.17,1],[.23,1.14,'hit'],[.29,1],[.34,1.09],[.46,1]]);
const heartPose=(x,y,t,s)=>{const k=beat(t,s);return[16+(x-16)*k,18.6+(y-18.6)*k];};
const heartRate=make('heart-rate','Heart rate',
 'Light: the pulse trace draws across in 0.64 s and the heart gives one beat as the pen crosses the spike. Strong: the trace draws in 0.34 s and the heart double-beats (lub-dub) to 114%. The trace is retained; preview reset rearms.',[
 ...posed(framed('heart',HEART),heartPose),
 ...shown(tube('trace',(t,s)=>trace(ECG,pen(t,s))),(t,s)=>pen(t,s)>.008?1:0)
],.72,.5,['trace']);
heartRate.duotoneFill=true;

// ---------- STEP COUNTER ----------
// Two bare footprints offset like walking steps (left behind, right ahead). Each print is one closed sole
// outline (heel to ball, inner arch) with three filled toe dots above the ball. Dots keep a Bold painted gap
// ≥1 to the sole and to each other. Parent review: ovals + dots read as eyes; sole + heel pad as "!!".
const PK=.82;
const SOLE='M0 -4.8 C2.4 -4.8 3.3 -2.8 3.1 -.6 C2.9 1.4 2 2.6 2 4.2 C2 6 1.2 7.2 -.2 7.2 C-1.6 7.2 -2.2 6 -2.1 4.4 C-2 2.8 -1.2 1.8 -1.4 .4 C-1.6 -1 -2.9 -1.8 -2.9 -3 C-2.9 -4.2 -1.6 -4.8 0 -4.8 Z';
// Local spacing for the 0.82 scale: toe–sole ≥2.74+r, toe–toe ≥1.22+r1+r2 (Bold painted gap ≥1).
const TOES=[[-1.6,-8.5,.85],[1.3,-9.1,.7],[3.7,-7.9,.6]];
const place=(d,c,a,m)=>svgpath(d).scale(m*PK,PK).rotate(a).translate(...c).toString();
const print=(name,c,a,m)=>[...framed(name+'-sole',place(SOLE,c,a,m)),...TOES.flatMap(([x,y,r],k)=>bothFill(name+'-toe-'+k,place(circle(x,y,r),c,a,m)))];
const lift=(d)=>F([[0,0],[d,0],[d+.16,1],[d+.34,0,'smooth']],[[0,0],[d,0],[d+.06,-.25],[d+.2,1],[d+.32,-.12,'hit'],[d+.42,0]]);
const stepPose=(c,d)=>(x,y,t,s)=>{const k=lift(d)(t,s),sc=1+(s?.05:.03)*k,[rx,ry]=rotate(x,y,(s?5:2)*k,...c);return[c[0]+(rx-c[0])*sc,c[1]+(ry-c[1])*sc-(s?1.4:.9)*k];};
const L1=[10.4,18.4],R1=[21.6,12.6];
const stepCounter=make('step-counter','Step counter',
 'Light: the back footprint lifts and plants, then the front one, each rising 0.9 units. Strong: each foot pushes off, lifts 1.4 units toward the viewer with a twist and lands with a small contact dip.',
 [...posed(print('left',L1,-10,-1),stepPose(L1,.02)),...posed(print('right',R1,10,1),stepPose(R1,.2))],.54,.64);
stepCounter.duotoneFill=true;

// ---------- DUMBBELL ----------
// Centreline plates and bar (open strokes, no tint). Plates 3.5 apart: Bold gap 1.0.
const lifted=F([[0,0],[.08,0],[.3,-1.6],[.56,0,'smooth']],[[0,0],[.1,.8],[.3,-4],[.44,-3.2],[.62,0]]);
const tilt=F([[0,0],[.3,4],[.56,0,'smooth']],[[0,0],[.1,-2],[.3,-8],[.46,3],[.62,0]]);
const dumbbell=make('dumbbell','Dumbbell',
 'Light: the dumbbell is curled up 1.6 units with a 4° tilt and lowered. Strong: it dips as the lifter braces, is driven up 4 units tilting 8°, counter-swings and is set down.',
 posed([...tube('plate-lo','M6 12.6 L6 19.4'),...tube('plate-li','M9.5 10.2 L9.5 21.8'),...tube('bar','M9.5 16 L22.5 16'),...tube('plate-ri','M22.5 10.2 L22.5 21.8'),...tube('plate-ro','M26 12.6 L26 19.4')],
 rigid((t,s)=>({dy:lifted(t,s),angle:tilt(t,s),pivot:[16,16]}))),.56,.62);
dumbbell.duotoneFill=false;

// ---------- RUNNING ----------
// Stick runner. Limbs are two-segment chains driven by joint angles; a stride swaps the front and back
// limbs by interpolating angles (never positions), so limb lengths stay constant.
const HEAD=[19.4,6.4],NECK=[17.4,12],HIP=[14.8,18.4],SH=[16.9,13.2];
const ang=(a,b)=>Math.atan2(b[1]-a[1],b[0]-a[0]);
const chain=(o,a1,l1,a2,l2)=>{const k=[o[0]+l1*Math.cos(a1),o[1]+l1*Math.sin(a1)];return [k,[k[0]+l2*Math.cos(a2),k[1]+l2*Math.sin(a2)]];};
const limb=(o,k,e)=>({a1:ang(o,k),l1:Math.hypot(k[0]-o[0],k[1]-o[1]),a2:ang(k,e),l2:Math.hypot(e[0]-k[0],e[1]-k[1])});
const LEG_F=limb(HIP,[19,20.8],[18.4,26.8]),LEG_B=limb(HIP,[11.8,22.4],[7.2,23.6]);
const ARM_F=limb(SH,[20.4,15],[23,12.6]),ARM_B=limb(SH,[13.8,14.6],[11,17]);
const mixL=(A,B,u)=>({a1:lerp(A.a1,B.a1,u),l1:A.l1,a2:lerp(A.a2,B.a2,u),l2:A.l2});
const stride=F([[0,0],[.06,0],[.36,1],[.66,0,'smooth']],[[0,0],[.05,-.12],[.22,1],[.4,0],[.56,1],[.72,-.06],[.8,0]]);
const bob=F([[0,0],[.21,-.5],[.36,0],[.51,-.5],[.66,0,'smooth']],[[0,0],[.13,-1],[.22,0],[.31,-1],[.4,0],[.48,-1],[.56,0],[.64,-.6],[.8,0]]);
const lean=F([[0,0],[.66,0]],[[0,0],[.12,6],[.64,6],[.8,0]]);
const body=(t,s)=>({dy:bob(t,s),angle:lean(t,s),pivot:[14.8,26.8]});
const limbD=(o,A,B)=>(t,s)=>{const L2=mixL(A,B,stride(t,s)),[k,e]=chain(o,L2.a1,L2.l1,L2.a2,L2.l2);return `M${o} L${k} L${e}`;};
const running=make('running','Running',
 'Light: one easy stride. Legs and arms swing through and back while the body bobs 0.5. Strong: the runner leans 6° into two quick strides with a 1-unit bob.',
 posed([...framed('head',circle(...HEAD,2.1)),...tube('torso',`M${NECK} L${HIP}`),
  ...tube('leg-f',limbD(HIP,LEG_F,LEG_B)),...tube('leg-b',limbD(HIP,LEG_B,LEG_F)),
  ...tube('arm-f',limbD(SH,ARM_F,ARM_B)),...tube('arm-b',limbD(SH,ARM_B,ARM_F))],rigid(body)),.66,.8);
running.duotoneFill=true;

// ---------- WATER INTAKE ----------
// Open glass (no rim line). Water rises with a surface whose ends ride the walls; Strong drops a
// droplet in first, and the splash drives a harder slosh. The level only rises.
const wallX=(y,side)=>side<0?8+.1031*(y-6):24-.1031*(y-6);
const GLASS='M8 6 L10 25.4 C10.1 26.4 10.9 27 11.8 27 L20.2 27 C21.1 27 21.9 26.4 22 25.4 L24 6';
const level=F([[0,22.5],[.06,22.5],[.7,11.4,'smooth']],[[0,22.5],[.24,22.5,'hold'],[.56,11.4]]);
const slosh=(t,s)=>{const u=Math.max(0,t-(s?.24:.06)),T=s?.6:.64;return u>=T?0:(s?1.5:.6)*Math.sin(u*(s?16:11))*Math.exp(-u*4)*(1-u/T)**1.5;};
const surf=(t,s)=>{const y=level(t,s),a=slosh(t,s),l=wallX(y,-1),r=wallX(y,1);return `M${l} ${y} C${lerp(l,r,1/3)} ${y-a} ${lerp(l,r,2/3)} ${y+a} ${r} ${y}`;};
const waterD=(t,s)=>surf(t,s)+' L22 25.4 C21.9 26.4 21.1 27 20.2 27 L11.8 27 C10.9 27 10.1 26.4 10 25.4 Z';
const dropY=F([[0,4.6],[.5,4.6]],[[0,4.6],[.06,4.6],[.24,level(0,true)-2.2,'in']]);
const dropD=(t,s)=>{const y=dropY(t,s);return `M16 ${y-1.6} C16.9 ${y-.4} 17.5 ${y+.3} 17.5 ${y+.9} C17.5 ${y+1.7} 16.8 ${y+2.3} 16 ${y+2.3} C15.2 ${y+2.3} 14.5 ${y+1.7} 14.5 ${y+.9} C14.5 ${y+.3} 15.1 ${y-.4} 16 ${y-1.6} Z`;};
const waterIntake=make('water-intake','Water intake',
 'Light: the water in the glass rises from a quarter to near the rim with a small slosh. Strong: a droplet falls in, and on impact the level rises in 0.32 s with a hard slosh that settles. The level is retained; preview reset rearms.',[
 stroke('glass',GLASS),{name:'glass-solid',d:GLASS,style:'solid',type:'stroke',width:2.6},
 fill('water-interior',waterD,'tint'),fill('water-solid',waterD,'solid'),
 ...tube('surface',surf),
 ...shown(bothFill('drop',dropD),(t,s)=>s&&t>.02&&t<.24?1:0)
],.7,.84,['water-interior','water-solid','surface','drop','drop-solid']);
waterIntake.duotoneFill=true;

export const icons=[heartRate,stepCounter,dumbbell,running,waterIntake];
