// developer-debug-events.mjs — Claude corrected draft (rev 2), batch 2 of Developer & version control.
import {make,toggle,geometricRearm,F,posed,rigid,tube,line,framed,bothFill,fill,circle,rect,capsule,holes,rotate,segment,mix} from './helpers.mjs';

const seg=(a,b)=>segment(a,b);
function along(route,u){let total=route.reduce((s,r)=>s+r.len,0),d=Math.max(0,Math.min(1,u))*total;
 for(const r of route){if(d<=r.len||r===route.at(-1)){const k=r.len?Math.min(1,d/r.len):1,[a,b,c,e]=r.p,m=1-k;
  return[m*m*m*a[0]+3*m*m*k*b[0]+3*m*k*k*c[0]+k*k*k*e[0],m*m*m*a[1]+3*m*m*k*b[1]+3*m*k*k*c[1]+k*k*k*e[1]];}d-=r.len;}}
const ln=(a,e,len)=>({len,p:[a,[mix(a[0],e[0],1/3),mix(a[1],e[1],1/3)],[mix(a[0],e[0],2/3),mix(a[1],e[1],2/3)],e]});
const cv=(a,b,c,e,len)=>({len,p:[a,b,c,e]});

/* ---------- git-stash ---------- */
// CLEARANCE CHANGE (flagged): drop 14 -> 13 so the card rests at y 18..23.
// Solid floor gap was 1.4 (card 25.3 vs floor paint 26.7); now 2.4. Bold outline 1.5 -> 2.5. Timing untouched.
const stashDrop=F([[0,0],[.5,13,'smooth']],[[0,0],[.14,-1.5],[.36,13,'hit'],[.48,12],[.62,13]]);
const gitStash=geometricRearm(make('git-stash','Git stash','Local change card drops into the stash tray and stays there. Strong lifts first, drops fast, lands with one bounce.',[
 ...tube('tray','M6 15 L6 25.5 C6 26.9 7.1 28 8.5 28 L23.5 28 C24.9 28 26 26.9 26 25.5 L26 15'),
 ...posed(framed('card',rect(11,5,10,5,1.6),rect(9.7,3.7,12.6,7.6,2.8)),rigid((t,s)=>({dy:stashDrop(t,s)})))
],.55,.7));
gitStash.duotoneFill=true; // card interior only; tray is open

/* ---------- debug ---------- */
const bugDy=F([[0,0],[.28,-1.2,'smooth'],[.55,0,'smooth']],[[0,0],[.14,1.2],[.32,-3.5,'launch'],[.5,0,'hit'],[.6,-.6],[.72,0]]);
const legAng=F([[0,0],[.28,6,'smooth'],[.55,0,'smooth']],[[0,0],[.14,-8],[.32,14],[.5,-10,'hit'],[.62,3],[.72,0]]);
const bodyPose=rigid((t,s)=>({dy:bugDy(t,s)}));
const leg=(name,a,b,sign)=>posed(line(name,a,b),rigid((t,s)=>({dy:bugDy(t,s),angle:sign*legAng(t,s),pivot:a})));
const debug=make('debug','Debug','The bug flexes and hops; legs tuck in the air and splay on landing. Light is a small breathing lift.',[
 // Solid body: seam is a real 2.6-wide capsule cutout (family solid width), open counter, 4.5 of fill each side.
 ...posed(framed('body',rect(11.5,8,9,15,4.5),holes(rect(10.2,6.7,11.6,17.6,5.8),capsule([16,12.5],[16,18.5],2.6))),bodyPose),
 // Outline/Duotone seam: stroke type, width follows the selected 1.5/2/2.5 preset.
 posed([{name:'seam',d:'M16 12.5 L16 18.5',style:'outline',type:'stroke',width:2}],bodyPose)[0],
 ...leg('leg-lu',[11.5,12],[7.5,9],1),...leg('leg-ll',[11.5,19],[7.5,22],-1),
 ...leg('leg-ru',[20.5,12],[24.5,9],-1),...leg('leg-rl',[20.5,19],[24.5,22],1)
],.55,.72);
debug.duotoneFill=true; // body interior tinted; seam stroke drawn over it, not tinted

/* ---------- breakpoint ---------- */
const bpSet=F([[0,0],[.35,2.6,'smooth']],[[0,0],[.1,0],[.3,3.2],[.46,2.6]]);
const bpClear=F([[0,2.6],[.35,0,'smooth']],[[0,2.6],[.12,3.1],[.34,0]]);
const bpR=(t,s,r)=>r?bpClear(t,s):bpSet(t,s);
const bpVis=(t,s,r)=>Math.max(0,Math.min(1,(bpR(t,s,r)-.3)/.6));
// CLEARANCE CHANGE (flagged): code lines start at x 13 (was 12). At the Strong pop (r 3.2, edge 9.7)
// a Bold cap at 12 reached 10.75, gap 1.05; now 2.05 at peak, 2.65 at rest. Profiles untouched.
const breakpoint=toggle('breakpoint','Breakpoint','A stop marker pops into the gutter beside the active line and holds; clearing shrinks it away.',[
 ...tube('gutter','M6.5 5 L6.5 27'),
 ...tube('line-1','M13 9 L25 9'),...tube('line-2','M13 16 L28 16'),...tube('line-3','M13 23 L22 23'),
 // Marker is an opaque filled disk in every style and sits over the gutter trunk on purpose:
 // gutter half-width (max 1.3) < dot radius, so the trunk is fully covered, never a seam.
 fill('dot',(t,s,r)=>circle(6.5,16,Math.max(.01,bpR(t,s,r))),'outline',{visibility:bpVis}),
 fill('dot-solid',(t,s,r)=>circle(6.5,16,Math.max(.01,bpR(t,s,r))),'solid',{visibility:bpVis})
],.4,.5,'cleared','set');
breakpoint.duotoneFill=false; // dot is a typographic filled mark, not a tint surface; Duotone equals Outline

/* ---------- debug-step ---------- */
const stepDy=F([[0,0],[.5,12,'smooth']],[[0,0],[.12,-1],[.3,6,'pass'],[.44,12,'hit'],[.54,12.6],[.66,12]]);
const stepDx=F([[0,0],[.25,-1.2,'smooth'],[.5,0,'smooth']],[[0,0],[.12,0],[.3,-2.5],[.44,0,'hit'],[.66,0]]);
const debugStep=geometricRearm(make('debug-step','Debug step','The execution pointer steps over the current line to the next one and holds there. Strong winds up and leaps in an outward arc, landing with a short settle.',[
 ...tube('line-1','M14 10 L27 10'),...tube('line-2','M14 16 L24 16'),...tube('line-3','M14 22 L27 22'),
 ...posed(tube('pointer','M6 7 L9.5 10 L6 13'),rigid((t,s)=>({dx:stepDx(t,s),dy:stepDy(t,s)})))
],.55,.7));
debugStep.duotoneFill=false;

/* ---------- regular-expression ---------- */
const spin=F([[0,0],[.55,120,'smooth']],[[0,0],[.12,-14],[.4,128],[.56,120]]);
const arm=(name,a)=>tube(name,seg(rotate(19,6,a,19,13),rotate(19,20,a,19,13)));
const regularExpression=make('regular-expression','Regular expression','The asterisk turns one symmetric third and stops at an equivalent pose. Strong recoils, snaps through with a small overshoot, settles.',[
 ...bothFill('dot',circle(8,23,2.2)),
 ...posed([...arm('arm-a',0),...arm('arm-b',60),...arm('arm-c',120)],rigid((t,s)=>({angle:spin(t,s),pivot:[19,13]})))
],.55,.56);
regularExpression.duotoneFill=false; // dot is a filled typographic counter in every style
regularExpression.cycleDegrees=120;   // aggregate silhouette at 120° equals rest; per-part endpoints intentionally differ

/* ---------- webhook ---------- */
const hookRoute=[ln([8,8],[10.5,8],2.5),ln([10.5,8],[18,8],7.5),cv([18,8],[21.9,8],[24,10.1],[24,14],9.4),ln([24,14],[24,19],5),cv([24,19],[24,22.9],[21.3,25],[18.3,25],9.2),cv([18.3,25],[15.6,25],[13,22.8],[13,20],8)];
const payload=F([[0,0],[.65,1,'smooth']],[[0,0],[.12,0],[.34,1,'hit']]);
const hookAng=F([[0,0],[.65,0],[.85,2,'smooth'],[1.05,0,'smooth']],[[0,0],[.12,-3],[.34,0,'hit'],[.46,6],[.62,-1.5],[.74,0]]);
const hookPose=rigid((t,s)=>({angle:hookAng(t,s),pivot:[8,8]}));
// Payload is a static circle at the source centre. Its rigid pose reproduces the previous world
// trajectory exactly: world = R(8,8,angle)(routePoint). With rigid() = rotate-then-translate and
// the same pivot, the required translation is R(routePoint) - routePoint's rest, i.e. the rotated route offset.
const payloadPose=rigid((t,s)=>{const q=along(hookRoute,payload(t,s)),a=hookAng(t,s),w=rotate(q[0],q[1],a,8,8);return{angle:a,pivot:[8,8],dx:w[0]-8,dy:w[1]-8};});
const webhook=geometricRearm(make('webhook','Webhook','An event payload leaves the source node, runs the hooked callback route and is caught at the hook tip. Strong cocks the hook back, fires the payload fast and the hook swings once on catch.',[
 ...bothFill('source',circle(8,8,2.6)),
 ...posed(tube('hook','M10.5 8 L18 8 C21.9 8 24 10.1 24 14 L24 19 C24 22.9 21.3 25 18.3 25 C15.6 25 13 22.8 13 20'),hookPose),
 ...posed([fill('payload',circle(8,8,2.1),'outline'),fill('payload-solid',circle(8,8,2.1),'solid')],payloadPose)
],1.05,.74));
webhook.duotoneFill=false; // source node and payload are filled marks; hook is open

export const icons=[gitStash,debug,breakpoint,debugStep,regularExpression,webhook];
