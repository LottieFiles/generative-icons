import {make,F,tube,rigid,posed,toggle,mix} from './helpers.mjs';
// ---------- letter-case (toggle lowercase <-> uppercase, compatible morph) ----------
// One path, one M, fixed command order: M C C L L C C L L (17 points).
// Lowercase: 4-cubic circle (16,19) r4.5 starting right, both inserted Ls at the left point,
// stem retraces from the right point up to 13.5 then down to 24.
// Uppercase: triangular counter right(21.5,19) top(16,8) left(10.5,19) with straight cubics
// (linearly placed controls), left leg detour to (8,24), crossbar left→mid→right, right leg to (24,24).
// Counter stays open throughout (uppercase inradius ≈3.4u > 1.3u half stroke). No opacity change.
const LC_LOWER=[[20.5,19],[20.5,16.515],[18.485,14.5],[16,14.5],[13.515,14.5],[11.5,16.515],[11.5,19],
 [11.5,19],[11.5,19],[11.5,21.485],[13.515,23.5],[16,23.5],[18.485,23.5],[20.5,21.485],[20.5,19],
 [20.5,13.5],[20.5,24]];
const LC_UPPER=[[21.5,19],[19.667,15.333],[17.833,11.667],[16,8],[14.167,11.667],[12.333,15.333],[10.5,19],
 [8,24],[10.5,19],[12.333,19],[14.167,19],[16,19],[17.833,19],[19.667,19],[21.5,19],
 [21.5,19],[24,24]];
const LC_CMDS=['M','C','','','C','','','L','L','C','','','C','','','L','L'];
const lcPath=u=>LC_LOWER.map((p,i)=>{
 const x=mix(p[0],LC_UPPER[i][0],u).toFixed(3),y=mix(p[1],LC_UPPER[i][1],u).toFixed(3);
 return (LC_CMDS[i]?(i?' ':'')+LC_CMDS[i]:' ')+x+' '+y;
}).join('');
// Strong: brief prep away from the target (apex dips / bowl tightens), fast drive with a small
// overshoot (apex 7.6, painted 6.3; reverse prep u=1.08 apex 7.1, painted 5.8), bounded settle.
const lcP=F([[0,0],[.5,1,'smooth']],[[0,0],[.1,-.08],[.32,1.06],[.48,1]]);
const lcU=(t,s,r)=>r?1-lcP(t,s):lcP(t,s);
export const letterCase=toggle('letter-case','Letter case',
 'Lowercase a morphs into uppercase A: the bowl rises into the triangular counter, the stem leans into the right leg and the left leg unfolds. Reverse returns the lowercase form.',
 tube('glyph',(t,s,r)=>lcPath(lcU(t,s,r))),
 .5,.5,'lowercase','uppercase');
Object.assign(letterCase,{duotoneFill:false});

