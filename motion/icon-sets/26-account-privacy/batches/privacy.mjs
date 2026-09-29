// account-privacy batch A, revision 2. 32-unit grid, all paint inside [2,30].
import {make,toggle,geometricRearm,F,posed,rigid,tube,stroke,fill,framed,circle,rect,arc,holes,capsule} from './helpers.mjs';

/* 1 face-authentication (gesture, open symbol, duotoneFill:false)
   Strong peak lowered 2.2 -> 2.0 so the tl/tr inner arms keep a 1.4u Solid gap
   (centerline 14.0 vs 18.0 at peak) and the bl/br arms keep 1.4u to the mouth. */
const focus=F([[0,0],[.25,.9,'smooth'],[.5,0,'smooth']],
              [[0,0],[.12,-.6],[.3,2.0,'hit'],[.42,1.5],[.7,0]]);
const corner=(sx,sy)=>rigid((t,s)=>{const k=focus(t,s);return{dx:sx*k,dy:sy*k};});
const faceAuth={...make('face-authentication','Face authentication',
 'The framing marks tighten on the face, then release: one inward focus in Light, a prepared snap with a single release in Strong.',[
 ...posed(tube('bracket-tl','M6 12 L6 8.2 C6 6.8 6.8 6 8.2 6 L12 6'),corner(1,1)),
 ...posed(tube('bracket-tr','M20 6 L23.8 6 C25.2 6 26 6.8 26 8.2 L26 12'),corner(-1,1)),
 ...posed(tube('bracket-br','M26 20 L26 23.8 C26 25.2 25.2 26 23.8 26 L20 26'),corner(-1,-1)),
 ...posed(tube('bracket-bl','M12 26 L8.2 26 C6.8 26 6 25.2 6 23.8 L6 20'),corner(1,-1)),
 ...tube('eye-l','M12.5 12.6 L12.5 14.2'),
 ...tube('eye-r','M19.5 12.6 L19.5 14.2'),
 ...tube('mouth',arc(16,15.5,4.5,25,155)),
],.5,.7),duotoneFill:false};

/* 2 two-factor-code (result + preview reset, duotoneFill:true)
   Two cells instead of three. Centers 10 and 22. Outline centerline 8 wide,
   Solid ring exterior 10.6 wide -> Solid paint gap 1.4u, Bold outline gap 1.5u.
   Ring counter 5.4 x 9.4; dot pops to r 1.62 -> 1.08u counter space at peak.
   Cell press dy 1.2 moves the frame only; the dot stays centred. */
const cellD=x=>rect(x-4,10,8,12,2.2);
const cellS=x=>holes(rect(x-5.3,8.7,10.6,14.6,3.5),rect(x-2.7,11.3,5.4,9.4,.9));
const twoFactor={...geometricRearm(make('two-factor-code','Two-factor code',
 'Two cells receive a code: dots bloom in sequence in Light; in Strong each cell takes a key press and its dot pops in with one bounded settle.',
 [10,22].flatMap((x,i)=>{
  const dl=i*.2,ds=i*.16;
  const g=F([[dl,0],[dl+.28,1,'smooth']],[[ds,0],[ds+.1,1.35],[ds+.3,1]]);
  const dip=F([[0,0]],[[ds,0],[ds+.09,1.2],[ds+.26,0]]);
  const dot=(t,s)=>circle(x,16,Math.max(.01,1.2*g(t,s))),vis=(t,s)=>g(t,s)>.04?1:0;
  return [
   ...posed(framed('cell-'+i,cellD(x),cellS(x)),rigid((t,s)=>({dy:dip(t,s)}))),
   {name:'dot-'+i,style:'outline',d:dot,visibility:vis},
   {name:'dot-'+i+'-solid',style:'solid',d:dot,visibility:vis},
  ];}),.55,.62)),duotoneFill:true};

/* 3 recovery-codes (toggle stored <-> open, duotoneFill:true)
   Code lines respaced 11.2 / 15 / 18.8: Bold paint gap 1.3u between lines and
   1.2u below the sheet top stroke. Solid counters follow the same centres. */
const lift=F([[0,0],[.5,3,'smooth']],[[0,0],[.14,-.6],[.36,3.8,'hit'],[.7,3]]);
const sheetPose=rigid((t,s,r)=>({dy:-(r?3-lift(t,s):lift(t,s))}));
const recovery={...toggle('recovery-codes','Recovery codes',
 'A sheet of backup codes slides up out of its sleeve and stays out; reverse tucks it back.',[
 ...tube('sleeve','M5 17 L5 26 C5 27.1 5.9 28 7 28 L25 28 C26.1 28 27 27.1 27 26 L27 17'),
 ...posed([
  ...framed('sheet',rect(9,7.5,14,16,1.6),
   holes(rect(7.7,6.2,16.6,18.6,2.9),rect(12,10.2,8,2,1),rect(12,14,8,2,1),rect(12,17.8,8,2,1))),
  stroke('code-1','M12.5 11.2 L19.5 11.2'),
  stroke('code-2','M12.5 15 L19.5 15'),
  stroke('code-3','M12.5 18.8 L19.5 18.8'),
 ],sheetPose),
],.5,.7,'stored','open'),duotoneFill:true};

/* 4 private-browsing (gesture, duotoneFill:true)
   Lens r 4.2, centres 9.5 / 22.5 (13 apart). Solid ring exterior r 5.5, so
   lens paint spans x 4.0-15.0 and 17.0-28.0: real 2.0u gap. Bridge and
   temple endpoints sit 4.5-4.8u from their lens centre, inside the ring paint
   (Solid 2.9-5.5, Light outline 3.45-4.95), so every cap is buried. */
const bob=F([[0,0],[.28,1.2,'smooth'],[.55,0,'smooth']],[[0,0],[.12,-.6],[.3,3,'hit'],[.42,2.2],[.7,0]]);
const ring=(x,y)=>holes(circle(x,y,5.5),circle(x,y,2.9));
const privateBrowsing={...make('private-browsing','Private browsing',
 'The glasses are pushed up the nose and settle back: a quiet nudge in Light, a decisive push with one drop in Strong.',
 posed([
  ...framed('lens-l',circle(9.5,17,4.2),ring(9.5,17)),
  ...framed('lens-r',circle(22.5,17,4.2),ring(22.5,17)),
  ...tube('bridge','M14.2 16 C14.6 14.4 17.4 14.4 17.8 16'),
  ...tube('temple-l','M5 16.4 L3.6 14.8'),
  ...tube('temple-r','M27 16.4 L28.4 14.8'),
 ],rigid((t,s)=>({dy:-bob(t,s)}))),.55,.7),duotoneFill:true};

/* 5 cookie-preferences (gesture, duotoneFill:true) unchanged geometry.
   Chips sit <= 5.4u from centre; Solid rim inner edge is 8.2, bite clearance 1.7.
   Rotation about (16,16) keeps every counter inside the fill. */
const cookieD=arc(16,16,9.5,-20,290)+' '+arc(23.5,8.5,4.48,198.6,71.4).replace(/^M/,'L')+' Z';
const chips=[[13,11.5],[18,15.5],[12,18.5],[18,21]];
const turn=F([[0,0],[.28,-14,'smooth'],[.55,0,'smooth']],[[0,0],[.14,12],[.42,-75,'hit'],[.56,-58],[.85,0]]);
const cookie={...make('cookie-preferences','Cookie preferences',
 'The cookie turns like a dial and settles: a small turn in Light, a wound-up swing with one return in Strong.',
 posed([
  fill('interior',cookieD,'tint'),
  fill('body-solid',holes(cookieD,...chips.map(([x,y])=>circle(x,y,1.4))),'solid'),
  ...tube('rim',cookieD),
  ...chips.map(([x,y],i)=>fill('chip-'+i,circle(x,y,1.1),'outline')),
 ],rigid((t,s)=>({angle:turn(t,s),pivot:[16,16]}))),.55,.85),duotoneFill:true};

/* 6 block-user (toggle allowed <-> blocked, duotoneFill:true for the head only)
   Badge ring is now a plain tube (open symbol, no tint, no annulus counter).
   Head stays framed. Flinch prep reduced to +0.3 and jolt to +0.5 so the
   shoulder cap/ring gap never drops below 1.35u and ring paint stays <= 29.8. */
const A=[21.1,20.6],B=[25.9,25.4];
const draw=F([[0,0],[.08,0],[.45,1,'smooth']],[[0,0],[.16,0],[.27,1]]);
// The reverse has no person flinch to carry the action. Give its actual slash
// retraction a readable drive at 24 px instead of borrowing the 110 ms strike.
const retract=F([[0,1],[.08,1],[.45,0,'smooth']],[[0,1],[.06,1],[.42,0]]);
const k=(t,s,r)=>r?retract(t,s):draw(t,s);
const tip=(t,s,r)=>[A[0]+(B[0]-A[0])*k(t,s,r),A[1]+(B[1]-A[1])*k(t,s,r)];
const flinch=F([[0,0]],[[0,0],[.14,.3],[.27,-1.4,'hit'],[.6,0]]);
const jolt=F([[0,0]],[[0,0],[.27,0],[.35,.5],[.6,0]]);
const blockUser={...toggle('block-user','Block user',
 'A prohibition slash strikes across the badge and stays; the person flinches in Strong. Unblock retracts the slash.',[
 ...posed([
  ...framed('head',circle(13,9.5,4.5),circle(13,9.5,5.8)),
  ...tube('shoulders','M5 26 C5 21 8.5 18 13 18 C14.1 18 15.1 18.2 16 18.5'),
 ],rigid((t,s,r)=>({dx:r?0:flinch(t,s)}))),
 ...posed([
  ...tube('ring',circle(23.5,23,4.5)),
  {name:'slash',style:'outline',type:'stroke',width:2,d:(t,s,r)=>`M${A} L${tip(t,s,r)}`,visibility:(t,s,r)=>k(t,s,r)>.05?1:0},
  {name:'slash-solid',style:'solid',d:(t,s,r)=>capsule(A,tip(t,s,r),2.6),visibility:(t,s,r)=>k(t,s,r)>.05?1:0},
 ],rigid((t,s,r)=>({dx:r?0:jolt(t,s)}))),
],.5,.6,'allowed','blocked'),duotoneFill:true};

export const icons=[faceAuth,twoFactor,recovery,privateBrowsing,cookie,blockUser];
