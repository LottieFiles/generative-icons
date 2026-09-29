// Weather batch D: temperature, humidity, uv-index, hurricane, moon-phase.
import {make,F,tube,framed,rigid,posed,shown,fill,stroke,circle,capsule,bothFill} from './helpers.mjs';
import {arcN,curveFrom,pt} from './sky.mjs';
const L=d=>d.replace(/^M/,'L');

// ---------- TEMPERATURE ----------
// Thermometer: stem half-width 3.2 (x 10.8–17.2), bulb r4.6 at (14,23.2), scale ticks on the right.
// Mercury is a filled capsule (1.8 wide) so its Bold/Solid gap to the wall stays ≥1.0; its level only rises.
const TX=14;
const bodyD=`M${TX-3.2} 19.9 L${TX-3.2} 7.2 ${L(arcN(TX,7.2,3.2,180,360,2))} L${TX+3.2} 19.9 ${L(arcN(TX,23.2,4.6,-45.88,225.88,4))} Z`;
const level=F([[0,18.6],[.06,18.6],[.62,10.4,'smooth']],[[0,18.6],[.16,18.6,'hold'],[.4,10.4]]);
const swell=F([[0,0],[.6,0]],[[0,0],[.1,.35],[.2,.35],[.34,0]]);
const temperature=make('temperature','Temperature',
 'Light: the mercury column rises steadily from low to high and holds. Strong: the bulb swells first, then the column shoots up in a quarter second and stops at the same reading. The reading is retained; preview reset rearms.',[
 fill('body-interior',bodyD,'tint'),stroke('body-frame',bodyD),{name:'body-solid',d:bodyD,style:'solid',type:'stroke',width:2.6},
 ...[9.4,13,16.6].flatMap((y,i)=>tube('tick-'+i,`M${TX+6.8} ${y} L${TX+8.6} ${y}`)),
 ...bothFill('bulb',(t,s)=>circle(TX,23.2,2.3+swell(t,s))),
 ...bothFill('column',(t,s)=>capsule([TX,level(t,s)],[TX,22.4],1.8))
],.62,.4,['column']);
temperature.duotoneFill=true;

// ---------- HUMIDITY ----------
// Drop: circle (16,19.2) r7.5 with a rounded apex at (16,4.6). Water fills below a sloshing surface whose
// ends ride the drop's centreline, so the surface meets the wall instead of floating inside it.
const DC=[16,19.2],DR=7.5;
const dropD=`M15.31 5.76 C15.77 4.99 16.23 4.99 16.69 5.76 L22.43 15.35 ${L(arcN(16,19.2,7.5,-30.9,210.9,4))} Z`;
const water=F([[0,24.2],[.06,24.2],[.66,18.4,'smooth']],[[0,24.2],[.08,24.2,'hold'],[.42,18.4]]);
// The slosh decays and is windowed to end exactly at the active time, so the rest is still.
const slosh=(t,s)=>{const u=Math.max(0,t-(s?.08:.06)),T=s?.7:.58;return u>=T?0:(s?1.6:.7)*Math.sin(u*(s?17:12))*Math.exp(-u*(s?4.2:4))*(1-u/T)**1.5;};
const surfaceAt=(t,s)=>{const y=water(t,s),w=Math.sqrt(DR*DR-(y-DC[1])**2),a=slosh(t,s);return {y,w,a};};
const surfD=(t,s)=>{const {y,w,a}=surfaceAt(t,s);return `M${16-w} ${y} C${16-w/3} ${y-a} ${16+w/3} ${y+a} ${16+w} ${y}`;};
const waterD=(t,s)=>{const {y}=surfaceAt(t,s),th=Math.asin((y-DC[1])/DR)*180/Math.PI;return surfD(t,s)+' '+L(arcN(16,19.2,7.5,th,180-th,3))+' Z';};
const humidity=make('humidity','Humidity',
 'Light: the water level inside the drop rises from low to just above the middle with a small slosh. Strong: it rises in 0.34 s and the surface sloshes harder before settling level. The level is retained; preview reset rearms.',[
 stroke('drop',dropD),{name:'drop-solid',d:dropD,style:'solid',type:'stroke',width:2.6},
 fill('water-interior',waterD,'tint'),fill('water-solid',waterD,'solid'),
 ...tube('surface',surfD)
],.66,.8,['water-interior','water-solid','surface']);
humidity.duotoneFill=true;

// ---------- UV INDEX ----------
// Sun (disc r3 at (16,12.8)) above a three-step meter. Each meter dash grows from a dot to full length
// in turn; the rays lengthen with the reading. Meter values are monotone; only the rays overshoot (Strong).
// Rays start at r6.9: Bold gap to the disc stays ≥1.1 even at Strong's 1.08 pulse.
const SC=[16,12.8],DASH=[[6.4,10.47],[13.97,18.03],[21.53,25.6]],MY=25.8;
// Strong fills each dash in 0.09 s after the pulse (Light 0.16 s), a decisive step reading.
const fillK=k=>F([[0,0],[.08+k*.16,0],[.24+k*.16,1,'smooth']],[[0,0],[.14+k*.09,0,'hold'],[.23+k*.09,1]]);
const pulse=F([[0,1],[.6,1]],[[0,1],[.06,1.08],[.14,1]]);
const rayLen=(t,s)=>{const lvl=[0,1,2].reduce((a,k)=>a+fillK(k)(t,s),0)/3;const flare=s?F([[0,0]],[[0,0],[.44,0],[.52,.55,'hit'],[.64,-.1],[.74,0]])(t,s):0;return 1+.9*lvl+flare;};
const uvIndex=make('uv-index','UV index',
 'Light: the three meter dashes fill left to right while the sun\'s rays lengthen with the reading. Strong: the sun pulses first, the dashes fill in a quick sequence and the rays flare past their final length and settle. The reading is retained; preview reset rearms.',[
 ...posed(framed('disc',circle(...SC,3)),(x,y,t,s)=>{const k=pulse(t,s);return[SC[0]+(x-SC[0])*k,SC[1]+(y-SC[1])*k];}),
 ...[0,45,90,135,180,225,270,315].flatMap((a,i)=>{const d=(t,s)=>`M${pt(a,6.9,SC)} L${pt(a,6.9+rayLen(t,s),SC)}`;return [stroke('ray-'+i,d),{name:'ray-'+i+'-solid',d,style:'solid',type:'stroke',width:2.6}];}),
 ...DASH.flatMap(([a,b],k)=>{const d=(t,s)=>`M${a} ${MY} L${a+Math.max(.02,(b-a)*fillK(k)(t,s))} ${MY}`;return [stroke('dash-'+k,d),{name:'dash-'+k+'-solid',d,style:'solid',type:'stroke',width:2.6}];})
],.62,.76,['dash-0','dash-1','dash-2','ray-0','ray-1','ray-2','ray-3','ray-4','ray-5','ray-6','ray-7']);
uvIndex.duotoneFill=true;

// ---------- HURRICANE ----------
// Eye ring r3.2 with two spiral arms leaving it on opposite sides and sweeping counterclockwise out to r11.
// Two-fold symmetric: a half turn is the next equivalent pose. Storms here turn counterclockwise.
const arm=phase=>curveFrom(u=>pt(phase-172*u,3.2+7.8*u**1.05),4);
const spin=F([[0,0],[.06,0],[.8,-180,'smooth']],[[0,0],[.1,14],[.56,-372],[.76,-360]]);
const hurricane=make('hurricane','Hurricane',
 'Light: the storm makes a calm counterclockwise half turn to its next equivalent pose. Strong: it winds 14° the other way, whips through a full counterclockwise turn and settles.',
 posed([...framed('eye',circle(16,16,3.2)),...tube('arm-0',arm(0)),...tube('arm-1',arm(180))],rigid((t,s)=>({angle:spin(t,s),pivot:[16,16]}))),.8,.76);
hurricane.duotoneFill=true;hurricane.cycleDegrees=180;

// ---------- MOON PHASE ----------
// Disc r10.5; the terminator is a half ellipse from pole to pole whose width k·r sweeps. Lit side is right.
const K=.5523,MR=10.5;
const term=k=>{const rx=k*MR;return `M16 5.5 C${16+K*rx} 5.5 ${16+rx} ${16-K*MR} ${16+rx} 16 C${16+rx} ${16+K*MR} ${16+K*rx} 26.5 16 26.5`;};
// v1 ended at the first quarter; its straight terminator read as a divided circle. Crescent → waxing gibbous.
const phase=F([[0,.5],[.06,.5],[.64,-.45,'smooth']],[[0,.5],[.1,.5,'hold'],[.38,-.45]]);
const rock=F([[0,0],[.64,0]],[[0,0],[.08,-4],[.38,9],[.54,-3],[.7,0]]);
const litD=(t,s)=>term(phase(t,s))+' '+L(arcN(16,16,MR,90,-90,2))+' Z';
const moonPose=rigid((t,s)=>({angle:rock(t,s),pivot:[16,16]}));
const moonPhase=make('moon-phase','Moon phase',
 'Light: the terminator sweeps from a waxing crescent through the half moon to a waxing gibbous. Strong: the moon tips back, the terminator sweeps in 0.28 s as the moon swings through 9° and settles. The phase is retained; preview reset rearms.',
 posed([stroke('rim',circle(16,16,MR)),{name:'rim-solid',d:circle(16,16,MR),style:'solid',type:'stroke',width:2.6},
  fill('lit-interior',litD,'tint'),fill('lit-solid',litD,'solid'),
  ...tube('terminator',(t,s)=>term(phase(t,s)))],moonPose),.64,.7,['lit-interior','lit-solid','terminator']);
moonPhase.duotoneFill=true;

export const icons=[temperature,humidity,uvIndex,hurricane,moonPhase];
