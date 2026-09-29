// Education batch B: notebook, pencil, ruler, lab-flask, microscope.
import {make,F,tube,framed,rigid,posed,shown,circle,polygon,rect,rotate,bothFill,capsule,fill,stroke} from './helpers.mjs';
import {container,pen,reveal,lerp} from './edu.mjs';

// ---------- NOTEBOOK ----------
// Spiral-bound page (binding bars cross the left edge). Notes are written line by line. Result retained.
const LINES=[[10,22.5],[14,22.5],[18,20]];
const lineK=k=>F([[0,0],[.06+k*.2,0],[.26+k*.2,1,'smooth']],[[0,0],[.1+k*.1,0,'hold'],[.19+k*.1,1]]);
const nudge=F([[0,0],[.8,0]],[[0,0],[.06,-.8],[.14,0,'hit'],[.42,0],[.46,.5,'hit'],[.56,0]]);
const book=rigid((t,s)=>({dy:nudge(t,s)}));
const notebook=make('notebook','Notebook',
 'Light: three lines of notes are written one after another. Strong: the notebook is picked up and set down, the lines are dashed off in 0.09 s each, and it taps down when done. The notes are retained; preview reset rearms.',[
 ...posed(container('page',rect(9,4.5,17,23,2)),book),
 ...[8.5,12,15.5,19,22.5].flatMap((y,i)=>posed(tube('ring-'+i,`M6.2 ${y} L11 ${y}`),book)),
 ...LINES.flatMap(([y,end],k)=>posed(shown(pen('line-'+k,(t,s)=>`M14.6 ${y} L${14.6+(end-14.6)*Math.max(.002,lineK(k)(t,s))} ${y}`),(t,s)=>lineK(k)(t,s)>.01?1:0),book))
],.72,.6,['line-0','line-0-solid','line-1','line-1-solid','line-2','line-2-solid']);
notebook.duotoneFill=true;

// ---------- PENCIL ----------
// A pencil at 60° writes a wave; its tip rides the same advancing curve position as the ink.
const A=[.5,-.866],N=[.866,.5];
const ink=u=>[5.4+11.6*u,24.2-1.8*Math.sin(2*Math.PI*1.5*u)];
const w=F([[0,0],[.08,0],[.72,1,'smooth']],[[0,0],[.14,0,'hold'],[.44,1]]);
const hover=F([[0,0],[.8,0]],[[0,0],[.08,-1.2],[.14,0,'hit'],[.44,0],[.52,-1],[.64,0]]);
const L=(u,v)=>[u*A[0]+v*N[0],u*A[1]+v*N[1]];
const penBody=polygon([L(0,0),L(3.4,-2.1),L(13,-2.1),L(13,2.1),L(3.4,2.1)],[.4,.5,1,1,.5]);
const penAt=(x,y,t,s)=>{const p=ink(Math.max(0,w(t,s)));return[x+p[0],y+p[1]+hover(t,s)];};
const pencil=make('pencil','Pencil',
 'Light: the pencil writes a wave line, its tip riding the ink as it grows. Strong: the pencil lifts and drops onto the page, writes in 0.3 s and flicks up at the end. The line is retained; preview reset rearms.',[
 ...shown(pen('ink',reveal(ink,w,4)),(t,s)=>w(t,s)>.01?1:0),
 ...posed(framed('body',penBody),penAt),
 ...posed([stroke('cone',`M${L(3.4,-2.1)} L${L(3.4,2.1)}`),stroke('ferrule',`M${L(10.6,-2.1)} L${L(10.6,2.1)}`)],penAt)
],.72,.64,['ink','ink-solid','body-interior','body-frame','body-solid','cone','ferrule']);
pencil.duotoneFill=true;

// ---------- RULER ----------
// A measuring pulse travels along the ticks: each tick extends and returns in turn. Strong also rocks the ruler.
const TICKS=[[9,4.2],[12.5,2.4],[16,4.2],[19.5,2.4],[23,4.2]];
const ext=(t,s,i)=>{const u=t-(s?.08:.04)-i*(s?.05:.07),T=s?.26:.32;if(u<=0||u>=T)return 0;return (s?1.5:.6)*Math.sin(Math.PI*u/T);};
const rock=F([[0,0],[.8,0]],[[0,0],[.06,-1.5],[.3,2.5],[.48,-1],[.62,0]]);
const tilt=rigid((t,s)=>({angle:rock(t,s),pivot:[16,16.25]}));
const ruler=make('ruler','Ruler',
 'Light: a measuring pulse runs along the ticks, each lengthening 0.6 units in turn. Strong: the pulse runs faster and deeper (1.5 units) while the ruler rocks 2.5° and settles.',[
 ...posed(container('rule',rect(5,11,22,10.5,1.6)),tilt),
 ...TICKS.flatMap(([x,l],i)=>posed(pen('tick-'+i,(t,s)=>`M${x} 11 L${x} ${11+l+ext(t,s,i)}`),tilt))
],.6,.62);
ruler.duotoneFill=true;

// ---------- LAB FLASK ----------
// Open-mouthed Erlenmeyer flask with liquid; bubbles rise from the liquid up the neck and fade.
const flaskD='M11.8 5 L12.6 5 C13 5 13.2 5.2 13.2 5.6 L13.2 10.6 C13.2 11.3 13.05 11.9 12.7 12.46 L6.2 23.6 C5.4 25 6.3 26.6 7.9 26.6 L24.1 26.6 C25.7 26.6 26.6 25 25.8 23.6 L19.3 12.46 C18.95 11.9 18.8 11.3 18.8 10.6 L18.8 5.6 C18.8 5.2 19 5 19.4 5 L20.2 5';
const slosh=(t,s)=>{const u=Math.max(0,t-.06),T=s?.7:.6;return u>=T?0:(s?1.2:.4)*Math.sin(u*14)*(1-u/T)**1.5;};
const surf=(t,s)=>{const a=slosh(t,s);return `M8.42 19.8 C13.47 ${19.8-a} 18.53 ${19.8+a} 23.58 19.8`;};
const liquid=(t,s)=>surf(t,s)+' L25.8 23.6 C26.6 25 25.7 26.6 24.1 26.6 L7.9 26.6 C6.3 26.6 5.4 25 6.2 23.6 Z';
const shake=F([[0,0],[.7,0]],[[0,0],[.06,-4],[.18,5],[.3,-3],[.42,1.5],[.54,0]]);
const flaskPose=rigid((t,s)=>({angle:shake(t,s),pivot:[16,26.6]}));
const BUB=[[12.6,23.4,0],[16,24.4,.1],[19.4,23.2,.2]];
const rise=(t,s,d)=>{const T=s?.34:.5,u=(t-(s?.08:.06)-d*(s?.6:1))/T;return Math.max(0,Math.min(1,u));};
const labFlask=make('lab-flask','Lab flask',
 'Light: three bubbles rise from the liquid up the neck and fade while the surface ripples. Strong: the flask is swirled 5° either way, the surface sloshes and the bubbles race up in 0.34 s.',[
 fill('liquid-interior',liquid,'tint',{pose:flaskPose}),fill('liquid-solid',liquid,'solid',{pose:flaskPose}),
 ...posed(pen('flask',flaskD),flaskPose),
 ...posed(tube('surface',surf),flaskPose),
 ...BUB.flatMap(([x,y,d],i)=>shown(posed(bothFill('bubble-'+i,circle(x,y,1.1)),(px,py,t,s)=>{const r=rise(t,s,d),target=[16,9.4],k=Math.min(1,r*1.6),cx=x+(target[0]-x)*k,cy=y+(target[1]-y)*r;return flaskPose(px-x+cx,py-y+cy,t,s);}),(t,s)=>{const r=rise(t,s,d);return r<=0||r>=1?0:Math.min(1,r*6,(1-r)*5);}))
],.86,.84);
labFlask.duotoneFill=true;

// ---------- MICROSCOPE ----------
// Tube on an inclined axis, held by a curved arm whose end rides the tube's side as it racks to focus.
const AX=[.382,.924],NM=[.924,-.382];
const TA=[10.8,7.4],TB=[14.4,16.1];
const tubeD=capsule(TA,TB,4.4);
const objA=[TB[0]+2.2*AX[0],TB[1]+2.2*AX[1]],objB=[TB[0]+3.6*AX[0],TB[1]+3.6*AX[1]];
const mid=lerp(TA,TB,.5),ARM_END=[mid[0]+2.2*NM[0],mid[1]+2.2*NM[1]];
const focus=F([[0,0],[.16,.4],[.44,0,'smooth']],[[0,0],[.08,-.6],[.24,.8,'hit'],[.36,.55],[.5,0]]);
const rack=rigid((t,s)=>({dx:AX[0]*focus(t,s),dy:AX[1]*focus(t,s)}));
const microscope=make('microscope','Microscope',
 'Light: the tube racks down 0.4 units toward the slide and back to focus. Strong: it lifts, drives down 0.8 units with a hard stop above the stage, and eases back.',[
 ...tube('base','M7 27.4 L25 27.4'),
 ...tube('arm',`M21.4 27.4 L21.4 21 C21.4 16.4 19.4 12.6 ${ARM_END}`),
 ...tube('stage','M8.6 24.4 L21.4 24.4'),
 ...bothFill('knob',circle(21.4,18.4,1.9)),
 ...posed([...framed('tube',tubeD),...tube('objective',`M${objA} L${objB}`)],rack)
],.44,.5);
microscope.duotoneFill=true;

export const icons=[notebook,pencil,ruler,labFlask,microscope];
