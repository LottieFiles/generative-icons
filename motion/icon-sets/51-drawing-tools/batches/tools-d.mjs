// Drawing & design tools batch D: rotate-tool, mirror-tool, snap-to-grid, ruler-guides, scissors.
import {make,F,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,rotate,bothFill,capsule,holes,annulus,arcN,pt} from './helpers.mjs';
const shell=(name,d)=>[stroke(name,d),fill(name+'-solid',d,'solid')];

// ---------- ROTATE TOOL ----------
// A square under a clockwise rotation cursor turns a quarter turn about its own centre and lands in the
// equivalent pose. Light: one 0.5 s turn. Strong: a 12° wind-up, a fast turn past 90° with a stop and a
// settle onto the quarter. The cursor arc stays fixed.
const RC=[15,18],RR=9.5,head=pt(340,RR,RC),tan=[Math.cos((340+90)*Math.PI/180),Math.sin((340+90)*Math.PI/180)];
const barb=deg=>{const a=deg*Math.PI/180,c=Math.cos(a),s=Math.sin(a),d=[-tan[0]*c+tan[1]*s,-tan[0]*s-tan[1]*c];return [head[0]+2.8*d[0],head[1]+2.8*d[1]];};
const turn=F([[0,0],[.06,0],[.56,90,'smooth']],[[0,0],[.1,-12],[.34,98,'hit'],[.46,88],[.56,90]]);
const rotateTool=make('rotate-tool','Rotate tool',
 'Light: the square turns a quarter turn about its centre in 0.5 s under the fixed rotation cursor. Strong: it winds back 12°, turns fast past 90° with a stop and settles onto the quarter turn.',[
 ...tube('arc',arcN(...RC,RR,200,340,2)),
 ...tube('head',`M${barb(-38)} L${head} L${barb(38)}`),
 ...posed(framed('square',rect(11,14,8,8,1.4)),rigid((t,s)=>({angle:turn(t,s),pivot:RC})))
],.6,.6);
rotateTool.duotoneFill=true;rotateTool.cycleDegrees=90;

// ---------- MIRROR TOOL ----------
// Two triangles mirror each other across a dashed axis. The right copy flips over about its own centre and
// lands mirrored again. Light: one 0.58 s flip. Strong: it lifts 10%, flips in 0.22 s with a stop and
// squashes back to size.
const flip=F([[0,0],[.06,0],[.64,2,'smooth']],[[0,0],[.1,0,'hold'],[.32,2,'hit'],[.36,2]]);
const liftM=F([[0,1],[.6,1]],[[0,1],[.1,1.1],[.32,1.1,'hold'],[.4,.94],[.5,1]]);
const MC=[23,16],mirrorPose=(x,y,t,s)=>{const sx=Math.cos(Math.PI*flip(t,s)),k=liftM(t,s);return [MC[0]+(x-MC[0])*sx*k,MC[1]+(y-MC[1])*k];};
const mirrorTool=make('mirror-tool','Mirror tool',
 'Light: the right triangle flips over about its own centre in 0.58 s and lands mirrored again across the dashed axis. Strong: it lifts 10%, flips in 0.22 s with a stop and squashes back to size.',[
 ...tube('axis-0','M16 6 L16 10.5'),...tube('axis-1','M16 13.75 L16 18.25'),...tube('axis-2','M16 21.5 L16 26'),
 ...framed('left',polygon([[5.5,10],[5.5,22],[12.5,16]],[1,1,1.2])),
 ...posed(framed('right',polygon([[26.5,10],[26.5,22],[19.5,16]],[1,1,1.2])),mirrorPose)
],.66,.52);
mirrorTool.duotoneFill=true;

// ---------- SNAP TO GRID ----------
// A square sits on the centre point of a 3×3 dot grid. It is dragged off diagonally, then snaps back onto
// the point, which pulses on contact. Light: a 2.2 unit drag and a quick return. Strong: a farther drag, a
// hard snap that overshoots the point and settles.
const dragS=F([[0,0],[.06,0],[.34,1,'smooth'],[.42,1],[.52,0,'in'],[.56,0]],[[0,0],[.06,0,'hold'],[.3,1],[.38,1,'hold'],[.45,-.12,'hit'],[.52,.04],[.58,0]]);
const pulse=F([[0,1],[.5,1],[.56,1.6,'out'],[.64,1,'smooth']],[[0,1],[.45,1,'hold'],[.5,1.8],[.6,1]]);
const snapToGrid=make('snap-to-grid','Snap to grid',
 'Light: the square is dragged 2.2 units off the centre grid point and snaps back in 0.1 s while the point pulses. Strong: it is dragged farther, snaps back hard past the point and settles as the point pulses.',[
 ...[7,16,25].flatMap(y=>[7,16,25].map(x=>bothFill(`dot-${x}-${y}`,x===16&&y===16?(t,s)=>circle(16,16,.8*pulse(t,s)):circle(x,y,.8))).flat()),
 ...posed(framed('object',rect(13,13,6,6,1.2)),rigid((t,s)=>({dx:-(s?2.8:2.2)*dragS(t,s),dy:-2.2*dragS(t,s),pivot:[16,16]})))
],.66,.6);
snapToGrid.duotoneFill=true;

// ---------- RULER GUIDES ----------
// A corner ruler with ticks frames the canvas; a vertical and a horizontal guide cross inside it. The guides
// are dragged 3 units to a new position, held for measuring, and set back. Strong drags 5 units with a stop.
const RULER=polygon([[3.5,3.5],[28.5,3.5],[28.5,9],[9,9],[9,28.5],[3.5,28.5]],1);
const TICKS=[13,17,21,25];
const rulerSolid=holes(RULER,...TICKS.map(x=>capsule([x,9.4],[x,7.4],1.4)),...TICKS.map(y=>capsule([9.4,y],[7.4,y],1.4)));
const slideG=F([[0,0],[.06,0],[.34,1,'smooth'],[.44,1],[.66,0,'smooth']],[[0,0],[.06,-.1],[.26,1,'hit'],[.36,.94],[.44,.94,'hold'],[.62,0]]);
const rulerGuides=make('ruler-guides','Ruler guides',
 'Light: the crossing guides are dragged 3 units right and down along the rulers, held for measuring and set back. Strong: they are dragged 5 units with a stop at the tick, held and returned with a spring.',[
 ...framed('ruler',RULER,rulerSolid),
 ...TICKS.flatMap(x=>stroke('tick-t'+x,`M${x} 9 L${x} 7.2`)),...TICKS.flatMap(y=>stroke('tick-l'+y,`M9 ${y} L7.2 ${y}`)),
 ...posed(tube('guide-v','M17 12.5 L17 28.3'),rigid((t,s)=>({dx:(s?5:3)*slideG(t,s),pivot:[17,12.5]}))),
 ...posed(tube('guide-h','M12.5 17 L28.3 17'),rigid((t,s)=>({dy:(s?5:3)*slideG(t,s),pivot:[12.5,17]})))
],.7,.66);
rulerGuides.duotoneFill=true;

// ---------- SCISSORS ----------
// Two blades cross at a hub with finger rings below. The scissors open about the hub and snip shut.
// Light: an 8° opening and a quick close. Strong: a 15° opening, a hold and a snap shut that overshoots
// 1.5° with a settle. Rings and blades are rigid about the hub.
const HUB=[16,17],DIR=[4.82,6.63],len=Math.hypot(...DIR),ud=DIR.map(v=>v/len);
const half=(sign)=>{const d=[ud[0]*sign,ud[1]],tip=[HUB[0]-d[0]*12,HUB[1]-d[1]*12],ring=[HUB[0]+d[0]*8.2,HUB[1]+d[1]*8.2],shaftEnd=[ring[0]-d[0]*2.3,ring[1]-d[1]*2.3];
 return {tip,ring,shaft:`M${tip} L${shaftEnd}`};};
const open=F([[0,0],[.06,0],[.24,8,'smooth'],[.3,8],[.42,0,'in'],[.5,0]],[[0,0],[.06,-2],[.22,15],[.34,15,'hold'],[.4,-1.5,'hit'],[.48,.8],[.56,0]]);
const bladeParts=(name,sign)=>{const h=half(sign);return posed([...tube(name+'-shaft',h.shaft),stroke(name+'-ring',circle(...h.ring,2.8)),fill(name+'-ring-solid',annulus(...h.ring,2.8,2.6),'solid')],rigid((t,s)=>({angle:-sign*open(t,s),pivot:HUB})));};
const scissors=make('scissors','Scissors',
 'Light: the scissors open 8° about the hub and snip shut in 0.12 s. Strong: they open 15°, hold, and snap shut with a stop that overshoots 1.5° before settling.',[
 ...bladeParts('blade-a',1),...bladeParts('blade-b',-1),
 ...bothFill('hub',circle(...HUB,1.3))
],.52,.58);
scissors.duotoneFill=false;

export const icons=[rotateTool,mirrorTool,snapToGrid,rulerGuides,scissors];
