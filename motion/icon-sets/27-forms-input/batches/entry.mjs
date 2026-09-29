// Forms and input — batch 1, revision 02. Import: ./helpers.mjs. 32-unit grid.
import {make,toggle,geometricRearm,F,tube,stroke,fill,bothFill,rect,circle,capsule,holes,rigid,posed,shown,polygon,clamp} from './helpers.mjs';

const X=(x,y,w,h,r)=>rect(x-1.3,y-1.3,w+2.6,h+2.6,r+1.3);      // Solid exterior = Outline centerline + 1.3
const dir=(f,b)=>(t,s,r)=>(r?b:f)(t,s);
const S=(name,d)=>({name,d,style:'solid'}),T=(name,d)=>({name,d,style:'tint'});
const L=(a,b,k)=>[a[0]+(b[0]-a[0])*k,a[1]+(b[1]-a[1])*k];
const poly=(...p)=>p.flatMap((q,i)=>i?[L(p[i-1],q,1/3),L(p[i-1],q,2/3),q]:[q]);
function trim(pts,u){const n=(pts.length-1)/3;let d='M'+pts[0],e=pts[0];for(let i=0;i<n;i++){const k=Math.min(1,Math.max(0,u*n-i));if(k<=0){d+=' C'+e+' '+e+' '+e;continue;}const [a,b,c,z]=pts.slice(3*i,3*i+4),ab=L(a,b,k),bc=L(b,c,k),cz=L(c,z,k),abc=L(ab,bc,k),bcz=L(bc,cz,k);e=L(abc,bcz,k);d+=' C'+ab+' '+abc+' '+e;}return {d,e};}
const inked=(name,pts,u,solid=true)=>(solid?[2,2.6]:[2]).map((w,i)=>({name:name+(i?'-solid':''),d:(t,s,r)=>trim(pts,u(t,s,r)).d,style:i?'solid':'outline',type:'stroke',width:w,visibility:(t,s,r)=>u(t,s,r)>.012?1:0}));
// Constant-topology stroke envelope for a 2-segment polyline (L-J-R, both dx>0).
// Top rail L->R, bottom rail R->L: positive winding at every pose, so hole() always
// reverses it the same way, including the zero-area pose.
const N=(a,b)=>{const dx=b[0]-a[0],dy=b[1]-a[1],l=Math.hypot(dx,dy)||1;return[-dy/l,dx/l];};
function band(Lp,J,R,h,rr){const n1=N(Lp,J),n2=N(J,R),k=h/(1+n1[0]*n2[0]+n1[1]*n2[1]),m=[(n1[0]+n2[0])*k,(n1[1]+n2[1])*k],o=(p,n,s)=>[p[0]+n[0]*s,p[1]+n[1]*s];return polygon([o(Lp,n1,-h),o(J,m,-1),o(R,n2,-h),o(R,n2,h),o(J,m,1),o(Lp,n1,h)],rr);}

// 1. SELECT FIELD — toggle closed -> open. Fixed: field 4..28 x 4..14.5, value bar.
// Chevron: V (20,8.35)-(22,10.15)-(24,8.35) flips through flat to Λ; a static-extent
// morph, so its swept paint equals its rest paint. Panel 4..28 x 19.5..28 drops 1 unit.
const op=dir(F([[0,0],[.42,1,'smooth']],[[0,0],[.1,-.075],[.32,1.075],[.48,1]]),
             F([[0,1],[.4,0,'smooth']],[[0,1],[.1,1.075],[.32,-.075],[.46,0]]));
const ca=(t,s,r)=>.9*(1-2*op(t,s,r)),CY=9.25;
const vee=a=>[[20,CY-a],[22,CY+a],[24,CY-a]];
const panelPose=rigid((t,s,r)=>({dy:-1*(1-op(t,s,r))}));
const selectField={...toggle('select-field','Select field','The chevron flips over and the option panel drops open below the field; Strong winds the chevron back, whips it over and lands the panel with one short settle.',[
 stroke('field',rect(4,4,24,10.5,2.2)),T('field-tint',rect(4,4,24,10.5,2.2)),stroke('value','M7.7 9.25 L14 9.25'),
 stroke('chevron',(t,s,r)=>{const [a,b,c]=vee(ca(t,s,r));return `M${a} L${b} L${c}`;}),
 S('field-solid',(t,s,r)=>holes(X(4,4,24,10.5,2.2),capsule([7.7,9.25],[14,9.25],2.6),band(...vee(ca(t,s,r)),1.3,[1.3,.9,1.3,1.3,.9,1.3]))),
 ...shown(posed([stroke('panel',rect(4,19.5,24,8.5,2.2)),T('panel-tint',rect(4,19.5,24,8.5,2.2)),stroke('option','M7.7 23.75 L19 23.75'),S('panel-solid',holes(X(4,19.5,24,8.5,2.2),capsule([7.7,23.75],[19,23.75],2.6)))],panelPose),(t,s,r)=>clamp(op(t,s,r)))
],.45,.5,'closed','open'),duotoneFill:true};

// 2. MULTIPLE CHOICE — toggle one-selected -> two-selected. Boxes 10x10 at (3.5,3.5) and (3.5,17.5); labels 17.5..28.
// Solid = static ring (expanded box, open well 4.8..12.2) + separately revealed 2.6 check stroke, as in the accepted Checkbox.
// Check grows from its joint outward; constant M L L topology at every pose, zero opacity while absent. No counter morph.
const CJ=[8.3,9.3],CL=[7.1,8.3],CR=[9.9,7.2],dn=p=>[p[0],p[1]+14];
const checkD=(J,Lp,R,u)=>`M${L(J,Lp,u)} L${J} L${L(J,R,u)}`;
const ring=(x,y)=>holes(X(x,y,10,10,2.4),rect(x+1.3,y+1.3,7.4,7.4,1.1));
const sel=dir(F([[0,0],[.38,1,'smooth']],[[0,0],[.08,0,'hold'],[.2,1]]),
              F([[0,1],[.36,0,'smooth']],[[0,1],[.06,1,'hold'],[.2,0]]));
const press=F([[0,0]],[[0,0],[.16,1.1,'hit'],[.3,-.3],[.44,0]]);
const multipleChoice={...toggle('multiple-choice','Multiple choice','A second option is selected: its check appears while the first stays checked; Strong presses the box down, stamps the check in at contact, and the box releases with one rebound.',[
 stroke('box-1',rect(3.5,3.5,10,10,2.4)),T('box-1-tint',rect(3.5,3.5,10,10,2.4)),S('box-1-solid',ring(3.5,3.5)),
 ...tube('check-1',checkD(CJ,CL,CR,1)),
 ...tube('label-1','M17.5 8.5 L28 8.5'),...tube('label-2','M17.5 22.5 L28 22.5'),
 ...posed([stroke('box-2',rect(3.5,17.5,10,10,2.4)),T('box-2-tint',rect(3.5,17.5,10,10,2.4)),S('box-2-solid',ring(3.5,17.5)),
  ...shown(tube('check-2',(t,s,r)=>checkD(dn(CJ),dn(CL),dn(CR),clamp(sel(t,s,r)))),(t,s,r)=>sel(t,s,r)>.012?1:0)],
  rigid((t,s)=>({dy:press(t,s)})))
],.4,.46,'one selected','two selected'),duotoneFill:true};

// 3. SIGNATURE — unchanged from entry-01.
const SG=[[7,21],[8,13],[12,12],[12.5,18],[13,23],[16,21],[17.5,15],[18.5,11],[20,12],[23,19]];
const inkU=F([[0,0],[.18,0],[.85,1,'smooth']],[[0,0],[.1,0,'hold'],[.48,1]]);
const lift=F([[0,-2.5],[.18,0,'smooth'],[.85,0],[1.05,-2.5,'smooth']],[[0,-2.5],[.09,0,'hit'],[.48,0,'hold'],[.6,-3.3],[.72,-2.5]]);
const signature={...geometricRearm(make('signature','Signature','The pen lowers, draws the signature stroke, and lifts; Strong taps down, drives the stroke fast, and flicks up before settling.',[
 ...tube('baseline','M6 26 L26 26'),
 ...inked('ink',SG,(t,s)=>inkU(t,s)),
 ...posed(tube('pen','M.9 -.9 L5 -7'),rigid((t,s)=>{const e=trim(SG,clamp(inkU(t,s))).e;return {dx:e[0],dy:e[1]+lift(t,s)};}))
],1.1,.8)),duotoneFill:false};

// 4. FORM BUILDER — retained output. Fixed sheet (tint), text field, radio row. Inserted bar 9 -> 22 (Strong peak 23.2, paint 24.5 vs sheet inner 25.7).
const grow=F([[0,0],[.45,13,'smooth']],[[0,0],[.06,.4],[.16,.4,'hold'],[.3,14.2],[.44,13]]);
const gl=(t,s)=>Math.max(.2,grow(t,s));
const formBuilder={...geometricRearm(make('form-builder','Form builder','A new field is inserted below the existing text field and radio row; Strong shows the cursor, holds, then shoots the field out with one overshoot.',[
 stroke('sheet',rect(5,3.5,22,25,2.5)),T('sheet-tint',rect(5,3.5,22,25,2.5)),
 stroke('text-field',rect(9,7.5,14,5,1.2)),
 fill('radio',circle(11,19,1.4),'outline'),stroke('radio-label','M16 19 L23 19'),
 stroke('added',(t,s)=>`M9 24.5 L${9+gl(t,s)} 24.5`),
 S('sheet-solid',(t,s)=>holes(X(5,3.5,22,25,2.5),rect(9,7.5,14,5,1.2),circle(11,19,1.5),capsule([16,19],[23,19],2.6),capsule([9,24.5],[9+gl(t,s),24.5],2.6)))
],.5,.5)),duotoneFill:true};

// 5. QR CODE — retained output. Fixed finders (4,4),(19,4),(4,19) and two static modules in the left corridor.
// Two modules drop in their own clear columns (x 20..23 and 25..28); nothing else paints in those columns.
const FD=[[4,4],[19,4],[4,19]],ST=[[15.5,19.5],[15.5,24.5]],MD=[{x:20,y:24.5,l:3.5,s:8},{x:25,y:20,l:2,s:4}];
const land=i=>F([[0,0],[.05+i*.15,0],[.4+i*.15,1,'smooth']],[[0,0],[i*.1,0,'hold'],[.18+i*.1,1.08,'hit'],[.3+i*.1,1]]);
const qrCode={...geometricRearm(make('qr-code','QR code','Data modules drop into the code between the fixed finder squares; Strong drops them from higher and each lands with one rebound.',[
 ...FD.flatMap(([x,y],i)=>[stroke(`finder-${i}`,rect(x,y,9,9,1.5)),T(`finder-${i}-tint`,rect(x,y,9,9,1.5)),...bothFill(`finder-${i}-dot`,rect(x+3,y+3,3,3,.8)),
  S(`finder-${i}-solid`,holes(X(x,y,9,9,1.5),rect(x+1.3,y+1.3,6.4,6.4,.6)))]),
 ...ST.flatMap(([x,y],i)=>bothFill(`static-${i}`,rect(x,y,3,3,.8))),
 ...MD.flatMap(({x,y,l,s:sd},i)=>{const p=land(i);return shown(posed(bothFill(`module-${i}`,rect(x,y,3,3,.8)),rigid((t,s)=>({dy:-(s?sd:l)*(1-p(t,s))}))),(t,s)=>clamp((p(t,s)>0?1:0)+(s?t-i*.1:t-.05-i*.15)*16));})
],.7,.5)),duotoneFill:true};

// 6. BARCODE — unchanged from entry-01.
const sy=F([[0,8],[.6,24,'smooth'],[.64,8]],[[0,8],[.08,6.6],[.28,24.4,'hit'],[.5,8]]);
const sv=F([[0,0],[.1,1],[.5,1],[.6,0]],[[0,0],[.05,1],[.44,1],[.5,0]]);
const barcode={...make('barcode','Barcode','A scanner line passes down across the bars and fades; Strong lifts, drives to the bottom, and sweeps back up before it clears.',[
 ...[5.75,9.75,13.75,23.25,27.25].flatMap((x,i)=>tube(`bar-${i}`,`M${x} 6 L${x} 26`)),
 ...bothFill('bar-wide',rect(16.75,5,3,22,1.5)),
 ...shown(posed(tube('laser','M4 0 L28 0'),rigid((t,s)=>({dy:sy(t,s)}))),(t,s)=>sv(t,s))
],.65,.55),duotoneFill:false};

export const icons=[selectField,multipleChoice,signature,formBuilder,qrCode,barcode];
