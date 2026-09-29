// Health & Fitness batch B: sleep-tracking, calories, stethoscope, pill, syringe.
import {make,F,P,tube,framed,rigid,posed,shown,fill,stroke,circle,bothFill,rotate,holes,rect} from './helpers.mjs';
import {arcN,lerp} from './sky.mjs';
import svgpath from 'svgpath';
const L=d=>d.replace(/^M/,'L');

// ---------- SLEEP TRACKING ----------
// The family moon (05-interface) placed lower-left; two Z marks rise from it and grow in sequence.
// A Z below 35% scale is hidden: its collapsed strokes painted a dot (native pose sheet).
const MOON='M12.05 6.82 A10 10 0 1 0 25.18 19.95 C25.42 19.41 24.77 19.35 24.43 19.5 A9.5 9.5 0 0 1 12.58 7.53 C12.77 7.16 12.60 6.57 12.05 6.82 Z';
const moon=svgpath(MOON).scale(.62).translate(.8,8.4).toString();
const MC=[.8+16*.62,8.4+16*.62];
const Z1=[[19,13],[23.8,13],[19,17.8],[23.8,17.8]],Z2=[[23.2,5.4],[26.4,5.4],[23.2,8.6],[26.4,8.6]];
const zc=Z=>[(Z[0][0]+Z[3][0])/2,(Z[0][1]+Z[3][1])/2];
const zD=Z=>`M${Z[0]} L${Z[1]} L${Z[2]} L${Z[3]}`;
const grow=d=>F([[0,1],[d,1],[d+.14,.45],[d+.36,1,'smooth']],[[0,1],[d,1],[d+.08,0],[d+.1,0],[d+.26,1.16],[d+.38,1]]);
const zPose=(Z,d)=>(x,y,t,s)=>{const k=grow(d)(t,s),c=zc(Z);return[c[0]+(x-c[0])*k,c[1]+(y-c[1])*k];};
const cradle=F([[0,0],[.6,0]],[[0,0],[.1,-8],[.28,6],[.44,-3],[.58,0]]);
const sleepTracking=make('sleep-tracking','Sleep tracking',
 'Light: the two Z marks shrink and breathe back in sequence beside the moon. Strong: the moon rocks like a cradle while each Z pops out of nothing, overshoots to 116% and settles.',[
 ...posed(framed('moon',moon),rigid((t,s)=>({angle:cradle(t,s),pivot:MC}))),
 ...shown(posed(tube('z-near',zD(Z1)),zPose(Z1,.04)),(t,s)=>grow(.04)(t,s)>.35?1:0),
 ...shown(posed(tube('z-far',zD(Z2)),zPose(Z2,.16)),(t,s)=>grow(.16)(t,s)>.35?1:0)
],.52,.56);
sleepTracking.duotoneFill=true;

// ---------- CALORIES ----------
// A flame with an inner flame (Solid: one surface with the inner flame cut out). Flexible: the tip sways
// more than the base, and the flame flares about its base.
const OUTER='M16 27.6 C10.8 27.6 7.6 24.2 7.6 19.9 C7.6 15.4 11.4 12.6 12.8 8.8 C13.2 7.8 13.9 6.9 14.8 6.5 C15.1 9.4 16.9 11.2 19.1 13.3 C21.9 15.9 24.4 17.9 24.4 20.9 C24.4 24.8 21.1 27.6 16 27.6 Z';
const INNER='M16 24 C14 24 12.8 22.6 12.8 20.9 C12.8 18.9 14.4 17.6 15.5 15.5 C16.6 17.4 19.2 18.7 19.2 20.9 C19.2 22.6 18 24 16 24 Z';
const flare=F([[0,1],[.12,1.05],[.5,1,'smooth']],[[0,1],[.06,.96],[.2,1.12],[.36,1.03],[.5,1.06],[.66,1]]);
const lick=(t,s)=>{const u=Math.max(0,t-.02),T=s?.64:.48;return u>=T?0:(s?2.1:.9)*Math.sin(u*(s?18:13))*(1-u/T)**1.4;};
const flamePose=(x,y,t,s)=>{const k=flare(t,s),h=(27.6-y)/21.1;return[16+(x-16)*(1+(k-1)*.4)+lick(t,s)*h*h,27.6+(y-27.6)*k];};
const calories=make('calories','Calories',
 'Light: the flame flickers, its tip swaying about 1 unit as it swells slightly. Strong: it gutters, flares to 112% with a 2-unit lick at the tip and settles.',
 posed([fill('flame-interior',OUTER,'tint'),stroke('flame',OUTER),stroke('core',INNER),fill('flame-solid',holes(OUTER,INNER),'solid')],flamePose),.5,.66);
calories.duotoneFill=true;

// ---------- STETHOSCOPE ----------
// Binaural U; the tubing hangs from the U's base to a chest piece that swings as a pendulum about that
// base. The tube is rebuilt each frame from the pivot to the chest piece's rim, so it never detaches.
const PIV=[16,17.4],CR=3,RLEN=Math.hypot(6.8,6),PHI0=Math.atan2(6,6.8)*180/Math.PI;
const swing=F([[0,0],[.08,0],[.28,-10],[.56,0,'smooth']],[[0,0],[.08,6],[.26,-18],[.42,7],[.56,-3],[.68,0]]);
const chest=(t,s)=>{const p=(PHI0+swing(t,s))*Math.PI/180;return {c:[PIV[0]+RLEN*Math.cos(p),PIV[1]+RLEN*Math.sin(p)],u:[Math.cos(p),Math.sin(p)]};};
const tubeD=(t,s)=>{const {c,u}=chest(t,s),e=[c[0]-CR*u[0],c[1]-CR*u[1]];return `M${PIV} C${PIV[0]} ${PIV[1]+4.2} ${e[0]-3.4} ${e[1]+2.2} ${e[0]} ${e[1]}`;};
const stethoscope=make('stethoscope','Stethoscope',
 'Light: the chest piece swings 10° on its tubing and returns. Strong: it swings back 6°, then out 18°, and damps through two smaller swings.',[
 ...tube('binaural','M9 4.8 L9 10.2 C9 14.2 12.1 17.4 16 17.4 C19.9 17.4 23 14.2 23 10.2 L23 4.8'),
 ...tube('tubing',tubeD),
 ...posed(framed('chest',circle(PIV[0]+6.8,PIV[1]+6,CR)),rigid((t,s)=>{const {c}=chest(t,s);return {dx:c[0]-PIV[0]-6.8,dy:c[1]-PIV[1]-6};}))
],.56,.68);
stethoscope.duotoneFill=true;

// ---------- PILL ----------
// Capsule at 45°, built horizontally then rotated. The halves share the seam in Outline; Solid halves are
// inset 0.55 from the seam so they stay two pieces. Halves separate along the axis.
const RC=4,HL=7.78;
const halfD=(side,inset=0)=>side<0
 ?`M${16-inset} ${16-RC} L${16-HL} ${16-RC} ${L(arcN(16-HL,16,RC,270,90,2))} L${16-inset} ${16+RC} Z`
 :`M${16+inset} ${16+RC} L${16+HL} ${16+RC} ${L(arcN(16+HL,16,RC,90,-90,2))} L${16+inset} ${16-RC} Z`;
const sep=F([[0,0],[.08,0],[.28,.9],[.54,0,'smooth']],[[0,0],[.06,-.2],[.24,2],[.4,-.15,'hit'],[.52,0]]);
const turn=F([[0,0],[.54,0]],[[0,0],[.06,-4],[.24,10],[.4,-2],[.52,0]]);
const pillPose=side=>(x,y,t,s)=>{const [lx,ly]=[x+side*sep(t,s),y];return rotate(lx,ly,-45+turn(t,s),16,16);};
const pill=make('pill','Pill',
 'Light: the capsule eases open 0.9 units along its axis and closes. Strong: it twists 10° as the halves pull 2 units apart, then snap shut with a small click.',
 [-1,1].flatMap(side=>{const n=side<0?'half-a':'half-b';return posed([fill(n+'-interior',halfD(side),'tint'),stroke(n,halfD(side)),fill(n+'-solid',halfD(side,.55),'solid')],pillPose(side));}),.54,.52);
pill.duotoneFill=true;

// ---------- SYRINGE ----------
// Built horizontally (needle left), rotated −45° so the needle points down-left. The plunger (head, rod,
// thumb rest) moves rigidly; the liquid tint ends at the head. The head seals against the barrel walls.
const push=F([[0,0],[.08,0],[.66,-3,'smooth']],[[0,0],[.06,.35],[.2,.35,'hold'],[.36,-3,'hit'],[.44,-2.85],[.52,-3]]);
const S=(x,y)=>rotate(x,y,-45,16,16);
const sPose=(dx=()=>0)=>(x,y,t,s)=>S(x+dx(t,s),y);
const liquidD=(t,s)=>{const h=19.2+push(t,s);return rect(9.9,13.1,Math.max(.3,h-9.9-.9),5.8,1);};
const dropK=F([[0,0]],[[0,0],[.36,0],[.4,1],[.56,1]]);
const syringe=make('syringe','Syringe',
 'Light: the plunger pushes 3 units in and the dose empties. Strong: it draws back slightly, is driven home in 0.16 s with a hit and a small rebound, and a drop leaves the needle. The dose is retained; preview reset rearms.',[
 ...posed(tube('needle','M4.8 16 L9 16'),sPose()),
 ...posed([stroke('barrel',rect(9,12.2,11.5,7.6,1.4)),fill('barrel-solid',holes(rect(7.7,10.9,14.1,10.2,2.7),rect(10.3,13.5,8.9,5,.4)),'solid')],sPose()),
 ...posed([fill('liquid-interior',liquidD,'tint')],sPose()),
 ...posed(tube('flange','M20.5 10.6 L20.5 21.4'),sPose()),
 ...posed([...tube('head','M19.2 14.9 L19.2 17.1'),...tube('rod','M19.2 16 L27 16'),...tube('thumb','M27 13.2 L27 18.8')],sPose(push)),
 ...shown(bothFill('drop',(t,s)=>circle(6.6,24.8+2.2*dropK(t,s),1)),(t,s)=>s&&t>.36&&t<.56?1-Math.max(0,(t-.46)/.1):0)
],.66,.58,['liquid-interior','head','head-solid','rod','rod-solid','thumb','thumb-solid','drop','drop-solid']);
syringe.duotoneFill=true;

export const icons=[sleepTracking,calories,stethoscope,pill,syringe];
