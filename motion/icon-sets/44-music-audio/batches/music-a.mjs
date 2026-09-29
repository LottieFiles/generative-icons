// Music & audio batch A: music-note, vinyl-record, cassette-tape, audio-waveform, loudspeaker, radio, metronome.
import {make,F,tube,framed,rigid,posed,fill,stroke,circle,rect,polygon,holes,cuts,bothFill,rotate,arcN,arcTubeN,dline,ellipse} from './helpers.mjs';

// ---------- MUSIC NOTE ----------
// Eighth note: filled head (12.5,23) r3.2, stem to y7, flag curling right. One rigid body pivoting at the head.
const hop=F([[0,0],[.08,0],[.36,-1.2],[.66,0,'smooth']],[[0,0],[.06,.5],[.26,-3.2],[.40,0,'hit'],[.48,-.6],[.6,0]]);
const tilt=F([[0,0],[.08,0],[.36,-5],[.66,0,'smooth']],[[0,0],[.06,4],[.26,-10],[.40,3,'hit'],[.58,0]]);
const notePose=rigid((t,s)=>({dy:hop(t,s),angle:tilt(t,s),pivot:[12.5,23]}));
const musicNote=make('music-note','Music note',
 'Light: the note bobs up 1.2 units with a 5° lean and settles. Strong: it hops 3.2 units after a small dip, leans 10° and lands with a short bounce.',
 posed([...bothFill('head',circle(12.5,23,3.2)),...tube('stem','M15.4 22.4 L15.4 7'),...tube('flag','M15.4 7 C17.2 9.8 21.6 10.4 20.8 16')],notePose),.68,.62);
musicNote.duotoneFill=false;

// ---------- VINYL RECORD ----------
// Platter (14,16.5) r10.5 with a label ring r3.3 and two opposite groove arcs at r6.9 that rotate. The tonearm
// pivots at (26.5,7) and rests on the rim. Solid: a filled platter with label and groove counters.
const VC=[14,16.5];
const spin=F([[0,0],[.06,0],[1.0,360,'io']],[[0,0],[.08,-15],[.72,720]]);
const armSwing=F([[0,0],[.5,0]],[[0,0],[.1,0],[.3,4],[.5,-1.5,'hit'],[.64,0]]);
const platterPose=rigid((t,s)=>({angle:spin(t,s),pivot:VC}));
const armPose=rigid((t,s)=>({angle:armSwing(t,s),pivot:[26.5,7]}));
const grooves=[[150,210],[330,390]];
const vinylRecord=make('vinyl-record','Vinyl record',
 'Light: the platter turns one smooth revolution while the tonearm rests. Strong: it winds back 15°, spins two fast revolutions and the tonearm dips onto the rim and settles.',[
 fill('platter-interior',circle(...VC,10.5),'tint'),stroke('platter',circle(...VC,10.5)),
 ...posed([fill('platter-solid',holes(circle(...VC,10.5),circle(...VC,4.4),...grooves.map(([a,b])=>arcTubeN(...VC,6.9,a,b,1.6,2))),'solid'),
  ...grooves.map(([a,b],i)=>stroke('groove-'+i,arcN(...VC,6.9,a,b,2)))],platterPose),
 stroke('label',circle(...VC,3.3)),fill('label-solid',circle(...VC,2.4),'solid'),
 ...posed(tube('arm','M26.5 7 L23.6 12.2'),armPose),...bothFill('arm-pivot',circle(26.5,7,1.6))
],1.0,.72);
vinylRecord.duotoneFill=true;

// ---------- CASSETTE TAPE ----------
// Shell x3.5–28.5, y7.5–25.5. Two open reels (270° arcs, r3.2) at (11,14.5) and (21,14.5) turn together; a tape
// window line sits below. Solid: the shell with rotating reel counters and a window slot.
const shell=rect(3.5,7.5,25,18,2.2);
const reelSpin=F([[0,0],[.06,0],[1.1,360,'io']],[[0,0],[.08,-20],[.74,720]]);
const REELS=[[11,14.5],[21,14.5]];
const cassetteTape=make('cassette-tape','Cassette tape',
 'Light: both reels turn one slow revolution. Strong: they wind back 20°, then spin two quick revolutions and stop together.',[
 ...framed('shell',shell,(t,s)=>cuts(shell,...REELS.map(([x,y])=>arcTubeN(x,y,3.2,45+reelSpin(t,s),315+reelSpin(t,s),1.9,4)),rect(9.5,20.3,13,2.4,1.2))),
 ...REELS.flatMap(([x,y],i)=>posed([stroke('reel-'+i,arcN(x,y,3.2,45,315,4))],rigid((t,s)=>({angle:reelSpin(t,s),pivot:[x,y]})))),
 ...tube('window','M10 21.5 L22 21.5')
],1.1,.74);
cassetteTape.duotoneFill=true;

// ---------- AUDIO WAVEFORM ----------
// Six centred bars, pitch 4. A pulse travels left to right, lifting each bar toward a 24-unit ceiling.
const BX=[6,10,14,18,22,26],BH=[6,13,20,9,16,7];
const pulse=(t,i,w,delay,start)=>{const p=t-start-i*delay;return p<=0||p>=w?0:Math.sin(Math.PI*p/w)**2;};
const barHeight=(i,t,s)=>{const k=s?pulse(t,i,.3,.05,.04):pulse(t,i,.5,.07,.06)*.35;return BH[i]+(24-BH[i])*k;};
const audioWaveform=make('audio-waveform','Audio waveform',
 'Light: a soft swell travels across the bars, each rising about a third of the way to the ceiling. Strong: a fast pulse throws every bar to full height in turn.',
 BX.flatMap((x,i)=>dline('bar-'+i,(t,s)=>{const h=barHeight(i,t,s)/2;return [[x,16-h],[x,16+h]];})),.88,.56);
audioWaveform.duotoneFill=false;

// ---------- LOUDSPEAKER ----------
// Cabinet x7.5–24.5, y3.5–28.5 with a tweeter dot (16,9) and a woofer ring (16,20) r4.2 with a dust cap.
// The cone kicks outward: ring and cap grow. Strong adds a cabinet shake on each kick.
const cab=rect(7.5,3.5,17,25,2.6);
const kick=F([[0,0],[.1,0],[.34,1],[.64,0,'smooth']],[[0,0],[.05,-.3],[.14,1,'hit'],[.26,.15],[.34,1,'hit'],[.46,.1],[.54,.85,'hit'],[.72,0]]);
const shake=F([[0,0],[.6,0]],[[0,0],[.14,.6,'hit'],[.22,-.4],[.34,.5,'hit'],[.42,-.3],[.54,.4,'hit'],[.72,0]]);
const wooferR=(t,s)=>4.2+.7*kick(t,s),capR=(t,s)=>1.3+.5*kick(t,s);
const cabPose=rigid((t,s)=>({dx:shake(t,s)})),shakePose=(x,y,t,s)=>[x+shake(t,s),y];
const loudspeaker=make('loudspeaker','Loudspeaker',
 'Light: the woofer cone pushes out once (ring and cap grow) and relaxes. Strong: three hard kicks in a row, each shaking the cabinet sideways.',[
 ...posed([fill('cabinet-interior',cab,'tint'),stroke('cabinet',cab),...bothFill('tweeter',circle(16,9,1.6))],cabPose),
 ...posed([fill('cabinet-solid',(t,s)=>holes(cab,circle(16,9,1.6),circle(16,20,wooferR(t,s))),'solid'),fill('cap-solid',(t,s)=>circle(16,20,2.4+.5*kick(t,s)),'solid'),
  stroke('woofer',(t,s)=>circle(16,20,wooferR(t,s))),fill('cap',(t,s)=>circle(16,20,capR(t,s)),'outline')],shakePose)
],.66,.74);
loudspeaker.duotoneFill=true;

// ---------- RADIO ----------
// Body x4–28, y11–27.5; grille ring (11.5,19.2) r3.6; tuning scale y17.5 x18–24.5 crossed by a needle; antenna
// from (22.5,11) to (27.5,4). Light: the needle scans the dial and returns. Strong: the antenna springs, the
// grille pulses and the needle scans decisively.
const radioBody=rect(4,11,24,16.5,2.5);
const needleX=F([[0,19.5],[.08,19.5],[.5,23,'io'],[.92,19.5,'smooth']],[[0,19.5],[.06,19.5],[.24,23.4,'hit'],[.38,22.6],[.64,19.5]]);
const antennaSwing=F([[0,0],[.5,0]],[[0,0],[.06,0],[.18,-9],[.3,7],[.42,-4],[.54,2],[.68,0]]);
const grilleR=F([[0,3.6],[.5,3.6]],[[0,3.6],[.06,3.6],[.2,4,'hit'],[.4,3.6]]);
const radio=make('radio','Radio',
 'Light: the tuning needle slides across the dial and drifts back. Strong: the antenna whips and springs, the grille pulses and the needle snaps across the dial before returning.',[
 fill('body-interior',radioBody,'tint'),stroke('body',radioBody),
 fill('body-solid',(t,s)=>holes(radioBody,circle(11.5,19.2,grilleR(t,s)),rect(needleX(t,s)-1.3,14.5,2.6,6,1.3)),'solid'),fill('grille-solid',circle(11.5,19.2,2),'solid'),
 stroke('grille',(t,s)=>circle(11.5,19.2,grilleR(t,s))),...tube('scale','M18 17.5 L24.5 17.5'),
 stroke('needle',(t,s)=>`M${needleX(t,s)} 15 L${needleX(t,s)} 20`),
 ...posed(tube('antenna','M22.5 11 L27.5 4'),rigid((t,s)=>({angle:antennaSwing(t,s),pivot:[22.5,11]})))
],.92,.68);
radio.duotoneFill=true;

// ---------- METRONOME ----------
// Body trapezoid (11,12)-(21,12)-(26,27)-(6,27). The pendulum rod pivots at (16,25.5), reaches y5 and carries a
// weight at (16,7). Light: one tick-tock of 7°. Strong: 14° swings with hard stops at each extreme.
const metroBody=polygon([[11,12],[21,12],[26,27],[6,27]],1.6);
const swing=F([[0,0],[.06,0],[.34,7],[.74,-7],[1.04,0,'smooth']],[[0,0],[.06,0],[.28,-14,'hit'],[.52,14,'hit'],[.76,0]]);
const pendulumPose=rigid((t,s)=>({angle:swing(t,s),pivot:[16,25.5]}));
const metronome=make('metronome','Metronome',
 'Light: the pendulum swings 7° each way once and returns to centre. Strong: it swings 14° each way with a hard stop at each extreme.',[
 ...framed('body',metroBody,holes(metroBody,rect(15,14,2,10,1))),
 ...posed([...tube('rod','M16 25.5 L16 5'),...bothFill('weight',circle(16,7,1.8))],pendulumPose)
],1.04,.76);
metronome.duotoneFill=true;

export const icons=[musicNote,vinylRecord,cassetteTape,audioWaveform,loudspeaker,radio,metronome];
