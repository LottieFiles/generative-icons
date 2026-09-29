// motion/icon-sets/22-teamwork/batches/discussion.mjs — Claude first draft
import {make,stroke,fill,tube,bothFill,framed,posed,circle,rect,polygon,rigid,curve,track,flow,timePair,holes,geometricRearm} from './helpers.mjs';

const lerp=(a,b,f)=>[a[0]+(b[0]-a[0])*f,a[1]+(b[1]-a[1])*f];
// Fixed-topology progressive draw. segs: ['L',len,x,y] | ['C',len,x1,y1,x2,y2,x,y]. u>1 extends a final 'L'.
function trace(start,segs,u){
 const total=segs.reduce((s,g)=>s+g[1],0);let cur=start,rem=u*total,d=`M${cur[0]} ${cur[1]}`,tip=start;
 segs.forEach((g,i)=>{const [k,len,...p]=g,last=i===segs.length-1,end=[p.at(-2),p.at(-1)];
  if(rem>=len){const e=(last&&k==='L'&&rem>len)?lerp(cur,end,rem/len):end;
   d+=k==='L'?` L${e[0]} ${e[1]}`:` C${p[0]} ${p[1]} ${p[2]} ${p[3]} ${e[0]} ${e[1]}`;rem-=len;cur=e;tip=e;}
  else if(rem>0){const f=rem/len;let e;
   if(k==='L'){e=lerp(cur,end,f);d+=` L${e[0]} ${e[1]}`;}
   else{const p1=[p[0],p[1]],p2=[p[2],p[3]],a=lerp(cur,p1,f),b=lerp(p1,p2,f),c=lerp(p2,end,f),ab=lerp(a,b,f),bc=lerp(b,c,f);e=lerp(ab,bc,f);d+=` C${a[0]} ${a[1]} ${ab[0]} ${ab[1]} ${e[0]} ${e[1]}`;}
   rem=-1;cur=e;tip=e;}
  else d+=k==='L'?` L${cur[0]} ${cur[1]}`:` C${cur[0]} ${cur[1]} ${cur[0]} ${cur[1]} ${cur[0]} ${cur[1]}`;});
 return {d,tip};}
const headLeft=(tip,L)=>`M${tip[0]+L} ${tip[1]-L} L${tip[0]} ${tip[1]} L${tip[0]+L} ${tip[1]+L}`;
// Single closed plus contour (transparent cutout), arms measured from center, w = painted width.
const plusPath=(cx,cy,v,hz,w)=>{const h=w/2,a=Math.max(h,hz);return polygon([[cx-h,cy-v],[cx+h,cy-v],[cx+h,cy-h],[cx+a,cy-h],[cx+a,cy+h],[cx+h,cy+h],[cx+h,cy+v],[cx-h,cy+v],[cx-h,cy+h],[cx-a,cy+h],[cx-a,cy-h],[cx-h,cy-h]],1.3);};

// 1. ADD COMMENT — bubble + insertion stem; bar draws outward to complete the plus.
const BUB='M8 5 L24 5 C25.66 5 27 6.34 27 8 L27 18 C27 19.66 25.66 21 24 21 L13.5 21 L9.8 24.6 C9.4 25 9 24.8 9 24.3 L9 21 L8 21 C6.34 21 5 19.66 5 18 L5 8 C5 6.34 6.34 5 8 5 Z';
function addComment(){
 const u=(t,s,r)=>r?track([[0,1],[.26,0,'out']],t):curve(t,s,[[0,0],[.06,0],[.5,1,'out']],[[0,0],[.08,0,'hold'],[.34,1]]);
 const press=rigid((t,s,r)=>({dy:(!r&&s)?flow([[0,0],[.16,2.2,'hit'],[.44,0]],t):0}));
 const vis=(t,s,r)=>u(t,s,r)>.03?1:0;
 const parts=[fill('body-tint',BUB,'tint'),stroke('body',BUB),stroke('stem','M16 9 L16 17'),
  stroke('bar',(t,s,r)=>{const a=4*u(t,s,r);return `M${16-a} 13 L${16+a} 13`;},{visibility:vis}), // opaque join: bar crosses stem
  fill('body-solid',(t,s,r)=>holes(BUB,plusPath(16,13,4,4*u(t,s,r),2.6)),'solid')];
 for(const p of parts)p.pose=press;
 return {...make('add-comment','Add comment','An insertion stem inside a comment bubble; the crossbar draws outward to complete a plus. Strong presses the bubble down while the bar fires.',parts,.55,.5),
  stateful:true,initialState:'prepared',finalState:'result',previewReset:true,reverseSuffix:'-reset',reverseTimings:timePair(.3,.3),duotoneFill:true};
}

// 2. REPLY THREAD — parent card; a reply arrow draws down and hooks left beneath it.
function replyThread(){
 const segs=[['L',2,22,17],['C',7.4,22,20,20,22,17,22],['L',6,11,22]],start=[22,15];
 const u=(t,s)=>curve(t,s,[[0,0],[.05,0],[.6,1,'out']],[[0,0],[.06,0,'hold'],[.3,1.1],[.5,1]]);
 const L=(t,s)=>curve(t,s,[[.52,0],[.62,3,'out']],[[.3,0],[.4,3.5],[.55,3]]);
 const lift=rigid((t,s)=>({dy:s?flow([[0,0],[.1,.35],[.3,0]],t):0}));
 const parts=[...posed(framed('card',rect(8,5,16,6,2.2)),lift),
  ...tube('arrow',(t,s)=>trace(start,segs,u(t,s)).d,{visibility:(t,s)=>u(t,s)>.02?1:0}),
  ...tube('head',(t,s)=>headLeft(trace(start,segs,u(t,s)).tip,L(t,s)),{visibility:(t,s)=>(u(t,s)>.985&&L(t,s)>.2)?1:0})];
 return {...make('reply-thread','Reply thread','A reply arrow draws from beneath the parent message, curves down and hooks left; the head opens on arrival. Strong dips the card, fires the draw past the tip and settles.',parts,.65,.55,['arrow','head']),duotoneFill:true};
}

// 3. MENTION TEAMMATE — @ glyph; the teammate lifts and turns when called.
function mentionTeammate(){
 const AT='M15.5 15 L15.5 19.5 C15.5 22 17.5 23 18.8 21.5 C20.5 19.5 20 14.5 18.5 12.5 C16 9.3 8 10 6 14.5 C4.2 19 7 25 12.5 25.8 C14 26 15.5 25.8 16.8 25.2';
 const person=rigid((t,s)=>({dy:-curve(t,s,[[0,0],[.22,1.2],[.55,0,'out']],[[0,0],[.2,3],[.65,0]]),
  angle:s?flow([[0,0],[.2,-12],[.65,0]],t):0,pivot:[26,17.5]}));
 const parts=[...tube('ring',circle(12.5,18,3)),...tube('tail',AT), // opaque join: tail stem tangent to ring
  ...posed([...bothFill('head',circle(26,9.8,2)),...tube('shoulders','M23.55 17.5 C23.55 14.9 28.7 14.9 28.7 17.5')],person)];
 return {...make('mention-teammate','Mention teammate','An @ glyph calls a teammate; the person lifts in response and settles back. Strong lifts higher and turns toward the mention.',parts,.6,.7),duotoneFill:false};
}

// Shared thread housing for the resolve/reopen state pair.
const thread=(row1End)=>[...tube('spine','M7 6 L7 24'),...tube('row-1',`M11 9 L${row1End} 9`),...tube('row-2','M11 16 L17 16')];

// 4. RESOLVE THREAD — a check draws across the thread and stays.
function resolveThread(){
 const segs=[['L',4.95,20,24],['L',13.04,27,13]],start=[16.5,20.5];
 const u=(t,s)=>curve(t,s,[[0,0],[.06,0],[.5,1,'out']],[[0,0],[.12,0,'hold'],[.3,1.08],[.48,1]]);
 const rows=rigid((t,s)=>({dx:s?flow([[0,0],[.14,1.5],[.4,0]],t):0}));
 const housing=thread(20);const parts=[...housing.slice(0,2),...posed(housing.slice(2),rows),
  ...tube('check',(t,s)=>trace(start,segs,u(t,s)).d,{visibility:(t,s)=>u(t,s)>.02?1:0})];
 return {...make('resolve-thread','Resolve thread','A check draws across the discussion thread and is retained. Strong gathers the rows, fires a decisive stroke past the tip and settles.',parts,.6,.55,['check']),duotoneFill:false};
}

// 5. REOPEN THREAD — a return arrow draws from the bottom back up into the first comment.
function reopenThread(){
 const segs=[['C',13,25,25,28,21.5,28,16],['L',1.5,28,14.5],['C',6.2,28,11.5,26,10,24,10],['L',3,21,10]],start=[18,25];
 const u=(t,s)=>curve(t,s,[[0,0],[.05,0],[.7,1,'out']],[[0,0],[.06,0,'hold'],[.4,1,'out']]);
 const L=(t,s)=>curve(t,s,[[.62,0],[.72,2.6,'out']],[[.38,0],[.46,3.1],[.6,2.6]]);
 const parts=[...thread(17.5),
  ...tube('arrow',(t,s)=>trace(start,segs,u(t,s)).d,{visibility:(t,s)=>u(t,s)>.02?1:0}),
  ...tube('head',(t,s)=>headLeft(trace(start,segs,u(t,s)).tip,L(t,s)),{visibility:(t,s)=>(u(t,s)>.985&&L(t,s)>.2)?1:0})];
 return {...make('reopen-thread','Reopen thread','A return arrow draws from the bottom of the thread up and back into the first comment. Strong winds up a stub, fires the full draw and snaps the head open.',parts,.8,.65,['arrow','head']),duotoneFill:false};
}

export const icons=[addComment(),geometricRearm(replyThread()),mentionTeammate(),resolveThread(),geometricRearm(reopenThread())];
