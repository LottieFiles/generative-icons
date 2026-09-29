// Everyday controls · batch 1, revision 2 (Claude draft, not integrated)
import {make,toggle,geometricRearm,F,rigid,posed,shown,stroke,fill,tube,framed,line,bothFill,rect,circle,arc,capsule,holes,segment,clamp,mix} from './helpers.mjs';
// Solid contour = full painted exterior of a 2.6 tube around an outline rect.
const outer=(x,y,w,h,r)=>rect(x-1.3,y-1.3,w+2.6,h+2.6,r+1.3);

/* ---------- CUT ----------
 Fixed: rivet PV [16,15] (the crossing is the rivet). Moving: blade-a rotates +angle about PV,
 blade-b mirror rotates -angle. Ring centers sit on the shank line through PV so each blade is one
 path: full ring from its pivot-facing point, then the shank through PV to the tip.
 Rest ring gap at Solid: (21.5-2.7-1.3)-(10.5+2.7+1.3)=3.0; the -1.5° contact moves each ring ~0.3
 inward, leaving ~2.4 >= 1.15. Rings are apertures: no tint, duotoneFill false. */
const RA=[10.5,25],RB=[21.5,25],PV=[16,15],RR=2.7;
const TIP=[PV[0]+(PV[0]-RA[0])*.95,PV[1]+(PV[1]-RA[1])*.95]; // [21.225,5.5]
const ca=Math.atan2(PV[1]-RA[1],PV[0]-RA[0])*180/Math.PI;
const bladeA=arc(...RA,RR,ca,ca+360)+' L'+TIP;
const bladeB=arc(...RB,RR,180-ca,180-ca-360)+' L'+[32-TIP[0],TIP[1]];
// Light: quiet 8° open and close. Strong: 20° open, held a beat, .09 s snap to the -1.5° contact (hit), one bounded rebound to 0.
const cutAngle=F([[0,0],[.28,8,'smooth'],[.6,0,'smooth']],[[0,0],[.18,20],[.24,20,'hold'],[.33,-1.5,'hit'],[.5,0]]);
const cut={...make('cut','Cut','Scissors open, then close decisively across the content; Strong opens wide, then snaps shut with one rebound.',[
 ...posed(tube('blade-a',bladeA),rigid((t,s)=>({angle:cutAngle(t,s),pivot:PV}))),
 ...posed(tube('blade-b',bladeB),rigid((t,s)=>({angle:-cutAngle(t,s),pivot:PV}))),
],.6,.5),duotoneFill:false};

/* ---------- PASTE ----------
 Fixed: board (open path ending on the clip sides), clip (closed). Moving: sheet + sheet-line, rigid dy.
 Sheet appears at top y=12 (Solid painted top 10.7 vs clip Solid bottom 9.3: gap 1.4) and lands at 16
 (Solid bottom 25.3+.7 overshoot vs board inner painted 27.2: gap 1.2). Solid contours are full exteriors. */
const board='M12.5 6.5 L8.5 6.5 C7.12 6.5 6 7.62 6 9 L6 26 C6 27.38 7.12 28.5 8.5 28.5 L23.5 28.5 C24.88 28.5 26 27.38 26 26 L26 9 C26 7.62 24.88 6.5 23.5 6.5 L19.5 6.5';
const sheetD=rect(10,16,12,8,1.6),sl=[[13.5,20],[18.5,20]];
// Light: fades in over the first 1.5 units while gliding 4 down. Strong: appears in .08 s, hangs (preparation), plunges to 16.7 (hit), settles to 16.
const sheetY=F([[0,12],[.55,16,'smooth']],[[0,12],[.12,12,'hold'],[.25,16.7,'hit'],[.42,16]]);
const sheetIn=(t,s)=>s?clamp(t/.08):clamp((sheetY(t,s)-12)/1.5);
const paste={...geometricRearm(make('paste','Paste','Clipboard content drops onto the board and stays; Strong hangs, then slaps down with one settle.',[
 ...tube('board',board),
 ...framed('clip',rect(12.5,4,7,4,1.5),outer(12.5,4,7,4,1.5)),
 ...posed(shown([...framed('sheet',sheetD,holes(outer(10,16,12,8,1.6),capsule(...sl,2.2))),stroke('sheet-line',segment(...sl))],sheetIn),rigid((t,s)=>({dy:sheetY(t,s)-16}))),
],.55,.55)),duotoneFill:true};

/* ---------- SELECT ALL ----------
 Fixed: four 4×4 tiles (real enclosed surfaces, tinted, full Solid exteriors). The marquee is a FIXED
 boundary at 4.5..27.5 (Solid painted inner 5.8 vs tile Solid exterior 7.2: gap 1.4, never crossed).
 Morph: a directed trace along that perimeter from left-mid, clockwise. 'one' = an L bracket over
 tile 0 (p=.25); 'all' = the complete boundary (p=1). Constant 9-segment topology: untraced segments
 collapse onto the pen point. Reverse erases the trace back to the bracket. */
const MQ=[4.5,4.5,23,23,2];
function traceRect([x,y,w,h,r],p){
 const k=.552285*r,S=[['L',[x,y+h/2],[x,y+r]],['C',[x,y+r],[x,y+r-k],[x+r-k,y],[x+r,y]],['L',[x+r,y],[x+w-r,y]],['C',[x+w-r,y],[x+w-r+k,y],[x+w,y+r-k],[x+w,y+r]],['L',[x+w,y+r],[x+w,y+h-r]],['C',[x+w,y+h-r],[x+w,y+h-r+k],[x+w-r+k,y+h],[x+w-r,y+h]],['L',[x+w-r,y+h],[x+r,y+h]],['C',[x+r,y+h],[x+r-k,y+h],[x,y+h-r+k],[x,y+h-r]],['L',[x,y+h-r],[x,y+h/2]]];
 const len=s=>s[0]==='L'?Math.hypot(s[2][0]-s[1][0],s[2][1]-s[1][1]):Math.PI*r/2;
 let rem=p*S.reduce((a,s)=>a+len(s),0),pen=S[0][1],d='M'+pen;
 for(const s of S){const L=len(s),u=clamp(rem/L);rem-=L;
  if(u<=0){d+=s[0]==='L'?' L'+pen:' C'+[...pen,...pen,...pen];continue;}
  const m=(P,Q)=>[mix(P[0],Q[0],u),mix(P[1],Q[1],u)];
  if(s[0]==='L'){pen=m(s[1],s[2]);d+=' L'+pen;}
  else{const [a,b,c,e]=s.slice(1),ab=m(a,b),bc=m(b,c),ce=m(c,e),abc=m(ab,bc),bce=m(bc,ce);pen=m(abc,bce);d+=' C'+[...ab,...abc,...pen];}
 }return d;}
const tiles=[[8.5,8.5],[19.5,8.5],[8.5,19.5],[19.5,19.5]].flatMap(([x,y],i)=>framed('tile-'+i,rect(x,y,4,4,1.2),outer(x,y,4,4,1.2)));
// Light: smooth draw .25→1. Strong: short retreat to .2 (preparation), then one decisive sweep to the closed boundary; progress stays monotone after the retreat.
const selOn=F([[0,.25],[.55,1,'smooth']],[[0,.25],[.08,.2],[.36,1,'out']]);
const selOff=F([[0,1],[.55,.25,'smooth']],[[0,1],[.08,1,'hold'],[.34,.25,'out']]);
const marquee=(t,s,r)=>traceRect(MQ,r?selOff(t,s):selOn(t,s));
const selectAll={...toggle('select-all','Select all','The selection boundary is drawn from a corner bracket on one object around all of them; Strong pulls back, then sweeps the whole boundary in one stroke.',[
 ...tiles,...tube('marquee',marquee),
],.55,.36,'one','all'),duotoneFill:true};

/* ---------- DRAG HANDLE ----------
 Fixed: row-c (y 24). Moving: item = 2×2 grip dots (x 4.2/7.2, y 6.4/9.6, r 1) + bar 10..14.5 at y 8,
 rigid dx/dy; row-b bar 10..14.5 at y 16, rigid dy. Route: the item is pulled OUT to the right
 (dx 13.8 → painted 17.0..29.6; row-b painted right edge 15.8: gap 1.2 at Solid), slides down one
 slot while row-b moves up through the vacated column, then reinserts. No crossing. Reverse mirrors dy. */
const grip=[4.2,7.2].flatMap((x,i)=>[6.4,9.6].flatMap((y,j)=>bothFill('dot-'+i+j,circle(x,y,1))));
// Light: pull out, slide, reinsert, all smooth. Strong: .5 grab-lift (preparation), fast pull-out, drop to +8.5 (hit), settle, brisk reinsert.
const dxF=F([[0,0],[.2,1,'smooth'],[.4,1],[.6,0,'smooth']],[[0,0],[.06,0,'hold'],[.16,1,'out'],[.32,1,'hold'],[.44,0]]);
const dyI=F([[0,0],[.2,0],[.4,1,'smooth']],[[0,0],[.06,-.06],[.16,-.06,'hold'],[.3,1.06,'hit'],[.4,1]]);
const dyR=F([[0,0],[.2,0],[.4,1,'smooth']],[[0,0],[.16,0,'hold'],[.3,1,'out']]);
const flip=(p,r)=>r?1-p:p;
const dragHandle={...toggle('drag-handle','Drag handle','The gripped row is pulled out of the list, moved one slot, and reinserted while the row below slides into its place; Strong lifts, yanks out, and drops with one settle.',[
 ...posed([...grip,...line('bar',[10,8],[14.5,8])],rigid((t,s,r)=>({dx:13.8*dxF(t,s),dy:8*flip(dyI(t,s),r)}))),
 ...posed(line('row-b',[10,16],[14.5,16]),rigid((t,s,r)=>({dy:-8*flip(dyR(t,s),r)}))),
 ...line('row-c',[10,24],[14.5,24]),
],.6,.46,'first','second'),duotoneFill:false};

/* ---------- ARCHIVE ----------
 Fixed: body (closed rect, top rim y=12 is the complete exposed rim), slot line (Solid hole).
 Moving: lid, rigid dy only. Outline: lid bottom edge coincides with the rim at rest (one stroke);
 lifted, lid bottom painted 9.25 vs rim 10.75 at Bold: gap 1.5. Solid: lid exterior bottom 13.3 overlaps
 the body exterior top 10.7 by 2.6 (one opaque closed box), hole top 15.3 stays 2.0 clear. Lid top
 painted 6.2-4=2.2 at full lift. Lid and body are real panels: duotoneFill true. */
const lidD=rect(5,7.5,22,4.5,1.5),bodyD=rect(7,12,18,15,2),slot=[[13,16.5],[19,16.5]];
// Light: lid lifts 1.6 and reseats. Strong: lifts 4, holds open .08 s, slams to +.3 (hit), settles 0.
const lidY=F([[0,0],[.25,-1.6,'smooth'],[.55,0,'smooth']],[[0,0],[.22,-4],[.3,-4,'hold'],[.4,.3,'hit'],[.55,0]]);
const archive={...make('archive','Archive','The archive-box lid lifts to receive the item and closes over it; Strong opens wide and shuts with a slam.',[
 ...posed(framed('lid',lidD,outer(5,7.5,22,4.5,1.5)),rigid((t,s)=>({dy:lidY(t,s)}))),
 ...framed('body',bodyD,holes(outer(7,12,18,15,2),capsule(...slot,2.4))),
 stroke('slot',segment(...slot)),
],.55,.55),duotoneFill:true};

export const icons=[cut,paste,selectAll,dragHandle,archive];
