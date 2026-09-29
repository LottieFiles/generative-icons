// Preferences batch D: auto-update, startup-apps, pointer-speed, advanced-settings, trackpad.
import svgpath from 'svgpath';
import {make,F,P,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,arc,arcN,hole,bothFill,arrowHead,L} from './helpers.mjs';

// ---------- AUTO-UPDATE ----------
// A clockwise arrow ring (r11, open on the left) around a down arrow. Light taps the arrow down;
// Strong spins the ring a full turn while the arrow drops with a stop and returns.
const URING=arc(16,16,11,210,510),UE=[16+11*Math.cos(150*Math.PI/180),16+11*Math.sin(150*Math.PI/180)];
const UHEAD=arrowHead(UE,[-Math.sin(150*Math.PI/180),Math.cos(150*Math.PI/180)],2.5,34);
const drop=F([[0,0],[.16,1.6],[.46,0,'smooth']],[[0,0],[.06,-.4],[.2,2.4,'hit'],[.3,2],[.44,0]]);
const uspin=F([[0,0],[.46,0]],[[0,0],[.48,360,'hit'],[.52,360]]);
const autoUpdate=make('auto-update','Auto-update',
 'Light: the update arrow taps down 1.6 inside its ring and rises back. Strong: the ring whips through a full clockwise turn while the arrow drops 2.4 with a stop and eases back up.',[
 ...posed([...tube('ring',URING),...tube('ring-head',UHEAD)],rigid((t,s)=>({angle:uspin(t,s),pivot:[16,16]}))),
 ...posed([...tube('shaft','M16 10 L16 20.6'),...tube('head','M12.4 17 L16 20.6 L19.6 17')],rigid((t,s)=>({dy:drop(t,s)})))
],.46,.52);
autoUpdate.duotoneFill=false;

// ---------- STARTUP APPS ----------
// A rocket (nose 16,6.2; base y23; fins to y25.5) with a window counter. It lifts off and settles
// back; exhaust dashes hang from the base while it rises and fade before it lands.
const ROCKET='M16 6.2 C19 9.5 20.5 14 20.5 18.5 L20.5 23 L11.5 23 L11.5 18.5 C11.5 14 13 9.5 16 6.2 Z';
const FIN_L='M11.5 18.5 L7.5 23 L7.5 25.5 L11.5 23.5',FIN_R='M20.5 18.5 L24.5 23 L24.5 25.5 L20.5 23.5';
const FIN_LS=polygon([[12.3,18],[7.5,23],[7.5,25.5],[12.3,23.5]],[.3,.7,.7,.3]),FIN_RS=polygon([[19.7,18],[24.5,23],[24.5,25.5],[19.7,23.5]],[.3,.7,.7,.3]);
const rise=F([[0,0],[.2,-1.4],[.56,0,'smooth']],[[0,0],[.08,.6],[.24,-2.8],[.34,-2.8],[.5,0,'hit'],[.58,-.3],[.66,0]]);
const exShow=P([[0,0],[.06,0],[.14,1],[.3,1],[.44,0]],[[0,0],[.08,0],[.14,1],[.36,1],[.5,0]]);
const exExt=P([[0,0],[.14,.3],[.44,0]],[[0,0],[.14,.5],[.5,0]]);
const exD=(x,len)=>(t,s)=>{const dy=rise(t,s);return `M${x} ${26.2+dy} L${x} ${26.2+len+dy*.4+exExt(t,s)}`;};
const rocketPose=rigid((t,s)=>({dy:rise(t,s)}));
const startupApps=make('startup-apps','Startup apps',
 'Light: the rocket lifts 1.4 on a short exhaust and eases back down. Strong: it crouches 0.6, launches 2.8 with a longer exhaust, hangs, drops back with a stop and settles.',[
 ...posed([stroke('body',ROCKET),fill('body-interior',ROCKET,'tint'),fill('body-solid',hole(ROCKET,circle(16,17.5,1.6)),'solid'),fill('window',circle(16,17.5,1.6),'outline'),
  stroke('fin-l',FIN_L),fill('fin-l-solid',FIN_LS,'solid'),stroke('fin-r',FIN_R),fill('fin-r-solid',FIN_RS,'solid')],rocketPose),
 ...shown([...tube('exhaust-0',exD(13,1.4)),...tube('exhaust-1',exD(16,1.8)),...tube('exhaust-2',exD(19,1.4))],exShow)
],.56,.66);
startupApps.duotoneFill=true;

// ---------- POINTER SPEED ----------
// A pointer arrow (x12–24.2) with three motion lines behind it. Light glides 1.6; Strong winds
// back, darts 4 with a stop and drifts home while the lines stretch after it.
const CURSOR=polygon([[12,6],[12,21],[15.8,17.8],[18.8,24.2],[22.2,22.6],[19.2,16.3],[24.2,16.3]],[.7,.7,.5,.6,.6,.5,.7]);
const dart=F([[0,0],[.16,1.6],[.5,0,'smooth']],[[0,0],[.06,-.5],[.18,4,'hit'],[.3,3.4],[.5,0]]);
const trailD=y=>(t,s)=>{const d=dart(t,s);return `M${4+d*.3} ${y} L${8+d*.6} ${y}`;};
const pointerSpeed=make('pointer-speed','Pointer speed',
 'Light: the pointer glides 1.6 to the right and back while its motion lines stretch slightly. Strong: it draws back half a unit, darts 4 with a stop and drifts home as the lines stretch after it.',[
 ...posed(framed('cursor',CURSOR),rigid((t,s)=>({dx:dart(t,s)}))),
 ...[9.5,13.5,17.5].flatMap((y,i)=>tube('trail-'+i,trailD(y)))
],.5,.5);
pointerSpeed.duotoneFill=true;

// ---------- ADVANCED SETTINGS ----------
// An open-end wrench (head r6, notch 4.2 wide, handle 12×4.6) at 45°, scaled 1.15 about its head
// at 19.5,12.7. It turns about the head: 14° (Light) or a 10° wind-up and 28° drive (Strong).
const WRENCH=(()=>{const R=6,HW=2.3,NW=2.1,HL=12,nx=Math.sqrt(R*R-NW*NW),hx=-Math.sqrt(R*R-HW*HW),deg=(y,x)=>Math.atan2(y,x)*180/Math.PI;
 const a0=deg(-NW,nx),a1=deg(-HW,hx),a2=deg(HW,hx),a3=deg(NW,nx);
 const d=arcN(0,0,R,a0,a1,2)+` L${-HL} ${-HW} `+L(arcN(-HL,0,HW,-90,-270,2))+` L${hx} ${HW} `+L(arcN(0,0,R,a2,a3,2))+` L1.4 ${NW} L1.4 ${-NW} Z`;
 return svgpath(d).scale(1.15).rotate(-45).translate(19.5,12.7).toString();})();
const wrenchTurn=F([[0,0],[.2,-14],[.5,0,'smooth']],[[0,0],[.08,10],[.24,-28,'hit'],[.34,-21],[.44,-25],[.54,0]]);
const advancedSettings=make('advanced-settings','Advanced settings',
 'Light: the wrench turns 14° about its head and eases back. Strong: it winds 10° the other way, drives 28° with a stop, rocks on the nut and returns.',[
 ...posed(framed('wrench',WRENCH),rigid((t,s)=>({angle:wrenchTurn(t,s),pivot:[19.5,12.7]})))
],.5,.54);
advancedSettings.duotoneFill=true;

// ---------- TRACKPAD ----------
// A trackpad (x4–28, y7–25, Solid ring) with a button divider at y20.2 and two finger dots that
// swipe right. Light glides 2.5; Strong draws back and swipes 6 with a stop before returning.
const PAD=rect(4,7,24,18,3);
const swipe=F([[0,0],[.2,2.5],[.5,0,'smooth']],[[0,0],[.06,-.6],[.2,6,'hit'],[.3,5.4],[.52,0]]);
const fingerPose=rigid((t,s)=>({dx:swipe(t,s)}));
const trackpad=make('trackpad','Trackpad',
 'Light: two fingertips glide 2.5 to the right across the pad and drift back. Strong: they draw back 0.6, swipe 6 with a stop and return.',[
 ...framed('pad',PAD,hole(PAD,rect(6.6,9.6,18.8,12.8,1.4))),...tube('divider','M4 20.2 L28 20.2'),
 ...posed([...bothFill('finger-0',circle(11.5,14.5,1.9)),...bothFill('finger-1',circle(16.5,14.5,1.9))],fingerPose)
],.5,.52);
trackpad.duotoneFill=true;

export const icons=[autoUpdate,startupApps,pointerSpeed,advancedSettings,trackpad];
