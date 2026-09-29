// Onboarding & tips (49). All artwork and motion are authored in this file on the 32-unit grid.
// Regular outline centrelines are width 2 (the builder makes the 1.5 / 2.5 presets and binds the
// stroke-width token). Solid parts are separate constructions; tint parts are real enclosed surfaces.
import {make,timePair,drive,track,fill,stroke,tube,bothFill,framed,posed,shown,rigid,rect,circle,arc,polygon,holes,annulus,segment,rotate,mix} from '../library/ai-authoring.mjs';

// ---------- shared assembly ----------
const toggle=(name,label,intent,parts,light,strong,a,b,rl=light,rs=strong)=>({...make(name,label,intent,parts,light,strong),stateful:true,initialState:a,finalState:b,reverseSuffix:'-reverse',forwardLabel:b,reverseLabel:a,reverseTimings:timePair(rl,rs)});
// Forward/reverse driver pair: Light uses eased tracks, Strong uses bounded cubic flow.
const dir=(l,h,rl=l,rh=h)=>(t,s,r)=>drive(t,s,r?rl:l,r?rh:h);
const F=(l,h)=>(t,s)=>drive(t,s,l,h);
// Eased tracks for both profiles: fades, pulses and bursts that need an explicit out/linear ease.
const T=(l,h)=>(t,s)=>track(s?h:l,t);
const lerp=(a,b,u)=>[mix(a[0],b[0],u),mix(a[1],b[1],u)];
const clamp01=x=>Math.max(0,Math.min(1,x));
// Four-point sparkle: cusped tips, concave sides (control points pulled toward the centre).
const sparkle=(cx,cy,R,rot=0,c=.3)=>{const tip=k=>{const a=(rot+90*k)*Math.PI/180;return[cx+R*Math.cos(a),cy+R*Math.sin(a)];},ctl=p=>[cx+(p[0]-cx)*c,cy+(p[1]-cy)*c];let d='M'+tip(0);for(let k=0;k<4;k++){const a=tip(k),b=tip(k+1);d+=' C'+[...ctl(a),...ctl(b),...b];}return d+' Z';};
// Scale-and-turn pose about a centre (a morph, not a native transform).
const spin=(cx,cy,angle,scale)=>(x,y,t,s,r)=>{const a=angle(t,s,r),k=scale(t,s,r),[rx,ry]=rotate(x,y,a,cx,cy);return[cx+(rx-cx)*k,cy+(ry-cy)*k];};
const star=(cx,cy,n,R,r,rot=-90)=>Array.from({length:2*n},(_,i)=>{const a=(rot+180*i/n)*Math.PI/180,rad=i%2?r:R;return[cx+rad*Math.cos(a),cy+rad*Math.sin(a)];});
const leafRing=holes(rect(3.7,4.7,24.6,24.6,4),rect(6.3,7.3,19.4,19.4,1.4));

// ---------- 1. SPARKLE (gesture) ----------
// Big sparkle (14,18.5) R8.5 and a small one (25,7) R3. Light: a quiet twinkle. Strong: the big
// sparkle shrinks, bursts to 1.12 with a 40° twist and settles; the small one blinks and pops.
const spA=F([[0,0],[.24,10],[.56,0,'smooth']],[[0,0],[.1,-14],[.28,40,'hit'],[.44,-6],[.56,2],[.64,0]]);
const spK=F([[0,1],[.24,.86],[.56,1,'smooth']],[[0,1],[.1,.6],[.28,1.12,'hit'],[.44,.94],[.56,1.02],[.64,1]]);
const smK=F([[0,1],[.3,.72],[.6,1,'smooth']],[[0,1],[.14,.5],[.36,1.2,'hit'],[.5,.95],[.6,1]]);
const smV=F([[0,1],[.3,.4],[.6,1,'smooth']],[[0,1],[.12,0,'out'],[.18,0],[.36,1,'out']]);
const sparkleIcon=make('sparkle','Sparkle',
 'Light: the large sparkle twinkles (scale 0.86, 10° turn) while the small one dims. Strong: it shrinks to 0.6, bursts to 1.12 with a 40° twist and settles; the small sparkle blinks out and pops back.',[
 ...posed([fill('big-interior',sparkle(14,18.5,8.5),'tint'),stroke('big',sparkle(14,18.5,8.5)),fill('big-solid',sparkle(14,18.5,8.5),'solid')],spin(14,18.5,spA,spK)),
 ...shown(posed(bothFill('small',sparkle(25,7,3)),spin(25,7,()=>0,smK)),smV)
],.6,.64);
sparkleIcon.duotoneFill=true;

// ---------- 2. LIGHTBULB (stateful off → on) ----------
// Glass circle (16,14) r5 with a neck to the base line y21.5; foot at y25. On: filament fades in,
// three rays extend from r8.5 to r10.4 (1.0 clear of the glass paint at Bold, top ray paint ≥2 from the edge).
const glassD='M13 21.5 L13 18 '+arc(16,14,5,126.87,413.13).replace(/^M/,'L')+' L19 21.5 Z';
const rayLen=dir([[0,0],[.08,0],[.42,1.9,'out']],[[0,0],[.06,0,'hold'],[.18,2.2],[.32,1.9]],[[0,1.9],[.34,0,'in']],[[0,1.9],[.1,2.1],[.26,0]]);
const filament=dir([[0,0],[.1,0],[.36,1,'out']],[[0,0],[.06,0],[.16,1,'out']],[[0,1],[.3,0,'in']],[[0,1],[.22,0,'in']]);
const jolt=dir([[0,0],[.5,0]],[[0,0],[.06,.4],[.18,-.7,'hit'],[.3,.2],[.42,0]],[[0,0],[.4,0]],[[0,0],[.35,0]]);
const bulbPose=rigid((t,s,r)=>({dy:jolt(t,s,r),pivot:[16,25]}));
const ray=(name,deg)=>{const p=len=>[16+len*Math.cos(deg*Math.PI/180),14+len*Math.sin(deg*Math.PI/180)];
 // Shorten the top ray by the upward jolt to retain the 2-unit Solid paint margin.
 return shown(posed(tube(name,(t,s,r)=>segment(p(8.5),p(8.5+Math.max(.001,rayLen(t,s,r)+(deg===-90?Math.min(0,jolt(t,s,r)):0))))),bulbPose),(t,s,r)=>clamp01((rayLen(t,s,r)-.15)/.6));};
const lightbulb=toggle('lightbulb','Lightbulb',
 'Off → on. Light: the filament fades in and three rays extend 1.9 units from the glass. Strong: the bulb jolts up 0.7 as the rays shoot out 2.2 and settle back to 1.9. The reverse retracts the rays and fades the filament.',[
 ...posed(framed('glass',glassD),bulbPose),
 ...posed([stroke('filament','M16 12.8 L16 15.2',{visibility:filament})],bulbPose),
 ...posed(tube('foot','M14 25 L18 25'),bulbPose),
 ...ray('ray-up',-90),...ray('ray-left',-135),...ray('ray-right',-45)
],.52,.5,'off','on',.42,.36);
lightbulb.duotoneFill=true;

// ---------- 3. PRODUCT TOUR (stateful stop 1 → stop 3) ----------
// Three stop rings r2 on a route; the active marker (disc r2.3) travels A→B→C along the route lines.
const TA=[7,22],TB=[16,10],TC=[25,22];
const routePos=u=>u<1?lerp(TA,TB,u):lerp(TB,TC,u-1);
const tourU=dir([[0,0],[.05,0],[.36,1,'smooth'],[.42,1],[.74,2,'smooth']],[[0,0],[.04,0,'hold'],[.22,1,'hit'],[.32,1,'hold'],[.5,2,'hit'],[.6,2]],
 [[0,2],[.05,2],[.36,1,'smooth'],[.42,1],[.74,0,'smooth']],[[0,2],[.04,2,'hold'],[.22,1,'hit'],[.32,1,'hold'],[.5,0,'hit'],[.6,0]]);
// Strong hops: the marker lifts up to 2.2 in the middle of each leg and lands on the stop.
const tourHop=(t,s,r)=>{if(!s)return 0;const u=tourU(t,s,r),f=u-Math.floor(u);return -2.2*Math.sin(Math.PI*Math.min(1,Math.max(0,f)));};
const trim=(a,b,k)=>{const d=Math.hypot(b[0]-a[0],b[1]-a[1]);return lerp(a,b,k/d);};
const productTour=toggle('product-tour','Product tour',
 'The tour marker travels from the first stop over the second to the last stop along the route. Light: two smooth glides with a pause at the middle stop. Strong: two quick hops that lift 2.2 and land hard on each stop. Reverse walks the route back.',[
 ...tube('stop-a',circle(...TA,2)),...tube('stop-b',circle(...TB,2)),...tube('stop-c',circle(...TC,2)),
 ...tube('route-ab',segment(trim(TA,TB,5.65),trim(TB,TA,5.65))),...tube('route-bc',segment(trim(TB,TC,5.65),trim(TC,TB,5.65))),
 ...posed(bothFill('marker',circle(...TA,2.3)),rigid((t,s,r)=>{const p=routePos(tourU(t,s,r));return{dx:p[0]-TA[0],dy:p[1]-TA[1]+tourHop(t,s,r),pivot:TA};}))
],.76,.64,'start','end',.76,.64);
productTour.duotoneFill=false;

// ---------- 4. TOOLTIP (gesture) ----------
// Bubble x6–26 y6.5–18.5 with a pointer to y21 above a target dot r1.5 at (16,26). Two text lines inside.
const tipBox=polygon([[6,6.5],[26,6.5],[26,18.5],[18,18.5],[16,21],[14,18.5],[6,18.5]],[2.5,2.5,2.5,.4,.5,.4,2.5]);
const tipLift=F([[0,0],[.22,-1.2,'out'],[.55,0,'smooth']],[[0,0],[.08,0,'hold'],[.24,-2.6],[.4,.4],[.52,-.15],[.6,0]]);
const tipTap=F([[0,0],[.55,0]],[[0,0],[.08,.6,'hit'],[.22,0]]);
const tipPose=rigid((t,s)=>({dy:tipLift(t,s)}));
const tooltip=make('tooltip','Tooltip',
 'Light: the bubble lifts 1.2 above its target and settles. Strong: the target dot taps down, the bubble pops up 2.6 and settles with one damped bounce.',[
 ...posed(framed('bubble',tipBox),tipPose),
 ...posed([stroke('line-0','M10 11 L22 11'),stroke('line-1','M10 14.5 L18 14.5')],tipPose),
 ...posed(bothFill('target',circle(16,26,1.5)),rigid((t,s)=>({dy:tipTap(t,s),pivot:[16,26]})))
],.55,.6);
tooltip.duotoneFill=true;

// ---------- 5. HOTSPOT (gesture) ----------
// Beacon: dot r2.4 inside a ring r6. Pulse rings expand from the ring and fade; they are invisible at rest.
const pulseR=[T([[0,6],[.62,9.6,'out']],[[0,6],[.46,11.5,'out']]),T([[0,6],[.62,6]],[[0,6],[.2,6],[.66,11.5,'out']])];
const pulseV=[T([[0,0],[.02,1,'linear'],[.62,0,'linear']],[[0,0],[.02,1,'linear'],[.46,0,'linear']]),T([[0,0],[.62,0]],[[0,0],[.2,0],[.22,1,'linear'],[.66,0,'linear']])];
const dotK=F([[0,1],[.6,1]],[[0,1],[.06,.8],[.18,1.35,'hit'],[.3,.95],[.4,1]]);
const hotspot=make('hotspot','Hotspot',
 'Light: one pulse ring expands from the beacon to r9.6 and fades. Strong: the dot pops to 1.35× and two faster rings expand to r11.5 in succession.',[
 fill('well',circle(16,16,6),'tint'),
 ...tube('ring',circle(16,16,6)),
 ...posed(bothFill('dot',circle(16,16,2.4)),spin(16,16,()=>0,dotK)),
 ...[0,1].flatMap(i=>shown(tube('pulse-'+i,(t,s)=>circle(16,16,pulseR[i](t,s))),pulseV[i]))
],.64,.68);
hotspot.duotoneFill=true;

// ---------- 6. NEW BADGE (gesture) ----------
// Ten-point burst seal (R11.6 / r8.9) with a five-point star (R4.4 / r1.9) counter. The seal turns
// rigidly about the centre; the star pops as a follower.
const sealD=polygon(star(16,16,10,11.6,8.9),[.7,.5]),starD=polygon(star(16,16,5,4.4,1.9),[.5,.3]);
const sealA=F([[0,0],[.26,8],[.6,0,'smooth']],[[0,0],[.08,-12],[.26,26,'hit'],[.4,-8],[.52,3],[.62,0]]);
const starK=F([[0,1],[.26,1.1],[.6,1,'smooth']],[[0,1],[.08,.7],[.26,1.2,'hit'],[.4,.92],[.52,1.03],[.62,1]]);
const newBadge=make('new-badge','New badge',
 'Light: the burst seal turns 8° and eases back while the star swells slightly. Strong: the seal winds back 12°, swings 26° with a hard stop and settles; the star pops to 1.2×.',[
 ...posed([fill('seal-interior',sealD,'tint'),stroke('seal',sealD),fill('seal-solid',holes(sealD,starD),'solid')],rigid((t,s)=>({angle:sealA(t,s)}))),
 {...stroke('star',starD),pose:spin(16,16,()=>0,starK)}
],.6,.62);
newBadge.duotoneFill=true;

// ---------- 7. CONFETTI (gesture) ----------
// Four strips, two dots and two ribbons rest scattered. Light: they sway and turn in place.
// Strong: the lower ribbon stays visible while the other pieces gather invisibly and burst out, spinning.
const strip=([x,y])=>rect(x-1.6,y-.85,3.2,1.7,.6);
const PIECES=[{n:'strip-0',c:[8,9],a:25,d:strip},{n:'strip-1',c:[23,7],a:-30,d:strip},{n:'strip-2',c:[13,22],a:60,d:strip},{n:'strip-3',c:[25,24],a:-20,d:strip},
 {n:'dot-0',c:[26,15],a:0,d:([x,y])=>circle(x,y,1.3)},{n:'dot-1',c:[6,19],a:0,d:([x,y])=>circle(x,y,1.3)},
 {n:'ribbon-0',c:[18.8,15.6],a:0,d:([x,y])=>`M${x-2.4} ${y+.6} C${x-1.4} ${y-1} ${x-.6} ${y+2} ${x+.4} ${y+.4} C${x+1.1} ${y-.7} ${x+1.8} ${y+1.3} ${x+2.5} ${y+.2}`,tube:true},
 {n:'ribbon-1',c:[7,27],a:0,d:([x,y])=>`M${x-1.4} ${y+.8} C${x-.4} ${y-.8} ${x+.4} ${y+2} ${x+1.4} ${y+.4}`,tube:true}];
const ORIGIN=[16,26];
const confettiParts=PIECES.flatMap((p,i)=>{
 const ph=i*.07,sway=T([[0,0],[.28+ph,p.n==='ribbon-1'?.5:1.1],[.72,0,'smooth']],[[0,0],[.06,0],[.061,ORIGIN[1]-p.c[1],'linear'],[.52,0,'out']]);
 const swayX=T([[0,0],[.28+ph,i%2?-.9:.9],[.72,0,'smooth']],[[0,0],[.06,0],[.061,ORIGIN[0]-p.c[0],'linear'],[.52,0,'out']]);
 const turn=T([[0,0],[.34+ph,i%2?22:-22],[.72,0,'smooth']],[[0,0],[.06,0],[.061,i%2?400:-400,'linear'],[.52,0,'out']]);
 const show=T([[0,1],[.72,1]],[[0,1],[.05,0,'linear'],[.16,0],[.34,1,'linear']]);
 const parts=p.tube?tube(p.n,p.d(p.c)):bothFill(p.n,p.d(p.c));
 const stays=s=>s&&p.n==='ribbon-1';
 return shown(posed(parts,rigid((t,s)=>stays(s)?{angle:p.a,pivot:p.c}:{dx:swayX(t,s),dy:sway(t,s),angle:p.a+turn(t,s),pivot:p.c})),(t,s)=>stays(s)?1:show(t,s));
});
const confetti=make('confetti','Confetti',
 'Light: the scattered pieces drift and turn gently in place. Strong: the lower ribbon stays visible while the other pieces burst from the bottom centre to their resting places with a full spin and fade back in.',confettiParts,.72,.54);
confetti.duotoneFill=false;

// ---------- 8. WELCOME WAVE (gesture) ----------
// Palm x10.5–24.5 y13.5–24 with four finger strokes (3.6 apart, 1.0 painted gap for Solid tubes) and a thumb.
// The whole hand waves rigidly about the wrist (17.5,24.5).
const wave=F([[0,0],[.22,-8],[.52,7],[.8,0,'smooth']],[[0,0],[.06,5],[.2,-14,'hit'],[.36,15,'hit'],[.5,-9],[.62,4],[.72,0]]);
const handPose=rigid((t,s)=>({angle:wave(t,s),pivot:[17.5,24.5]}));
const welcomeWave=make('welcome-wave','Welcome wave',
 'Light: the hand waves once, 8° each way about the wrist. Strong: a bigger, faster wave (15° each way with two hard reversals) that damps out over four beats.',[
 ...posed([...framed('palm',rect(10.5,13.5,14,10.5,3)),
  ...tube('finger-0','M12.1 13.5 L12.1 9'),...tube('finger-1','M15.7 13.5 L15.7 6.8'),...tube('finger-2','M19.3 13.5 L19.3 7.2'),...tube('finger-3','M22.9 13.5 L22.9 9.4'),
  ...tube('thumb','M10.5 17.7 L7.1 14')],handPose)
],.8,.72);
welcomeWave.duotoneFill=true;

// ---------- 9. PROGRESS CHECKLIST (stateful unchecked → complete) ----------
// Card frame with three rows; each check is drawn by a pen tip (M L L, future points collapsed) and
// its row line nudges right when the check lands (Strong only).
const ROWS=[10,16,22];
const checkProgress=i=>dir([[.05+.2*i,0],[.27+.2*i,1,'out']],[[.02+.1*i,0],[.14+.1*i,1,'out']],[[.05+.15*(2-i),1],[.25+.15*(2-i),0,'in']],[[.02+.08*(2-i),1],[.12+.08*(2-i),0,'in']]);
const checkD=(y,prog)=>(t,s,r)=>{const p=prog(t,s,r),a=[8,y+.3],b=[9.8,y+2.1],c=[12.6,y-1.5];if(p<.4){const tip=lerp(a,b,p/.4);return`M${a} L${tip} L${tip}`;}return`M${a} L${b} L${lerp(b,c,(p-.4)/.6)}`;};
const rowNudge=i=>dir([[0,0],[.7,0]],[[0,0],[.13+.1*i,0,'hold'],[.19+.1*i,.6,'hit'],[.34+.1*i,0]],[[0,0],[.5,0]],[[0,0],[.4,0]]);
const progressChecklist=toggle('progress-checklist','Progress checklist',
 'Unchecked → complete. Light: three checks are drawn in sequence, each in 0.22 s. Strong: fast overlapping strokes 0.1 s apart, and each row line nudges right as its check lands. Reverse erases the checks from the bottom up.',[
 ...framed('card',rect(4.5,4.5,23,23,3),holes(rect(3.2,3.2,25.6,25.6,4.3),rect(5.7,5.7,20.6,20.6,1.7))),
 ...ROWS.flatMap((y,i)=>{const prog=checkProgress(i),nudge=rowNudge(i);return[
  ...shown(tube('check-'+i,checkD(y,prog)),(t,s,r)=>clamp01(prog(t,s,r)/.08)),
  ...posed(tube('row-'+i,`M16 ${y} L23.4 ${y}`),rigid((t,s,r)=>({dx:nudge(t,s,r),pivot:[16,y]})))];})
],.7,.56,'unchecked','complete',.56,.36);
progressChecklist.duotoneFill=true;

// ---------- 10. SKIP STEP (gesture) ----------
// Two chevrons and a stop bar. Light: both nudge right 1.4. Strong: a backswing, then the leading
// chevron strikes the bar (tip paint meets bar paint at Bold), the bar kicks 0.7 and everything settles.
const skipLead=F([[0,0],[.22,1.4,'out'],[.55,0,'smooth']],[[0,0],[.08,-1.2],[.2,2.5,'hit'],[.34,-.4],[.46,.2],[.56,0]]);
const skipTrail=F([[0,0],[.24,1.4,'out'],[.55,0,'smooth']],[[0,0],[.1,-1.2],[.24,2.4,'hit'],[.38,-.3],[.5,0]]);
const skipBar=F([[0,0],[.55,0]],[[0,0],[.2,0,'hold'],[.26,.7,'hit'],[.4,-.2],[.5,0]]);
const skipStep=make('skip-step','Skip step',
 'Light: both chevrons nudge 1.4 toward the stop bar and ease back. Strong: they swing back 1.2, the leading chevron drives 2.5 into the bar, the bar kicks 0.7 and both chevrons rebound to rest.',[
 ...posed(tube('chevron-0','M6 9.5 L12.5 16 L6 22.5'),rigid((t,s)=>({dx:skipTrail(t,s),pivot:[6,16]}))),
 ...posed(tube('chevron-1','M14 9.5 L20.5 16 L14 22.5'),rigid((t,s)=>({dx:skipLead(t,s),pivot:[14,16]}))),
 ...posed(tube('bar','M25.5 9.5 L25.5 22.5'),rigid((t,s)=>({dx:skipBar(t,s),pivot:[25.5,16]})))
],.55,.56);
skipStep.duotoneFill=false;

// ---------- 11. NEXT STEP (gesture) ----------
// Round button r10.5 with a stem-and-barb arrow. The arrow nudges (Light) or winds up and drives 3.2 (Strong);
// the whole button dips 0.6 in response to the Strong drive.
const nextDx=F([[0,0],[.2,1.4,'out'],[.55,0,'smooth']],[[0,0],[.08,-1.2],[.22,3.2,'hit'],[.36,-.5],[.48,.2],[.58,0]]);
const nextDy=F([[0,0],[.55,0]],[[0,0],[.22,0,'hold'],[.28,.6,'hit'],[.42,-.1],[.52,0]]);
const nextStep=make('next-step','Next step',
 'Light: the arrow slides 1.4 to the right and eases back. Strong: it draws back 1.2, drives 3.2 with a hard stop, the button dips 0.6 and both settle.',[
 ...posed([fill('button-interior',circle(16,16,10.5),'tint'),stroke('button',circle(16,16,10.5)),fill('button-solid',annulus(16,16,10.5,2.6),'solid')],rigid((t,s)=>({dy:nextDy(t,s)}))),
 ...posed([...tube('stem','M11.5 16 L19.5 16'),...tube('barb','M15.5 12 L19.5 16 L15.5 20')],rigid((t,s)=>({dx:nextDx(t,s),dy:nextDy(t,s)})))
],.55,.58);
nextStep.duotoneFill=true;

// ---------- 12. SPARK WAND (gesture) ----------
// Wand (7,26)→(18.5,14.5), a sparkle at (24,8.5) R4 and two spark dots. Light: the sparkle twinkles.
// Strong: the wand flicks 6° / −12° about its base and the sparkle bursts with a 45° twist.
const wandA=F([[0,0],[.56,0]],[[0,0],[.08,6],[.22,-12,'hit'],[.36,4],[.48,-1],[.56,0]]);
const wandK=F([[0,1],[.26,.8],[.6,1,'smooth']],[[0,1],[.12,.5],[.3,1.12,'hit'],[.44,.92],[.56,1]]);
const wandSpA=F([[0,0],[.26,18],[.6,0,'smooth']],[[0,0],[.12,-20],[.3,45,'hit'],[.44,-8],[.56,0]]);
const sparkV=d=>F([[0,1],[.2+d,.3],[.5+d,1,'smooth']],[[0,1],[.1+d,0,'out'],[.18+d,0],[.34+d,1,'out']]);
const sparkWand=make('spark-wand','Spark wand',
 'Light: the sparkle twinkles (0.8×, 18° turn) and the spark dots dim. Strong: the wand flicks 6° back and 12° forward about its base while the sparkle bursts to 1.12× with a 45° twist and the dots blink.',[
 ...posed(tube('wand','M7 26 L18.5 14.5'),rigid((t,s)=>({angle:wandA(t,s),pivot:[7,26]}))),
 ...posed([stroke('sparkle',sparkle(24,8.5,4)),fill('sparkle-solid',sparkle(24,8.5,4),'solid')],spin(24,8.5,wandSpA,wandK)),
 ...shown(bothFill('spark-0',circle(28,15.5,1)),sparkV(0)),...shown(bothFill('spark-1',circle(17.5,4.5,.9)),sparkV(.08))
],.6,.56);
sparkWand.duotoneFill=false;

// ---------- 13. HOTKEY HINT (gesture) ----------
// Keycap: top face x8–24 y7–20 over a skirt x4.5–27.5 (1.0 painted gap at Bold). The face presses down.
const keyDy=F([[0,0],[.18,1.2,'out'],[.5,0,'smooth']],[[0,0],[.08,2.4,'hit'],[.18,2.4,'hold'],[.3,-.5],[.42,.15],[.5,0]]);
const keyPose=rigid((t,s)=>({dy:keyDy(t,s)}));
const hotkeyHint=make('hotkey-hint','Hotkey hint',
 'Light: the keycap face presses down 1.2 and rises. Strong: a hard 2.4 press, a short hold, then the face springs 0.5 past rest and settles.',[
 ...tube('skirt','M4.5 13.5 L4.5 23 C4.5 24.66 5.84 26 7.5 26 L24.5 26 C26.16 26 27.5 24.66 27.5 23 L27.5 13.5'),
 ...posed(framed('face',rect(8,7,16,13,2.5)),keyPose),
 ...posed([stroke('legend','M13 13.5 L19 13.5')],keyPose)
],.5,.5);
hotkeyHint.duotoneFill=true;

// ---------- 14. SPOTLIGHT (stateful left → right) ----------
// A dimmed page (framed rect) with a round spotlight cut out of it; the spot glides from the left
// target to the right one. Solid and Duotone paint the page with a real counter.
const pageD=rect(3.5,5.5,25,21,2.5);
const spotX=dir([[0,11.5],[.06,11.5],[.62,20.5,'smooth']],[[0,11.5],[.08,10.8],[.3,21.2,'hit'],[.44,20.3],[.54,20.5]],
 [[0,20.5],[.06,20.5],[.62,11.5,'smooth']],[[0,20.5],[.08,21.2],[.3,10.8,'hit'],[.44,11.7],[.54,11.5]]);
const spotR=dir([[0,3.8],[.6,3.8]],[[0,3.8],[.34,3.8,'hold'],[.42,4.4],[.58,3.8]]);
const spotD=(t,s,r)=>circle(spotX(t,s,r),16,spotR(t,s,r));
const spotlight=toggle('spotlight','Spotlight',
 'Left → right. Light: the spotlight glides 9 units across the dimmed page. Strong: it draws back 0.7, drives across with a hard stop 0.7 past the target, settles and swells once. Reverse glides back.',[
 fill('page-interior',(t,s,r)=>holes(pageD,spotD(t,s,r)),'tint'),
 stroke('page',pageD),
 fill('page-solid',(t,s,r)=>holes(pageD,spotD(t,s,r)),'solid'),
 stroke('spot',spotD)
],.64,.6,'left','right',.64,.6);
spotlight.duotoneFill=true;

// ---------- 15. COACHMARK (stateful hidden → shown) ----------
// A pill button stays; the callout card with a tail slides in from up-left and fades in.
const cardD=polygon([[4.5,4.5],[21.5,4.5],[21.5,16],[18,16],[17,18],[14,16],[4.5,16]],[2.2,2.2,2.2,.3,.4,.3,2.2]);
const cmX=dir([[0,-1],[.08,-1],[.4,0,'out']],[[0,-1],[.08,-1,'hold'],[.22,.5,'hit'],[.34,-.15],[.44,0]],[[0,0],[.36,-1,'in']],[[0,0],[.06,.3],[.24,-1]]);
const cmY=dir([[0,-1],[.08,-1],[.4,0,'out']],[[0,-1],[.08,-1,'hold'],[.2,0,'hit'],[.3,-.3],[.4,0]],[[0,0],[.36,-1,'in']],[[0,0],[.06,0],[.24,-1]]);
const cmV=dir([[0,0],[.08,0],[.32,1,'out']],[[0,0],[.08,0],[.18,1,'out']],[[0,1],[.3,0,'in']],[[0,1],[.06,1],[.22,0,'in']]);
const cmPress=dir([[0,0],[.4,0]],[[0,0],[.08,.8,'hit'],[.24,0]],[[0,0],[.3,0]],[[0,0],[.24,0]]);
const cardPose=rigid((t,s,r)=>({dx:cmX(t,s,r),dy:cmY(t,s,r)}));
const coachmark=toggle('coachmark','Coachmark',
 'Hidden → shown. Light: the callout card fades in while sliding 1 unit down-right to its tail target. Strong: the button is pressed 0.8, then the card springs in with a 0.5 horizontal overshoot and settles. Reverse slides it out.',[
 ...posed([stroke('button',rect(17,21.5,10,5,2.5)),fill('button-solid',rect(17,21.5,10,5,2.5),'solid')],rigid((t,s,r)=>({dy:cmPress(t,s,r),pivot:[22,24]}))),
 ...shown(posed([...framed('card',cardD),stroke('line-0','M8 8.5 L18 8.5'),stroke('line-1','M8 12 L14 12')],cardPose),cmV)
],.46,.46,'hidden','shown',.4,.3);
coachmark.duotoneFill=true;

// ---------- 16. GETTING STARTED (gesture) ----------
// Rocket with rounded fin tips, a window dot and three exhaust trails. Light: a 1.4 hop.
// Strong: it squats 0.6, launches 3.6 with stretched trails, then lands with one soft bounce.
const rocketD='M16 7 C18.4 8.6 20.4 13 20.4 17 L23.4 22 C23.9 22.8 23.7 23 23 23 L9 23 C8.3 23 8.1 22.8 8.6 22 L11.6 17 C11.6 13 13.6 8.6 16 7 Z';
const rocketDy=F([[0,0],[.22,-1.4,'out'],[.6,0,'smooth']],[[0,0],[.1,.6,'launch'],[.28,-3.6],[.4,-3.2],[.56,.3,'hit'],[.66,-.15],[.74,0]]);
const trailLen=F([[0,1.6],[.22,2.4,'out'],[.6,1.6,'smooth']],[[0,1.6],[.1,1],[.28,4,'hit'],[.42,3],[.54,1.4],[.66,1.6]]);
const rocketPose=rigid((t,s)=>({dy:rocketDy(t,s)}));
const trail=(name,x,k)=>posed(tube(name,(t,s)=>`M${x} 26.6 L${x} ${26.6+k*trailLen(t,s)}`),(px,py,t,s)=>[px,py+rocketDy(t,s)]);
const gettingStarted=make('getting-started','Getting started',
 'Light: the rocket hops 1.4 and settles while its exhaust stretches slightly. Strong: it squats 0.6, launches 3.6 with the exhaust stretching to 4 units, hangs, then lands with a single soft bounce.',[
 ...posed([fill('body-interior',rocketD,'tint'),stroke('body',rocketD),fill('body-solid',holes(rocketD,circle(16,14,1.6)),'solid')],rocketPose),
 ...posed(bothFill('window',circle(16,14,1.4)),rocketPose),
 ...trail('trail-c',16,1),...trail('trail-l',12.2,.62),...trail('trail-r',19.8,.62)
],.6,.74);
gettingStarted.duotoneFill=true;

// ---------- 17. TIP OF THE DAY (gesture) ----------
// Calendar leaf (frame, two pegs, header) with a sparkle on the day. Light: a twinkle. Strong: a pop-twist.
const todK=F([[0,1],[.26,.85],[.6,1,'smooth']],[[0,1],[.1,.55],[.28,1.1,'hit'],[.42,.93],[.54,1]]);
const todA=F([[0,0],[.26,15],[.6,0,'smooth']],[[0,0],[.1,-25],[.28,45,'hit'],[.42,-8],[.54,0]]);
const tipOfTheDay=make('tip-of-the-day','Tip of the day',
 'Light: the day\'s sparkle twinkles (0.85×, 15° turn). Strong: it shrinks to 0.55, bursts to 1.1× with a 45° twist and settles inside the calendar leaf.',[
 ...framed('leaf',rect(5,6,22,22,2.7),leafRing),
 ...tube('peg-l','M11 4 L11 8.5'),...tube('peg-r','M21 4 L21 8.5'),...tube('header','M5.5 11.5 L26.5 11.5'),
 ...posed(bothFill('sparkle',sparkle(16,19.8,4.2)),spin(16,19.8,todA,todK))
],.6,.54);
tipOfTheDay.duotoneFill=true;

// ---------- 18. WELCOME GIFT (stateful closed → open) ----------
// Box x7–25 y16.5–27 with a ribbon slot; lid x5.5–26.5 y12–16.5 carrying the ribbon and bow.
// Open: the lid group lifts 4.2 (1.15 painted clearance above the box at Bold).
const boxD=rect(7,16.5,18,10.5,1.5),lidD=rect(5.5,12,21,4.5,1.2);
const lidDy=dir([[0,0],[.06,0],[.52,-4.2,'smooth']],[[0,0],[.1,.5,'launch'],[.26,-4.5],[.4,-4.05],[.5,-4.2]],[[0,-4.2],[.06,-4.2],[.48,0,'smooth']],[[0,-4.2],[.06,-4.6],[.24,0,'hit'],[.32,-.3],[.4,0]]);
const boxDy=dir([[0,0],[.5,0]],[[0,0],[.1,.5,'hit'],[.26,0]],[[0,0],[.48,0]],[[0,0],[.24,0,'hold'],[.3,.5,'hit'],[.42,0]]);
const lidPose=rigid((t,s,r)=>({dy:lidDy(t,s,r),pivot:[16,14.25]})),boxPose=rigid((t,s,r)=>({dy:boxDy(t,s,r),pivot:[16,21.75]}));
const welcomeGift=toggle('welcome-gift','Welcome gift',
 'Closed → open. Light: the lid with its bow lifts 4.2 off the box. Strong: the box is pressed 0.5, the lid launches 4.5 and settles at 4.2. Reverse lowers the lid; Strong slams it with a small box squat.',[
 ...posed([fill('box-interior',boxD,'tint'),stroke('box',boxD),fill('box-solid',holes(boxD,rect(14.9,18,2.2,7.5,1.1)),'solid'),stroke('ribbon','M16 16.5 L16 27')],boxPose),
 ...posed([...framed('lid',lidD),stroke('lid-ribbon','M16 12 L16 16.5'),
  ...tube('bow-l','M16 12 C13.2 12 11.4 10.2 12.4 8.6 C13.3 7.2 15.4 9.2 16 12'),...tube('bow-r','M16 12 C18.8 12 20.6 10.2 19.6 8.6 C18.7 7.2 16.6 9.2 16 12')],lidPose)
],.56,.52,'closed','open',.5,.44);
welcomeGift.duotoneFill=true;

// ---------- 19. FIRST RUN (gesture) ----------
// A start pill (x5–27 y19–26) and a pointer whose tip rests 1.0 above the pill paint. The pointer
// presses the pill: contact at 1.0, then both travel together; Strong lifts first and presses harder.
const pointerD=polygon([[0,0],[0,-9.35],[-2.55,-6.97],[-4.25,-10.54],[-6.12,-9.69],[-4.42,-6.12],[-7.65,-6.12]].map(([x,y])=>[x+18,y+15.5]),[.3,.5,.4,.4,.4,.4,.5]);
const ptrDy=F([[0,0],[.18,1,'out'],[.26,1.5,'out'],[.6,0,'smooth']],[[0,0],[.1,-1.2],[.22,1,'hit'],[.32,2.4,'hit'],[.46,-.3],[.56,0]]);
const runDy=F([[0,0],[.18,0],[.26,.5,'out'],[.6,0,'smooth']],[[0,0],[.22,0,'hold'],[.32,1.4,'hit'],[.44,-.4],[.56,0]]);
const runPose=rigid((t,s)=>({dy:runDy(t,s),pivot:[16,22.5]}));
const firstRun=make('first-run','First run',
 'Light: the pointer moves down 1.0 to touch the start button, presses it 0.5 and both return. Strong: the pointer lifts 1.2, drives down to contact, presses the button 1.4 and both spring back with a small rebound.',[
 ...posed(framed('button',rect(5,19,22,7,3.5)),runPose),
 ...posed([stroke('label','M10 22.5 L22 22.5')],runPose),
 ...posed(bothFill('pointer',pointerD),rigid((t,s)=>({dy:ptrDy(t,s),pivot:[18,15.5]})))
],.6,.56);
firstRun.duotoneFill=true;

// ---------- 20. DISMISS TIP (stateful shown → dismissed) ----------
// A tip card (x3.5–28.5 y5–24) with two lines and a close X collapses into a pill (x9–23 y22–27).
// The details fade before the collapse; the reverse expands the pill and fades the details back in.
const dismissQ=dir([[0,0],[.12,0],[.56,1,'smooth']],[[0,0],[.1,-.03],[.3,1.06,'hit'],[.42,.98],[.5,1]],[[0,1],[.46,0,'smooth']],[[0,1],[.06,1.04],[.26,-.03,'hit'],[.36,.01],[.42,0]]);
const dismissV=dir([[0,1],[.02,1],[.12,0,'in']],[[0,1],[.1,0,'out']],[[0,0],[.46,0],[.6,1,'out']],[[0,0],[.42,0],[.54,1,'out']]);
// Keep the pill's vertical edges nonzero so native forward/reverse morphs share stable corners.
const cardShape=(t,s,r)=>{const q=dismissQ(t,s,r),h=mix(19,5,q);return rect(mix(3.5,9,q),mix(5,22,q),mix(25,14,q),h,Math.min(2.5,h/2-.35));};
const dismissTip=toggle('dismiss-tip','Dismiss tip',
 'Shown → dismissed. Light: the text and close mark fade, then the card folds into a small pill at the bottom. Strong: a short lift, then the card snaps into the pill with a 6% squash and settles. Reverse expands the pill and fades the details back in.',[
 ...framed('card',cardShape),
 ...shown([stroke('line-0','M7.5 13.5 L17.5 13.5'),stroke('line-1','M7.5 18.5 L14 18.5'),stroke('close-a','M21.4 8.4 L25 12'),stroke('close-b','M25 8.4 L21.4 12')],dismissV)
],.58,.52,'shown','dismissed',.62,.56);
dismissTip.duotoneFill=true;

// ---------- set assembly ----------
const ORDER=[sparkleIcon,lightbulb,productTour,tooltip,hotspot,newBadge,confetti,welcomeWave,progressChecklist,skipStep,nextStep,sparkWand,hotkeyHint,spotlight,coachmark,gettingStarted,tipOfTheDay,welcomeGift,firstRun,dismissTip];
const fixed={'product-tour':['stop-a','stop-b','stop-c','route-ab','route-bc'],hotspot:['ring','well'],'progress-checklist':['card-frame','card-interior','card-solid'],spotlight:['page'],'tip-of-the-day':['leaf-frame','peg-l','peg-r','header'],'hotkey-hint':['skirt']};
const rigidParts={'welcome-wave':['palm-frame','finger-0','finger-1','thumb'],'new-badge':['seal'],confetti:['strip-0','strip-2','ribbon-0'],'welcome-gift':['lid-frame','bow-l','box'],'skip-step':['chevron-0','chevron-1','bar'],'first-run':['pointer','button-frame'],'hotkey-hint':['face-frame'],'next-step':['button','stem','barb'],tooltip:['bubble-frame']};
export const icons=ORDER.map((icon,index)=>({...icon,number:806+index,constraints:[...(fixed[icon.name]||[]).map(p=>({type:'fixed',part:icon.name+'-'+p})),...(rigidParts[icon.name]||[]).map(p=>({type:'rigid',part:icon.name+'-'+p}))]}));
