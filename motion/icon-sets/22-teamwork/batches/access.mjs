// Teamwork batch: share-workspace, invite-link, guest-access, view-access, edit-access.
import {make,stroke,tube,bothFill,framed,rect,circle,polygon,hole,rigid,track,timePair,curve} from './helpers.mjs';
const T=(t,s,L,S)=>curve(t,s,L,S);
const ring=(x,y,w,h)=>hole(rect(x-1.3,y-1.3,w+2.6,h+2.6,4.8),rect(x+1.3,y+1.3,w-2.6,h-2.6,2.2));

// ---- share-workspace: panel + outgoing arrow (gesture) ----
const shareTip=(t,s)=>T(t,s,[[0,25.5],[.36,27,'out'],[.56,27],[1,25.5,'io']],[[0,25.5],[.12,25.05],[.4,28.5,'hit'],[.6,28.1],[1.05,25.5]]);
const panelD=rect(4,7,15,18,2.5);
const shareWorkspace=()=>{
 const parts=[
  ...framed('panel',panelD,hole(panelD,rect(11.3,9.3,5.4,13.4,1.2))),
  stroke('divider','M9.5 7 L9.5 25'),                       // opaque join with panel frame (top/bottom)
  ...tube('shaft',(t,s)=>`M22.5 16 L${shareTip(t,s)} 16`),   // root fixed, tip extends
  ...tube('head','M23.1 13.6 L25.5 16 L23.1 18.4').map(p=>({...p,pose:rigid((t,s)=>({dx:shareTip(t,s)-25.5}))})) // opaque join with shaft at tip
 ];
 return {...make('share-workspace','Share workspace','The workspace panel pushes its share arrow outward; Strong prepares, drives 3 units to a hit and settles back.',parts,1,1.05),duotoneFill:true};
};

// ---- invite-link: chain offered outward, plus fixed (gesture) ----
const chainDx=(t,s)=>T(t,s,[[0,0],[.32,1.5,'out'],[.5,1.5],[.95,0,'io']],[[0,0],[.1,-.7],[.38,4,'hit'],[.56,3.4],[1.05,0]]);
const inviteLink=()=>{
 const pose=rigid((t,s)=>({dx:chainDx(t,s)}));
 const loopA=rect(4.5,15,10,7,3.5),loopB=rect(12.5,15,10,7,3.5);
 const parts=[
  {...stroke('chain-loop-a',loopA),pose},{...stroke('chain-loop-b',loopB),pose},   // interlock overlap 12.5..14.5 is an opaque join
  {name:'chain-loop-a-solid',d:ring(4.5,15,10,7),style:'solid',pose},{name:'chain-loop-b-solid',d:ring(12.5,15,10,7),style:'solid',pose},
  ...tube('plus-h','M23 8.5 L27 8.5'),...tube('plus-v','M25 6.5 L25 10.5')      // cross overlap named
 ];
 return {...make('invite-link','Invite link','The interlocked chain is offered to the right beneath the fixed plus; Strong pulls back slightly, drives 4 units to a hit and settles.',parts,.95,1.05),duotoneFill:false};
};

// ---- guest-access: boom gate lifts for a waiting visitor, then closes (gesture) ----
const gateAngle=(t,s)=>T(t,s,[[0,0],[.38,-18,'out'],[.6,-18],[1.05,0,'io']],[[0,0],[.1,2.5],[.42,-55,'hit'],[.58,-49],[1.1,0]]);
const guestAccess=()=>{
 const postD=rect(5.5,15.5,4,11,1.8);
 const parts=[
  ...framed('post',postD),
  ...tube('bar','M7.5 17.5 L23.5 17.5').map(p=>({...p,pose:rigid((t,s)=>({angle:gateAngle(t,s),pivot:[7.5,17.5]}))})), // bar root buried in post: opaque join
  ...bothFill('visitor',circle(22.5,24,1.8))
 ];
 return {...make('guest-access','Guest access','The gate bar lifts from its post to admit the waiting visitor and closes again; Strong dips, swings 55 degrees to a hit and settles before closing.',parts,1.05,1.1),duotoneFill:true};
};

// ---- view-access: eye reads across two content lines (gesture) ----
const pupilX=(t,s)=>T(t,s,[[0,0],[.3,-1,'out'],[.75,1],[1.1,0,'io']],[[0,0],[.14,-2],[.5,2,'hit'],[.64,1.7],[1.1,0]]);
const viewAccess=()=>{
 const almond='M6 11 C10 3 22 3 26 11 C22 19 10 19 6 11 Z';
 const parts=[
  ...framed('lids',almond,(t,s)=>hole(almond,circle(16+pupilX(t,s),11,3.2))),
  ...bothFill('pupil',circle(16,11,2)).map(p=>({...p,pose:rigid((t,s)=>({dx:pupilX(t,s)}))})),
  ...tube('line-a','M9 21.5 L23 21.5'),...tube('line-b','M9 25.5 L20 25.5')
 ];
 return {...make('view-access','View access','The pupil glances across the shared content and returns; Strong looks further left, sweeps right to a hit and settles.',parts,1.1,1.1),duotoneFill:true};
};

// ---- edit-access: pencil draws a retained mark; geometric reverse (stateful) ----
const penD=(()=>{const c=Math.SQRT1_2,m=([u,v])=>[6.5+(17.5-u)*c+v*c,22-(17.5-u)*c+v*c];return polygon([[0,-2.5],[13,-2.5],[17.5,0],[13,2.5],[0,2.5]].map(m),[1.4,1.2,1.2,1.2,1.4]);})();
const penX=(t,s,r)=>r?track([[0,14],[.3,6.5,'io']],t):T(t,s,[[0,6.5],[.95,14,'io']],[[0,6.5],[.3,6.5,'hold'],[.6,14,'hit'],[.68,14]]);
const penY=(t,s,r)=>r?0:T(t,s,[[0,0]],[[0,0],[.16,-2.4],[.3,0],[.6,0,'hold'],[.66,-.5],[.78,0]]);
const editAccess=()=>{
 const pose=rigid((t,s,r)=>({dx:penX(t,s,r)-6.5,dy:penY(t,s,r)}));
 const markD=(t,s,r)=>`M6.5 22 L${Math.max(6.5,penX(t,s,r))} 22`,markVis=(t,s,r)=>penX(t,s,r)-6.5>.6?1:0;
 const parts=[
  ...tube('rule','M6.5 26 L22 26'),
  ...tube('mark',markD).map(p=>({...p,visibility:markVis})),        // mark end sits under the pencil tip: opaque join
  ...framed('pencil',penD).map(p=>({...p,pose}))
 ];
 return {...make('edit-access','Edit access','The pencil draws a real mark along the field and keeps it; Strong lifts, lands and draws the full stroke fast with a short recoil. Reset withdraws the mark and slides the pencil back.',parts,.95,.8),
  stateful:true,initialState:'prepared',finalState:'result',previewReset:true,reverseSuffix:'-reset',forwardLabel:'Grant edit',reverseLabel:'Reset preview',reverseTimings:timePair(.3,.3),duotoneFill:true};
};

export const icons=[shareWorkspace(),inviteLink(),guestAccess(),viewAccess(),editAccess()];
