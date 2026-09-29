// Smart home batch D: smoke-alarm, water-leak, power-outlet, energy-usage, motion-sensor.
import {make,F,P,tube,framed,rigid,posed,shown,stroke,fill,circle,rect,bothFill,annulus} from './helpers.mjs';
import {arcN,pt,ringRect} from './sky.mjs';
const grown=(name,d)=>[stroke(name,d),{name:name+'-solid',d,style:'solid',type:'stroke',width:2.6}];

// ---------- SMOKE ALARM ----------
// Ceiling-mounted puck with an LED; two alarm arcs below it pulse outward. Strong shakes the puck.
const AC=[16,13];
// Both arcs pulse together (a stagger closed their gap) and never move inward toward the puck.
const blare=()=>F([[0,0],[.06,.8],[.4,0,'smooth']],[[0,0],[.14,1.6],[.26,.4],[.36,1],[.52,0]]);
const buzz=(t,s)=>{if(!s)return 0;const u=t-.04;return u<=0||u>=.44?0:.6*Math.sin(u*70)*(1-u/.44)**1.5;};
const blink=P([[0,1],[.06,.3],[.2,1]],[[0,1],[.04,.2],[.1,1],[.16,.2],[.22,1],[.28,.2],[.36,1]]);
const smokeAlarm=make('smoke-alarm','Smoke alarm',
 'Light: the LED blinks and the two alarm arcs swell 0.8 units and return. Strong: the puck buzzes on its mount, the LED flashes three times and the arcs push out 1.6 units with a second pulse.',[
 ...tube('ceiling','M4.5 5 L27.5 5'),
 ...posed(framed('puck',rect(7.5,8.5,17,7,3),ringRect(7.5,8.5,17,7,3)),rigid((t,s)=>({dx:buzz(t,s)}))),
 ...posed(shown(tube('led','M15.8 12 L16.2 12'),blink),rigid((t,s)=>({dx:buzz(t,s)}))),
 ...[8,11.5].flatMap((r,k)=>tube('arc-'+k,(t,s)=>arcN(...AC,r+blare(k)(t,s),50,130,2)))
],.45,.58);
smokeAlarm.duotoneFill=true;

// ---------- WATER LEAK ----------
// A pipe with a coupling; a drop hangs under the coupling, falls into the puddle (which spreads) and a
// new drop forms. The drop grows from its tip; at rest one drop hangs.
const TIP=[16,14.8];
const dropD='M16 14.8 C16.6 15.8 18.4 16.8 18.4 18.8 C18.4 20.1 17.3 21.2 16 21.2 C14.7 21.2 13.6 20.1 13.6 18.8 C13.6 16.8 15.4 15.8 16 14.8 Z';
const fall=P([[0,0],[.06,0],[.3,4,'in'],[.31,4],[.32,0,'linear']],[[0,0],[.1,0],[.24,4,'in'],[.25,4],[.26,0,'linear']]);
const grow=P([[0,1],[.3,1],[.31,0,'linear'],[.36,0],[.62,1,'out']],[[0,1],[.04,1],[.1,1.15],[.24,1.15],[.25,0,'linear'],[.3,0],[.5,1.12,'out'],[.6,1]]);
const dropV=P([[0,1],[.16,1],[.3,0,'in'],[.36,0],[.4,1]],[[0,1],[.14,1],[.24,0,'in'],[.3,0],[.34,1]]);
const spread=F([[0,0],[.26,0],[.4,1.2],[.6,0,'smooth']],[[0,0],[.2,0],[.28,2.6,'hit'],[.4,.6],[.56,0]]);
const waterLeak=make('water-leak','Water leak',
 'Light: the hanging drop falls into the puddle, the puddle spreads 1.2 units and a new drop forms. Strong: the drop swells heavy, drops fast, the puddle splashes 2.6 wider and the next drop forms with a small overshoot.',[
 ...tube('pipe-l','M4.5 8.5 L13.5 8.5'),...tube('pipe-r','M18.5 8.5 L27.5 8.5'),
 ...framed('coupling',rect(13.5,5.5,5,6,1.5)),
 ...posed(shown(framed('drop',dropD),dropV),(x,y,t,s)=>{const k=Math.max(.001,grow(t,s));return[TIP[0]+(x-TIP[0])*k,TIP[1]+(y-TIP[1])*k+fall(t,s)];}),
 ...tube('puddle',(t,s)=>{const w=5.5+spread(t,s);return `M${16-w} 26.8 L${16+w} 26.8`;})
],.62,.6);
waterLeak.duotoneFill=true;

// ---------- POWER OUTLET ----------
// Smart wall outlet: plate, socket with two slots, a top button and a status LED. Pressing the button
// switches the outlet on; the LED appears and stays on.
const push=F([[0,0],[.06,.6],[.24,0,'smooth']],[[0,0],[.04,-.2],[.12,1.2,'hit'],[.22,-.15],[.3,0]]);
const led=F([[0,0],[.14,0],[.3,1,'out']],[[0,0],[.12,0],[.18,1.2,'out'],[.28,.94],[.36,1]]);
const powerOutlet=make('power-outlet','Power outlet',
 'Light: the button presses 0.6 units and the status light comes on. Strong: a hard 1.2-unit press with a small rebound, and the status light pops on past full size and settles. The on state is retained; preview reset rearms.',[
 ...framed('plate',rect(6,3.8,20,24.9,4),ringRect(6,3.8,20,24.9,4)),
 ...framed('socket',circle(16,19.2,6),annulus(16,19.2,6,2.6)),
 ...tube('slot-l','M14.2 17.6 L14.2 20'),...tube('slot-r','M17.8 17.6 L17.8 20'),
 ...posed(tube('button','M13.2 8.5 L18 8.5'),rigid((t,s)=>({dy:push(t,s)}))),
 // LED r1.2 at (22.2,8.5): ≥1.1 clear of the button and plate wall at its 1.2× pop.
 ...shown(posed(bothFill('led',circle(22.2,8.5,1.2)),(x,y,t,s)=>{const k=Math.max(.001,led(t,s));return[22.2+(x-22.2)*k,8.5+(y-8.5)*k];}),(t,s)=>led(t,s)>.1?1:0)
],.36,.4,['led','led-solid']);
powerOutlet.duotoneFill=true;

// ---------- ENERGY USAGE ----------
// A lightning bolt beside four consumption bars. Bars grow from dots to their readings in turn and only
// rise; Strong flashes the bolt first and drives the bars up in a quick sequence.
const BARS=[[14,21.5],[18.5,18],[23,19.5],[27.5,14]];
const barK=k=>F([[0,0],[.06+k*.12,0],[.3+k*.12,1,'smooth']],[[0,0],[.18+k*.06,0,'hold'],[.3+k*.06,1]]);
const zap=P([[0,1],[.8,1]],[[0,1],[.04,.2],[.08,1],[.12,.25],[.18,1]]);
const jolt=F([[0,0],[.8,0]],[[0,0],[.04,-.6],[.18,0]]);
const energyUsage=make('energy-usage','Energy usage',
 'Light: the four usage bars rise from the baseline in turn to their readings. Strong: the bolt flashes and jolts, then the bars shoot up in a quick sequence. The readings are retained; preview reset rearms.',[
 ...posed(shown(tube('bolt','M10 5 L6 11.3 L10.4 11.3 L6.6 17.6'),zap),rigid((t,s)=>({dy:jolt(t,s)}))),
 ...BARS.flatMap(([x,top],k)=>grown('bar-'+k,(t,s)=>`M${x} 26.5 L${x} ${26.5-Math.max(.02,(26.5-top)*barK(k)(t,s))}`))
],.72,.52,['bar-0','bar-1','bar-2','bar-3']);
energyUsage.duotoneFill=false;

// ---------- MOTION SENSOR ----------
// Wall-mounted PIR dome; two detection beams open out from it in turn, then the detected person appears.
// v1 ended on a dot and read as a speaker broadcasting; a person makes it a motion sensor.
const DC=[5.8,16];
const arcK=k=>F([[0,0],[.06+k*.12,0],[.32+k*.12,1,'smooth']],[[0,0],[.06+k*.06,0,'hold'],[.18+k*.06,1]]);
// The person rises into range and fades in (rigid, so the head–shoulder gap never closes); Strong
// overshoots upward and settles.
const rise=F([[0,2.4],[.42,2.4],[.62,0,'out']],[[0,2.4],[.26,2.4],[.34,-.8,'out'],[.44,.25],[.52,0]]);
const seen=P([[0,0],[.42,0],[.54,1,'out']],[[0,0],[.26,0],[.32,1,'out']]);
const pop=rigid((t,s)=>({dy:rise(t,s)}));
const motionSensor=make('motion-sensor','Motion sensor',
 'Light: two detection beams open out from the dome in turn and a person appears in range. Strong: the beams flash open in a tight burst and the person rises in fast, overshoots upward and settles. The detection is retained; preview reset rearms.',[
 ...tube('wall','M4.2 9 L4.2 23'),
 ...framed('dome',`${arcN(...DC,4.2,-90,90,2)} Z`),
 ...[8.6,12.1].flatMap((r,k)=>shown(grown('beam-'+k,(t,s)=>{const p=Math.max(.002,arcK(k)(t,s));return arcN(...DC,r,-35*p,35*p,2);}),(t,s)=>arcK(k)(t,s)>.03?1:0)),
 ...shown(posed(bothFill('head',circle(26,12.6,1.7)),pop),seen),
 ...shown(posed(tube('shoulders','M23.5 20.4 C23.5 18.1 24.6 16.6 26 16.6 C27.4 16.6 28.5 18.1 28.5 20.4'),pop),seen)
],.66,.56,['beam-0','beam-1','head','head-solid','shoulders']);
motionSensor.duotoneFill=true;

export const icons=[smokeAlarm,waterLeak,powerOutlet,energyUsage,motionSensor];
