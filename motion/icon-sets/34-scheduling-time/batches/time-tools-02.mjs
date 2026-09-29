// scheduling-time-tools-b.mjs — revision 02 (spacing fixes only), 32-unit grid
import {make,F,tube,framed,bothFill,circle,rect,annulus,holes,rigid,posed,geometricRearm,toggle} from './helpers.mjs';

const arcN=(cx,cy,r,a,b,n=4)=>{const A=d=>d*Math.PI/180,p=t=>[cx+r*Math.cos(t),cy+r*Math.sin(t)];let s=A(a),d='M'+p(s);for(let i=1;i<=n;i++){const e=A(a+(b-a)*i/n),k=4/3*Math.tan((e-s)/4),u=p(s),v=p(e);d+=' C'+[u[0]-k*r*Math.sin(s),u[1]+k*r*Math.cos(s),v[0]+k*r*Math.sin(e),v[1]-k*r*Math.cos(e),...v];s=e;}return d;};
const K=.5523,ellipse=(x,y,rx,ry)=>`M${x} ${y-ry} C${x+rx*K} ${y-ry} ${x+rx} ${y-ry*K} ${x+rx} ${y} C${x+rx} ${y+ry*K} ${x+rx*K} ${y+ry} ${x} ${y+ry} C${x-rx*K} ${y+ry} ${x-rx} ${y+ry*K} ${x-rx} ${y} C${x-rx} ${y-ry*K} ${x-rx*K} ${y-ry} ${x} ${y-ry} Z`;
const duo=icon=>({...icon,duotoneFill:true}); // every icon has a real enclosed frame surface

// ---------- STOPWATCH ----------
// Fix 1: ticks now radial 8.4..9.5 (inner paint 7.1); hand r4.6 (paint 5.9) → 1.2 gap.
const swPress=F([[0,0],[.1,.6,'smooth'],[.3,0,'smooth']],[[0,0],[.08,-.3],[.16,1,'hit'],[.34,0]]);
const swHand=F([[0,0],[.1,0],[.6,120,'smooth']],[[0,0],[.16,0,'hold'],[.36,126],[.52,120]]);
const stopwatch=duo(geometricRearm(make('stopwatch','Stopwatch',
 'Plunger presses down and the sweep hand runs clockwise from 12; Strong presses harder and drives the hand faster with one short settle. Hand time is retained; preview reset rearms.',
 [...framed('case',circle(16,17.5,9.5),annulus(16,17.5,9.5,2.6)),
  ...tube('tick-3','M24.4 17.5 L25.5 17.5'),...tube('tick-6','M16 25.9 L16 27'),...tube('tick-9','M7.6 17.5 L6.5 17.5'),
  ...posed([...tube('stem','M16 7 L16 5.4'),...tube('button','M13.4 4.4 L18.6 4.4')],rigid((t,s)=>({dy:swPress(t,s)}))),
  ...posed(tube('hand','M16 17.5 L16 12.9'),rigid((t,s)=>({angle:swHand(t,s),pivot:[16,17.5]}))),
  ...bothFill('hub',circle(16,17.5,1.3))],
 .62,.55)));

// ---------- COUNTDOWN TIMER ---------- (unchanged geometry and profiles)
const cdSpan=F([[0,270],[.1,270],[.62,150,'smooth']],[[0,270],[.12,270,'hold'],[.2,296],[.42,146,'hit'],[.56,150]]);
const countdownTimer=duo(geometricRearm(make('countdown-timer','Countdown timer',
 'The remaining-time arc anchored at 12 shrinks as the dial is set; Strong winds up past full first, then drops fast with one bounded catch. Result retained; preview reset rearms.',
 [...framed('dial',circle(16,16,10.5),annulus(16,16,10.5,2.6)),
  ...tube('marker','M16 5.5 L16 3.4'),
  ...tube('remaining',(t,s)=>arcN(16,16,6.5,-90,-90+cdSpan(t,s)))],
 .66,.6)));

// ---------- WORLD CLOCK ----------
// Fix 2: dial (22.4,9.6) r6.2, inner paint 4.9; minute 2.4 (paint 3.7) → 1.2 gap; hour 1.8.
// Globe recomposed to (9.9,20.6) r6.5: center distance 16.65, painted sum 15.3 → 1.35 gap.
const wcMin=F([[0,0],[.08,0],[.62,180,'smooth']],[[0,0],[.1,-8],[.36,186],[.52,180]]);
const wcHour=F([[0,0],[.08,0],[.62,15,'smooth']],[[0,0],[.1,0,'hold'],[.36,15.6],[.52,15]]);
const worldClock=duo(geometricRearm(make('world-clock','World clock',
 'A globe sits beside a small zone dial; the minute hand jumps half a turn to the other zone while the hour hand follows. Strong winds back slightly then snaps with one settle. Time retained; preview reset rearms.',
 [...framed('globe',circle(9.9,20.6,6.5),annulus(9.9,20.6,6.5,2.6)),
  ...tube('equator','M3.4 20.6 L16.4 20.6'),
  ...tube('meridian',ellipse(9.9,20.6,2.7,6.5)),
  ...framed('dial',circle(22.4,9.6,6.2),annulus(22.4,9.6,6.2,2.6)),
  ...posed(tube('minute','M22.4 9.6 L22.4 7.2'),rigid((t,s)=>({angle:wcMin(t,s),pivot:[22.4,9.6]}))),
  ...posed(tube('hour','M22.4 9.6 L24.2 9.6'),rigid((t,s)=>({angle:wcHour(t,s),pivot:[22.4,9.6]})))],
 .66,.56)));

// ---------- AVAILABILITY ----------
// Fix 3: rows at 4.5 / 12.8 / 21.1 → Solid adjacent gap 1.1 (was .9). Counter moved to y14.1.
const busy=(name,y)=>[{name,d:(t,s,r,w=2)=>rect(7-w/2,y-w/2,18+w,4.6+w,2.3+w/2),style:'outline'},{name:name+'-solid',d:rect(5.7,y-1.3,20.6,7.2,3.6),style:'solid'}];
const avOut=F([[0,0],[.45,3,'smooth']],[[0,0],[.1,-.6],[.3,3.5],[.46,3]]);
const avBack=F([[0,3],[.4,0,'smooth']],[[0,3],[.08,3.4],[.26,-.3],[.4,0]]);
const availability=duo(toggle('availability','Availability',
 'Two busy slots stay put; the open slot pulls out to the right and holds as the picker opens. Strong takes a short backswing then drives farther with one bounded settle; the reverse slides it home.',
 [...busy('busy-top',4.5),
  ...posed(framed('free',rect(7,12.8,18,4.6,2.3),holes(rect(5.7,11.5,20.6,7.2,3.6),rect(8.3,14.1,15.4,2,1))),rigid((t,s,r)=>({dx:r?avBack(t,s):avOut(t,s)}))),
  ...busy('busy-bottom',21.1)],
 .5,.5,'closed','open',.44,.44));

export const icons=[stopwatch,countdownTimer,worldClock,availability];
