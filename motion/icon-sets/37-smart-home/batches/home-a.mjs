// Smart home batch A: smart-bulb, thermostat, smart-lock, door-sensor, garage-door.
import {make,F,P,tube,framed,rigid,posed,shown,stroke,circle,rect,bothFill,annulus} from './helpers.mjs';
import {arcN,pt,ringRect} from './sky.mjs';
const L=d=>d.replace(/^M/,'L');
const grown=(name,d)=>[stroke(name,d),{name:name+'-solid',d,style:'solid',type:'stroke',width:2.6}];

// ---------- SMART BULB ----------
// Glass r6.2 at (16,12) narrowing to a neck at y20, two screw lines below. Rays start at r9.7
// (Bold gap to the glass ≥1); the top ray is omitted so rays keep the 2-unit margin.
const glass=`M12.444 17.079 ${L(arcN(16,12,6.2,125,415,4))} L18.8 19.3 C18.8 19.7 18.5 20 18.1 20 L13.9 20 C13.5 20 13.2 19.7 13.2 19.3 Z`;
const bulbPulse=F([[0,1],[.6,1]],[[0,1],[.05,1.06],[.18,1]]);
const glow=(t,s,i)=>{const d=i*(s?.02:.05),a=(s?.18:.1)+d,b=(s?.3:.4)+d;if(t<=a)return 0;const u=Math.min(1,(t-a)/(b-a));if(!s)return u*u*(3-2*u);if(t<=b)return 1.4*(1-(1-u)**3);const v=Math.min(1,(t-b)/.14);return 1.4-.4*v*v*(3-2*v);};
const smartBulb=make('smart-bulb','Smart bulb',
 'Light: the bulb switches on and four rays grow out in a calm stagger. Strong: the glass swells as the power arrives, the rays burst out past full length and settle. The lit state is retained; preview reset rearms.',[
 ...posed(framed('glass',glass),(x,y,t,s)=>{const k=bulbPulse(t,s);return[16+(x-16)*k,20+(y-20)*k];}),
 ...tube('thread-0','M13.3 23.5 L18.7 23.5'),...tube('thread-1','M14.3 27 L17.7 27'),
 ...[180,225,315,0].flatMap((a,i)=>shown(grown('ray-'+i,(t,s)=>`M${pt(a,9.7,[16,12])} L${pt(a,9.7+1.6*Math.max(.001,glow(t,s,i)),[16,12])}`),(t,s)=>glow(t,s,i)>.03?1:0))
],.6,.54,['ray-0','ray-1','ray-2','ray-3']);
smartBulb.duotoneFill=true;

// ---------- THERMOSTAT ----------
// Round wall thermostat: fixed ring r11.2, a setpoint arc (r6.8) that grows clockwise from 135°, a knob
// riding its end and a hub. The setpoint only increases.
const setA=F([[0,200],[.06,200],[.62,330,'smooth']],[[0,200],[.1,200,'hold'],[.36,330]]);
const heat=F([[0,1],[.6,1]],[[0,1],[.34,1],[.42,1.45,'hit'],[.56,1]]);
const thermostat=make('thermostat','Thermostat',
 'Light: the setpoint arc sweeps clockwise to a higher temperature with its knob riding the end. Strong: the setting is turned up in a quarter second and the hub pulses as the heating engages. The setting is retained; preview reset rearms.',[
 ...framed('ring',circle(16,16,11.2),annulus(16,16,11.2,2.6)),
 ...grown('setpoint',(t,s)=>arcN(16,16,6.8,135,setA(t,s),4)),
 ...bothFill('knob',(t,s)=>circle(...pt(setA(t,s),6.8),1.9)),
 ...posed(bothFill('hub',circle(16,16,1.6)),(x,y,t,s)=>{const k=heat(t,s);return[16+(x-16)*k,16+(y-16)*k];})
],.62,.56,['setpoint','setpoint-solid','knob','knob-solid']);
thermostat.duotoneFill=true;

// ---------- SMART LOCK ----------
// Door-lock escutcheon (v3): a tall rounded plate, a 2×2 keypad in the upper half, and a lever handle
// projecting sideways from a round rose lower down. The lever is pressed down and springs back; the status
// light above the keypad comes on. (v1/v2 read as a remote or phone.)
const ROSE=[14.5,21.2];
const leverA=F([[0,0],[.06,0],[.24,30],[.46,0,'smooth']],[[0,0],[.04,-5],[.16,45,'hit'],[.3,-6],[.4,2],[.48,0]]);
const status=F([[0,0],[.34,0],[.5,1,'out']],[[0,0],[.28,0],[.36,1.35,'out'],[.46,.92],[.54,1]]);
const smartLock=make('smart-lock','Smart lock',
 'Light: the lever handle is pressed 30° down and springs back, and the status light above the keypad comes on. Strong: it is yanked 45° down with a hard hit, rebounds and settles, and the status light pops on. The unlocked state is retained; preview reset rearms.',[
 ...framed('plate',rect(8,3.8,13,24.4,6.5),ringRect(8,3.8,13,24.4,6.5)),
 ...[[12.5,10.8],[16.5,10.8],[12.5,14.8],[16.5,14.8]].flatMap(([x,y],i)=>tube('key-'+i,`M${x-.2} ${y} L${x+.2} ${y}`)),
 ...framed('rose',circle(...ROSE,2.8)),
 ...posed(tube('lever','M17.3 21.2 L25 21.2'),rigid((t,s)=>({angle:leverA(t,s),pivot:ROSE}))),
 ...shown(posed(bothFill('status',circle(14.5,7.3,.9)),(x,y,t,s)=>{const k=Math.max(.001,status(t,s));return[14.5+(x-14.5)*k,7.3+(y-7.3)*k];}),(t,s)=>status(t,s)>.1?1:0)
],.54,.56,['status','status-solid']);
smartLock.duotoneFill=true;

// ---------- DOOR SENSOR ----------
// Contact sensor (left) and its magnet (right, 1-unit painted gap when closed). The door opens: the
// magnet slides away and two alert arcs appear above the sensor.
const slide=F([[0,0],[.06,0],[.42,4.5,'smooth']],[[0,0],[.08,0,'hold'],[.22,5.5],[.34,4.3],[.44,4.5]]);
const alertK=(i)=>F([[0,0],[.3+i*.08,0],[.44+i*.08,1,'smooth']],[[0,0],[.2+i*.05,0,'hold'],[.3+i*.05,1]]);
const SEN=[9.75,12.2];
const doorSensor=make('door-sensor','Door sensor',
 'Light: the magnet slides 4.5 units away from the sensor as the door opens, then two alert arcs appear. Strong: the magnet is yanked 5.5 units and settles while the arcs snap in quickly. The open state is retained; preview reset rearms.',[
 ...framed('sensor',rect(5,13,9.5,12.5,2.5),ringRect(5,13,9.5,12.5,2.5)),
 ...tube('led','M9.55 19.25 L9.95 19.25'),
 ...posed(framed('magnet',rect(18,15,4.5,8.5,1.8)),rigid((t,s)=>({dx:slide(t,s)}))),
 ...[3.8,7.3].flatMap((r,i)=>{const k=alertK(i);return shown(grown('alert-'+i,(t,s)=>{const p=Math.max(.002,k(t,s));return arcN(...SEN,r,270-35*p,270+35*p,2);}),(t,s)=>k(t,s)>.03?1:0);})
],.6,.44,['magnet','magnet-frame','magnet-interior','magnet-solid','alert-0','alert-1']);
doorSensor.duotoneFill=true;

// ---------- GARAGE DOOR ----------
// House outline (open at the ground) with a door opening. Three door panels roll up toward the header
// and disappear into it; the empty doorway is the result.
const house='M4.5 27.5 L4.5 12.3 C4.5 11.7 4.8 11.2 5.3 10.9 L15.2 5.3 C15.7 5 16.3 5 16.8 5.3 L26.7 10.9 C27.2 11.2 27.5 11.7 27.5 12.3 L27.5 27.5';
const opening='M8.5 27.5 L8.5 15.5 C8.5 14.9 8.9 14.5 9.5 14.5 L22.5 14.5 C23.1 14.5 23.5 14.9 23.5 15.5 L23.5 27.5';
const roll=F([[0,0],[.06,0],[.7,1,'smooth']],[[0,0],[.08,-.04],[.4,1.02],[.5,1]]);
// Top panel starts 1.25 below the header (full opacity); panels fade out between y18.2 and 17.6.
const PANELS=[18.25,21.75,25.25];
const panelY=(k,t,s)=>PANELS[k]-10.5*roll(t,s);
const garageDoor=make('garage-door','Garage door',
 'Light: the three door panels roll up one after another into the header, leaving the doorway open. Strong: the door dips, then rolls up in a third of a second and knocks against its stop. The open state is retained; preview reset rearms.',[
 ...tube('house',house),...tube('opening',opening),
 ...PANELS.flatMap((y,k)=>shown(posed(tube('panel-'+k,`M12 ${y} L20 ${y}`),rigid((t,s)=>({dy:panelY(k,t,s)-y}))),(t,s)=>{const v=panelY(k,t,s);return v>=18.2?1:v<=17.6?0:(v-17.6)/.6;}))
],.7,.5,['panel-0','panel-1','panel-2']);
garageDoor.duotoneFill=false;

export const icons=[smartBulb,thermostat,smartLock,doorSensor,garageDoor];
