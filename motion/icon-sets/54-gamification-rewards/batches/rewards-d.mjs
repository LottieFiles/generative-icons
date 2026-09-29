// Arcade rewards: a score display, multiplier ticket, power bolt, life hearts and round clock.
import {make,toggle,F,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,bothFill,holes,heartD,starPts,arcN} from './helpers.mjs';
const dir=(forward,reverse)=>(t,s,r)=>(r?reverse:forward)(t,s);

// HIGH SCORE: a framed arcade display bearing a clear 1 and a crown.
// The display is a ring in Solid, retaining the numeral counter and crown detail.
const panel=rect(6,12,20,15,2),panelSolid=holes(rect(4.7,10.7,22.6,17.6,3.3),rect(7.3,13.3,17.4,12.4,.7));
const scoreAngle=F([[0,0],[.18,5],[.5,0]],[[0,0],[.08,-4],[.24,11],[.42,-4],[.62,0]]);
const crown=polygon([[10,9],[8.5,4.8],[13,7],[16,3.8],[19,7],[23.5,4.8],[22,9]],.5);
const highScore=make('high-score','High score',
 'Light: the number-one display tilts 5 degrees to acknowledge a record. Strong: it prepares left, rocks 11 degrees right and settles after one response. The crown follows the case so their gap stays open.',
 posed([...bothFill('crown',crown),fill('panel-interior',panel,'tint'),stroke('panel',panel),fill('panel-solid',panelSolid,'solid'),
 ...tube('one','M13 18 L16 15.7 L16 23.3'),...tube('foot','M13 23.3 L19 23.3')],rigid((t,s)=>({angle:scoreAngle(t,s),pivot:[16,27]}))),.5,.62);

// BONUS: a ticket with a multiplier cross and two. Not a second Gift drawing.
// The multiplier and numeral are centreline paths inside a separate Solid ring.
const ticket='M7 6 L25 6 C26.1 6 27 6.9 27 8 L27 12 C24.5 12 24.5 16 27 16 L27 24 C27 25.1 26.1 26 25 26 L7 26 C5.9 26 5 25.1 5 24 L5 20 C7.5 20 7.5 16 5 16 L5 8 C5 6.9 5.9 6 7 6 Z';
const ticketTilt=F([[0,0],[.2,-6],[.52,0]],[[0,0],[.08,4],[.24,-14],[.42,5],[.6,0]]);
const bonus=make('bonus','Bonus',
 'Light: the double-bonus ticket tilts 6 degrees. Strong: it flicks back, swings 14 degrees and settles after a small response. Ticket, notches and multiplier move together as rigid paper.',
 posed([fill('ticket-interior',ticket,'tint'),stroke('ticket',ticket),{name:'ticket-solid',d:ticket,style:'solid',type:'stroke',width:2.6},
 ...tube('times-a','M9.5 13.7 L13 17.2'),...tube('times-b','M9.5 17.2 L13 13.7'),
 ...tube('two','M17 13 C17.4 10.8 22 10.8 22 13.5 C22 15.2 17 17.2 17 20 L22 20')],rigid((t,s)=>({angle:ticketTilt(t,s),pivot:[16,16]}))),.52,.6);

// POWER UP: a lightning bolt inside a power capsule. It surges upward; the
// shell remains fixed, with enough swept clearance for Strong at Bold/Solid.
const shell=rect(7,4.5,18,23,7.5),shellSolid=holes(rect(5.7,3.2,20.6,25.6,8.8),rect(8.3,5.8,15.4,20.4,6.2));
const bolt=polygon([[17,11],[12,18],[16,18],[15,23],[21,15],[17,15]],.4);
const boltRise=F([[0,0],[.22,-1.2],[.58,0]],[[0,0],[.07,.4],[.25,-3.2],[.42,.4],[.6,0]]);
const powerUp=make('power-up','Power up',
 'Light: the bolt rises 1.2 units inside its fixed capsule. Strong: a short charge dip sends the bolt up 3.2 units before it settles. The shell never scales and its internal clearance remains open.',[
 fill('shell-interior',shell,'tint'),stroke('shell',shell),fill('shell-solid',shellSolid,'solid'),
 ...posed(bothFill('bolt',bolt),rigid((t,s)=>({dy:boltRise(t,s),pivot:[16,17]})))
 ],.58,.6);

// LIVES: one full-size heart and two reserve hearts. The main heart beats while
// the reserves stay still; flexed path points keep the family stroke width.
const beat=F([[0,1],[.18,1.08],[.48,1]],[[0,1],[.08,.94],[.22,1.16],[.34,1],[.45,1.08],[.62,1]]);
const lifeY=F([[0,0],[.48,0]],[[0,0],[.08,.3],[.22,-1.5],[.42,0],[.62,0]]);
const lifePose=(x,y,t,s)=>[16+(x-16)*beat(t,s),12+(y-12)*beat(t,s)+lifeY(t,s)];
const lives=make('lives','Lives',
 'Light: the main life heart beats once by eight percent. Strong: it draws in, leaps 1.5 units through a sixteen-percent beat and gives one smaller heartbeat. The two reserve hearts remain still.',[
 ...posed(framed('main-heart',heartD(16,12,6.3)),lifePose),
 ...framed('reserve-left',heartD(8.5,24,3)),...framed('reserve-right',heartD(23.5,24,3))
 ],.48,.62);

// ROUND COUNTDOWN: a round clock with a start tab. The main hand advances three
// quarters of a turn and keeps the elapsed state. The return is a directed reset.
const advance=dir(F([[0,0],[.05,0],[.9,270]],[[0,0],[.04,0,'hold'],[.44,270]]),F([[0,270],[.04,270],[.8,0]],[[0,270],[.04,270],[.48,0]]));
const clockFace=circle(16,17.5,9.5),clockRing=holes(circle(16,17.5,10.8),circle(16,17.5,8.2));
const handPose=rigid((t,s,r)=>({angle:advance(t,s,r),pivot:[16,17.5]}));
const countdown=toggle('round-countdown','Round countdown',
 'Light: the round timer hand advances 270 degrees over 0.85 seconds and retains the elapsed state. Strong drives the full hand sweep in 0.4 seconds. Reverse rewinds the hand to the next round without moving the fixed case.',[
 fill('face-interior',clockFace,'tint'),stroke('case',clockFace),fill('case-solid',clockRing,'solid'),
 ...tube('tab','M13.5 4.5 L18.5 4.5'),...tube('neck','M16 4.5 L16 8'),
 ...posed(tube('hand','M16 17.5 L16 11.5'),handPose),...bothFill('hub',circle(16,17.5,1.35))
 ],.9,.44,'round-start','round-elapsed',.8,.48);

export const icons=[highScore,bonus,powerUp,lives,countdown].map(icon=>({...icon,duotoneFill:true}));
