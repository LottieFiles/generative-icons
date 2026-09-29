// Health & Fitness batch C: first-aid-kit, bandage, weight-scale, blood-pressure, yoga.
import {make,F,tube,framed,rigid,posed,shown,fill,stroke,circle,bothFill,rotate,holes,rect,polygon,annulus} from './helpers.mjs';
import {arcN,curveFrom,lerp,ellipse} from './sky.mjs';

// ---------- FIRST AID KIT ----------
// Case with a carry handle; Solid cuts the cross out of the case. The kit is lifted and swings about the
// top of its handle.
const CASE=rect(5.5,10,21,17,2.6);
const PLUS=polygon([[14.7,13.3],[17.3,13.3],[17.3,17.2],[21.2,17.2],[21.2,19.8],[17.3,19.8],[17.3,23.7],[14.7,23.7],[14.7,19.8],[10.8,19.8],[10.8,17.2],[14.7,17.2]],.6);
const up=F([[0,0],[.08,0],[.26,-1],[.52,0,'smooth']],[[0,0],[.08,.3],[.24,-2.2],[.46,-.8],[.62,0]]);
const sway=F([[0,0],[.26,2.5],[.52,0,'smooth']],[[0,0],[.14,-4.5],[.32,3],[.48,-1.5],[.62,0]]);
const firstAidKit=make('first-aid-kit','First aid kit',
 'Light: the kit is lifted 1 unit by its handle and swings 2.5°. Strong: it is jerked up 2.2 units and swings −4.5°, 3°, −1.5° before settling.',
 posed([fill('case-interior',CASE,'tint'),stroke('case',CASE),fill('case-solid',holes(rect(4.2,8.7,23.6,19.6,3.9),PLUS),'solid'),
  // The cross is Outline/Duotone detail; Solid shows it as the cut-out.
  stroke('cross-v','M16 14.6 L16 22.4'),stroke('cross-h','M12.1 18.5 L19.9 18.5'),
  ...tube('handle','M12 10 L12 7.6 C12 6.7 12.7 6 13.6 6 L18.4 6 C19.3 6 20 6.7 20 7.6 L20 10')],
 rigid((t,s)=>({dy:up(t,s),angle:sway(t,s),pivot:[16,6]}))),.52,.62);
firstAidKit.duotoneFill=true;

// ---------- BANDAGE ----------
// Adhesive strip at 45°, built horizontally. Pad edges are two cross lines; four vent dots sit on the pad.
// Solid is the strip with the dots cut out. The strip stretches along its length and snaps back.
const STRIP=rect(5,11,22,10,5);
// Dots 2.8 apart, pad lines at 11.8/20.2: Bold gap from dot to pad line ≈0.8.
const DOTS=[[14.6,14.6],[17.4,14.6],[14.6,17.4],[17.4,17.4]];
const stretch=F([[0,1],[.06,1],[.26,1.04],[.5,1,'smooth']],[[0,1],[.06,.97],[.22,1.12],[.34,.97,'hit'],[.44,1.02],[.54,1]]);
const bPose=(x,y,t,s)=>rotate(16+(x-16)*stretch(t,s),y,-45,16,16);
const bandage=make('bandage','Bandage',
 'Light: the strip stretches 4% along its length and relaxes. Strong: it is pinched in, pulled 12% long, snaps back past rest and settles.',
 posed([fill('strip-interior',STRIP,'tint'),stroke('strip',STRIP),fill('strip-solid',holes(STRIP,...DOTS.map(d=>circle(...d,.8))),'solid'),
  stroke('pad-l','M11.8 11 L11.8 21'),stroke('pad-r','M20.2 11 L20.2 21'),
  ...DOTS.map((d,i)=>fill('dot-'+i,circle(...d,.75),'outline'))],bPose),.5,.54);
bandage.duotoneFill=true;

// ---------- WEIGHT SCALE ----------
// Bathroom scale: body, dial window arc, needle and hub. Result: the needle swings to the reading.
// Strong: the body compresses as someone steps on, and the spring-loaded needle overshoots and rings down.
const BODY=rect(5,5,22,22,5),PV=[16,15.5];
const needle=F([[0,-55],[.08,-55],[.62,20,'smooth']],[[0,-55],[.12,-55,'hold'],[.3,38],[.42,12],[.52,23],[.62,20]]);
const press=F([[0,1],[.6,1]],[[0,1],[.06,.95,'hit'],[.2,1.01],[.3,1]]);
const squash=(x,y,t,s)=>{const k=press(t,s);return[16+(x-16)*(2-k)**.4,27+(y-27)*k];};
const needlePose=(x,y,t,s)=>squash(...rotate(x,y,needle(t,s),...PV),t,s);
const weightScale=make('weight-scale','Weight scale',
 'Light: the needle sweeps from −55° to the reading at 20°. Strong: the scale body compresses 5% as weight lands, then the needle overshoots to 38° and rings down to 20°. The reading is retained; preview reset rearms.',[
 ...posed([fill('body-interior',BODY,'tint'),stroke('body',BODY),fill('body-solid',holes(rect(3.7,3.7,24.6,24.6,6.3),rect(6.3,6.3,19.4,19.4,3.7)),'solid'),
  ...tube('window',arcN(16,15.5,6.5,205,335,2)),...bothFill('hub',circle(...PV,1.3))],squash),
 ...posed(tube('needle','M16 15.5 L16 12.5'),needlePose)
],.62,.62,['needle','needle-solid']);
weightScale.duotoneFill=true;

// ---------- BLOOD PRESSURE ----------
// Gauge with a pump bulb on a tube. Squeezing the bulb drives the needle up; the tube tracks the bulb's top.
// The bulb is a teardrop whose tip takes the hose (an ellipse read as a ring beside a magnifier).
const DC=[12.4,12.4],DR=7.2,BC=[23,23.4],BB=27.6,TIP=[23,18.6];
const BULB='M23 18.6 C24.6 20.4 26.4 21.8 26.4 24.2 C26.4 26.2 24.9 27.6 23 27.6 C21.1 27.6 19.6 26.2 19.6 24.2 C19.6 21.8 21.4 20.4 23 18.6 Z';
const squeeze=F([[0,0],[.06,0],[.18,1],[.34,0,'smooth']],[[0,0],[.04,0],[.12,1,'hit'],[.22,0],[.3,1,'hit'],[.42,0]]);
const bpNeedle=F([[0,-120],[.14,-120],[.62,40,'smooth']],[[0,-120],[.1,-120,'hold'],[.18,-40],[.26,-40,'hold'],[.36,48],[.46,38],[.54,40]]);
const bulbPose=(x,y,t,s)=>{const k=squeeze(t,s);return[BC[0]+(x-BC[0])*(1+.14*k),BB+(y-BB)*(1-.16*k)];};
const topY=(t,s)=>bulbPose(...TIP,t,s)[1];
const hoseD=(t,s)=>{const y=topY(t,s);return `M${DC[0]+DR*Math.SQRT1_2} ${DC[1]+DR*Math.SQRT1_2} C19.6 19.6 ${BC[0]} ${y-2.4} ${BC[0]} ${y}`;};
const bloodPressure=make('blood-pressure','Blood pressure',
 'Light: one squeeze of the bulb and the gauge needle rises to the reading. Strong: two hard squeezes step the needle up, then it overshoots 8° and settles. The reading is retained; preview reset rearms.',[
 ...framed('dial',circle(...DC,DR),annulus(...DC,DR,2.6)),
 ...posed(tube('needle',`M${DC} L${DC[0]} ${DC[1]-3.6}`),rigid((t,s)=>({angle:bpNeedle(t,s),pivot:DC}))),
 ...bothFill('hub',circle(...DC,1.2)),
 ...tube('hose',hoseD),
 ...posed(framed('bulb',BULB),bulbPose)
],.62,.54,['needle','needle-solid']);
bloodPressure.duotoneFill=true;

// ---------- YOGA ----------
// Yoga mat unrolling, side view (parent review: every figure pictogram read as something else). The mat is a
// line on the floor from a fixed left end to the roll's contact point; the roll is a circle resting on the
// mat end with an inner spiral turn. As the roll travels right it pays out mat and its radius shrinks with
// the rolled length (area kept). The spiral turns with rolling contact. Result retained; preview reset rearms.
// Rest radius 5.6 keeps the inner turn an open curl (r 2.1) at Bold; 4.6 collapsed into a blob.
const FL=23,X0=4.6,XA=10.6,XB=21.4,RA=6.8,RB=5.6,KR=(RA*RA-RB*RB)/(XB-XA);
const roll=F([[0,XA],[.06,XA],[.72,XB,'smooth']],[[0,XA],[.04,XA-.4],[.14,XA-.4,"hold"],[.4,XB+1.1],[.52,XB-.5],[.62,XB+.2],[.7,XB]]);
const R=(t,s)=>Math.sqrt(Math.min(RA*RA,Math.max(RB*RB-.5,RA*RA-KR*(roll(t,s)-XA))));
const spinA=(t,s)=>{const x=roll(t,s);return (x-XA)/((RA+RB)/2)*180/Math.PI;};
const rollC=(t,s)=>[roll(t,s),FL-R(t,s)];
const yoga=make('yoga','Yoga',
 'Light: the rolled mat travels right, paying out the mat, and settles partly unrolled. Strong: it is drawn back, pushed, rolls out fast past its mark and rocks back once. The unrolled mat is retained; preview reset rearms.',[
 ...tube('mat',(t,s)=>`M${X0} ${FL} L${roll(t,s)} ${FL}`),
 ...framed('roll',(t,s)=>circle(...rollC(t,s),R(t,s)),(t,s)=>annulus(...rollC(t,s),R(t,s),2.6)),
 ...tube('spiral',(t,s)=>{const [cx,cy]=rollC(t,s),a=90+spinA(t,s);return arcN(cx,cy,R(t,s)-3.5,a,a+250,3);})
],.72,.7,['mat','mat-solid','roll-interior','roll-frame','roll-solid','spiral','spiral-solid']);
yoga.duotoneFill=true;

export const icons=[firstAidKit,bandage,weightScale,bloodPressure,yoga];
