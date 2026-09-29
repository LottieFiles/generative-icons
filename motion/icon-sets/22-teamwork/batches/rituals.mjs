// Teamwork batch 8: standup, raise-hand, team-poll, decision-log, team-celebration. Claude first draft.
import {make,stroke,tube,bothFill,framed,circle,rect,rigid,curve,posed} from './helpers.mjs';

const dr=(L,S)=>(t,s)=>curve(t,s,L,S);
// Closed rounded torso: flat feet edge with r=.6 corners, shoulder arch on top.
const body=(x,hw,top,bot,r=.6)=>`M${x-hw+r} ${bot} L${x+hw-r} ${bot} C${x+hw-r*.45} ${bot} ${x+hw} ${bot-r*.45} ${x+hw} ${bot-r} L${x+hw} ${top+3.6} C${x+hw} ${top+1.4} ${x+hw*.55} ${top} ${x} ${top} C${x-hw*.55} ${top} ${x-hw} ${top+1.4} ${x-hw} ${top+3.6} L${x-hw} ${bot-r} C${x-hw} ${bot-r*.45} ${x-hw+r*.45} ${bot} ${x-hw+r} ${bot} Z`;

// 1. standup — middle speaker stands; flanks and floor hold.
function standup(){
 const person=(id,x)=>[...framed(id+'-head',circle(x,11,2.2)),...framed(id+'-body',body(x,2.6,17.4,22))];
 const rise=rigid((t,s)=>({dy:dr([[0,0],[.08,0],[.35,-1.8,'out'],[.6,-1.8],[.95,0]],
  [[0,0],[.06,0,'hold'],[.22,-4.2],[.34,-3.6],[.58,-3.6,'hold'],[1,0]])(t,s)}));
 const parts=[...person('left',7),...posed(person('speaker',16),rise),...person('right',25),...tube('floor','M5 26.5 L27 26.5')];
 return {...make('standup','Standup','The middle teammate stands up to give the daily update, then sits back.',parts,.95,1),duotoneFill:true};
}

// 2. raise-hand — arm and palm rotate about the shoulder; root cap buried in body (named join: shoulder).
function raiseHand(){
 const swing=rigid((t,s)=>({angle:dr([[0,0],[.06,0],[.42,-75,'out'],[.7,-75],[1.05,0]],
  [[0,0],[.05,0,'hold'],[.09,8],[.3,-120],[.4,-113],[.75,-113,'hold'],[1.1,0]])(t,s),pivot:[16.5,19.5]}));
 const parts=[...framed('head',circle(11.5,10.6,2.4)),...framed('body',body(11.5,4,17.4,25)),
  ...posed(tube('arm','M16.5 19.5 L19.04 23.82'),swing),...posed(bothFill('palm',circle(19.5,24.6,1.7)),swing)];
 return {...make('raise-hand','Raise hand','A teammate raises a hand to ask for the floor, holds, then lowers it.',parts,1.05,1.1),duotoneFill:true};
}

// 3. team-poll — bars grow from a short start to the same result in both profiles; bullets fixed.
function teamPoll(){
 const bar=(id,y,L,S)=>tube(id,(t,s)=>`M11 ${y} L${dr(L,S)(t,s)} ${y}`);
 const parts=[...bothFill('bullet-a',circle(7,9,1.2)),...bothFill('bullet-b',circle(7,16,1.2)),...bothFill('bullet-c',circle(7,23,1.2)),
  ...bar('bar-a',9,[[.05,14],[.75,24,'out']],[[.04,14],[.34,24]]),
  ...bar('bar-b',16,[[.12,14],[.7,18,'out']],[[.1,14],[.3,18]]),
  ...bar('bar-c',23,[[.2,14],[.85,21,'out']],[[.16,14],[.46,21]])];
 return {...make('team-poll','Team poll','Votes arrive and the option bars fill to the team result, which holds.',parts,.9,.55,['bar-a','bar-b','bar-c']),duotoneFill:false};
}

// 4. decision-log — stamp descends onto the ledger, the check mark is recorded and stays.
function decisionLog(){
 const press=rigid((t,s)=>({dy:dr([[0,0],[.06,0],[.34,4,'in'],[.4,4],[.7,0,'out']],
  [[0,0],[.05,0,'hold'],[.14,-1.2],[.26,4,'hit'],[.5,0]])(t,s)}));
 const prog=dr([[.44,0],[.72,1,'out']],[[.3,0],[.46,1]]);
 const AB=Math.hypot(2.2,2.2),BC=Math.hypot(3.8,4.1);
 const markD=(t,s)=>{const L=prog(t,s)*(AB+BC);if(L<=AB){const u=L/AB,x=7+2.2*u,y=18.8+2.2*u;return `M7 18.8 L${x} ${y} L${x} ${y}`;}const u=(L-AB)/BC;return `M7 18.8 L9.2 21 L${9.2+3.8*u} ${21-4.1*u}`;};
 const parts=[...posed(tube('handle','M10 5 L10 8'),press),               // handle-root buried in base rim (named join)
  ...posed(framed('base',rect(5.5,8,9,4.5,1)),press),                        // flat contact plane at y=12.5 → 16.5 when seated
  ...tube('row-a','M18.5 17.5 L27 17.5'),...tube('row-b','M5 24.7 L27 24.7'),
  ...tube('mark',markD,{visibility:(t,s)=>prog(t,s)>.02?1:0})];
 return {...make('decision-log','Decision log','The stamp strikes the log entry; the recorded check stays.',parts,.75,.55,['mark']),duotoneFill:true};
}

// 5. team-celebration — glasses tilt to one authored contact angle where inner rims meet.
// Strong adds outward wind-up and a damped recoil; it never passes the contact pose.
function teamCelebration(){
 const bowl=cx=>`M${cx-2.9} 9.4 L${cx+2.9} 9.4 C${cx+3.3} 9.4 ${cx+3.5} 9.6 ${cx+3.5} 10 C${cx+3.5} 15 ${cx+2} 19.6 ${cx} 19.6 C${cx-2} 19.6 ${cx-3.5} 15 ${cx-3.5} 10 C${cx-3.5} 9.6 ${cx-3.3} 9.4 ${cx-2.9} 9.4 Z`;
 const C=8.3; // contact angle: inner rim corners (cx±3.5,10) reach x≈14.87/17.13
 const L=[[0,0],[.06,0],[.4,C,'out'],[.55,C],[.95,0]];
 const S=[[0,0],[.06,0,'hold'],[.18,-6],[.32,C,'hit'],[.42,4.2],[1,0]];
 const glass=(id,cx,sign)=>{const tilt=rigid((t,s)=>({angle:sign*dr(L,S)(t,s),pivot:[cx,26.5]}));
  return posed([...framed(id+'-bowl',bowl(cx)),
   ...tube(id+'-stem',`M${cx} 19 L${cx} 26.5`),        // stem-root buried in bowl, stem-foot T-join on base (named joins)
   ...tube(id+'-base',`M${cx-3} 26.5 L${cx+3} 26.5`)],tilt);};
 const parts=[...glass('left',9,1),...glass('right',23,-1)];
 return {...make('team-celebration','Team celebration','Two glasses swing in and clink for the shared win, then settle upright.',parts,.95,1),duotoneFill:true};
}

export const icons=[standup(),raiseHand(),teamPoll(),decisionLog(),teamCelebration()];
