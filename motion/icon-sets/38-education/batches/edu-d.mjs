// Education batch D: language-learning, alphabet-blocks, homework, exam-grade, geometry-compass.
import {make,F,tube,framed,rigid,posed,shown,circle,rect,rotate,bothFill,fill,stroke} from './helpers.mjs';
import {arcN} from './sky.mjs';
import {container,pen,drawPolyline,curveFrom} from './edu.mjs';

// ---------- LANGUAGE LEARNING ----------
// An open book; a speech bubble holding a vector "A" pops up out of it (the new word spoken aloud).
const leftPage='M16 21.4 C13 20 9 19.6 5 20.2 L5 27.4 C9 26.8 13 27.2 16 28.6 Z';
const rightPage='M16 21.4 C19 20 23 19.6 27 20.2 L27 27.4 C23 26.8 19 27.2 16 28.6 Z';
const bubble='M11.5 3.6 L20.5 3.6 C22.2 3.6 23.5 4.9 23.5 6.6 L23.5 12.2 C23.5 13.9 22.2 15.2 20.5 15.2 L17.8 15.2 L16 17.3 L14.2 15.2 L11.5 15.2 C9.8 15.2 8.5 13.9 8.5 12.2 L8.5 6.6 C8.5 4.9 9.8 3.6 11.5 3.6 Z';
const TIP=[16,17.8];
// Strong pops wide (squash-stretch 120% × 102%) rather than tall: the canvas has no headroom above the bubble.
const popX=F([[0,0],[.08,0],[.42,1.06],[.56,1,'smooth']],[[0,0],[.12,0,'hold'],[.28,1.2],[.4,.95],[.5,1]]);
const pop=F([[0,0],[.08,0],[.42,1.02],[.56,1,'smooth']],[[0,0],[.12,0,'hold'],[.28,1.02],[.4,.97],[.5,1]]);
const tiltB=F([[0,0],[.6,0]],[[0,0],[.12,0],[.28,-3],[.42,1.5],[.54,0]]);
const flap=F([[0,0],[.6,0]],[[0,0],[.04,.9],[.16,-.4],[.3,0]]);
const bub=(x,y,t,s)=>{y+=.5;const k=pop(t,s),kx=popX(t,s),[a,b]=rotate(x,y,tiltB(t,s),...TIP);return[TIP[0]+(a-TIP[0])*kx,TIP[1]+(b-TIP[1])*k];};
const pages=(x,y,t,s)=>[x,y-flap(t,s)*Math.abs(x-16)/11];
const languageLearning=make('language-learning','Language learning',
 'Light: a speech bubble with the letter A grows up out of the open book. Strong: the pages flap as the bubble pops out wide (120%), tilts and settles. The bubble is retained; preview reset rearms.',[
 ...posed(framed('page-l',leftPage),pages),...posed(framed('page-r',rightPage),pages),
 ...shown(posed([...container('bubble',bubble),...tube('a-legs','M12.9 11.9 L16 6.9 L19.1 11.9'),...tube('a-bar','M14.08 10 L17.92 10')],bub),(t,s)=>pop(t,s)>.08?1:0)
],.56,.5,['bubble-interior','bubble-frame','bubble-solid','a-legs','a-legs-solid','a-bar','a-bar-solid']);
languageLearning.duotoneFill=true;

// ---------- ALPHABET BLOCKS ----------
// Two letter blocks (L, T) on the floor; the A block drops onto them and settles. The stack is retained.
// v1's small C and I closed into blobs at Bold; the letters are open strokes with no counters.
const drop=F([[0,-5],[.06,-5],[.46,0,'smooth']],[[0,-7],[.06,-7],[.26,0,'in'],[.36,-1.4,'out'],[.46,0,'in'],[.52,-.3,'out'],[.58,0,'in']]);
const spinT=F([[0,0],[.6,0]],[[0,10],[.06,10],[.26,0,'in']]);
const thud=F([[0,0],[.6,0]],[[0,0],[.26,0],[.3,.4,'hit'],[.42,0]]);
const topPose=(x,y,t,s)=>{const [a,b]=rotate(x,y,spinT(t,s),16,13);return[a,b+drop(t,s)];};
const floorPose=rigid((t,s)=>({dy:thud(t,s)}));
const alphabetBlocks=make('alphabet-blocks','Alphabet blocks',
 'Light: the A block is lowered onto the two blocks below and settles. Strong: it falls from higher while turning 10°, lands hard with a double bounce, and the blocks below take the thud. The stack is retained; preview reset rearms.',[
 ...posed([...container('block-c',rect(4.25,18,10,10,1.6)),...tube('letter-c','M8.25 21.2 L8.25 24.8 L10.9 24.8')],floorPose),
 ...posed([...container('block-i',rect(17.75,18,10,10,1.6)),...tube('letter-i','M21.2 21.2 L24.3 21.2'),...tube('letter-t','M22.75 21.2 L22.75 24.8')],floorPose),
 ...shown(posed([...container('block-a',rect(11,8,10,10,1.6)),...tube('letter-a','M14.6 14.7 L16 11.3 L17.4 14.7')],topPose),// Light fades in from −4.5 (it has no rotation); Strong's turning block waits until −3.5 to keep the top margin.
 (t,s)=>Math.max(0,Math.min(1,s?(drop(t,s)+3.5)/1.5:(drop(t,s)+4.5)/1.2)))
],.58,.58,['block-a-interior','block-a-frame','block-a-solid','letter-a','letter-a-solid']);
alphabetBlocks.duotoneFill=true;

// ---------- HOMEWORK ----------
// A worksheet with two answer lines; the finished work is ticked. The tick is retained.
const tickP=F([[0,0],[.08,0],[.5,1,'smooth']],[[0,0],[.12,0,'hold'],[.26,1]]);
const hop=F([[0,0],[.6,0]],[[0,0],[.04,.5],[.1,-.6],[.16,0,'hit'],[.26,0],[.32,-1],[.44,0,'hit']]);
const sheet=rigid((t,s)=>({dy:hop(t,s)}));
const homework=make('homework','Homework',
 'Light: a tick is drawn across the finished worksheet. Strong: the sheet is slapped down, the tick is struck in 0.14 s and the sheet hops once in approval. The tick is retained; preview reset rearms.',[
 ...posed(container('sheet',rect(7,4.5,18,23,2)),sheet),
 ...posed(tube('line-0','M10.5 9 L21.5 9'),sheet),...posed(tube('line-1','M10.5 12.4 L18 12.4'),sheet),
 ...posed(shown(pen('tick',drawPolyline([[11.2,19.6],[14.2,22.6],[20.2,16.2]],tickP)),(t,s)=>tickP(t,s)>.01?1:0),sheet)
],.5,.46,['tick','tick-solid']);
homework.duotoneFill=true;

// ---------- EXAM GRADE ----------
// An A+ grade is circled with a hand-drawn ring (slightly widening so its end passes outside its start).
const ringP=F([[0,0],[.06,0],[.62,1,'smooth']],[[0,0],[.08,0,'hold'],[.34,1]]);
const gradeK=F([[0,1],[.7,1]],[[0,1],[.34,1],[.42,1.06,'hit'],[.52,.98],[.6,1]]);
const ringAt=u=>{const a=(-110+380*u)*Math.PI/180;return[16+(11.8+.7*u)*Math.cos(a),16+(8.6+.6*u)*Math.sin(a)];};
// The grade is drawn at 88% about the centre so the widening ring clears it by ≥1 even at Strong's pop.
const grade=(x,y,t,s)=>{const k=gradeK(t,s)*.88;return[16.4+(x-16)*k,16+(y-16)*k];};
const examGrade=make('exam-grade','Exam grade',
 'Light: a hand-drawn ring circles the A+ grade. Strong: the ring is struck round in 0.26 s and the grade pops 6% when circled. The ring is retained; preview reset rearms.',[
 ...posed([...tube('a-legs','M9.8 20.5 L13.4 11 L17 20.5'),...tube('a-bar','M11.1 17.1 L15.7 17.1'),...tube('plus-h','M18.8 15.8 L23.6 15.8'),...tube('plus-v','M21.2 13.4 L21.2 18.2')],grade),
 ...shown(pen('ring',(t,s)=>curveFrom(u=>ringAt(Math.max(.002,ringP(t,s))*u),6)),(t,s)=>ringP(t,s)>.01?1:0)
],.62,.6,['ring','ring-solid']);
examGrade.duotoneFill=false;

// ---------- GEOMETRY COMPASS ----------
// Drafting compass: hinge disc and handle at the top, two straight legs spread 40°, a needle point on the
// left leg and a collared pencil tip on the right. The swept arc is a separate stroke on the paper beneath
// both tips (radius 20.6 about the hinge, ≥1 painted clear of each tip); it is drawn left to right while
// the compass swivels counterclockwise on its needle and returns upright. v1/v2 read as a bent stick or hook
// because an arc concentric with the needle is nearly tangent to the pencil leg at its tip.
const HG=[16,8],LEG=15,deg=Math.PI/180,ARC_R=20.6;
const legEnd=a=>[HG[0]+LEG*Math.sin(a*deg),HG[1]+LEG*Math.cos(a*deg)];
const NE=legEnd(-20),PE=legEnd(20),dirN=[NE[0]-HG[0],NE[1]-HG[1]].map(v=>v/LEG),dirP=[PE[0]-HG[0],PE[1]-HG[1]].map(v=>v/LEG);
const NT=[NE[0]+dirN[0]*2,NE[1]+dirN[1]*2],PT=[PE[0]+dirP[0]*1.6,PE[1]+dirP[1]*1.6];
const perp=[-dirP[1],dirP[0]],COL=[PE[0]-dirP[0]*.6,PE[1]-dirP[1]*.6];
const sweepP=F([[0,0],[.1,0],[.62,1,'smooth']],[[0,0],[.18,0,'hold'],[.34,1]]);
const swivel=F([[0,0],[.1,0],[.34,-6],[.62,0,'smooth']],[[0,0],[.06,-9],[.18,-9,'hold'],[.34,0,'hit'],[.42,-1.5],[.5,0]]);
const compassPose=rigid((t,s)=>({angle:swivel(t,s),pivot:NT}));
const geometryCompass=make('geometry-compass','Geometry compass',
 'Light: the compass swivels 6° on its needle and back while an arc is drawn on the paper beneath it. Strong: it cocks 9°, holds, then snaps upright as the arc is struck in 0.16 s, with a small recoil. The arc is retained; preview reset rearms.',[
 ...shown(pen('arc',(t,s)=>arcN(...HG,ARC_R,108,108-36*Math.max(.01,sweepP(t,s)),2)),(t,s)=>sweepP(t,s)>.02?1:0),
 ...posed([...tube('handle',`M${HG[0]} ${HG[1]-1.7} L${HG[0]} ${HG[1]-3.4}`),
  ...tube('needle-leg',`M${HG} L${NE}`),...tube('needle',`M${NE} L${NT}`),
  ...tube('pencil-leg',`M${HG} L${PE}`),...tube('collar',`M${COL[0]+perp[0]*1.3} ${COL[1]+perp[1]*1.3} L${COL[0]-perp[0]*1.3} ${COL[1]-perp[1]*1.3}`),
  ...tube('pencil-tip',`M${PE} L${PT}`),...bothFill('hinge',circle(...HG,1.7))],compassPose)
],.62,.5,['arc','arc-solid']);
geometryCompass.duotoneFill=false;

export const icons=[languageLearning,alphabetBlocks,homework,examGrade,geometryCompass];
