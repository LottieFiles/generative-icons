// Music & audio batch C: mute-audio, crossfade, equalizer-preset, audio-jack, spatial-audio, tuning-fork.
import {make,toggle,F,G,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,holes,bothFill,arcN,capsule,clamp01} from './helpers.mjs';

// ---------- MUTE AUDIO (stateful: Sound on → Muted) ----------
// Speaker (box + cone, x4.5–15.5) with two sound arcs to the right. Muting fades the arcs, then an X draws
// from its centre at (24.2,16). Strong draws the X past full size and snaps back while the speaker recoils.
const speaker=polygon([[4.5,12.5],[9.5,12.5],[15.5,7.5],[15.5,24.5],[9.5,19.5],[4.5,19.5]],[1,.6,1.2,1.2,.6,1]);
const waveAlpha=G([[0,1],[.04,1],[.22,0,'out']],[[0,1],[.03,1],[.13,0]],[[0,0],[.24,0],[.52,1,'out']],[[0,0],[.14,0],[.4,1]]);
const xSize=G([[0,0],[.18,0],[.46,1,'out']],[[0,0],[.1,0],[.24,1.15],[.34,.95,'hit'],[.44,1]],[[0,1],[.04,1],[.28,0,'in']],[[0,1],[.03,1],[.17,0]]);
const recoil=G([[0,0],[.4,0]],[[0,0],[.1,0],[.24,-.7,'hit'],[.42,0]],[[0,0],[.3,0]],[[0,0],[.3,0]]);
const cross=(t,s,r,sign)=>{const k=2.4*xSize(t,s,r);return `M${24.2-k} ${16-sign*k} L${24.2+k} ${16+sign*k}`;};
const muteAudio=toggle('mute-audio','Mute audio',
 'Light: the sound arcs fade and an X draws beside the speaker. Strong: the arcs cut instantly, the X snaps out past full size and settles while the speaker recoils. Reverse shrinks the X and restores the arcs.',[
 ...posed([fill('speaker-interior',speaker,'tint'),stroke('speaker',speaker),fill('speaker-solid',speaker,'solid')],rigid((t,s,r)=>({dx:recoil(t,s,r)}))),
 ...shown([...tube('wave-0',arcN(15.5,16,6.5,-42,42,2)),...tube('wave-1',arcN(15.5,16,10,-42,42,2))],waveAlpha),
 ...shown([...tube('cross-0',(t,s,r)=>cross(t,s,r,1)),...tube('cross-1',(t,s,r)=>cross(t,s,r,-1))],(t,s,r)=>xSize(t,s,r)<.08?0:1)
],.46,.44,'Sound on','Muted',.52,.4);
muteAudio.duotoneFill=true;

// ---------- CROSSFADE (stateful: Track A → Track B) ----------
// Two fade curves cross between x5.5 and 26.5 (y7–19). A fader knob rides a track at y25 from x8 (A) to x24 (B);
// curve A dims as curve B brightens.
const knobX=G([[0,8],[.06,8],[.62,24,'io']],[[0,8],[.05,7.2],[.32,24.9],[.44,23.6,'hit'],[.58,24]],
 [[0,24],[.06,24],[.58,8,'io']],[[0,24],[.05,24.8],[.3,7.3],[.42,8.3,'hit'],[.54,8]]);
const mixU=(t,s,r)=>clamp01((knobX(t,s,r)-8)/16);
const crossfade=toggle('crossfade','Crossfade',
 'Light: the fader knob slides from A to B while curve A dims and curve B brightens. Strong: the knob pulls back, overshoots the B end and settles. Reverse rides it back to A.',[
 ...shown(tube('curve-a','M5.5 7 C13 7 19 19 26.5 19'),(t,s,r)=>1-.55*mixU(t,s,r)),
 ...shown(tube('curve-b','M5.5 19 C13 19 19 7 26.5 7'),(t,s,r)=>.45+.55*mixU(t,s,r)),
 ...tube('track','M6 25 L26 25'),
 ...posed(bothFill('knob',rect(6.4,22.4,3.2,5.2,1.6)),rigid((t,s,r)=>({dx:knobX(t,s,r)-8})))
],.62,.58,'Track A','Track B',.58,.54);
crossfade.duotoneFill=false;

// ---------- EQUALIZER PRESET (stateful: Flat → Preset) ----------
// Three vertical tracks at x9/16/23 (y6–26) with round knobs. Flat holds every knob at y16; the preset lifts
// bass to y10, drops mids to y20 and lifts treble to y12, staggered left to right.
const EQX=[9,16,23],EQY=[10,20,12];
const knobY=EQY.map((yf,i)=>{const d=i*.06,o=yf-16;return G([[0,16],[.06+d,16],[.6+d,yf,'io']],[[0,16],[.04+d,16],[.3+d,yf+o*.2],[.42+d,yf-o*.08,'hit'],[.52+d,yf]],
 [[0,yf],[.06+d,yf],[.56+d,16,'io']],[[0,yf],[.04+d,yf],[.28+d,16-o*.15],[.4+d,16+o*.05,'hit'],[.5+d,16]]);});
const equalizerPreset=toggle('equalizer-preset','Equalizer preset',
 'Light: the three knobs glide from flat to the preset curve, bass first. Strong: each knob overshoots its target and springs back. Reverse returns every knob to the centre line.',[
 ...EQX.flatMap((x,i)=>tube('track-'+i,`M${x} 6 L${x} 26`)),
 ...EQX.flatMap((x,i)=>posed(bothFill('knob-'+i,circle(x,16,2.3)),rigid((t,s,r)=>({dy:knobY[i](t,s,r)-16}))))
],.72,.64,'Flat','Preset',.68,.62);
equalizerPreset.duotoneFill=false;

// ---------- AUDIO JACK (stateful: Unplugged → Plugged) ----------
// Socket slot x10.5–21.5, y3.5–8. Plug: pin capsule (16,12)-(16,18.5) 3 wide, body x12–20 y18.5–24, cable to
// y27.2. The plug rises 5.5 units so the pin enters the slot.
const socket=rect(10.5,3.5,11,4.5,2.25),plugBody=rect(12,18.5,8,5.5,1.6);
const insert=G([[0,0],[.06,0],[.52,1,'io']],[[0,0],[.06,-.15],[.24,1,'hit'],[.32,.94],[.44,1]],
 [[0,1],[.06,1],[.52,0,'io']],[[0,1],[.06,1.05],[.28,0,'hit'],[.36,.06],[.46,0]]);
const plugPose=rigid((t,s,r)=>({dy:-5.5*insert(t,s,r)}));
const audioJack=toggle('audio-jack','Audio jack',
 'Light: the plug rises 5.5 units and its pin seats in the socket. Strong: it dips first, jabs in with a hard stop and a small recoil. Reverse pulls it out.',[
 stroke('socket',socket),fill('socket-solid',holes(socket,rect(11.8,4.8,8.4,1.9,.95)),'solid'),
 ...posed([...bothFill('pin',capsule([16,12],[16,18.5],3)),...framed('body',plugBody),...tube('cable','M16 24 L16 27.2')],plugPose)
],.52,.5,'Unplugged','Plugged',.52,.48);
audioJack.duotoneFill=true;

// ---------- SPATIAL AUDIO ----------
// A listener (head (16,13.5) r2.4, shoulders arc) surrounded by three sound arcs at r10.5 about (16,16): left,
// right and top. Light: the arcs breathe outward. Strong: the arc trio orbits one full turn around the listener.
const breathe=F([[0,0],[.08,0],[.36,1],[.72,0,'smooth']],[[0,0],[.6,0]]);
const orbit=F([[0,0],[.5,0]],[[0,0],[.06,0],[.74,360]]);
const arcR=(t,s)=>10.5+.9*breathe(t,s);
const ring=(name,a,b)=>posed(tube(name,(t,s)=>arcN(16,16,arcR(t,s),a,b,2)),rigid((t,s)=>({angle:orbit(t,s),pivot:[16,16]})));
const spatialAudio=make('spatial-audio','Spatial audio',
 'Light: the three surrounding arcs breathe 0.9 units outward and back. Strong: the arcs orbit the listener through one full turn and stop where they started.',[
 ...bothFill('head',circle(16,13.5,2.4)),...tube('shoulders',arcN(16,23,4.8,200,340,2)),
 ...ring('arc-right',-20,20),...ring('arc-top',250,290),...ring('arc-left',160,200)
],.72,.76);
spatialAudio.duotoneFill=false;

// ---------- TUNING FORK ----------
// Prongs at x11/21 from y4.5 joined by a U at y≈14.8, stem to y26, knob (16,26.4). The prong tips vibrate in
// opposite directions with a decaying envelope. Strong: the fork drops 1.6 units, strikes and rings harder.
const vib=(t,s)=>{const t0=s?.12:.05,T=s?.9:.8,A=s?1.4:.6,f=s?11:9,k=s?3:3.5;if(t<=t0||t>=T)return 0;const u=t-t0,w=1-u/(T-t0);return A*Math.sin(2*Math.PI*f*u)*Math.exp(-k*u)*w*w;};
const strike=F([[0,0],[.5,0]],[[0,0],[.04,0],[.12,1.6],[.24,0]]);
const forkPose=(x,y,t,s)=>[x,y+strike(t,s)],forkRigid=rigid((t,s)=>({dy:strike(t,s)}));
const prongs=(t,s)=>{const a=vib(t,s);return `M${11-a} 4.5 L${11-a*.3} 12.5 C11 15.6 21 15.6 ${21+a*.3} 12.5 L${21+a} 4.5`;};
const tuningFork=make('tuning-fork','Tuning fork',
 'Light: the prong tips vibrate 0.6 units in opposite directions and die away. Strong: the fork drops, strikes and rings with a 1.4-unit vibration that decays over 0.8 s.',[
 ...posed(tube('prongs',prongs),forkPose),
 ...posed([...tube('stem','M16 14.8 L16 26'),...bothFill('knob',circle(16,26.4,1.5))],forkRigid)
],.8,.9);
tuningFork.duotoneFill=false;

export const icons=[muteAudio,crossfade,equalizerPreset,audioJack,spatialAudio,tuningFork];
