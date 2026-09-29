// account-privacy batch A — Claude revision 2 (draft, not live)
import {make,toggle,geometricRearm,F,tube,framed,fill,stroke,rigid,posed,shown,rect,circle,arc,hole,holes,capsule,annulus} from './helpers.mjs';

/* 1. sign-in-out — toggle signed-out ⇄ signed-in.
 Shaft moved to start x6 (was 4) and Strong prep reduced to -1.2: painted left
 margin 6-1.2-1.3 = 3.5 (Solid) at the prep frame; leave overshoot -.8 gives 3.9.
 Travel stays 7. Shaft end 13 / head tip 14 keep a 1-unit joint overlap.
 Entered overshoot 7.9: tip 21.9+1.3 = 23.2 vs Solid rim inner edge 25.2 (gap 2.0). */
const enter=F([[0,0],[.5,7,'smooth']],[[0,0],[.14,-1.2],[.42,7.9,'hit'],[.62,7]]);
const leave=F([[0,7],[.5,0,'smooth']],[[0,7],[.14,8.2],[.42,-.8,'hit'],[.62,0]]);
const arrowPose=rigid((t,s,r)=>({dx:(r?leave:enter)(t,s)}));
const signInOut=toggle('sign-in-out','Sign in / out','Arrow enters the open doorway and stays inside; Sign out slides it back out.',[
 ...tube('doorway','M14 5.5 L24.5 5.5 C25.9 5.5 26.5 6.1 26.5 7.5 L26.5 24.5 C26.5 25.9 25.9 26.5 24.5 26.5 L14 26.5'),
 ...posed(tube('shaft','M6 16 L13 16'),arrowPose),
 ...posed(tube('head','M10.5 12.5 L14 16 L10.5 19.5'),arrowPose),
],.5,.62,'signed-out','signed-in',.5,.62);
signInOut.duotoneFill=false;

/* 2. switch-account — toggle personal ⇄ work, same endpoints (dx 0 → 14).
 Persons raised: head cy 9, shoulders bottom 20.8; marker lowered to y 26.5.
 Shoulder paint bottom 20.8+1.3 = 22.1, nod 1.0 → 23.1; marker paint top 25.2.
 Worst-case gap 2.1 (need ≥1.15). Marker overshoot 15.2 → 27.7+1.3 = 29.0 (margin 3). */
const slide=F([[0,0],[.45,14,'smooth']],[[0,0],[.12,-1.2],[.38,15.2,'hit'],[.58,14]]);
const slideBack=F([[0,14],[.45,0,'smooth']],[[0,14],[.12,15.2],[.38,-1.2,'hit'],[.58,0]]);
const nod=F([[0,0]],[[0,0],[.38,0,'hold'],[.46,1],[.62,0]]);
const person=(n,cx,pose)=>posed([
 ...framed(n+'-head',circle(cx,9,2.6),circle(cx,9,3.9)),
 ...tube(n+'-shoulders',`M${cx-4.5} 20.8 C${cx-4.5} 16.8 ${cx-2.6} 15.4 ${cx} 15.4 C${cx+2.6} 15.4 ${cx+4.5} 16.8 ${cx+4.5} 20.8`),
],pose);
const switchAccount=toggle('switch-account','Switch account','The selection marker slides from the personal account to the work account; the chosen person nods.',[
 ...person('personal',9,rigid((t,s,r)=>({dy:r?nod(t,s):0}))),
 ...person('work',23,rigid((t,s,r)=>({dy:r?0:nod(t,s)}))),
 ...posed(tube('marker','M5.5 26.5 L12.5 26.5'),rigid((t,s,r)=>({dx:(r?slideBack:slide)(t,s)}))),
],.45,.62,'personal','work',.45,.62);
switchAccount.duotoneFill=true;

/* 3. id-card — result + preview reset. Main action = detail fill.
 Strong: tap -1 at .10, contact +.7 'hit' at .20; details fire .16–.34 (.18 s,
 monotone, no overshoot) overlapping the contact, then card settles by .38.
 Light: details ease in over .5 with no tap. Full width now 24.2:
 Bold cap edge 25.45 vs frame inner edge 26.75 → gap 1.3. Rows 5.4 apart (2.9 clear at Bold). */
const reveal=F([[0,0],[.5,1,'smooth']],[[0,0],[.16,0,'hold'],[.34,1]]);
const tap=F([[0,0]],[[0,0],[.1,-1],[.2,.7,'hit'],[.38,0]]);
const cardPose=rigid((t,s)=>({dy:tap(t,s)}));
const lineEnd=(t,s)=>18.2+6*reveal(t,s);
const lineHole=(y,t,s)=>capsule([16.2,y],[lineEnd(t,s),y],2.4);
const idSolid=(t,s)=>holes(rect(2.7,5.7,26.6,20.6,3.8),circle(10.5,16,2.2),lineHole(13.3,t,s),lineHole(18.7,t,s));
const idCard=geometricRearm(make('id-card','ID card','The card taps and its identity details fill in beside the photo; details stay revealed.',posed([
 ...framed('card',rect(4,7,24,18,2.5),idSolid),
 fill('photo',circle(10.5,16,2.2),'outline'),
 stroke('line-1',(t,s)=>`M16.2 13.3 L${lineEnd(t,s)} 13.3`),
 stroke('line-2',(t,s)=>`M16.2 18.7 L${lineEnd(t,s)} 18.7`),
],cardPose),.5,.5));
idCard.duotoneFill=true;

/* 4. password — result + preview reset. Field grown to y 9–23 (14 tall).
 Solid well inner top 10.3; Strong drop starts dy -1.8 → dot top 12.9 (gap 2.6);
 Bold outline inner edge 10.25 → gap 2.65. Contact +.4 → dot bottom 17.7 vs well floor 21.7.
 Strong: each dot pops visible in .04 s at its own onset, drops, 'hit', settles; stagger .09.
 Light: sequential fade, stagger .12. Dot 1 is fixed in both profiles. */
const typed=i=>F([[0,0],[.08+.12*i,0],[.24+.12*i,1,'smooth']],[[0,0],[.1+.09*i,0,'hold'],[.14+.09*i,1]]);
const drop=i=>F([[0,0]],[[0,-1.8],[.1+.09*i,-1.8,'hold'],[.2+.09*i,.4,'hit'],[.3+.09*i,0]]);
const dot=(i,x)=>{const parts=[fill('dot-'+(i+1),circle(x,16,1.3),'outline'),fill('dot-'+(i+1)+'-solid',circle(x,16,1.3),'solid')];
 if(!i)return parts;const v=typed(i-1),d=drop(i-1);return shown(posed(parts,rigid((t,s)=>({dy:d(t,s)}))),v);};
const password=geometricRearm(make('password','Password','Masked characters type into the field one by one and stay entered.',[
 ...framed('field',rect(4,9,24,14,3.5),hole(rect(2.7,7.7,26.6,16.6,4.8),rect(5.3,10.3,21.4,11.4,2.2))),
 ...dot(0,10.5),...dot(1,14.2),...dot(2,17.9),...dot(3,21.6),
],.5,.55));
password.duotoneFill=true;

/* 5. passkey — gesture. Person shifted to cx 9.5 (shoulders x 4.2–14.8):
 Solid left margin 4.2-1.3 = 2.9. Key ring centre (21.5,10.5), pivot there.
 At -32°: shaft end → (27.3,19.8), tooth end → (27.9,16.5); +1.3 cap = 29.2 (margin 2.8).
 At +6° prep the shaft paint edge is ≈19.0 vs shoulder paint 16.1 (gap 2.9). Ring counter r 1.3 at Solid, 1.35 at Bold. */
const turn=F([[0,0],[.22,-15,'smooth'],[.5,0,'smooth']],[[0,0],[.14,6],[.36,-32,'hit'],[.62,0]]);
const keyNod=F([[0,0]],[[0,0],[.36,0,'hold'],[.44,1],[.62,0]]);
const keyPose=rigid((t,s)=>({angle:turn(t,s),pivot:[21.5,10.5]}));
const passkey=make('passkey','Passkey','A device key swings beside the account holder and settles; the person acknowledges.',[
 ...posed([...framed('head',circle(9.5,9,3),circle(9.5,9,4.3)),
  ...tube('shoulders','M4.2 23 C4.2 17.8 6.4 15.6 9.5 15.6 C12.6 15.6 14.8 17.8 14.8 23')],rigid((t,s)=>({dy:keyNod(t,s)}))),
 ...posed([stroke('ring',circle(21.5,10.5,2.6)),fill('ring-solid',annulus(21.5,10.5,2.6,2.6),'solid'),
  ...tube('shaft','M21.5 13.1 L21.5 21.5'),...tube('tooth','M21.5 19 L23.7 19')],keyPose),
],.5,.62);
passkey.duotoneFill=true;

/* 6. fingerprint — gesture, redrawn for measured clearance. Same three parts.
 Every neighboring pair is a true offset: concentric arcs (shared centre) or
 parallel lines, so the closest curve distance equals the nominal spacing.
 Centreline spacing outer→middle = 4.0 everywhere; core→middle ≥ 4.2.
 Paint gap = spacing − 2.6 (Solid) / − 2.5 (Bold): ≥ 1.4 / 1.5.
 Core is a short straight bar: inside the middle ridge only x 15.8–16.2 is
 ≥4.0 clear of both sides, so a curled hook cannot fit at this weight.
 Rest bounds: top 4.8−1.3 = 3.5, bottom 26.2+1.3 = 27.5, x 6.5–25.5.
 Strong lift −.8 → top 2.7; press +2 → bottom 29.5, margin 2.5 (≥2.0).
 flow: hit key at .26 is the extremum (out-tangent negative), no overshoot. */
const seg=(...p)=>p.map((s,i)=>i?s.replace(/^M/,'L'):s).join(' ');
const R1='M16 13 L16 17.5';
const R2=seg('M11.8 25.4 L11.8 13',arc(16,13,4.2,180,360),'M20.2 18.7',arc(16.7,18.7,3.5,0,90));
const R3=seg('M7.8 23.6 L7.8 13',arc(16,13,8.2,180,360),'M24.2 18.7',arc(16.7,18.7,7.5,0,90));
const press=F([[0,0],[.25,.6,'smooth'],[.6,0,'smooth']],[[0,0],[.12,-.8],[.26,2,'hit'],[.5,0]]);
const glow=i=>F([[0,1],[.05+.1*i,1],[.2+.1*i,.45,'smooth'],[.5+.1*i,1,'smooth']],[[0,1],[.26+.06*i,1,'hold'],[.33+.06*i,.3],[.52+.06*i,1]]);
const printPose=rigid((t,s)=>({dy:press(t,s)}));
const ridge=(i,d)=>shown(posed(tube('ridge-'+(i+1),d),printPose),glow(i));
const fingerprint=make('fingerprint','Fingerprint','The finger presses onto the sensor and its ridges light from the core outward.',[
 ...ridge(0,R1),
 ...ridge(1,R2),
 ...ridge(2,R3),
],.7,.72);
fingerprint.duotoneFill=false;

export const icons=[signInOut,switchAccount,idCard,password,passkey,fingerprint];
