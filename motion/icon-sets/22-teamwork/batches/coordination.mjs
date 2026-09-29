// teamwork batch 02 rev2: task coordination. First design revision, unverified estimates.
import {make,stroke,fill,tube,framed,bothFill,circle,rect,hole,annulus,rigid,curve,mix,clamp,geometricRearm} from './helpers.mjs';

// Fixed-topology partial polyline: unrevealed vertices collapse onto the draw front.
function trace(pts,p){const L=[0];for(let i=1;i<pts.length;i++)L.push(L[i-1]+Math.hypot(pts[i][0]-pts[i-1][0],pts[i][1]-pts[i-1][1]));const d=L.at(-1)*clamp(p);let front=pts[0],out='M'+front.join(' ');for(let i=1;i<pts.length;i++){if(d>=L[i])front=pts[i];else if(d>L[i-1]){const u=(d-L[i-1])/(L[i]-L[i-1]);front=[mix(pts[i-1][0],pts[i][0],u),mix(pts[i-1][1],pts[i][1],u)];}out+=' L'+front[0]+' '+front[1];}return out;}
const barb=(tip,a,b,p)=>`M${mix(tip[0],a[0],p)} ${mix(tip[1],a[1],p)} L${tip[0]} ${tip[1]} L${mix(tip[0],b[0],p)} ${mix(tip[1],b[1],p)}`;
const on=p=>p>0?1:0;
// Bracket with real rounded spine corners (cubic), arms pointing toward ax.
const bracket=(ax,sx,y0,y1,r=1.6)=>{const k=.552*r,dir=Math.sign(sx-ax),ix=sx-dir*r;return `M${ax} ${y0} L${ix} ${y0} C${ix+dir*k} ${y0} ${sx} ${y0+r-k} ${sx} ${y0+r} L${sx} ${y1-r} C${sx} ${y1-r+k} ${ix+dir*k} ${y1} ${ix} ${y1} L${ax} ${y1}`;};
const avatar=(head,shoulders,solidShoulders)=>[stroke('avatar-head',circle(...head)),fill('avatar-head-solid',circle(...head)),stroke('avatar-shoulders',shoulders),fill('avatar-shoulders-solid',solidShoulders)];

// 1 assign-task — avatar docks onto the task tile, check confirms ownership
const assign=(()=>{
 const drop=(t,s)=>curve(t,s,[[0,-3.5],[.5,0,'out']],[[0,-3.5],[.05,-3.5],[.22,0,'hit'],[.32,-.6],[.42,0]]);
 const chk=(t,s)=>curve(t,s,[[.45,0],[.8,1,'out']],[[.24,0],[.46,1]]);
 const tileD=rect(3.4,7,13.8,18,3),tilePose=rigid((t,s)=>({dy:drop(t,s)}));
 const parts=[
  ...framed('tile',tileD,hole(tileD,rect(5.6,11,9.4,9.8,1.8))),
  ...tube('check',(t,s)=>trace([[8,16.4],[9.9,18.3],[12.6,14.4]],chk(t,s)),{visibility:(t,s)=>on(chk(t,s))}),
  ...avatar([24,11.5,2.3],'M20.8 21.5 C20.8 17.3 27.2 17.3 27.2 21.5','M20.5 21.6 C20.5 16.6 27.5 16.6 27.5 21.6 Z').map(p=>({...p,pose:tilePose})),
 ];
 return {...make('assign-task','Assign task','The assignee drops onto the task tile; the check confirms ownership. Strong lands with a hit and recoil.',parts,.85,.6,['avatar','check']),duotoneFill:true};
})();

// 2 delegate-task — owner pushes the task tile away; the arrow trails, head lands at the stem tip
const delegate=(()=>{
 const dx=(t,s)=>curve(t,s,[[0,0],[.12,0],[.7,5.2,'out']],[[0,0],[.06,0],[.18,-.5],[.4,5.2,'hit'],[.5,5.2]]);
 const stemEnd=(t,s)=>Math.min(15.6,Math.max(12.6,14+dx(t,s)-3.5));
 const hp=(t,s)=>curve(t,s,[[0,0],[.7,0],[.82,1,'out']],[[0,0],[.4,0],[.5,1]]);
 const tilePose=rigid((t,s)=>({dx:dx(t,s)}));
 const parts=[
  ...avatar([6.9,11,2.2],'M3.3 19 C3.3 15 10.5 15 10.5 19','M3.1 19.1 C3.1 14.4 10.7 14.4 10.7 19.1 Z'),
  ...tube('arrow-stem',(t,s)=>`M12.6 12 L${stemEnd(t,s)} 12`,{visibility:(t,s)=>clamp((stemEnd(t,s)-12.6)/.4)}),
  ...tube('arrow-head',(t,s)=>barb([15.6,12],[13.9,10.3],[13.9,13.7],hp(t,s)),{visibility:(t,s)=>on(hp(t,s))}),
  ...framed('tile',rect(14,8,8,8,2)).map(p=>({...p,pose:tilePose})),
 ];
 return {...geometricRearm(make('delegate-task','Delegate task','The owner pushes the task tile away; the arrow trails behind and its head lands when the stem reaches the tip. Strong anticipates then pushes decisively.',parts,.85,.5)),duotoneFill:true};
})();

// 3 task-handoff — baton slides from giver bracket into receiver bracket
const handoff=(()=>{
 const bat=(t,s)=>curve(t,s,[[0,0],[.1,0],[.75,6.8,'out']],[[0,0],[.06,0],[.3,6.8,'hit'],[.5,6.8]]);
 const give=(t,s)=>curve(t,s,[[0,0],[.1,0],[.35,-.6],[.7,0,'out']],[[0,0],[.06,0],[.2,-.9,'launch'],[.55,0]]);
 const take=(t,s)=>curve(t,s,[[0,0],[.65,0],[.8,.2],[.95,0,'out']],[[0,0],[.3,0],[.38,.5,'hit'],[.6,0]]);
 const parts=[
  ...tube('giver',bracket(8.5,5,9.5,22.5)).map(p=>({...p,pose:rigid((t,s)=>({dx:give(t,s)}))})),
  ...tube('receiver',bracket(23.3,26.8,9.5,22.5)).map(p=>({...p,pose:rigid((t,s)=>({dx:take(t,s)}))})),
  ...framed('baton',rect(8.65,13.5,7.7,5,2.5)).map(p=>({...p,pose:rigid((t,s)=>({dx:bat(t,s)}))})),
 ];
 return {...make('task-handoff','Task handoff','The giver releases and the baton slides into the receiver, who nods on receipt. Strong launches faster and the receiver recoils on contact.',parts,.95,.65,['baton']),duotoneFill:true};
})();

// 4 task-dependency — link draws from done task A down and across to waiting task B
const dependency=(()=>{
 const lnk=(t,s)=>curve(t,s,[[.1,0],[.6,1,'out']],[[.04,0],[.22,1]]);
 const hp=(t,s)=>curve(t,s,[[.6,0],[.75,1,'out']],[[.22,0],[.3,1]]);
 const nudge=(t,s)=>curve(t,s,[[0,0]],[[0,0],[.22,0],[.28,.5,'hit'],[.48,0]]);
 const route=[[8.5,12.2],[8.5,21.5],[8.94,22.56],[10,23],[15.8,23]];
 const parts=[
  ...framed('tile-a',rect(4.5,4.5,8,8,2)),
  ...tube('link',(t,s)=>trace(route,lnk(t,s)),{visibility:(t,s)=>on(lnk(t,s))}),
  ...tube('head',(t,s)=>barb([15.8,23],[14.1,21.3],[14.1,24.7],hp(t,s)),{visibility:(t,s)=>on(hp(t,s))*(lnk(t,s)>=1?1:0)}),
  ...framed('tile-b',rect(19.5,19,8,8,2)).map(p=>({...p,pose:rigid((t,s)=>({dx:nudge(t,s)}))})),
 ];
 return {...geometricRearm(make('task-dependency','Task dependency','The dependency link draws from task A down and across; its head points at waiting task B. Strong draws it as one decisive stroke and B registers the contact.',parts,.9,.55)),duotoneFill:true};
})();

// 5 shared-goal — two arrows converge on one goal ring, centre lights
const goal=(()=>{
 const pl=(t,s)=>curve(t,s,[[.05,0],[.45,1,'out']],[[0,0],[.04,0],[.2,1]]);
 const pr=(t,s)=>curve(t,s,[[.2,0],[.6,1,'out']],[[0,0],[.04,0],[.2,1]]);
 const hl=(t,s)=>curve(t,s,[[.45,0],[.6,1,'out']],[[.2,0],[.28,1]]);
 const hr=(t,s)=>curve(t,s,[[.6,0],[.75,1,'out']],[[.2,0],[.28,1]]);
 const dot=(t,s)=>curve(t,s,[[.7,0],[.9,1,'out']],[[.26,0],[.4,1]]);
 const ringDy=(t,s)=>curve(t,s,[[0,0]],[[0,0],[.2,0],[.25,-.6,'hit'],[.48,0]]);
 const ringPose=rigid((t,s)=>({dy:ringDy(t,s)}));
 const parts=[
  ...framed('ring',circle(16,11,4.5),annulus(16,11,4.5,2.6)).map(p=>({...p,pose:ringPose})),
  ...bothFill('dot',circle(16,11,1.4),{visibility:(t,s)=>dot(t,s)}).map(p=>({...p,pose:ringPose})),
  ...tube('arrow-l',(t,s)=>trace([[5,22],[10.3,16.7]],pl(t,s)),{visibility:(t,s)=>on(pl(t,s))}),
  ...tube('head-l',(t,s)=>barb([10.3,16.7],[7.7,16.7],[10.3,19.3],hl(t,s)),{visibility:(t,s)=>on(hl(t,s))*(pl(t,s)>=1?1:0)}),
  ...tube('arrow-r',(t,s)=>trace([[27,22],[21.7,16.7]],pr(t,s)),{visibility:(t,s)=>on(pr(t,s))}),
  ...tube('head-r',(t,s)=>barb([21.7,16.7],[24.3,16.7],[21.7,19.3],hr(t,s)),{visibility:(t,s)=>on(hr(t,s))*(pr(t,s)>=1?1:0)}),
 ];
 return {...geometricRearm(make('shared-goal','Shared goal','Two teammates aim at the same goal ring; both arrows arrive and the centre lights. Strong arrives together with a ring recoil.',parts,.95,.6)),duotoneFill:true};
})();

export const icons=[assign,delegate,handoff,dependency,goal];
