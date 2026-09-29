// motion/icon-sets/29-version-control/batch-1-revision.mjs — Claude revision draft
// Only the five revised icons. Repository and repository-fork stay in the accepted module.
import {make,F,tube,stroke,fill,circle,annulus,posed,rigid,geometricRearm,mix,rotate} from './helpers.mjs';

const clamp=x=>Math.max(0,Math.min(1,x));
const open=i=>({...i,duotoneFill:false});
// Ring node used at rest: Outline stroke circle, Solid annulus. Open counter, never tinted.
const node=(n,x,y)=>[stroke(n,circle(x,y,2.5)),fill(n+'-solid',annulus(x,y,2.5,2.6))];
// Function-path stroke at family width; Solid is the same centerline as a 2.6 tube.
const morph=(n,d,extra={})=>[{name:n,d,style:'outline',type:'stroke',width:2,...extra},{name:n+'-solid',d,style:'solid',type:'stroke',width:2.6,...extra}];

// Constant-topology progressive draw. Segments are [p0,p3] lines or [p0,p1,p2,p3] cubics.
// Completed segments print in full, the current one is cut by de Casteljau, and
// future segments collapse onto the pen tip. The command sequence never changes.
const lerp=(a,b,u)=>[mix(a[0],b[0],u),mix(a[1],b[1],u)];
const approxLen=s=>{const chord=Math.hypot(s.at(-1)[0]-s[0][0],s.at(-1)[1]-s[0][1]);if(s.length===2)return chord;let poly=0;for(let i=1;i<4;i++)poly+=Math.hypot(s[i][0]-s[i-1][0],s[i][1]-s[i-1][1]);return (chord+poly)/2;};
function drawn(segments,u){
 const lens=segments.map(approxLen),total=lens.reduce((a,b)=>a+b,0);let cum=0,pen=segments[0][0],d='M'+pen;
 segments.forEach((s,i)=>{
  const v=clamp((u*total-cum)/lens[i]);cum+=lens[i];
  if(s.length===2){const e=lerp(s[0],s[1],v);d+=' L'+e;if(v>0)pen=e;return;}
  const [a,b,c,e]=s;
  if(v<=0){d+=` C${pen} ${pen} ${pen}`;return;}
  const ab=lerp(a,b,v),bc=lerp(b,c,v),cd=lerp(c,e,v),abc=lerp(ab,bc,v),bcd=lerp(bc,cd,v),q=lerp(abc,bcd,v);
  d+=` C${ab} ${abc} ${q}`;pen=q;
 });
 return d;
}
// Four quarter-cubic segments of a circle, starting at startDeg and running clockwise on screen.
function ringSegments(cx,cy,r,startDeg){
 const k=.552285,seg=[];
 for(let i=0;i<4;i++){
  const s=(startDeg+i*90)*Math.PI/180,e=s+Math.PI/2,u=[cx+r*Math.cos(s),cy+r*Math.sin(s)],v=[cx+r*Math.cos(e),cy+r*Math.sin(e)];
  seg.push([u,[u[0]-k*r*Math.sin(s),u[1]+k*r*Math.cos(s)],[v[0]+k*r*Math.sin(e),v[1]-k*r*Math.cos(e)],v]);
 }
 return seg;
}
const shownWhen=(parts,fn)=>parts.map(p=>({...p,visibility:(t,s)=>fn(t,s)>.02?1:0}));

// 1 GIT BRANCH — trunk x=9 with rings (9,6),(9,26). One drive: the lane draws out
// of the trunk at (9,18) to (20.5,10), then the tip ring draws from that arrival
// point around 360 degrees. Both are opaque strokes joined at the pen. Result holds.
const branchLane=[[[9,18],[9,13],[14,10],[20.5,10]]];
const branchRing=ringSegments(23,10,2.5,180);
const grow=F([[0,0],[.7,1,'smooth']],[[0,0],[.12,.14,'pass'],[.42,1]]);
const laneU=(t,s)=>clamp(grow(t,s)/.47),ringU=(t,s)=>clamp((grow(t,s)-.47)/.53);
const gitBranch=open(geometricRearm(make('git-branch','Git branch','A new lane draws out of the trunk and its tip ring draws closed at the arrival point; Strong drives the same stroke decisively.',[
 ...node('root',9,6),...node('base',9,26),...tube('trunk','M9 8.5 L9 23.5'),
 ...shownWhen(morph('lane',(t,s)=>drawn(branchLane,laneU(t,s))),laneU),
 ...shownWhen(morph('tip',(t,s)=>drawn(branchRing,ringU(t,s))),ringU),
],.7,.5)));

// 2 GIT MERGE — Y silhouette. Source rings (9,6),(23,6). Left lane and the continuing
// trunk to the target ring (16,26) are fixed. The incoming right lane draws from its
// ring down into the junction (16,16). Result holds.
const mergeLane=[[[23,8.5],[23,11]],[[23,11],[23,13.5],[19,14.5],[16,16]]];
const join=F([[0,0],[.65,1,'smooth']],[[0,0],[.1,.12,'pass'],[.36,1]]);
const gitMerge=open(geometricRearm(make('git-merge','Git merge','The incoming lane draws from its source ring into the junction where history continues to the target; Strong joins with a faster decisive draw.',[
 ...node('left',9,6),...node('right',23,6),...node('target',16,26),
 ...tube('lane-left','M9 8.5 L9 11 C9 13.5 13 14.5 16 16'),...tube('trunk','M16 16 L16 23.5'),
 ...shownWhen(morph('lane-right',(t,s)=>drawn(mergeLane,join(t,s))),join),
],.65,.45)));

// 3 GIT COMMIT — history lane y=16 broken by a ring r4.5 at (16,16). The point is a
// filled disk that grows in place from radius 0 to 1.7. Strong presses to 1.95 then
// settles. Solid inner wall 3.2: gap at 1.95 = 1.25. Bold inner 3.25 minus 1.25 = 1.30.
const record=F([[0,0],[.5,1.7,'smooth']],[[0,0],[.2,1.95],[.36,1.7]]);
const point=(t,s)=>circle(16,16,record(t,s));
const gitCommit=open(geometricRearm(make('git-commit','Git commit','A point is recorded inside the empty history ring and stays; Strong presses it in larger and settles once.',[
 ...tube('lane-left','M3.5 16 L11.5 16'),...tube('lane-right','M20.5 16 L28.5 16'),
 stroke('ring',circle(16,16,4.5)),fill('ring-solid',annulus(16,16,4.5,2.6)),
 fill('point',point,'outline'),fill('point-solid',point,'solid'),
],.5,.4)));

// 4 PULL REQUEST — trunk x=9 with rings; branch ring (24,6), tail down to y=15, bend
// radius 2 ending at (22,17), straight approach to the head tip at 15.5. Head is rigid
// dx, tail end follows the same push so they stay attached. Gesture, returns.
const push=F([[0,0],[.3,1,'smooth'],[.7,0,'smooth']],[[0,0],[.12,-.6],[.3,2.5,'hit'],[.42,2.1],[.72,0]]);
const pullRequest=open(make('pull-request','Pull request','The proposal arrow nudges toward the trunk and returns without joining; Strong draws back, thrusts closer with a hit and recoils once.',[
 ...node('root',9,6),...node('base',9,26),...tube('trunk','M9 8.5 L9 23.5'),...node('branch',24,6),
 ...morph('tail',(t,s)=>`M24 8.5 L24 15 C24 16.1 23.1 17 22 17 L${15.5-push(t,s)} 17`),
 ...posed(tube('head','M18.5 14 L15.5 17 L18.5 20'),rigid((t,s)=>({dx:-push(t,s),pivot:[15.5,17]}))),
],.7,.75));

// 5 MERGE CONFLICT — unchanged lanes and junction ring r6.5 at (16,18.5). The mark is
// smaller: every point stays within 2.69 of the center. Solid inner wall 5.2, mark
// half width 1.3: gap 1.21 at the widest pose. Bold: 5.25 - 2.69 - 1.25 = 1.31.
const S=[[14.1,20.4],[16,18.5],[17.9,16.6]],C=[[13.6,18.6],[15.5,20.4],[18.2,17.1]];
const solve=F([[0,0],[.55,1,'smooth']],[[0,0],[.34,1.08],[.48,1]]);
const twist=F([[0,0]],[[0,0],[.14,-14],[.34,0]]);
const mergeConflict=open(geometricRearm(make('merge-conflict','Merge conflict','The blocking slash at the junction resolves into a check; Strong winds the slash back, then snaps to the check with one bounded overshoot.',[
 ...tube('lane-left','M7 7 L7 10 C7 13 8.5 13.9 11.4 13.9'),...tube('lane-right','M25 7 L25 10 C25 13 23.5 13.9 20.6 13.9'),
 stroke('junction',circle(16,18.5,6.5)),fill('junction-solid',annulus(16,18.5,6.5,2.6)),
 ...morph('mark',(t,s)=>{const p=solve(t,s),a=twist(t,s);return S.map((q,i)=>rotate(mix(q[0],C[i][0],p),mix(q[1],C[i][1],p),a,16,18.5)).map((q,i)=>(i?'L':'M')+q.join(' ')).join(' ');}),
],.6,.5)));

export const icons=[gitBranch,gitCommit,gitMerge,pullRequest,mergeConflict];
