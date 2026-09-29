// motion/icon-sets/29-version-control/batch-1.mjs — Claude first-design draft
import {make,F,tube,stroke,fill,bothFill,circle,annulus,rect,holes,posed,rigid,geometricRearm,mix,rotate} from './helpers.mjs';

// Ring node: Outline stroke circle, Solid annulus. Line-symbol counter, never tinted.
const node=(n,x,y)=>[stroke(n,circle(x,y,2.5)),fill(n+'-solid',annulus(x,y,2.5,2.6))];
// Stroked part with a function path (true morph) at family width / Solid 2.6.
const morph=(n,d)=>[{name:n,d,style:'outline',type:'stroke',width:2},{name:n+'-solid',d,style:'solid',type:'stroke',width:2.6}];
// Progressive cubic: same 4-point topology at every u (de Casteljau), no re-topology.
const cut=(P,u)=>{const l=(a,b)=>[mix(a[0],b[0],u),mix(a[1],b[1],u)];const [a,b,c,d]=P,ab=l(a,b),bc=l(b,c),cd=l(c,d),abc=l(ab,bc),bcd=l(bc,cd),e=l(abc,bcd);return `M${a} C${ab} ${abc} ${e}`;};
const open=i=>({...i,duotoneFill:false});

// 1 REPOSITORY — bound book. Fixed: cover frame (7..25 x 6..27) + spine x=11.
// Moving: bookmark ribbon (rigid dy). Gesture: lifts out of the spine top, returns.
const lift=F([[0,0],[.3,-1.2,'smooth'],[.75,0,'smooth']],[[0,0],[.12,.5],[.3,-2.5],[.46,-2],[.78,0]]);
const repository=make('repository','Repository','Opening a repository lifts the bookmark ribbon from the spine; Strong tucks first then lifts higher and settles.',[
 fill('cover-tint',rect(7,6,18,21,3),'tint'),                       // enclosed cover surface (Duotone)
 stroke('cover',rect(7,6,18,21,3)),
 fill('cover-solid',holes(rect(5.7,4.7,20.6,23.6,4.3),rect(12.5,7.3,11.2,18.4,1.4))), // exterior +1.3, page counter right of spine
 ...tube('spine','M11 6 L11 27'),
 ...posed(tube('mark','M19 6 L19 12'),rigid((t,s)=>({dy:lift(t,s)}))),
],.8,.8);

// 2 GIT BRANCH — trunk x=9 with rings (9,6),(9,26). New lane draws OUT of the trunk
// at (9,18) to a new ring (23,10). Result holds; preview reset via geometricRearm.
const grow=F([[0,0],[.65,1,'smooth']],[[0,0],[.1,.08,'pass'],[.36,1]]);
const tipShow=F([[.55,0],[.65,1,'smooth']],[[.3,0],[.36,1]]);
const gitBranch=open(geometricRearm(make('git-branch','Git branch','A new lane diverges from the trunk and its branch ring appears; Strong drives the draw decisively.',[
 ...node('root',9,6),...node('base',9,26),...tube('trunk','M9 8.5 L9 23.5'),
 ...morph('lane',(t,s)=>cut([[9,18],[9,13],[14,10],[20.5,10]],grow(t,s))).map(p=>({...p,visibility:(t,s)=>grow(t,s)>.02?1:0})), // zero-length guard
 ...node('tip',23,10).map(p=>({...p,visibility:tipShow})),
],.7,.5)));

// 3 GIT COMMIT — history lane y=16 broken by a ring r4.5 at (16,16). Moving: a filled
// point (r1.7) drops into the ring and is recorded. Result holds; geometricRearm.
const drop=F([[0,-10],[.6,0,'smooth']],[[0,-10],[.14,-11.5],[.32,.4,'hit'],[.46,0]]);
const gitCommit=open(geometricRearm(make('git-commit','Git commit','A point drops into the empty history ring and stays recorded; Strong lifts first, then lands hard with one bounded settle.',[
 ...tube('lane-left','M3.5 16 L11.5 16'),...tube('lane-right','M20.5 16 L28.5 16'),
 stroke('ring',circle(16,16,4.5)),fill('ring-solid',annulus(16,16,4.5,2.6)),
 ...posed(bothFill('point',circle(16,16,1.7)),rigid((t,s)=>({dy:drop(t,s)}))),
],.7,.5)));

// 4 GIT MERGE — same trunk; branch ring (23,10) already exists. Its lane draws INTO
// the trunk at (9,18), tangent to the downward flow. Result holds; geometricRearm.
const join=F([[0,0],[.65,1,'smooth']],[[0,0],[.1,.1,'pass'],[.34,1]]);
const gitMerge=open(geometricRearm(make('git-merge','Git merge','The branch lane draws down into the continuing trunk; Strong joins with a faster decisive draw.',[
 ...node('root',9,6),...node('base',9,26),...tube('trunk','M9 8.5 L9 23.5'),...node('branch',23,10),
 ...morph('lane',(t,s)=>cut([[20.5,10],[14,10],[9,13],[9,18]],join(t,s))).map(p=>({...p,visibility:(t,s)=>join(t,s)>.02?1:0})),
],.7,.5)));

// 5 PULL REQUEST — lanes stay separate: trunk x=9 with rings; branch ring (23,6) with
// lane down to y=14, curving to a horizontal proposal arrow at y=17 pointing at the
// trunk. Moving: arrowhead (rigid dx) + tail end (function path). Gesture, returns.
const push=F([[0,0],[.3,1,'smooth'],[.7,0,'smooth']],[[0,0],[.12,-.7],[.32,2.5,'hit'],[.46,2.1],[.72,0]]);
const pullRequest=open(make('pull-request','Pull request','The proposal arrow nudges toward the trunk and returns without joining; Strong pulls back, thrusts closer and recoils once.',[
 ...node('root',9,6),...node('base',9,26),...tube('trunk','M9 8.5 L9 23.5'),...node('branch',23,6),
 ...morph('tail',(t,s)=>`M23 8.5 L23 14 C23 15.7 21.7 17 20 17 L${16.5-push(t,s)} 17`),
 ...posed(tube('head','M18.5 14 L15.5 17 L18.5 20'),rigid((t,s)=>({dx:-push(t,s)}))),
],.7,.75));

// 6 REPOSITORY FORK — one source ring (16,6), trunk to junction (16,14), two lanes fan
// to independent rings (9,23),(23,23). Moving: each lane+ring rigid about pivot (16,14).
const fan=F([[0,0],[.3,7,'smooth'],[.75,0,'smooth']],[[0,0],[.12,-5],[.3,18],[.46,15],[.8,0]]);
const fork=(side,x)=>posed([...tube('lane-'+side,`M16 14 L${x} 21`),...node('copy-'+side,x<16?9:23,23)],rigid((t,s)=>({angle:(x<16?1:-1)*fan(t,s),pivot:[16,14]})));
const repositoryFork=open(make('repository-fork','Repository fork','Both copies fan outward from the junction and settle back; Strong gathers first, fans wider, and damps once.',[
 ...node('source',16,6),...tube('trunk','M16 8.5 L16 14'),...fork('left',10.5),...fork('right',21.5),
],.75,.8));

// 7 MERGE CONFLICT — two lanes enter a junction ring r6.5 at (16,18.5). Inside: a
// slash (blocked) that morphs to a check (resolved). Function path, 3 points, fixed
// topology. Result holds; geometricRearm.
const S=[[14,20.5],[15.5,19],[18.1,16.4]],C=[[14,18.7],[15.6,20.3],[18.4,17]];
const solve=F([[0,0],[.55,1,'smooth']],[[0,0],[.14,0],[.34,1.08],[.48,1]]);
const twist=F([[0,0]],[[0,0],[.14,-14],[.34,0]]);
const mergeConflict=open(geometricRearm(make('merge-conflict','Merge conflict','The blocking slash at the junction resolves into a check; Strong winds the slash back, then snaps to the check with one bounded overshoot.',[
 ...tube('lane-left','M7 7 L7 10 C7 13 8.5 13.9 11.4 13.9'),...tube('lane-right','M25 7 L25 10 C25 13 23.5 13.9 20.6 13.9'),
 stroke('junction',circle(16,18.5,6.5)),fill('junction-solid',annulus(16,18.5,6.5,2.6)),
 ...morph('mark',(t,s)=>{const p=solve(t,s),a=twist(t,s);return S.map((q,i)=>rotate(mix(q[0],C[i][0],p),mix(q[1],C[i][1],p),a,16,18.5)).map((q,i)=>(i?'L':'M')+q.join(' ')).join(' ');}),
],.6,.5)));

export const icons=[repository,gitBranch,gitCommit,gitMerge,pullRequest,repositoryFork,mergeConflict];
