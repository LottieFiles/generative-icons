// Preferences batch B: quiet-hours, privacy-shield, shortcut-key, mouse, auto-rotate.
import {make,toggle,F,U,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,arc,hole,capsule,bothFill,clamp01,arrowHead} from './helpers.mjs';

// ---------- DO NOT DISTURB ----------
// A bell (crown 16,7; lip y21.5 x8–24; clapper arc under the lip) is crossed by a slash that draws
// from top-left to bottom-right. bellD(i) is the same construction inset by i, so Solid is a 2.6 ring.
const bellD=i=>{const lipY=21.5-i,lx=8+i,rx=24-i,sx=22.2-i,slx=9.8+i,topY=7+i,shY=14.5;
 return `M${lx} ${lipY} L${rx} ${lipY} C${rx} ${lipY-2.4} ${sx} ${shY+3} ${sx} ${shY} C${sx} ${topY+3.2} ${16+3.4-i*.5} ${topY} 16 ${topY} C${16-3.4+i*.5} ${topY} ${slx} ${topY+3.2} ${slx} ${shY} C${slx} ${shY+3} ${lx} ${lipY-2.4} ${lx} ${lipY} Z`;};
const SA=[8.5,7.5],SB=[23.5,24.5];
const slashU=U([[0,0],[.05,0],[.45,1,'smooth']],[[0,0],[.2,0],[.36,1,'hit']],[[0,0],[.04,0],[.4,1,'smooth']],[[0,0],[.16,1,'hit']]);
const slashEnd=(t,s,r)=>{const w=slashU(t,s,r);return [SA[0]+(SB[0]-SA[0])*w,SA[1]+(SB[1]-SA[1])*w];};
const bellSwing=(t,s,r)=>r?F([[0,0],[.1,0],[.24,-4],[.4,0,'smooth']],[[0,0],[.16,0],[.24,12],[.36,-8],[.46,3],[.54,0]])(t,s)
 :F([[0,0],[.15,-5],[.45,0,'smooth']],[[0,0],[.08,-12],[.2,10],[.28,-4],[.34,0,'hit']])(t,s);
const bellPose=rigid((t,s,r)=>({angle:bellSwing(t,s,r),pivot:[16,7]}));
const doNotDisturb=toggle('quiet-hours','Quiet hours',
 'Light: the bell nods 5° while a slash draws across it. Strong: the bell rings 12° each way, the slash strikes through it with a stop and the bell freezes. Reverse: the slash wipes back and the bell rings once, free again.',[
 ...posed([...framed('bell',bellD(0),hole(bellD(0),bellD(2.6))),...tube('clapper',arc(16,21.2,2.8,25,155))],bellPose),
 ...shown([stroke('slash',(t,s,r)=>`M${SA} L${slashEnd(t,s,r)}`),fill('slash-solid',(t,s,r)=>capsule(SA,slashEnd(t,s,r),2.6),'solid')],(t,s,r)=>slashU(t,s,r)>.04?1:0)
],.45,.36,'on','muted',.4,.54);
doNotDisturb.duotoneFill=true;

// ---------- PRIVACY SHIELD ----------
// A shield (top 16,4.5; sides x6/26; bottom 16,27.5) holds an eye whose upper lid drops onto the
// lower lid. shieldD(i) is the same construction inset by i for the Solid ring.
const shieldD=i=>`M16 ${4.5+1.4*i} L${25.2-i} ${7.7+.5*i} C${25.7-i} ${7.9+.5*i} ${26-i} ${8.3+.5*i} ${26-i} ${8.8+.5*i} L${26-i} 15 C${26-i} ${21.3-.8*i} ${21.6-.6*i} ${25.7-1.2*i} 16.3 ${27.4-1.4*i} C16.1 ${27.5-1.4*i} 15.9 ${27.5-1.4*i} 15.7 ${27.4-1.4*i} C${10.4+.6*i} ${25.7-1.2*i} ${6+i} ${21.3-.8*i} ${6+i} 15 L${6+i} ${8.8+.5*i} C${6+i} ${8.3+.5*i} ${6.3+i} ${7.9+.5*i} ${6.8+i} ${7.7+.5*i} Z`;
const open=(t,s,r)=>r?F([[0,0],[.06,0],[.42,1,'smooth']],[[0,0],[.06,0],[.2,1.18,'hit'],[.3,.96],[.38,1]])(t,s)
 :F([[0,1],[.08,1],[.42,0,'smooth']],[[0,1],[.1,1.15],[.24,0,'hit'],[.3,0]])(t,s);
const eyeD=(t,s,r)=>{const k=open(t,s,r),top=15.5+3.7*(1-2*k);return `M11 15.5 C13 ${top} 19 ${top} 21 15.5 C19 19.2 13 19.2 11 15.5 Z`;};
const shieldDip=(t,s,r)=>r?0:F([[0,0],[.42,0]],[[0,0],[.24,0,'hold'],[.3,.5,'hit'],[.4,-.12],[.48,0]])(t,s);
const shieldPose=rigid((t,s,r)=>({dy:shieldDip(t,s,r)}));
const privacyShield=toggle('privacy-shield','Privacy shield',
 'Light: the eye inside the shield closes, its upper lid settling onto the lower lid. Strong: the eye widens 15%, snaps shut with a stop and the shield dips half a unit. Reverse: the eye opens; Strong pops it open past full.',[
 ...posed([...framed('shield',shieldD(0),hole(shieldD(0),shieldD(2.6))),stroke('eye',eyeD),{name:'eye-solid',d:eyeD,style:'solid',type:'stroke',width:2.6},
  ...bothFill('pupil',circle(16,15.5,1.7),{visibility:(t,s,r)=>clamp01((open(t,s,r)-.55)/.25)})],shieldPose)
],.42,.48,'visible','private',.42,.38);
privacyShield.duotoneFill=true;

// ---------- SHORTCUT KEY ----------
// A keycap (x7–25, y4.5–21.5) on a fixed base skirt (bottom y25) carrying a filled bolt. The cap
// presses down 1.2 (Light) or 2.6 with a stop, a brief hold and a sprung return (Strong).
const CAP=rect(7,4.5,18,17,3.2),BASE='M7 18.5 L7 22 C7 23.7 8.3 25 10 25 L22 25 C23.7 25 25 23.7 25 22 L25 18.5';
const BOLT=polygon([[17.2,9],[13.4,14.6],[16.2,14.6],[14.8,18.2],[18.6,12.9],[15.8,12.9]],.5);
const press=F([[0,0],[.12,1.2],[.4,0,'smooth']],[[0,0],[.1,2.6,'hit'],[.18,2.6],[.3,-.4],[.42,0]]);
const capPose=rigid((t,s)=>({dy:press(t,s)}));
const shortcutKey=make('shortcut-key','Shortcut key',
 'Light: the keycap presses down 1.2 units and eases back up. Strong: it slams down 2.6 with a stop, holds pressed for a beat, springs 0.4 above rest and settles.',[
 ...tube('base',BASE),
 ...posed([...framed('cap',CAP,hole(CAP,rect(9.6,7.1,12.8,11.8,1.6))),...bothFill('bolt',BOLT)],capPose)
],.4,.42);
shortcutKey.duotoneFill=true;

// ---------- MOUSE ----------
// A mouse body (x9.5–22.5, y4.5–27.5, Solid ring) with a divider at y14.5 and a filled scroll wheel
// pill above it. Light nudges the wheel; Strong lifts the mouse, clicks it down and scrolls.
const BODY=rect(9.5,4.5,13,23,6.5);
const clickY=F([[0,0],[.4,0]],[[0,0],[.08,-.6],[.2,.4,'hit'],[.3,-.1],[.4,0]]);
const scroll=F([[0,0],[.14,.9],[.4,0,'smooth']],[[0,0],[.08,0],[.2,1.2,'hit'],[.32,-.2],[.42,0]]);
const bodyPose=rigid((t,s)=>({dy:clickY(t,s)}));
const wheelPose=rigid((t,s)=>({dy:clickY(t,s)+scroll(t,s)}));
const mouse=make('mouse','Mouse',
 'Light: the scroll wheel rolls down 0.9 and back. Strong: the mouse lifts 0.6, clicks down with a stop while the wheel scrolls 1.2, and both settle.',[
 ...posed([...framed('body',BODY,hole(BODY,rect(12.1,7.1,7.8,17.8,3.9))),...tube('divider','M9.5 14.5 L22.5 14.5')],bodyPose),
 ...posed(bothFill('wheel',capsule([16,7.8],[16,10.2],2)),wheelPose)
],.4,.42);
mouse.duotoneFill=true;

// ---------- AUTO-ROTATE ----------
// A phone (8×15 about 16,16) turns 90° clockwise from portrait to landscape inside two fixed
// clockwise arrows (r12.2). Its swept corners (r8.5) keep 1.2 clear of the arrows at Bold.
const PHONE=rect(12,8.5,8,15,1.8);
const turn=U([[0,0],[.06,0],[.5,1,'smooth']],[[0,0],[.08,-.09],[.3,1.09,'hit'],[.4,.98],[.48,1]]);
const rotHop=(t,s)=>F([[0,0],[.5,0]],[[0,0],[.08,.5],[.22,-1.2],[.34,0,'hit'],[.44,0]])(t,s);
const phonePose=rigid((t,s,r)=>({angle:90*turn(t,s,r),dy:rotHop(t,s),pivot:[16,16]}));
const arrowArc=(a,b)=>{const R=12.2,E=[16+R*Math.cos(b*Math.PI/180),16+R*Math.sin(b*Math.PI/180)],dir=[-Math.sin(b*Math.PI/180),Math.cos(b*Math.PI/180)];return [arc(16,16,R,a,b),arrowHead(E,dir,2.4,32)];};
const [ARC_A,HEAD_A]=arrowArc(22,62),[ARC_B,HEAD_B]=arrowArc(202,242);
const autoRotate=toggle('auto-rotate','Auto-rotate',
 'Light: the phone turns a quarter turn clockwise into landscape. Strong: it winds back 8°, whips through the turn past 90° with a stop and settles while hopping 1.2. Reverse turns it back to portrait.',[
 ...tube('arrow-a',ARC_A),...tube('head-a',HEAD_A),...tube('arrow-b',ARC_B),...tube('head-b',HEAD_B),
 ...posed([...framed('phone',PHONE),stroke('speaker','M14.5 12 L17.5 12')],phonePose)
],.5,.48,'portrait','landscape');
autoRotate.duotoneFill=true;

export const icons=[doNotDisturb,privacyShield,shortcutKey,mouse,autoRotate];
