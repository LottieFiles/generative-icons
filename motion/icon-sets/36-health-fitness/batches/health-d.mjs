// Health & Fitness batch D: meditation, nutrition, jump-rope, hospital, dna.
import {make,F,tube,framed,rigid,posed,shown,fill,stroke,circle,bothFill,rotate,holes,rect,polygon,capsule} from './helpers.mjs';
import {curveFrom,lerp} from './sky.mjs';

// ---------- MEDITATION ----------
// Lotus over a water line. v1/v2 drew a seated figure; at icon sizes its lap read as a UFO rim, then a
// bell. The lotus is the familiar meditation mark. It breathes: side petals open about the shared base
// while the centre petal lifts; Strong closes in first, blooms wide and settles.
const BASE=[16,23.4];
const CENTER='M16 23.4 C12.4 19.8 12.6 12.2 16 7.8 C19.4 12.2 19.6 19.8 16 23.4 Z';
const SIDE_L='M16 23.4 C10.8 23.6 6.4 19.4 5.6 13.8 C9.4 14 12.2 16 13.6 18.6';
const SIDE_R='M16 23.4 C21.2 23.6 25.6 19.4 26.4 13.8 C22.6 14 19.8 16 18.4 18.6';
const bloom=F([[0,0],[.08,0],[.36,1],[.7,0,'smooth']],[[0,0],[.08,-.6],[.34,1.6],[.5,.9],[.62,1.1],[.8,0]]);
const lotus=side=>(x,y,t,s)=>rotate(x,y,side*7*bloom(t,s),...BASE);
const meditation=make('meditation','Meditation',
 'Light: the lotus breathes; its side petals open 7° about the base as the centre petal lifts, then close. Strong: the petals draw in first, bloom 11° wide with the centre lifting, and settle.',[
 ...posed(framed('center',CENTER),(x,y,t,s)=>{const k=1+.05*bloom(t,s);return[x,BASE[1]+(y-BASE[1])*k];}),
 ...posed(tube('petal-l',SIDE_L),lotus(-1)),...posed(tube('petal-r',SIDE_R),lotus(1)),
 ...tube('water','M6.4 27.2 L25.6 27.2')
],.72,.84);
meditation.duotoneFill=true;

// ---------- NUTRITION ----------
// Apple with stem and a filled leaf. It rocks on its base while the leaf flutters about the stem.
const APPLE='M16 10.6 C13.8 9 9 8.6 7.2 12.6 C5.6 16.2 7 22 9.6 25.4 C11.2 27.6 13.2 28.6 16 27.2 C18.8 28.6 20.8 27.6 22.4 25.4 C25 22 26.4 16.2 24.8 12.6 C23 8.6 18.2 9 16 10.6 Z';
const LEAF='M18.6 7.6 C19.2 5.4 21.4 4.4 23.8 4.6 C23.2 6.8 21 7.8 18.6 7.6 Z';
const rock=F([[0,0],[.08,0],[.26,3],[.5,0,'smooth']],[[0,0],[.06,-2],[.2,7],[.34,-4],[.46,1.5],[.58,0]]);
const flutter=F([[0,0],[.12,0],[.3,-10],[.56,0,'smooth']],[[0,0],[.12,3],[.26,-16],[.4,5],[.52,-3],[.64,0]]);
const base=[16,27.6],STEM=[16.2,10.2];
const leafPose=(x,y,t,s)=>{const [a,b]=rotate(x,y,flutter(t,s),18.6,7.6);return rotate(a,b,rock(t,s),...base);};
const nutrition=make('nutrition','Nutrition',
 'Light: the apple rocks 3° on its base and the leaf flutters 10°. Strong: it rocks −2°, 7°, −4° and settles while the leaf whips 16° and flutters back.',[
 ...posed([...framed('apple',APPLE),...tube('stem','M16.2 10.2 C16.2 8.4 16.8 6.8 17.8 5.6')],rigid((t,s)=>({angle:rock(t,s),pivot:base}))),
 ...posed(bothFill('leaf',LEAF),leafPose)
],.58,.64);
nutrition.duotoneFill=true;

// ---------- JUMP ROPE ----------
// Two handles; the rope's bulge is the projected loop of a turning rope: b = B·cos(θ). One turn passes the
// rope overhead and brings it back under. Handles jerk with each turn.
// Handles are outlined capsule grips (v1 thin lines read as a bare U). The rope ends at each grip's top.
const HL=[7.4,15.8],HR=[24.6,15.8],B=10.6;
const turn=F([[0,0],[.06,0],[.72,360,'smooth']],[[0,0],[.06,-20],[.4,380],[.66,730],[.8,720]]);
const ropeD=(t,s)=>{const b=B*Math.cos(turn(t,s)*Math.PI/180)*1.33;return `M${HL} C10.6 ${HL[1]+b} 21.4 ${HR[1]+b} ${HR}`;};
const jerk=F([[0,0],[.72,0]],[[0,0],[.1,1],[.26,-1],[.44,1],[.6,-.6],[.8,0]]);
const handle=(name,a,b,side)=>posed(framed(name,capsule([a[0]+(b[0]-a[0])*.2,a[1]+1.7],b,3.4)),rigid((t,s)=>({angle:side*6*jerk(t,s),pivot:a})));
const jumpRope=make('jump-rope','Jump rope',
 'Light: the rope makes one turn, over the top and back under. Strong: it winds back, then whips through two fast turns while the handles jerk with each pass.',[
 ...tube('rope',ropeD),
 ...handle('handle-l',HL,[6,23.4],-1),...handle('handle-r',HR,[26,23.4],1)
],.72,.8);
jumpRope.duotoneFill=true;

// ---------- HOSPITAL ----------
// Building with a cross sign and automatic sliding doors; the doors part and close. Solid cuts the cross
// and the doorway out of the building.
const BLD=rect(5.5,6.5,21,21,2.4);
const PLUS=polygon([[14.7,9.2],[17.3,9.2],[17.3,12.2],[20.3,12.2],[20.3,14.8],[17.3,14.8],[17.3,17.8],[14.7,17.8],[14.7,14.8],[11.7,14.8],[11.7,12.2],[14.7,12.2]],.6);
const part=F([[0,0],[.08,0],[.24,.7],[.52,0,'smooth']],[[0,0],[.06,-.15],[.2,1.5],[.4,1.3],[.6,0]]);
const pulse=F([[0,1],[.6,1]],[[0,1],[.14,1],[.22,1.18],[.34,1]]);
const hospital=make('hospital','Hospital',
 'Light: the sliding doors part 0.7 units and close. Strong: they jolt, slide 1.5 units open as the cross sign pulses 18%, hold and close.',[
 fill('building-interior',BLD,'tint'),stroke('building',BLD),fill('building-solid',holes(rect(4.2,5.2,23.6,23.6,3.7),PLUS,rect(11,20.2,10,6.1,.6)),'solid'),
 ...posed([stroke('cross-v','M16 10.5 L16 16.5'),stroke('cross-h','M13 13.5 L19 13.5')],(x,y,t,s)=>{const k=pulse(t,s);return[16+(x-16)*k,13.5+(y-13.5)*k];}),
 stroke('doorway','M11 27.5 L11 21 L21 21 L21 27.5'),
 // Door edges have Solid variants so the doors also move inside Solid's doorway cut-out.
 ...posed(tube('door-l','M16 21 L16 27.5'),rigid((t,s)=>({dx:-part(t,s)}))),
 ...posed(tube('door-r','M16 21 L16 27.5'),rigid((t,s)=>({dx:part(t,s)})))
],.52,.6);
hospital.duotoneFill=true;

// ---------- DNA ----------
// Double helix, 0.7 period over the height, phased so the strands cross exactly twice (u ≈ .14, .86) and
// their ends stay apart and open. Three short rungs sit only in the wide middle lobe. Each rung's inset is
// computed from the local strand slope so the Bold gap to both strands stays ≥1 (perpendicular), and a
// rung hides when the twist brings the strands too close for a readable rung.
// Parent review: v1 crossings blobbed and the end rungs read as lids.
const Y0=4,Y1=28,A=7.4,P=.7,RUNGS=[.34,.5,.66],H=Y1-Y0;
const phaseT=F([[0,0],[.06,0],[.86,360,'smooth']],[[0,0],[.06,-24],[.62,736],[.78,720]]);
const th=(u,t,s)=>2*Math.PI*P*(u-.5)+phaseT(t,s)*Math.PI/180;
// Strand profile sin(π/2·cos θ): flat through the wide middle lobe (room for three rungs), steeper at the
// two crossings (a clean X instead of a shallow blob).
const xAt=(u,t,s)=>A*Math.sin(Math.PI/2*Math.cos(th(u,t,s)));
// Rung half-length: the largest h whose end stays ≥3.6 (Bold gap ≥1.1) from every sampled strand point.
const half=(u,t,s)=>{const y=Y0+H*u;let h=Infinity;for(let k=0;k<=160;k++){const v=k/160,dy=Y0+H*v-y;if(Math.abs(dy)>=3.6)continue;h=Math.min(h,Math.abs(xAt(v,t,s))-Math.sqrt(3.6*3.6-dy*dy));}return h;};
const strand=sign=>(t,s)=>curveFrom(u=>[16+sign*xAt(u,t,s),Y0+H*u],8);
const rungD=u=>(t,s)=>{const h=Math.max(.02,half(u,t,s)),y=Y0+H*u;return `M${16-h} ${y} L${16+h} ${y}`;};
const dna=make('dna','DNA',
 'Light: the helix makes one full calm twist; rungs fade out wherever the strands close in. Strong: it winds back 24°, twists through two turns with a short overrun and settles.',[
 ...tube('strand-a',strand(1)),...tube('strand-b',strand(-1)),
 ...RUNGS.flatMap((u,i)=>shown(tube('rung-'+i,rungD(u)),(t,s)=>half(u,t,s)>.8?1:0))
],.86,.8);
dna.duotoneFill=false;

export const icons=[meditation,nutrition,jumpRope,hospital,dna];
