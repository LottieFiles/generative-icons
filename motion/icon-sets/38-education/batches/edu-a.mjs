// Education batch A: graduation-cap, chalkboard, flashcards, quiz, certificate.
import {make,F,tube,framed,rigid,posed,shown,circle,polygon,rect,rotate,bothFill} from './helpers.mjs';
import {arcN} from './sky.mjs';
import {container,pen,reveal,lerp} from './edu.mjs';

// ---------- GRADUATION CAP ----------
// Mortarboard diamond over a skull band whose ends sit exactly on the board's lower edges.
// The tassel hangs from the cord end and swings about it; it lags the cap's toss.
const board=polygon([[16,8],[28,13],[16,18],[4,13]],.9);
// v1 ran a cord from the centre to the tassel; at Bold it fused with the board's edges near the tip.
// The tassel now hangs straight from a point on the board's lower-right edge.
const TP=[25.4,14.08];
const capDy=F([[0,0],[.18,-1.4],[.46,0,'smooth']],[[0,0],[.08,.7],[.28,-3.1],[.48,.25,'hit'],[.6,0]]);
const capA=F([[0,0],[.18,-4],[.46,0,'smooth']],[[0,0],[.08,3],[.28,-11],[.48,1],[.62,0]]);
// Positive angles are clockwise, which swings the hanging tassel toward the band: Strong's big swing goes outward.
const swing=F([[0,0],[.14,6],[.3,-5],[.46,2],[.6,0,'smooth']],[[0,0],[.1,3],[.3,-9],[.48,4],[.64,-2],[.8,0]]);
const cap=(x,y,t,s)=>{const [a,b]=rotate(x,y,capA(t,s),16,16);return [a,b+capDy(t,s)];};
const graduationCap=make('graduation-cap','Graduation cap',
 'Light: the cap lifts 1.4 units with a 4° tilt and the tassel swings behind it. Strong: it dips, is tossed 3.1 units with an 11° tilt, lands with a small bounce, and the tassel swings wide and damps out.',[
 ...posed(framed('board',board),cap),
 ...posed(tube('band','M10.5 15.5 L10.5 21 C10.5 23.1 13 24.5 16 24.5 C19 24.5 21.5 23.1 21.5 21 L21.5 15.5'),cap),
 ...posed(tube('tassel',`M${TP} L${TP[0]} 19.6`),(x,y,t,s)=>{const [a,b]=rotate(x,y,swing(t,s),...TP);return cap(a,b,t,s);})
],.6,.8);
graduationCap.duotoneFill=true;

// ---------- CHALKBOARD ----------
// Board on an easel. Chalk writes one wavy line, then underlines it. Result retained.
const write=F([[0,0],[.06,0],[.58,1,'smooth']],[[0,0],[.08,0,'hold'],[.3,1]]);
const under=F([[0,0],[.58,0],[.78,1,'smooth']],[[0,0],[.34,0,'hold'],[.42,1]]);
const tap=F([[0,0],[.8,0]],[[0,0],[.42,0],[.46,.6,'hit'],[.56,-.2],[.66,0]]);
const inkFn=u=>[8.6+13.8*u,12.4-2*Math.sin(2*Math.PI*1.5*u)];
const easel=rigid((t,s)=>({dx:tap(t,s)}));
const chalkboard=make('chalkboard','Chalkboard',
 'Light: chalk writes a wavy line across the board, then underlines it. Strong: the line is written in 0.22 s, the underline snaps in, and the chalk tap jolts the board. The writing is retained; preview reset rearms.',[
 ...posed(container('board',rect(4.5,4.5,23,17,2)),easel),
 ...posed(tube('leg-l','M9.5 21.5 L7.6 27.6'),easel),...posed(tube('leg-r','M22.5 21.5 L24.4 27.6'),easel),
 ...posed(shown(pen('ink',reveal(inkFn,write,4)),(t,s)=>write(t,s)>.01?1:0),easel),
 ...posed(shown(pen('underline',(t,s)=>`M9 17.9 L${9+9.5*Math.max(.002,under(t,s))} 17.9`),(t,s)=>under(t,s)>.01?1:0),easel)
],.8,.66,['ink','ink-solid','underline','underline-solid']);
chalkboard.duotoneFill=true;

// ---------- FLASHCARDS ----------
// A landscape card flips about its vertical centre (edge-on at the midpoint) from the question mark to the
// answer "A"; a line beneath shows the deck. v1 was a square card ending on a check, which read as a checkbox.
const flip=F([[0,0],[.06,0],[.56,1,'smooth']],[[0,0],[.1,0,'hold'],[.36,1]]);
const lift=F([[0,0],[.56,0]],[[0,0],[.1,-1.4],[.36,-1],[.46,0,'hit'],[.52,-.2],[.58,0]]);
const card=(x,y,t,s)=>[16+(x-16)*Math.abs(Math.cos(Math.PI*flip(t,s))),y+lift(t,s)];
// Face marks show only while the card is ≥60% wide; foreshortened past that they would crowd its edges.
const front=(t,s)=>Math.cos(Math.PI*flip(t,s))>.6?1:0,back=(t,s)=>Math.cos(Math.PI*flip(t,s))<-.6?1:0;
const flashcards=make('flashcards','Flashcards',
 'Light: the card turns edge-on and over, from the question mark to the answer. Strong: the card lifts off the deck, flips in a quarter second and lands with a small tap. The answer is retained; preview reset rearms.',[
 ...tube('deck','M7 25.8 L25 25.8'),
 ...posed(container('card',rect(4.5,6,23,16.4,2.4)),card),
 ...posed(shown(tube('q-hook','M13.4 11.9 C13.4 10.2 14.6 9.2 16 9.2 C17.5 9.2 18.6 10.3 18.6 11.7 C18.6 13.5 16 13.8 16 15.8'),front),card),
 ...posed(shown(tube('q-dot','M16 19.2 L16 19.3'),front),card),
 ...posed(shown([...tube('answer-legs','M12.8 18.6 L16 10.2 L19.2 18.6'),...tube('answer-bar','M14 15.4 L18 15.4')],back),card)
],.56,.56,['q-hook','q-hook-solid','q-dot','q-dot-solid','answer-legs','answer-legs-solid','answer-bar','answer-bar-solid']);
flashcards.duotoneFill=true;

// ---------- QUIZ ----------
// A quiz card: a stroke "?" at the top and two answer rows (marker ring + line). v1 was a buzzer dome that
// read as a hat. The first answer is selected: a filled dot grows in its ring (retained); Strong pops the
// whole marker (12%). Rows are 6.7 apart so the rings keep ≥1 painted gap at Bold, even at the pop.
const ROWQ=[17.3,24],MK=[9.4,17.3];
const pick=F([[0,0],[.1,0],[.4,1,'smooth']],[[0,0],[.12,0,'hold'],[.2,1]]);
const popQ=F([[0,1],[.5,1]],[[0,1],[.18,1],[.26,1.12,'hit'],[.36,.96],[.46,1]]);
const marker=(x,y,t,s)=>{const k=popQ(t,s);return[MK[0]+(x-MK[0])*k,MK[1]+(y-MK[1])*k];};
const quiz=make('quiz','Quiz',
 'Light: the first answer on the quiz card is selected as a dot grows inside its ring. Strong: the dot snaps in and the whole marker pops 12% and settles. The answer is retained; preview reset rearms.',[
 ...container('card',rect(4.4,3.3,23.2,25.4,2.5)),
 ...tube('q-hook','M14 8.7 C14 7.6 14.9 7 16 7 C17.1 7 18 7.7 18 8.7 C18 9.9 16 10.1 16 11.2'),
 ...tube('q-dot','M16 13.7 L16 13.8'),
 ...ROWQ.flatMap((y,i)=>[...(i===0?posed(tube('ring-0',circle(9.4,y,1.5)),marker):tube('ring-1',circle(9.4,y,1.5))),...tube('row-'+i,`M14.8 ${y} L23 ${y}`)]),
 ...shown(posed(bothFill('answer-dot',(t,s)=>circle(...MK,Math.max(.02,1.5*pick(t,s)))),marker),(t,s)=>pick(t,s)>.03?1:0)
],.46,.5,['answer-dot','answer-dot-solid']);
quiz.duotoneFill=true;

// ---------- CERTIFICATE ----------
// Portrait certificate with two text lines. The seal is stamped on (Strong: slammed from 1.9× scale),
// then its two ribbon tails unfurl from the seal's rim. Result retained.
const SEAL=[16,19];
const sealK=F([[0,.01],[.08,.01],[.4,1,'smooth']],[[0,1.9],[.1,1.9,'hold'],[.22,1,'hit'],[.3,.95],[.38,1]]);
const sealShow=F([[0,0],[.08,0],[.2,1]],[[0,0],[.1,0],[.2,1]]);
const tails=F([[0,0],[.34,0],[.56,1,'smooth']],[[0,0],[.24,0,'hold'],[.36,1]]);
const jolt=F([[0,0],[.6,0]],[[0,0],[.22,0],[.26,.6,'hit'],[.4,0]]);
const paper=rigid((t,s)=>({dy:jolt(t,s)}));
const tail=(a,b)=>(t,s)=>{const g=Math.max(.01,tails(t,s)),e=lerp(a,b,g);return `M${a} L${e}`;};
const sealPose=(x,y,t,s)=>{const k=sealK(t,s);return[SEAL[0]+(x-SEAL[0])*k,SEAL[1]+(y-SEAL[1])*k+jolt(t,s)];};
const certificate=make('certificate','Certificate',
 'Light: the seal grows onto the certificate and its ribbon tails unfurl. Strong: the seal slams down from twice its size, the paper jolts, and the tails snap out. The seal is retained; preview reset rearms.',[
 ...posed(container('paper',rect(7,4.5,18,23.5,2)),paper),
 ...posed(tube('line-0','M10.5 8.6 L21.5 8.6'),paper),...posed(tube('line-1','M10.5 12.2 L18 12.2'),paper),
 ...shown(posed(framed('seal',circle(...SEAL,3.1)),sealPose),(t,s)=>sealShow(t,s)),
 ...posed(shown(pen('tail-l',tail([14.6,21.8],[13.6,24.6])),(t,s)=>tails(t,s)>.03?1:0),paper),
 ...posed(shown(pen('tail-r',tail([17.4,21.8],[18.4,24.6])),(t,s)=>tails(t,s)>.03?1:0),paper)
],.56,.44,['seal-interior','seal-frame','seal-solid','tail-l','tail-l-solid','tail-r','tail-r-solid']);
certificate.duotoneFill=true;

export const icons=[graduationCap,chalkboard,flashcards,quiz,certificate];
