// Education batch C: abacus, school-bell, course-progress, lecture, library-shelf.
import {make,F,tube,framed,rigid,posed,shown,circle,polygon,rect,rotate,bothFill,fill,stroke} from './helpers.mjs';
import {container,pen} from './edu.mjs';

// ---------- ABACUS ----------
// Three rods with tall capsule beads (3.6 × 5.5): each bead stands 1.5 clear of the Bold rod stroke on both
// sides, beads are 4.6 apart (≥1 painted gap) and keep ≥1 to the frame. v1's round r1.7 beads fused onto
// the rods at Bold and read as a list. Beads are counted across to the right, one after another.
const ROWS=[8.8,16,23.2],LEFT=[7.6,12.2,16.8],RIGHT=[24.4,19.8],COUNT=[3,3,2];
const MOVES=[[0,2,0],[1,2,0],[1,1,1],[2,1,0]]; // [row, bead index, right slot]
const move=(k)=>F([[0,0],[.06+k*.14,0],[.28+k*.14,1,'smooth']],[[0,0],[.08+k*.08,0,'hold'],[.18+k*.08,1,'hit'],[.22+k*.08,.96],[.27+k*.08,1]]);
const beadX=(r,b)=>{const m=MOVES.findIndex(([rr,bb])=>rr===r&&bb===b);if(m<0)return ()=>LEFT[b];const [, ,slot]=MOVES[m],p=move(m);return (t,s)=>LEFT[b]+(RIGHT[slot]-LEFT[b])*p(t,s);};
const bead=(x,y)=>rect(x-1.8,y-2.75,3.6,5.5,1.8);
const abacus=make('abacus','Abacus',
 'Light: four beads slide across their rods one after another to record a count. Strong: each bead is flicked across in 0.1 s and knocks to a stop with a small recoil. The count is retained; preview reset rearms.',[
 ...container('frame',rect(3.5,3.5,25,25,2)),
 ...ROWS.flatMap((y,r)=>tube('rod-'+r,`M3.5 ${y} L28.5 ${y}`)),
 ...ROWS.flatMap((y,r)=>LEFT.slice(0,COUNT[r]).flatMap((x,b)=>{const bx=beadX(r,b);return posed(bothFill(`bead-${r}-${b}`,bead(x,y)),(px,py,t,s)=>[px-x+bx(t,s),py]);}))
],.7,.52,MOVES.flatMap(([r,b])=>[`bead-${r}-${b}`,`bead-${r}-${b}-solid`]));
abacus.duotoneFill=true;

// ---------- SCHOOL BELL ----------
// A hand bell with a wooden handle, shaken from the grip; the clapper swings about the lip centre and lags.
const bellD='M13.6 11.6 L18.4 11.6 C19.6 11.6 20.4 12.4 20.6 13.6 L21.4 19.2 C21.6 20.8 22.6 22.2 24 23 C24.6 23.3 24.4 23.6 23.8 23.6 L8.2 23.6 C7.6 23.6 7.4 23.3 8 23 C9.4 22.2 10.4 20.8 10.6 19.2 L11.4 13.6 C11.6 12.4 12.4 11.6 13.6 11.6 Z';
const handleD='M14.2 11.6 L14.2 6.8 C14.2 5.8 15 5 16 5 C17 5 17.8 5.8 17.8 6.8 L17.8 11.6';
const GRIP=[16,7.5],LIP=[16,23.6];
const shakeA=F([[0,0],[.1,-8],[.26,7],[.42,-3],[.56,0,'smooth']],[[0,0],[.08,5],[.2,-15,'hit'],[.36,13],[.52,-6],[.66,2],[.76,0]]);
const lag=F([[0,0],[.14,9],[.3,-9],[.46,4],[.62,0,'smooth']],[[0,0],[.12,-6],[.24,16],[.4,-14],[.56,7],[.7,-2],[.82,0]]);
const bell=(x,y,t,s)=>rotate(x,y,shakeA(t,s),...GRIP);
const schoolBell=make('school-bell','School bell',
 'Light: the bell is shaken side to side (8°) from its handle and the clapper swings after it. Strong: a small back-swing, a hard 15° shake either way, and the clapper rings wide before damping out.',[
 ...posed(framed('body',bellD),bell),
 ...posed([fill('handle-interior',handleD+' Z','tint'),stroke('handle',handleD),fill('handle-solid',handleD+' Z','solid')],bell),
 ...posed(bothFill('clapper',circle(16,26.9,1.5)),(x,y,t,s)=>{const [a,b]=rotate(x,y,lag(t,s),...LIP);return bell(a,b,t,s);})
],.64,.84);
schoolBell.duotoneFill=true;

// ---------- COURSE PROGRESS ----------
// A lesson card (play button) above a progress scrubber; pressing play advances the course to 75%.
const HEAD=F([[0,6.5],[.08,6.5],[.72,20.5,'smooth']],[[0,6.5],[.16,6.5,'hold'],[.4,20.5]]);
const playK=F([[0,1],[.72,1]],[[0,1],[.06,.84],[.16,1.1,'hit'],[.26,1]]);
const PC=[16.2,12.5];
const courseProgress=make('course-progress','Course progress',
 'Light: the progress scrubber advances steadily to three quarters of the course. Strong: the play button is pressed in and springs out, then the scrubber drives to the same point in 0.24 s. Progress is retained; preview reset rearms.',[
 ...container('card',rect(5,4.5,22,16,2.5)),
 ...posed(framed('play',polygon([[13.6,9],[13.6,16],[19.6,12.5]],.9)),(x,y,t,s)=>{const k=playK(t,s);return[PC[0]+(x-PC[0])*k,PC[1]+(y-PC[1])*k];}),
 ...shown(tube('track','M6.5 25.6 L25.5 25.6'),()=>.35),
 ...shown(pen('fill',(t,s)=>`M6.5 25.6 L${Math.max(6.52,HEAD(t,s))} 25.6`),(t,s)=>HEAD(t,s)>6.6?1:0),
 ...posed(bothFill('knob',circle(6.5,25.6,2.1)),(x,y,t,s)=>[x+HEAD(t,s)-6.5,y])
],.72,.44,['fill','fill-solid','knob','knob-solid']);
courseProgress.duotoneFill=true;

// ---------- LECTURE ----------
// A presenter beside a screen: the pointer taps up at the screen as each bullet line is presented.
const HAND=[10.6,22.8],REST=-52;
const point=F([[0,-40],[.12,-40],[.3,-57],[.46,-50],[.62,-52,'smooth']],[[0,-40],[.06,-36],[.16,-58,'hit'],[.26,-49],[.34,-58,'hit'],[.46,-52]]);
const b1=F([[0,0],[.2,0],[.42,1,'smooth']],[[0,0],[.16,0,'hold'],[.24,1]]);
const b2=F([[0,0],[.42,0],[.62,1,'smooth']],[[0,0],[.34,0,'hold'],[.42,1]]);
const tip=[HAND[0]+5.4*Math.cos(REST*Math.PI/180),HAND[1]+5.4*Math.sin(REST*Math.PI/180)];
const bullet=(y,x1,p)=>shown(pen('bullet-'+(y<10?0:1),(t,s)=>`M16 ${y} L${16+(x1-16)*Math.max(.002,p(t,s))} ${y}`),(t,s)=>p(t,s)>.01?1:0);
const lecture=make('lecture','Lecture',
 'Light: the presenter raises the pointer to the screen as two bullet lines appear in turn. Strong: two sharp taps of the pointer, each snapping a bullet line onto the screen. The slide is retained; preview reset rearms.',[
 ...container('screen',rect(12.5,4.5,15,10.7,1.8)),
 ...framed('head',circle(7,16.6,2.2)),
 ...tube('shoulders','M3.6 27.6 C3.6 24.2 5.1 22.4 7 22.4 C8.9 22.4 10.4 24.2 10.4 27.6'),
 ...posed(tube('pointer',`M${HAND} L${tip}`),rigid((t,s)=>({angle:point(t,s)-REST,pivot:HAND}))),
 ...bullet(8.2,23.5,b1),...bullet(11.6,20.5,b2)
],.62,.46,['pointer','bullet-0','bullet-0-solid','bullet-1','bullet-1-solid']);
lecture.duotoneFill=true;

// ---------- LIBRARY SHELF ----------
// Three 4.6-wide books on a shelf, the right one leaning on its bottom-right corner. One band per spine,
// 3.5 below the top (v1's bands sat 3 from narrow tops and fused at Bold). The middle book is pulled up and back.
const pull=F([[0,0],[.14,-2.2],[.46,0,'smooth']],[[0,0],[.18,-3.6],[.32,-2.8],[.46,0,'hit'],[.54,-.35],[.62,0]]);
const BK=[[4.8,10],[12.9,7],[21,12.5]];
const lean=(x,y)=>rotate(x,y,8,25.6,26);
const book=(i,[x,top])=>[...container('book-'+i,rect(x,top,4.6,26-top,1.2)),...tube('band-'+i,`M${x} ${top+3.5} L${x+4.6} ${top+3.5}`)];
const libraryShelf=make('library-shelf','Library shelf',
 'Light: the middle book is drawn 2.2 units up off the shelf and slid back. Strong: it is yanked 3.6 units, then pushed home with a knock and a small rebound.',[
 ...tube('shelf','M4 26 L28 26'),
 ...book(0,BK[0]),
 ...posed(book(1,BK[1]),(x,y,t,s)=>[x,y+pull(t,s)]),
 ...posed(book(2,BK[2]),(x,y)=>lean(x,y))
],.46,.64);
libraryShelf.duotoneFill=true;

export const icons=[abacus,schoolBell,courseProgress,lecture,libraryShelf];
