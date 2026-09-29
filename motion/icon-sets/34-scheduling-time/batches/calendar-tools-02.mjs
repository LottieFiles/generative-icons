// scheduling-tools-a.mjs — date-range, calendar-view, reschedule, alarm-clock (revision 02)
import {make,toggle,geometricRearm,F,tube,rect,circle,arc,fill,bothFill,framed,holes,rigid,posed,mix,stroke} from './helpers.mjs';

// Fixed leaf. Outline: centerline rect 5,6 22x21. Solid: ring 3.7..28.3 with a real
// counter 6.3..25.7 x 7.3..25.7 so interior marks stay visible. Header Solid paint ends y12.8.
const leaf=n=>[
 ...framed(n,rect(5,6,22,21,2.7),holes(rect(3.7,4.7,24.6,23.6,4),rect(6.3,7.3,19.4,18.4,1.4))),
 ...tube('peg-l','M11 4 L11 8'),...tube('peg-r','M21 4 L21 8'),   // caps buried in frame paint
 ...tube('header','M5.5 11.5 L26.5 11.5')];                         // caps buried in side rails

// 1. DATE RANGE — one driver for the end dot; the bar can never pass it, and follows it exactly after contact.
const dotX=F([[0,22],[.5,22]],[[0,22],[.08,21.1],[.28,22.7,'hit'],[.46,22]]);
const barX=F([[0,10],[.5,22,'smooth']],[[0,10],[.06,10,'hold'],[.28,22.7,'hit'],[.46,22]]);
const barEnd=(t,s)=>Math.min(barX(t,s),dotX(t,s));
const dateRange=geometricRearm(make('date-range','Date range',
 'A range bar sweeps from the fixed start date to the end date and stays. Strong: the end dot backs off, the bar drives fast, knocks it 0.7 past the date, and both settle together.',[
 ...leaf('leaf'),
 ...bothFill('start',circle(10,19,1.8)),
 ...tube('bar',(t,s)=>`M10 19 L${barEnd(t,s)} 19`,{visibility:(t,s)=>barEnd(t,s)-10<.35?0:1}), // cap hidden inside start disc until then
 ...posed(bothFill('end',circle(22,19,1.8)),rigid((t,s)=>({dx:dotX(t,s)-22})))
],.5,.5));

// Supervisor integration repair: geometricRearm freezes the equal-endpoint end dot.
// Clamp only the demo-reset bar to that stationary receiver; forward drawing and motion stay exact.
for (const part of dateRange.parts.filter(p => ['date-range-bar','date-range-bar-solid'].includes(p.name))) {
 const original = part.d;
 part.d = (t,s,r,w) => {
  const d = original(t,s,r,w);
  if (!r) return d;
  const x = Number(d.slice(d.indexOf('L')+1).split(' ')[0]);
  return `M10 19 L${Math.min(22,x)} 19`;
 };
}

// 2. CALENDAR VIEW — three cells, one route: contract in own column into own row lane, then shift left and extend along the row.
const COL=[8.3,14.3,20.3],ACT={light:.72,strong:.56};
const contract=i=>F([[.05*i,0],[.29+.05*i,1,'smooth']],[[.03*i,0],[.15+.03*i,1.08,'hit'],[.24+.03*i,1]]);
const extend=i=>F([[.36+.04*i,0],[.64+.04*i,1,'smooth']],[[.2+.03*i,0],[.36+.03*i,1.06,'hit'],[.5+.03*i,1]]);
const cell=i=>{const c=contract(i),e=extend(i);return (t,s,r)=>{
 const tt=r?(s?ACT.strong:ACT.light)-t:t,h=mix(9.5,2.4,c(tt,s)),u=e(tt,s),w=mix(3.4,15.4,u),x=mix(COL[i],8.3,u);
 const y=i===0?14.5:i===1?19.25-h/2:24-h;                     // A parks at its top, B at its centre, C at its bottom
 return rect(x,y,w,h,Math.min(1.2,w/2,h/2));};};
const calendarView=toggle('calendar-view','Calendar view',
 'Week columns contract into three squares, each already in its own row lane, then slide left and extend into month rows. Strong: a snappier contraction with a slight squash, then a fast extension with a bounded width overshoot. Reverse runs the same route backwards.',[
 ...leaf('leaf'),...bothFill('cell-a',cell(0)),...bothFill('cell-b',cell(1)),...bothFill('cell-c',cell(2))
],ACT.light,ACT.strong,'week','month',ACT.light,ACT.strong);

// 3. RESCHEDULE — fixed leaf, open tray (origin slot), fixed target dot; the event chip is a rigid pill.
const chipX=F([[0,0],[.1,0],[.56,8.9,'smooth']],[[0,0],[.12,0,'hold'],[.3,8.9,'hit'],[.42,8.9]]);
const chipY=F([[0,0],[.24,-3.4],[.56,-1.65,'smooth']],[[0,0],[.12,-3.4],[.3,-1.65,'hit'],[.37,-2.3],[.46,-1.65]]);
const reschedule=geometricRearm(make('reschedule','Reschedule',
 'The event chip lifts out of its open tray, crosses to the marked date and lands on it; the empty tray stays behind. Strong: quicker lift, fast drive, one landing hop.',[
 ...leaf('leaf'),
 ...tube('tray','M8.6 21 L8.6 22.8 L17.4 22.8 L17.4 21'),        // seated chip keeps 1.15 painted gap on three sides, open above
 ...bothFill('target',circle(22.25,17.3,1.3)),                   // fixed date guide, covered by the landed chip
 ...posed(bothFill('chip',rect(11.05,17.55,4.6,2.8,1.4)),rigid((t,s)=>({dx:chipX(t,s),dy:chipY(t,s)})))
],.6,.48));

// 4. ALARM CLOCK — open dial ring, two hands of distinct length, two legs; bells are D-shaped semicaps
// whose chord sits 0.6 outside the rim centreline and rocks on the rim point. Body only thumps as a follower.
const thump=F([[0,0],[.45,0]],[[0,0],[.16,0,'hold'],[.22,.45,'hit'],[.34,-.15],[.48,0]]);
const ring=F([[0,0],[.1,-6],[.42,0,'smooth']],[[0,0],[.06,4],[.16,-15,'hit'],[.28,11],[.4,-6],[.52,3],[.68,0]]);
const body=rigid((t,s)=>({dy:thump(t,s)}));
const bellD=(cx,cy,a,b)=>arc(cx,cy,2.2,a,b)+' Z';
const bell=(n,d,pivot,sign)=>posed([stroke(n,d),{name:n+'-solid',d,style:'solid',type:'stroke',width:2.6},fill(n+'-solid-core',d,'solid')],
 rigid((t,s)=>({dy:thump(t,s),angle:sign*ring(t,s),pivot})));
const alarmClock=make('alarm-clock','Alarm clock',
 'Light: one restrained outward strike of both bells. Strong: a small inward wind-up, a hard outward strike and a damped ring that decays to rest while the body thumps once on its feet.',[
 ...posed(tube('body',circle(16,17.5,8.5)),body),
 ...posed([fill('dial',circle(16,17.5,8.5),'tint')],body),
 ...posed(tube('minute','M16 17.5 L16 13'),body),               // tip cap 11.7 vs Solid inner dial 10.3
 ...posed(tube('hour','M16 17.5 L19.3 17.5'),body),
 ...posed(tube('leg-l','M12.2 25.2 L10.6 27.6'),body),           // starts on the rim centreline
 ...posed(tube('leg-r','M19.8 25.2 L21.4 27.6'),body),
 ...bell('bell-l',bellD(9.57,11.07,135,315),[10,11.5],1),
 ...bell('bell-r',bellD(22.43,11.07,225,405),[22,11.5],-1)
],.45,.7);

for(const i of [dateRange,calendarView,reschedule,alarmClock])i.duotoneFill=true;
export const icons=[dateRange,calendarView,reschedule,alarmClock];
