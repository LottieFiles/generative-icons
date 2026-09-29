// Music & audio batch B: tuner, piano-keys, guitar, drum, lyrics, music-album, earbuds.
import {make,toggle,F,G,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,holes,bothFill,rotate,arcN,earcN,ellipse,arcTubeN,cutRev,pt,L,clamp01} from './helpers.mjs';

// ---------- TUNER (stateful: Flat → In tune) ----------
// Dial arc centred (16,21) r11 from 200° to 340°, ticks outside at 225/270/315 (r14.5–16), hub (16,21) and a
// needle of length 7.2 that starts at 235° (flat) and comes to rest at 270° (in tune).
const TC=[16,21];
const needleAngle=G([[0,235],[.06,235],[.7,270,'io']],[[0,235],[.06,235],[.34,283],[.5,266,'hit'],[.62,271],[.72,270]],
 [[0,270],[.06,270],[.62,235,'io']],[[0,270],[.06,270],[.32,231],[.46,237,'hit'],[.58,235]]);
const needlePose=rigid((t,s,r)=>({angle:needleAngle(t,s,r)-270,pivot:TC}));
const tuner=toggle('tuner','Tuner',
 'Light: the needle rises smoothly from flat to the centre mark. Strong: it swings past the mark to 283°, springs back and settles on centre. Reverse drops it back to flat.',[
 ...tube('dial',arcN(...TC,11,200,340,4)),
 ...[225,270,315].flatMap((a,i)=>tube('tick-'+i,`M${pt(a,14.5,TC)} L${pt(a,16,TC)}`)),
 ...posed(tube('needle','M16 21 L16 13.8'),needlePose),
 fill('hub',circle(...TC,1.6),'outline'),fill('hub-solid',circle(...TC,2),'solid')
],.7,.72,'Flat','In tune',.62,.58);
tuner.duotoneFill=false;

// ---------- PIANO KEYS ----------
// Keyboard x3.5–28.5, y7–25; dividers at x9.75/16/22.25; black keys 3.2 wide hang from the top edge to y15.5.
// A black key presses down. Solid: 3-over-4 slot pattern; the black slots move with the press.
const keyboard=rect(3.5,7,25,18,2.2),BK=[9.75,16,22.25];
const press=BK.map((_,i)=>F(i===1?[[0,0],[.1,0],[.26,1.4,'out'],[.54,0,'io']]:[[0,0],[.5,0]],[[0,0],[.06+i*.14,0],[.13+i*.14,2.2,'hit'],[.2+i*.14,2.2],[.34+i*.14,0]]));
const pianoKeys=make('piano-keys','Piano keys',
 'Light: the middle black key presses 1.4 units and lifts. Strong: the three black keys press 2.2 units in a quick run from left to right.',[
 ...framed('keyboard',keyboard,(t,s)=>holes(keyboard,...[0,1,2,3].map(i=>rect(4.8+6.25*i,17.6,3.65,5.8,1.5)),...BK.map((x,i)=>rect(x-1.6,8.6+press[i](t,s),3.2,5.6,1.4)))),
 ...BK.flatMap((x,i)=>tube('divider-'+i,`M${x} 7 L${x} 25`)),
 ...BK.flatMap((x,i)=>posed(bothFill('black-'+i,rect(x-1.6,7,3.2,8.5,1.2)),rigid((t,s)=>({dy:press[i](t,s)}))))
],.56,.64);
pianoKeys.duotoneFill=true;

// ---------- GUITAR ----------
// Acoustic body (figure eight, x7.2–22.8, y13.5–28.2) with a sound hole at (15,21), neck to y5 and a head block.
// The guitar rocks about the sound hole. Strong: a strum rocks it 8° with a spring and the sound hole pumps.
const guitarBody='M15 13.5 C10.8 13.5 9.2 16.2 10.2 19 C7.2 20.6 7.2 28.2 15 28.2 C22.8 28.2 22.8 20.6 19.8 19 C20.8 16.2 19.2 13.5 15 13.5 Z';
const rock=F([[0,0],[.08,0],[.34,3],[.66,-2],[.9,0,'smooth']],[[0,0],[.06,2],[.2,-8,'hit'],[.36,5],[.5,-2.5],[.62,1],[.74,0]]);
const pump=F([[0,0],[.5,0]],[[0,0],[.06,0],[.2,1,'hit'],[.44,0]]);
const holeR=(t,s)=>1.9+.6*pump(t,s);
const guitarPose=rigid((t,s)=>({angle:rock(t,s),pivot:[15,21]})),guitarPosePoint=(x,y,t,s)=>rotate(x,y,rock(t,s),15,21);
const guitar=make('guitar','Guitar',
 'Light: the guitar rocks 3° one way and 2° back before settling. Strong: a strum kicks it 8° with a spring settle while the sound hole pumps once.',[
 ...posed([fill('body-interior',guitarBody,'tint'),stroke('body',guitarBody),...tube('neck','M15 13.5 L15 5'),...bothFill('head',rect(13.2,3.2,3.6,3.6,1.2))],guitarPose),
 ...posed([fill('body-solid',(t,s)=>holes(guitarBody,circle(15,21,holeR(t,s)+.5)),'solid'),fill('sound-hole',(t,s)=>circle(15,21,holeR(t,s)),'outline')],guitarPosePoint)
],.9,.74);
guitar.duotoneFill=true;

// ---------- DRUM ----------
// Snare: rim ellipse (16,14) rx10 ry3, shell to y25 with a lower elliptical bottom. Two sticks from the top
// corners strike the head. Light: one soft hit. Strong: alternating hard hits that jolt the shell.
const rim=ellipse(16,14,10,3);
const shellD='M6 14 L6 25 '+L(earcN(16,25,10,3,180,0,2))+' L26 14';
const shellSolid='M6 14 L6 25 '+L(earcN(16,25,10,3,180,0,2))+' L26 14 '+L(earcN(16,14,10,3,360,180,2))+' Z';
const liftA=F([[0,0],[.08,0],[.3,-10],[.46,0,'in']],[[0,0],[.04,0],[.16,-22],[.26,0,'hit'],[.32,-3],[.4,0]]);
const liftB=F([[0,0],[.5,0]],[[0,0],[.27,0],[.38,22],[.48,0,'hit'],[.54,3],[.62,0]]);
const jolt=F([[0,0],[.5,0]],[[0,0],[.25,0],[.29,.7],[.4,0],[.47,0],[.51,.7],[.64,0]]);
const drumPose=rigid((t,s)=>({dy:jolt(t,s)}));
const drum=make('drum','Drum',
 'Light: the left stick lifts 10° and taps the head. Strong: the sticks alternate two hard 22° hits and the shell jolts down on each contact.',[
 ...posed([fill('shell-interior',shellSolid,'tint'),stroke('shell',shellD),stroke('rim',rim),fill('shell-solid',holes(shellSolid,ellipse(16,14,8,2.1)),'solid')],drumPose),
 ...posed(tube('stick-a','M5.5 4.5 L13 13.2'),rigid((t,s)=>({angle:liftA(t,s),pivot:[5.5,4.5]}))),
 ...posed(tube('stick-b','M26.5 4.5 L19 13.2'),rigid((t,s)=>({angle:liftB(t,s),pivot:[26.5,4.5]})))
],.5,.66);
drum.duotoneFill=true;

// ---------- LYRICS ----------
// A note at left and three dimmed lyric lines. The middle line is sung: a bright sweep grows along it then fades.
// Strong: the note hops on two beats and the sweep is fast.
const sweep=F([[0,0],[.08,0],[.72,1,'linear']],[[0,0],[.05,0],[.4,1]]);
const sweepAlpha=F([[0,1],[.72,1],[1.02,0,'io']],[[0,1],[.6,1],[.86,0]]);
const beat=F([[0,0],[.5,0]],[[0,0],[.05,0],[.15,-2],[.25,0,'hit'],[.33,-1.2],[.43,0,'hit'],[.5,-.4],[.58,0]]);
const notePose=rigid((t,s)=>({dy:beat(t,s),pivot:[8.2,22]}));
const lyrics=make('lyrics','Lyrics',
 'Light: a bright highlight sweeps along the middle lyric line and fades back. Strong: the note hops on two beats while the highlight sweeps in 0.35 s.',[
 ...posed([...bothFill('head',circle(8.2,22,2.5)),...tube('stem','M10.6 21.4 L10.6 8.5'),...tube('flag','M10.6 8.5 C12.3 10.8 14.6 11.2 14 15.2')],notePose),
 ...[10,16,22].flatMap((y,i)=>shown(tube('line-'+i,`M18 ${y} L26.5 ${y}`),()=>.45)),
 ...shown(tube('sung',(t,s)=>`M18 16 L${18+8.5*sweep(t,s)} 16`),(t,s)=>8.5*sweep(t,s)<.4?0:sweepAlpha(t,s))
],1.02,.86);
lyrics.duotoneFill=false;

// ---------- MUSIC ALBUM (stateful: In sleeve → Disc out) ----------
// Sleeve x5.2–22.2 (moves 1.5 left), disc r7 whose centre slides from 18.7 to 20.7. Only the exposed part of the
// disc is drawn: its rim arc ends are buried in the sleeve edge; a groove arc appears once it clears the sleeve.
const slide=G([[0,0],[.06,0],[.62,1,'io']],[[0,0],[.05,-.12],[.3,1.25],[.42,.92,'hit'],[.54,1.04],[.66,1]],
 [[0,1],[.06,1],[.58,0,'io']],[[0,1],[.05,1.1],[.3,-.1],[.46,0]]);
const albumGeo=(t,s,r)=>{const p=slide(t,s,r),cx=18.7+2*p,edge=22.2-1.5*p;return {p,cx,edge};};
const acos=v=>Math.acos(Math.max(-1,Math.min(1,v)))*180/Math.PI;
const sleeve=rect(5.2,7,17,18,2);
const sleevePose=rigid((t,s,r)=>({dx:-1.5*slide(t,s,r)}));
const grooveSpan=(t,s,r)=>{const {cx,edge}=albumGeo(t,s,r);return acos((edge+3.5-cx)/4.3);};
const musicAlbum=toggle('music-album','Music album',
 'Light: the disc slides 2 units out of the sleeve while the sleeve shifts 1.5 the other way, revealing a groove. Strong: the disc pops out past its stop and settles back. Reverse slides it home.',[
 ...posed([fill('sleeve-interior',sleeve,'tint'),stroke('sleeve',sleeve),fill('sleeve-solid',sleeve,'solid')],sleevePose),
 stroke('disc',(t,s,r)=>{const {cx,edge}=albumGeo(t,s,r),a=acos((edge+.6-cx)/7);return arcN(cx,16,7,-a,a,4);}),
 fill('disc-solid',(t,s,r)=>{const {cx,edge}=albumGeo(t,s,r),a=acos((edge-.5-cx)/7),g=grooveSpan(t,s,r);return cutRev(arcN(cx,16,7,-a,a,4)+' Z',arcTubeN(cx,16,4.3,-g,g,1.6,2));},'solid'),
 ...shown([stroke('groove',(t,s,r)=>{const {cx}=albumGeo(t,s,r),g=grooveSpan(t,s,r);return arcN(cx,16,4.3,-g,g,2);})],(t,s,r)=>clamp01((slide(t,s,r)-.8)/.15))
],.62,.66,'In sleeve','Disc out',.58,.46);
musicAlbum.duotoneFill=true;

// ---------- EARBUDS ----------
// Two wireless buds: rings r3.3 at (10.6,12) and (21.4,12) with stems angling outward to y21. Light: they bob in
// turn. Strong: both hop 2.5 units and tilt outward with a spring.
const bobA=F([[0,0],[.06,0],[.3,-.9],[.62,0,'smooth']],[[0,0],[.05,.4],[.2,-2.6],[.36,0,'hit'],[.44,-.5],[.54,0]]);
const bobB=F([[0,0],[.2,0],[.44,-.9],[.76,0,'smooth']],[[0,0],[.11,.4],[.26,-2.6],[.42,0,'hit'],[.5,-.5],[.6,0]]);
const tiltA=F([[0,0],[.5,0]],[[0,0],[.05,3],[.2,-9],[.36,2,'hit'],[.5,0]]);
const bud=(name,cx,dir,bob,tilt)=>posed([stroke(name,circle(cx,12,3.3)),fill(name+'-solid',circle(cx,12,3.9),'solid'),...tube(name+'-stem',`M${cx+dir*1.9} 14.7 L${cx+dir*3.6} 21`)],rigid((t,s)=>({dy:bob(t,s),angle:dir*tilt(t,s),pivot:[cx,12]})));
const earbuds=make('earbuds','Earbuds',
 'Light: the two buds bob 0.9 units one after the other. Strong: each bud hops 2.6 units and tilts outward 9° with a spring, left then right.',[
 ...bud('bud-a',10.6,-1,bobA,tiltA),...bud('bud-b',21.4,1,bobB,(t,s)=>tiltA(t-.06,s))
],.76,.6);
earbuds.duotoneFill=false;

export const icons=[tuner,pianoKeys,guitar,drum,lyrics,musicAlbum,earbuds];
