// Preferences batch C: sound-toggle, battery-saver, mobile-data, haptic-feedback, reset-settings.
import {make,toggle,F,P,U,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,arc,hole,bothFill,clamp01,arrowHead,line} from './helpers.mjs';

// ---------- SOUND TOGGLE ----------
// A speaker (box x5–10, cone to x16) with two sound arcs. Muting fades the arcs out (sliding into
// the cone) and then a cross fades in at 23.1,15.9; the two never overlap at partial opacity.
const SPK=polygon([[5,12.5],[10,12.5],[16,7.5],[16,24.5],[10,19.5],[5,19.5]],[.8,.3,1,1,.3,.8]);
const waveShow=(t,s,r)=>r?P([[0,0],[.2,0],[.4,1,'out']],[[0,0],[.16,0],[.26,1,'out']])(t,s):P([[0,1],[.04,1],[.2,0,'in']],[[0,1],[.1,0,'in']])(t,s);
const waveSlide=(t,s,r)=>-1.2*(1-waveShow(t,s,r));
const crossShow=(t,s,r)=>r?P([[0,1],[.14,0,'in']],[[0,1],[.1,0,'in']])(t,s):P([[0,0],[.22,0],[.4,1,'out']],[[0,0],[.12,0],[.22,1,'out']])(t,s);
const crossScale=(t,s,r)=>r?1:F([[0,.8],[.22,.8],[.4,1,'smooth']],[[0,.7],[.12,.7],[.22,1.2,'hit'],[.34,1]])(t,s);
const recoil=(t,s,r)=>r?0:F([[0,0],[.4,0]],[[0,0],[.06,-.7,'hit'],[.16,.2],[.24,0]])(t,s);
const XC=[23.1,15.9];
const crossPose=(x,y,t,s,r)=>{const k=crossScale(t,s,r);return [XC[0]+(x-XC[0])*k,XC[1]+(y-XC[1])*k];};
const soundToggle=toggle('sound-toggle','Sound toggle',
 'Light: the sound arcs fade back into the speaker and a cross fades in beside it. Strong: the speaker recoils 0.7 as the arcs cut out, then the cross snaps in 20% large with a stop. Reverse: the cross fades and the arcs return.',[
 ...posed(framed('speaker',SPK),rigid((t,s,r)=>({dx:recoil(t,s,r)}))),
 ...shown(posed([...tube('wave-0',arc(16,16,5.5,-40,40)),...tube('wave-1',arc(16,16,9.5,-40,40))],rigid((t,s,r)=>({dx:waveSlide(t,s,r)}))),waveShow),
 ...shown(posed([...tube('cross-0','M20.6 13.4 L25.6 18.4'),...tube('cross-1','M25.6 13.4 L20.6 18.4')],crossPose),crossShow)
],.4,.34,'on','muted',.4,.26);
soundToggle.duotoneFill=true;

// ---------- BATTERY SAVER ----------
// A battery (x4–25, y10–22, stub to 27.3; Solid ring) whose low charge block gives way to a leaf.
// The block fades first, then the leaf scales in from 85% (Light) or pops in past full (Strong).
const BAT=rect(4,10,21,12,2.5);
const LEAF='M10.4 18.6 C10.4 14.6 13.8 13.2 18.6 13.4 C18.2 17.8 15 18.9 10.4 18.6 Z',LC=[14.5,16];
const blockShow=(t,s,r)=>r?P([[0,0],[.16,0],[.36,1,'out']],[[0,0],[.14,0],[.24,1,'out']])(t,s):P([[0,1],[.16,0,'in']],[[0,1],[.08,0,'in']])(t,s);
const leafShow=(t,s,r)=>r?P([[0,1],[.12,0,'in']],[[0,1],[.1,0,'in']])(t,s):P([[0,0],[.2,0],[.36,1,'out']],[[0,0],[.1,0],[.18,1,'out']])(t,s);
const leafScale=(t,s,r)=>r?F([[0,1],[.12,.85,'smooth']],[[0,1],[.1,.7]])(t,s):F([[0,.85],[.2,.85],[.44,1,'smooth']],[[0,.7],[.1,.7],[.22,1.1,'hit'],[.34,1]])(t,s);
const leafPose=(x,y,t,s,r)=>{const k=leafScale(t,s,r);return [LC[0]+(x-LC[0])*k,LC[1]+(y-LC[1])*k];};
const batterySaver=toggle('battery-saver','Battery saver',
 'Light: the low charge block fades out and a leaf grows into the battery. Strong: the block drops away and the leaf pops in 10% past full with a stop before settling. Reverse restores the charge block.',[
 ...framed('body',BAT,hole(BAT,rect(6.6,12.6,15.8,6.8,1.2))),...tube('stub','M25 16 L27.3 16'),
 ...shown(bothFill('block',rect(7.8,13.4,3.8,5.2,.9)),blockShow),
 ...shown(posed([fill('leaf-interior',LEAF,'tint'),stroke('leaf',LEAF),fill('leaf-solid',LEAF,'solid'),stroke('rib','M11.2 18 L17.6 14')],leafPose),leafShow)
],.44,.34,'normal','saving',.36,.24);
batterySaver.duotoneFill=true;

// ---------- MOBILE DATA ----------
// Transfer arrows (up at x6, down at x11.5) beside four signal bars (x15.5–27.5, feet y26).
// Light: the arrows slide 1 and the bars bob in turn. Strong: the arrows dart 2.4 and the bars
// drop to a third, then spring up in sequence past full height.
const BARS=[[15.5,5.5],[19.5,9.5],[23.5,13.5],[27.5,17.5]];
const barG=i=>F([[0,1],[.05*i,1],[.14+.05*i,1.14],[.34+.05*i,1,'smooth']],[[0,1],[.08,.35],[.16+.05*i,.35],[.28+.05*i,1.12],[.4+.05*i,1]]);
const barD=i=>(t,s)=>`M${BARS[i][0]} 26 L${BARS[i][0]} ${26-BARS[i][1]*barG(i)(t,s)}`;
const shift=F([[0,0],[.14,1],[.4,0,'smooth']],[[0,0],[.04,-.3],[.14,2.4,'hit'],[.26,2],[.44,0]]);
const upPose=rigid((t,s)=>({dy:-shift(t,s)})),downPose=rigid((t,s)=>({dy:shift(t,s)}));
const mobileData=make('mobile-data','Mobile data',
 'Light: the transfer arrows slide apart by 1 and the signal bars bob taller one after another. Strong: the arrows dart 2.4 apart with a stop while the bars collapse to a third and spring back up in sequence past full.',[
 ...posed([...tube('up',`M6 22 L6 12`),...tube('up-head',arrowHead([6,12],[0,-1],1.8,35))],upPose),
 ...posed([...tube('down',`M11.5 12 L11.5 22`),...tube('down-head',arrowHead([11.5,22],[0,1],1.8,35))],downPose),
 ...BARS.flatMap((_,i)=>tube('bar-'+i,barD(i)))
],.5,.6);
mobileData.duotoneFill=false;

// ---------- HAPTIC FEEDBACK ----------
// A phone (x11.5–20.5, y5.5–26.5) buzzes side to side at 11 Hz under a shaped envelope while two
// fixed vibration arcs (r9.8) kick outward. Strong shakes 1.4 with a 4° twist; Light 0.6.
const HPH=rect(11.5,5.5,9,21,2.2);
const env=P([[0,0],[.04,1],[.28,1],[.42,0,'smooth']],[[0,0],[.03,1],[.36,1],[.52,0,'smooth']]);
const buzz=(t,s)=>env(t,s)*Math.sin(2*Math.PI*11*t);
const hapticPose=rigid((t,s)=>({dx:(s?1.4:.6)*buzz(t,s),angle:(s?4:1.5)*buzz(t,s),pivot:[16,16]}));
const kick=(t,s)=>(s?1.2:.4)*env(t,s);
const hapticFeedback=make('haptic-feedback','Haptic feedback',
 'Light: the phone buzzes 0.6 side to side for a third of a second and the vibration arcs ease out 0.4. Strong: it rattles 1.4 with a 4° twist for half a second while the arcs kick 1.2 outward, then everything settles.',[
 ...posed([...framed('phone',HPH),stroke('speaker','M14.5 9.2 L17.5 9.2')],hapticPose),
 ...posed(tube('arc-l',arc(16,16,9.8,150,210)),rigid((t,s)=>({dx:-kick(t,s)}))),
 ...posed(tube('arc-r',arc(16,16,9.8,-30,30)),rigid((t,s)=>({dx:kick(t,s)})))
],.42,.52);
hapticFeedback.duotoneFill=true;

// ---------- RESET SETTINGS ----------
// A counter-clockwise arrow ring (r11, open at the bottom) around two sliders (y12.5/19.5).
// Resetting slides both knobs back to centre while the ring turns a full turn anticlockwise.
const RING=arc(16,16,11,60,-240),RE=[16+11*Math.cos(120*Math.PI/180),16+11*Math.sin(120*Math.PI/180)];
const RHEAD=arrowHead(RE,[Math.sin(120*Math.PI/180),-Math.cos(120*Math.PI/180)],2.5,34);
const reset=U([[0,0],[.08,0],[.5,1,'smooth']],[[0,0],[.06,-.06],[.26,1.15,'hit'],[.36,.96],[.44,1]],[[0,0],[.06,0],[.5,1,'smooth']],[[0,0],[.06,0],[.3,1.1,'hit'],[.4,.98],[.48,1]]);
const ringSpin=(t,s,r)=>r?0:F([[0,0],[.04,0],[.52,-360,'smooth']],[[0,0],[.06,20],[.36,-360,'hit'],[.44,-352],[.5,-360]])(t,s);
const resetSettings=toggle('reset-settings','Reset settings',
 'Light: the two slider knobs glide back to centre while the ring makes one anticlockwise turn. Strong: the ring winds 20° back and spins fast to a stop while the knobs snap 15% past centre and settle. Reverse slides the knobs back to their custom positions.',[
 ...posed([...tube('ring',RING),...tube('ring-head',RHEAD)],rigid((t,s,r)=>({angle:ringSpin(t,s,r),pivot:[16,16]}))),
 ...tube('track-0','M10 12.5 L22 12.5'),...tube('track-1','M10 19.5 L22 19.5'),
 ...posed(bothFill('knob-0',circle(19.5,12.5,2.3)),rigid((t,s,r)=>({dx:-3.5*reset(t,s,r)}))),
 ...posed(bothFill('knob-1',circle(12,19.5,2.3)),rigid((t,s,r)=>({dx:4*reset(t,s,r)})))
],.52,.5,'custom','default',.5,.48);
resetSettings.duotoneFill=false;

export const icons=[soundToggle,batterySaver,mobileData,hapticFeedback,resetSettings];
