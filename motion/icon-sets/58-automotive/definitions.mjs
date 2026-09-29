// Automotive (58). Designs, reviews and corrections: evidence/design-review.md.
// Every drawing is authored here on the 32-unit grid with round caps and joins.
// Outline parts are the Regular centreline (the builder derives Light/Bold);
// Solid parts are separate painted silhouettes; tint parts are real enclosed surfaces.
import {make,F,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,rotate,holes,capsule,bothFill,arc,mix,drive,track,timePair} from '../library/ai-authoring.mjs';

// ---------- shared assembly ----------
// A true two-state icon: the reverse action is directed, not a rewind.
const toggle=(name,label,intent,parts,light,strong,initial,final,rl=light,rs=strong)=>({...make(name,label,intent,parts,light,strong),stateful:true,initialState:initial,finalState:final,reverseSuffix:'-reverse',forwardLabel:final,reverseLabel:initial,reverseTimings:timePair(rl,rs)});
// Forward/reverse driver pairs. Light uses eased tracks; Strong uses the bounded cubic flow.
const D=(lf,sf,lr,sr)=>(t,s,r)=>drive(t,s,r?lr:lf,r?sr:sf);
const deg=a=>a*Math.PI/180;
const P2=(cx,cy,r,a)=>[cx+r*Math.cos(deg(a)),cy+r*Math.sin(deg(a))];
// Open rounded polyline: interior corners are rounded, the two ends stay free.
function openPoly(v,r){const n=v.length;let d='M'+v[0];for(let i=1;i<n-1;i++){const p=v[i],a=v[i-1],b=v[i+1];const step=q=>{const dd=Math.hypot(q[0]-p[0],q[1]-p[1]);const k=Math.min(r,dd*.35)/dd;return[p[0]+(q[0]-p[0])*k,p[1]+(q[1]-p[1])*k];};const s=step(a),e=step(b);d+=` L${s} C${mix(s[0],p[0],2/3)} ${mix(s[1],p[1],2/3)} ${mix(e[0],p[0],2/3)} ${mix(e[1],p[1],2/3)} ${e}`;}return d+' L'+v[n-1];}
// Hermite → cubic for a sampled curve fn(u) on [0,1]; always n segments so a partial draw keeps its topology.
function curveFrom(fn,n=4){const h=1e-4,dv=u=>{const a=fn(Math.max(0,u-h)),b=fn(Math.min(1,u+h));const w=Math.min(1,u+h)-Math.max(0,u-h);return[(b[0]-a[0])/w,(b[1]-a[1])/w];};let d='M'+fn(0);for(let i=0;i<n;i++){const u0=i/n,u1=(i+1)/n,p0=fn(u0),p1=fn(u1),d0=dv(u0),d1=dv(u1),k=(u1-u0)/3;d+=' C'+[p0[0]+d0[0]*k,p0[1]+d0[1]*k,p1[0]-d1[0]*k,p1[1]-d1[1]*k,...p1];}return d;}
// A straight strap drawn from a to b by fraction p (round-cap dot hidden by the caller below 3%).
const drawn=(a,b)=>p=>`M${a} L${a[0]+(b[0]-a[0])*Math.max(p,.001)} ${a[1]+(b[1]-a[1])*Math.max(p,.001)}`;
const cross=(cx,cy,l,w)=>polygon([[cx-w,cy-l],[cx+w,cy-l],[cx+w,cy-w],[cx+l,cy-w],[cx+l,cy+w],[cx+w,cy+w],[cx+w,cy+l],[cx-w,cy+l],[cx-w,cy+w],[cx-l,cy+w],[cx-l,cy-w],[cx-w,cy-w]],w*.9);
// A wheel: ring in Outline, filled disc in Solid (the ring would show the body edge through its hub).
const wheel=(name,x,y,r)=>[stroke(name,circle(x,y,r)),fill(name+'-solid',circle(x,y,r+1.3),'solid')];

// Side-view car facing right: hood right, tail left. Wheel tops meet the body's bottom edge.
const carD=polygon([[5.5,18.4],[5.5,15],[7.6,14.6],[10.8,9.6],[19,9.6],[23,14.6],[26.5,15],[26.5,18.4]],1.6);
const W=[[10,21.2],[22,21.2]],WR=3;

// ---------- CAR ----------
// Light: the car rolls forward 1 unit with a hint of nose lift and eases back.
// Strong: it rocks back, lunges 1.9 units with a 5° nose lift about the rear wheel and settles.
const surge=F([[0,0],[.06,0],[.42,1,'out'],[.82,0,'smooth']],[[0,0],[.1,-.5],[.34,1.9],[.48,1.6],[.78,0]]);
const lift=F([[0,0],[.06,0],[.36,-1.4,'out'],[.82,0,'smooth']],[[0,0],[.1,1.2],[.32,-5],[.48,1.4],[.6,-.5],[.74,0]]);
const carPose=rigid((t,s)=>({dx:surge(t,s),angle:lift(t,s),pivot:W[0]}));
const car=make('car','Car',
 'Light: the car rolls forward 1 unit with a hint of nose lift and eases back. Strong: it rocks back, lunges 1.9 units with a 5° nose lift about the rear wheel and settles.',
 posed([...framed('body',carD),...wheel('wheel-rear',...W[0],WR),...wheel('wheel-front',...W[1],WR)],carPose),.82,.78);
car.duotoneFill=true;
car.constraints=[{type:'rigid',part:'car-body-frame'},{type:'rigid',part:'car-wheel-rear'},{type:'rigid',part:'car-wheel-front'}];

// ---------- ELECTRIC CAR ----------
// A bolt on the door is a filled mark in Outline and a real counter in Solid.
// Light: the bolt swells 15% and eases back. Strong: it pops to 120% with a snap and the car dips on its suspension.
const BC=[15.3,14];
const boltV=[[16.6,11],[12.9,16],[15.1,16],[14,19.4],[17.9,14.3],[15.6,14.3]].map(([x,y])=>[BC[0]+(x-15.2)*.52,BC[1]+(y-15.2)*.52]);
const boltAt=k=>polygon(boltV.map(([x,y])=>[BC[0]+(x-BC[0])*k,BC[1]+(y-BC[1])*k]),.4);
const swell=F([[0,1],[.06,1],[.26,1.15,'out'],[.62,1,'smooth']],[[0,1],[.06,.92],[.22,1.2,'hit'],[.36,1.03],[.48,1.08],[.62,1]]);
const dip=F([[0,0],[.62,0]],[[0,0],[.1,0],[.24,.7,'hit'],[.42,-.2],[.56,0]]);
const evPose=rigid((t,s)=>({dy:dip(t,s)}));
const electricCar=make('electric-car','Electric car',
 'Light: the lightning bolt on the door swells 15% and eases back. Strong: it pops to 120% with a snap while the car dips 0.7 units on its suspension and settles.',[
 ...posed([fill('body-interior',carD,'tint'),stroke('body-frame',carD),fill('body-solid',(t,s)=>holes(carD,boltAt(swell(t,s))),'solid'),
  fill('bolt',(t,s)=>boltAt(swell(t,s)),'outline'),
  ...wheel('wheel-rear',...W[0],WR),...wheel('wheel-front',...W[1],WR)],evPose)
],.62,.62);
electricCar.duotoneFill=true;
electricCar.constraints=[{type:'rigid',part:'electric-car-body-frame'},{type:'rigid',part:'electric-car-wheel-rear'}];

// ---------- FUEL PUMP ----------
// Pump body with a display window; the hose leaves the top right and hangs to a hooked nozzle.
// Light: the nozzle lifts 1.2 units off its hook and drops back. Strong: it is pulled up 2.6 units, tips 14° outward and drops back onto the hook.
const pumpD=rect(6.5,5.5,12.5,21.5,2.2),windowD=rect(9,8.5,7.5,5.5,1.2);
const NZ=[25.5,19];
const liftN=F([[0,0],[.06,0],[.34,-1.2,'out'],[.7,0,'smooth']],[[0,0],[.08,.3],[.3,-2.6],[.44,-2.1],[.66,0,'hit'],[.74,0]]);
const tipN=F([[0,0],[.7,0]],[[0,0],[.1,0],[.34,-14],[.5,-9],[.66,0]]);
const nozzlePose=rigid((t,s)=>({dy:liftN(t,s),angle:tipN(t,s),pivot:NZ}));
const fuelPump=make('fuel-pump','Fuel pump',
 'Light: the nozzle lifts 1.2 units off its hook and drops back. Strong: it is pulled up 2.6 units, tips 14° outward and drops back onto the hook.',[
 fill('body-interior',pumpD,'tint'),stroke('body-frame',pumpD),fill('body-solid',holes(pumpD,windowD),'solid'),stroke('window',windowD),
 ...tube('hose',(t,s)=>`M19 9.5 C23 9.5 25.5 10.8 25.5 14.5 L25.5 ${NZ[1]+liftN(t,s)}`),
 ...posed(tube('nozzle',`M25.5 19 L25.5 23.6 L23 23.6`),nozzlePose)
],.7,.74);
fuelPump.duotoneFill=true;
fuelPump.constraints=[{type:'fixed',part:'fuel-pump-body-frame'},{type:'fixed',part:'fuel-pump-window'},{type:'rigid',part:'fuel-pump-nozzle'}];

// ---------- SPEEDOMETER ----------
// Half dial with three ticks; the needle pivots on a filled hub.
// Light: the needle sweeps 70° up the dial and eases back. Strong: it snaps 145° to the red line, springs back through zero and settles.
const SC=[16,20],SR=10.5;
const sweep=F([[0,0],[.06,0],[.4,70,'out'],[.9,0,'smooth']],[[0,0],[.08,-4],[.3,145],[.42,139],[.52,145],[.86,0],[.96,-3],[1.06,0]]);
const needlePose=rigid((t,s)=>({angle:sweep(t,s),pivot:SC}));
const speedometer=make('speedometer','Speedometer',
 'Light: the needle sweeps 70° up the dial and eases back. Strong: it snaps 145° to the red line, shivers, falls back through zero and settles.',[
 ...tube('dial',arc(...SC,SR,180,360)),
 ...[210,270,330].flatMap((a,i)=>tube('tick-'+i,`M${P2(...SC,SR,a)} L${P2(...SC,SR-1.6,a)}`)),
 ...posed(tube('needle',`M${SC} L${P2(...SC,5.2,190)}`),needlePose),
 ...bothFill('hub',circle(...SC,1.6))
],.9,1.06);
speedometer.duotoneFill=false;
speedometer.constraints=[{type:'fixed',part:'speedometer-dial'},{type:'fixed',part:'speedometer-hub'},{type:'rigid',part:'speedometer-needle'}];

// ---------- PARKING BRAKE ----------
// The parking-brake tell-tale: a P in a ring with two side brackets. Off: the brackets are dark.
// On: each bracket grows from its midpoint to a 60° arc; Strong snaps them out past full and the lamp bumps.
const PR=8,BR=12.1,PC=[16,16];
const pD='M13.8 19.8 L13.8 12.2 L16 12.2 C17.5 12.2 18.2 13 18.2 14.1 C18.2 15.2 17.5 16 16 16 L13.8 16';
const bracket=D([[0,0],[.06,0],[.5,30,'smooth']],[[0,0],[.06,0,'hold'],[.24,34],[.36,29],[.46,30]],[[0,30],[.06,30],[.44,0,'smooth']],[[0,30],[.06,31],[.28,0,'in'],[.32,0]]);
const lampBump=F([[0,1],[.5,1]],[[0,1],[.06,1],[.2,1.07,'hit'],[.34,.99],[.44,1]]);
const lampPose=(x,y,t,s,r)=>{const k=r?1:lampBump(t,s);return[PC[0]+(x-PC[0])*k,PC[1]+(y-PC[1])*k];};
const parkingBrake=toggle('parking-brake','Parking brake',
 'Forward: the two brackets of the tell-tale grow out from their midpoints to 60° arcs; Strong snaps them past full while the lamp bumps 7%. Reverse: the brackets shrink back and go dark.',[
 ...posed([fill('ring-interior',circle(...PC,PR),'tint'),...tube('ring',circle(...PC,PR)),...tube('p',pD)],lampPose),
 ...[180,0].flatMap((a,i)=>shown(tube('bracket-'+i,(t,s,r)=>{const w=Math.max(bracket(t,s,r),.05);return arc(...PC,BR,a-w,a+w);}),(t,s,r)=>bracket(t,s,r)>.8?1:0))
],.5,.46,'off','on',.44,.32);
parkingBrake.duotoneFill=true;
parkingBrake.constraints=[{type:'fixed',part:'parking-brake-ring',profile:'light'},{type:'fixed',part:'parking-brake-p',profile:'light'}];

// ---------- TIRE PRESSURE ----------
// The tyre cross-section warning: an open U with a flat tread and an exclamation mark inside.
// Light: the tyre sags 8% and re-inflates. Strong: it slumps 18% with bulging walls, the mark bounces and it re-inflates.
const tireD='M9 6.5 L9 18.5 C9 23.5 12 27 16 27 C20 27 23 23.5 23 18.5 L23 6.5';
const sag=F([[0,1],[.06,1],[.32,.92,'out'],[.74,1,'smooth']],[[0,1],[.06,1.02],[.26,.82,'hit'],[.44,.95],[.56,.9],[.7,1]]);
const tirePose=(x,y,t,s)=>{const k=sag(t,s),w=Math.max(0,(y-6.5)/20.5);return[16+(x-16)*(1+(1-k)*.9*w),27-(27-y)*k];};
const markY=F([[0,0],[.74,0]],[[0,0],[.06,0],[.26,1.6],[.4,-1.1],[.52,.3],[.64,0]]);
const markPose=rigid((t,s)=>({dy:markY(t,s)}));
const tirePressure=make('tire-pressure','Tire pressure',
 'Light: the tyre sags 8% and re-inflates. Strong: it slumps 18% with bulging side walls, the exclamation mark bounces on the tread and the tyre re-inflates.',[
 ...posed(tube('tire',tireD),tirePose),
 ...posed([...tube('mark','M16 11 L16 17.8'),...bothFill('dot',circle(16,21.8,1.35))],markPose)
],.74,.7);
tirePressure.duotoneFill=false;
tirePressure.constraints=[{type:'rigid',part:'tire-pressure-mark'},{type:'rigid',part:'tire-pressure-dot'}];

// ---------- ENGINE ----------
// The stepped engine-block glyph. Light: it idles with a 0.3-unit shiver that dies away.
// Strong: it revs, shaking 0.7 units and rocking 2.5° before settling.
const engineD=polygon([[8,23],[8,20],[5.5,20],[5.5,15.5],[8,15.5],[8,12.5],[12.5,12.5],[12.5,9.5],[19.5,9.5],[19.5,12.5],[23.5,12.5],[23.5,15.5],[26.5,15.5],[26.5,20],[23.5,20],[23.5,23]],.7);
const shake=F([[0,0],[.08,.3],[.5,.3],[.72,0,'smooth']],[[0,0],[.06,.7],[.4,.7],[.74,0]]);
const rev=F([[0,0],[.72,0]],[[0,0],[.08,1.5],[.3,-2.5],[.5,.8],[.66,0]]);
const enginePose=rigid((t,s)=>{const a=shake(t,s),w=s?14:9;return {dx:a*Math.sin(t*w*Math.PI*2),dy:a*.5*Math.sin(t*w*Math.PI*2*1.4+1),angle:rev(t,s),pivot:[16,17]};});
const engine=make('engine','Engine',
 'Light: the engine idles with a 0.3-unit shiver that dies away. Strong: it revs, shaking 0.7 units and rocking 2.5° before it settles.',
 posed(framed('block',engineD),enginePose),.72,.74);
engine.duotoneFill=true;
engine.constraints=[{type:'rigid',part:'engine-block-frame'}];

// ---------- SEATBELT ----------
// A seated figure; the belt is drawn from the shoulder across to the hip. Solid keeps the torso as a ring so the belt reads inside it.
const SB=[9.3,13.6],SE=[20.4,24.6],torso=rect(9.5,12.5,13,15,3.5),torsoRing=holes(torso,rect(11.4,14.4,9.2,11.2,2));
const buckle=capsule([19.3,25.7],[21.5,23.5],2.8);
const beltP=D([[0,0],[.06,0],[.56,1,'smooth']],[[0,0],[.06,0,'hold'],[.3,1.06],[.4,1]],[[0,1],[.06,1],[.5,0,'smooth']],[[0,1],[.06,1.03],[.3,0,'in'],[.34,0]]);
const click=F([[0,0],[.56,0]],[[0,0],[.3,0],[.38,.45,'hit'],[.5,0]]);
const beltD=drawn(SB,SE);
const seatbelt=toggle('seatbelt','Seatbelt',
 'Forward: the belt is pulled from the shoulder across to the hip; Strong pulls it taut past the hip and clicks home while the figure settles. Reverse: the belt retracts to the shoulder.',[
 ...posed([...bothFill('head',circle(16,7.6,2.6)),fill('torso-interior',torso,'tint'),stroke('torso-frame',torso),fill('torso-solid',torsoRing,'solid')],rigid((t,s,r)=>({dy:r?0:click(t,s)}))),
 ...shown(tube('belt',(t,s,r)=>beltD(beltP(t,s,r))),(t,s,r)=>beltP(t,s,r)>.03?1:0),
 ...shown(bothFill('buckle',buckle),(t,s,r)=>beltP(t,s,r)>.96?1:0)
],.56,.5,'unfastened','fastened',.5,.34);
seatbelt.duotoneFill=true;
seatbelt.constraints=[{type:'rigid',part:'seatbelt-torso-frame'},{type:'rigid',part:'seatbelt-head'}];

// ---------- HEADLIGHTS ----------
// A lamp (D shape) throws three beams to the left. Off: no beams. On: beams extend out from the lamp face.
const lampD='M18.5 9 L19 9 C23.4 9 26 12.2 26 16 C26 19.8 23.4 23 19 23 L18.5 23 C17.7 23 17 22.3 17 21.5 L17 10.5 C17 9.7 17.7 9 18.5 9 Z';
const BEAMS=[[[13.5,12.5],[6.5,10.5]],[[13.5,16],[6.5,16]],[[13.5,19.5],[6.5,21.5]]];
const beam=i=>D([[0,0],[.06+i*.08,0],[.5+i*.08,1,'smooth']],[[0,0],[.04+i*.05,0,'hold'],[.22+i*.05,1.1],[.34+i*.05,1]],[[0,1],[.04+i*.06,1],[.4+i*.06,0,'smooth']],[[0,1],[.03+i*.04,1,'hold'],[.2+i*.04,0,'in'],[.26+i*.04,0]]);
const flare=F([[0,0],[.5,0]],[[0,0],[.04,0],[.16,.5],[.34,0]]);
const headlights=toggle('headlights','Headlights',
 'Forward: three beams extend out of the lamp in sequence; Strong flashes them out past full length and the lamp recoils. Reverse: the beams draw back into the lamp.',[
 ...posed(framed('lamp',lampD),rigid((t,s,r)=>({dx:r?0:flare(t,s)}))),
 ...BEAMS.flatMap(([a,b],i)=>{const p=beam(i),d=drawn(a,b);return shown(tube('beam-'+i,(t,s,r)=>d(p(t,s,r))),(t,s,r)=>p(t,s,r)>.03?1:0);})
],.66,.5,'off','on',.52,.34);
headlights.duotoneFill=true;
headlights.constraints=[{type:'rigid',part:'headlights-lamp-frame'}];

// ---------- WIPER ----------
// A windscreen (Solid ring) with a wiper arm pivoting from the bottom centre.
// Light: one 70° wipe up and back. Strong: a fast 90° wipe with a hard turn and a snap back.
const shieldD=polygon([[6,7.5],[26,7.5],[27.5,25],[4.5,25]],2),shieldRing=holes(shieldD,polygon([[8.5,9.8],[23.5,9.8],[24.6,23.3],[7.4,23.3]],1));
const wipe=F([[0,0],[.06,0],[.42,-70,'smooth'],[.9,0,'smooth']],[[0,0],[.06,0,'hold'],[.28,-90,'hit'],[.52,0,'hit'],[.6,-4],[.68,0]]);
const wiperPose=rigid((t,s)=>({angle:wipe(t,s),pivot:[16,25]}));
const wiper=make('wiper','Wiper',
 'Light: the wiper arm sweeps 70° across the windscreen and returns. Strong: it whips 90° across with a hard turn at each end and settles at its park position.',[
 fill('shield-interior',shieldD,'tint'),stroke('shield-frame',shieldD),fill('shield-solid',shieldRing,'solid'),
 ...posed(tube('arm',`M16 25 L${P2(16,25,8.5,-45)}`),wiperPose),
 ...bothFill('pivot',circle(16,25,1.5))
],.9,.72);
wiper.duotoneFill=true;
wiper.constraints=[{type:'fixed',part:'wiper-shield-frame'},{type:'fixed',part:'wiper-pivot'},{type:'rigid',part:'wiper-arm'}];

// ---------- STEERING WHEEL ----------
// Rim, three spokes and a filled hub turn as one rigid piece. Light: a 25° turn and back. Strong: a short counter-steer, an 85° turn and a settle back to centre.
const turn=F([[0,0],[.06,0],[.4,25,'out'],[.86,0,'smooth']],[[0,0],[.1,-10],[.36,85],[.5,80],[.84,0],[.92,3],[1,0]]);
const wheelPose=rigid((t,s)=>({angle:turn(t,s),pivot:[16,16]}));
const steeringWheel=make('steering-wheel','Steering wheel',
 'Light: the wheel turns 25° and eases back to centre. Strong: a short counter-steer, an 85° turn and a settle back to centre.',[
 fill('rim-interior',circle(16,16,10.5),'tint'),
 ...posed([...tube('rim',circle(16,16,10.5)),...[180,0,90].flatMap((a,i)=>tube('spoke-'+i,`M${P2(16,16,2,a)} L${P2(16,16,10.5,a)}`)),...bothFill('hub',circle(16,16,2.9))],wheelPose)
],.86,1);
steeringWheel.duotoneFill=true;
steeringWheel.constraints=[{type:'rigid',part:'steering-wheel-rim'},{type:'rigid',part:'steering-wheel-spoke-0'},{type:'rigid',part:'steering-wheel-hub'}];

// ---------- CAR KEY ----------
// A remote key: fob head with a button, a blade with two teeth, and two signal arcs on the right.
// Light: the arcs blink in sequence. Strong: the key bumps and both arcs radiate outward, fade and reform.
const headD=rect(9,5,10,12,3.5),btn=circle(14,10.5,1.6);
const blink=i=>F([[0,1],[.08+i*.14,1],[.2+i*.14,.2,'out'],[.42+i*.14,1,'smooth']],[[0,1],[.08,1],[.3,1],[.42,0,'in'],[.46,0],[.62,1]]);
const radiate=i=>F([[0,0],[.6,0]],[[0,0],[.08,0],[.42,1,'linear'],[.44,1],[.45,0,'linear']]);
const bump=F([[0,0],[.6,0]],[[0,0],[.04,0],[.12,-.7],[.24,0,'hit'],[.34,0]]);
const carKey=make('car-key','Car key',
 'Light: the two signal arcs blink in sequence. Strong: the key bumps as the button is pressed and both arcs radiate a unit outward, fade and reform.',[
 ...posed([fill('head-interior',headD,'tint'),stroke('head-frame',headD),fill('head-solid',holes(headD,btn),'solid'),fill('button',btn,'outline'),
  ...tube('blade','M14 17 L14 27'),...tube('tooth-0','M14 22 L17.2 22'),...tube('tooth-1','M14 25.6 L16.6 25.6')],rigid((t,s)=>({dy:bump(t,s)}))),
 ...[5,8.5].flatMap((r,i)=>{const o=blink(i),g=radiate(i);return shown(tube('signal-'+i,(t,s)=>arc(19,11,r+g(t,s),-38,38)),o);})
],.72,.76);
carKey.duotoneFill=true;
carKey.constraints=[{type:'rigid',part:'car-key-head-frame'},{type:'rigid',part:'car-key-blade'}];

// ---------- CAR LOCK ----------
// A lower side-view car with a padlock badge over its tail. Unlocked: the shackle is raised and swung open. Locked: it drops in.
const lockCarD=polygon([[5,22.2],[5,18.8],[7.2,18.4],[10.5,14],[19.2,14],[22.6,18.4],[27,18.8],[27,22.2]],1.6);
const LW=[[9.8,25],[22.2,25]],LWR=2.8;
const padD=rect(21.2,8,6.6,5,1.5),SHK=[26.7,9.6];
const shackleD='M22.3 11 L22.3 8 '+arc(24.5,8,2.2,180,360).replace(/^M[^C]*/,'')+' L26.7 11';
const swingS=D([[0,28],[.06,28],[.5,0,'smooth']],[[0,28],[.06,30],[.26,0,'hit'],[.34,0]],[[0,0],[.06,0],[.5,28,'smooth']],[[0,0],[.06,-2],[.26,31],[.38,27],[.46,28]]);
const liftS=D([[0,-1.3],[.06,-1.3],[.5,0,'smooth']],[[0,-1.3],[.06,-1.5],[.26,0,'hit'],[.34,0]],[[0,0],[.06,0],[.5,-1.3,'smooth']],[[0,0],[.06,.2],[.26,-1.6],[.38,-1.2],[.46,-1.3]]);
const lockDip=F([[0,0],[.5,0]],[[0,0],[.24,0],[.36,.5,'hit'],[.5,-.15],[.6,0]]);
const carLock=toggle('car-lock','Car lock',
 'Forward: the raised shackle swings 28° closed and drops into the padlock; Strong snaps it shut and the car dips on its suspension. Reverse: the shackle lifts and swings open.',[
 ...posed([...framed('body',lockCarD),...wheel('wheel-rear',...LW[0],LWR),...wheel('wheel-front',...LW[1],LWR)],rigid((t,s,r)=>({dy:r?0:lockDip(t,s)}))),
 ...bothFill('pad',padD),
 ...posed(tube('shackle',shackleD),rigid((t,s,r)=>({angle:swingS(t,s,r),dy:liftS(t,s,r),pivot:SHK})))
],.5,.6,'unlocked','locked',.5,.46);
carLock.duotoneFill=true;
carLock.constraints=[{type:'fixed',part:'car-lock-pad'},{type:'rigid',part:'car-lock-shackle'},{type:'rigid',part:'car-lock-body-frame'}];

// ---------- CAR BATTERY ----------
// A 12 V battery: box, two filled terminal caps, + and − marks (real counters in Solid).
// Light: the + mark pulses. Strong: the battery jolts up, rocks 2° and the marks pulse harder.
const battD=rect(5.5,12,21,14,2.2),caps=[rect(8.5,9.5,4,3.2,1),rect(19.5,9.5,4,3.2,1)];
const plusAt=k=>cross(11,19,2.2*k,1.05*k),minusAt=k=>capsule([18.4,19],[22.4,19],2.1*k);
const pulseB=F([[0,1],[.06,1],[.26,1.2,'out'],[.6,1,'smooth']],[[0,1],[.06,1],[.2,1.35,'hit'],[.36,.95],[.5,1]]);
const jolt=F([[0,0],[.6,0]],[[0,0],[.06,0],[.14,-1],[.28,0,'hit'],[.36,-.25],[.46,0]]);
const rock=F([[0,0],[.6,0]],[[0,0],[.06,0],[.16,-2],[.3,1.4],[.42,-.5],[.52,0]]);
const battPose=rigid((t,s)=>({dy:jolt(t,s),angle:rock(t,s),pivot:[16,19]}));
const carBattery=make('car-battery','Car battery',
 'Light: the plus mark pulses 20% and settles. Strong: the battery jolts up a unit, rocks 2° and its plus mark pulses 35% before settling.',
 posed([fill('box-interior',battD,'tint'),stroke('box-frame',battD),fill('box-solid',(t,s)=>holes(battD,plusAt(pulseB(t,s)),minusAt(pulseB(t,s))),'solid'),
  ...bothFill('cap-0',caps[0]),...bothFill('cap-1',caps[1]),
  fill('plus',(t,s)=>plusAt(pulseB(t,s)),'outline'),fill('minus',(t,s)=>minusAt(pulseB(t,s)),'outline')],battPose),.6,.56);
carBattery.duotoneFill=true;
carBattery.constraints=[{type:'rigid',part:'car-battery-box-frame'},{type:'rigid',part:'car-battery-cap-0'}];

// ---------- GEAR SHIFT ----------
// An H gate with the knob in first. Forward: the knob slides down the slot into second; Strong snaps it with a bounded overshoot. Reverse: back up into first.
const shiftY=D([[0,0],[.06,0],[.5,13,'smooth']],[[0,0],[.06,-.8],[.28,13.6],[.4,12.8],[.48,13]],[[0,13],[.06,13],[.5,0,'smooth']],[[0,13],[.06,13.8],[.28,-.6],[.4,.2],[.48,0]]);
const gearShift=toggle('gear-shift','Gear shift',
 'Forward: the knob slides down the left slot from first into second; Strong lifts it out and snaps it in with a bounded overshoot. Reverse: it slides back up into first.',[
 ...tube('slot-0','M10 7.5 L10 24.5'),...tube('slot-1','M22 7.5 L22 24.5'),...tube('bar','M10 16 L22 16'),
 ...posed(bothFill('knob',circle(10,9.5,2.6)),rigid((t,s,r)=>({dy:shiftY(t,s,r)})))
],.5,.48,'first','second',.5,.48);
gearShift.duotoneFill=false;
gearShift.constraints=[{type:'fixed',part:'gear-shift-slot-0'},{type:'fixed',part:'gear-shift-slot-1'},{type:'fixed',part:'gear-shift-bar'},{type:'rigid',part:'gear-shift-knob'}];

// ---------- CAR NAVIGATION ----------
// A dashboard screen (Solid ring) with a start dot and a destination pin. Forward: the route draws from the car to the pin; Strong draws it fast and the pin hops as it arrives. Reverse: the route withdraws.
const screenD=rect(4,7,24,18,2.5),screenRing=holes(screenD,rect(6,9,20,14,1.2));
const RS=[9.5,20],RE=[20.6,12];
const route=u=>{const k=u*u*(3-2*u);return[RS[0]+(RE[0]-RS[0])*u,RS[1]+(RE[1]-RS[1])*k];};
const routeP=D([[0,0],[.06,0],[.6,1,'smooth']],[[0,0],[.06,0,'hold'],[.3,1,'out'],[.34,1]],[[0,1],[.06,1],[.5,0,'smooth']],[[0,1],[.04,1,'hold'],[.26,0,'in'],[.3,0]]);
const hop=F([[0,0],[.6,0]],[[0,0],[.28,0],[.36,-1.6],[.46,0,'hit'],[.52,-.3],[.58,0]]);
const carNavigation=toggle('car-navigation','Car navigation',
 'Forward: the route draws from the car dot up to the destination pin; Strong draws it in 0.24 s and the pin hops as the route arrives. Reverse: the route withdraws to the car.',[
 fill('screen-interior',screenD,'tint'),stroke('screen-frame',screenD),fill('screen-solid',screenRing,'solid'),
 ...bothFill('start',circle(...RS,1.6)),
 ...posed(bothFill('pin',circle(23.2,12,1.7)),rigid((t,s,r)=>({dy:r?0:hop(t,s)}))),
 ...shown(tube('route',(t,s,r)=>{const p=Math.max(routeP(t,s,r),.002);return curveFrom(u=>route(u*p),4);}),(t,s,r)=>routeP(t,s,r)>.03?1:0)
],.6,.58,'idle','routed',.5,.3);
carNavigation.duotoneFill=true;
carNavigation.constraints=[{type:'fixed',part:'car-navigation-screen-frame'},{type:'fixed',part:'car-navigation-start'},{type:'rigid',part:'car-navigation-pin'},{type:'inside',part:'car-navigation-route',box:[6.4,9.4,25.6,22.6]}];

// ---------- CAR WASH ----------
// A front-view car under falling water. Three drops fall from above the roof, fade out at the roof and reform at the top.
// Light: one gentle fall. Strong: two fast falls while the car rocks under the spray.
const frontD=rect(7,17,18,10,2.5),roofD='M10.5 17 L12 11.5 L20 11.5 L21.5 17';
const DROPS=[[11.5,0],[16,.12],[20.5,.06]];
const dropFall=d=>F([[0,0],[d,0],[d+.5,4,'in'],[d+.52,4],[d+.53,0,'linear']],[[0,0],[d*.6,0],[d*.6+.26,4,'in'],[d*.6+.28,4],[d*.6+.29,0,'linear'],[d*.6+.34,0],[d*.6+.6,4,'in'],[d*.6+.62,4],[d*.6+.63,0,'linear']]);
const dropShow=d=>F([[0,1],[d+.3,1],[d+.5,0,'in'],[d+.56,0],[d+.72,1,'out']],[[0,1],[d*.6+.14,1],[d*.6+.26,0,'in'],[d*.6+.3,0],[d*.6+.34,1,'out'],[d*.6+.48,1],[d*.6+.6,0,'in'],[d*.6+.64,0],[d*.6+.78,1,'out']]);
const rockW=F([[0,0],[.8,0]],[[0,0],[.1,0],[.24,2],[.44,-1.6],[.62,.8],[.78,0]]);
const carWash=make('car-wash','Car wash',
 'Light: three water drops fall 4 units onto the roof, fade and reform above. Strong: they fall twice as fast in two bursts while the car rocks 2° under the spray.',[
 ...posed([fill('roof-interior',roofD+' Z','tint'),stroke('roof',roofD),fill('roof-solid',roofD+' Z','solid'),
  fill('body-interior',frontD,'tint'),stroke('body-frame',frontD),fill('body-solid',holes(frontD,circle(10.5,22.5,1.3),circle(21.5,22.5,1.3)),'solid'),
  fill('lamp-0',circle(10.5,22.5,1.3),'outline'),fill('lamp-1',circle(21.5,22.5,1.3),'outline')],rigid((t,s)=>({angle:rockW(t,s),pivot:[16,27]}))),
 ...DROPS.flatMap(([x,d],i)=>{const f=dropFall(d),o=dropShow(d);return shown(bothFill('drop-'+i,circle(x,4.6,1)),o).map(p=>({...p,pose:(px,py,t,s)=>[px,py+f(t,s)]}));})
],.9,.88);
carWash.duotoneFill=true;
carWash.constraints=[{type:'rigid',part:'car-wash-body-frame'},{type:'rigid',part:'car-wash-roof'}];

// ---------- ENGINE OIL ----------
// The oil-can lamp: a can with a cap, a loop handle and a spout; a drop falls from the spout tip.
// Light: one drop falls. Strong: the can tips 10° toward the spout and the drop falls fast and far.
const canD=rect(8,13,12,7,1.6),capD=rect(12,10,4,3.4,1);
const oilDrop=F([[0,0],[.08,0],[.5,3.8,'in'],[.52,3.8],[.53,0,'linear']],[[0,0],[.14,0],[.4,5.2,'in'],[.42,5.2],[.43,0,'linear']]);
const oilShow=F([[0,1],[.3,1],[.5,0,'in'],[.56,0],[.72,1,'out']],[[0,1],[.26,1],[.4,0,'in'],[.46,0],[.62,1,'out']]);
const tipCan=F([[0,0],[.72,0]],[[0,0],[.06,-2],[.24,10],[.42,8],[.62,0]]);
const canPose=rigid((t,s)=>({angle:tipCan(t,s),pivot:[20,16.5]}));
const engineOil=make('engine-oil','Engine oil',
 'Light: a drop falls 3.8 units from the spout, fades and reforms. Strong: the can tips 10° toward its spout and the drop falls 5.2 units fast before the can settles.',[
 ...posed([...framed('can',canD),...bothFill('cap',capD),...tube('handle','M8 14.5 C4.6 14.5 4.6 18.5 8 18.5'),...tube('spout','M20 16.5 L25.6 13.2')],canPose),
 ...shown(bothFill('drop',circle(25.8,17.4,1.1)),oilShow).map(p=>({...p,pose:(x,y,t,s)=>[x,y+oilDrop(t,s)]}))
],.72,.62);
engineOil.duotoneFill=true;
engineOil.constraints=[{type:'rigid',part:'engine-oil-can-frame'},{type:'rigid',part:'engine-oil-spout'}];

// ---------- CAR WINDOW ----------
// A door panel with a window opening (Solid ring) and a handle. The glass edge descends as the window rolls down; the glass is tinted and Solid.
const doorD=polygon([[5.2,5],[21.5,5],[26.5,11],[26.5,26.5],[5.2,26.5]],2);
const WX0=9,WX1=22.7,WY0=8.75,WY1=17.5,SLANT=[[19.8,8.75],[22.7,13]];
const winD=polygon([[WX0,WY0],SLANT[0],SLANT[1],[WX1,WY1],[WX0,WY1]],1);
const slantX=y=>SLANT[0][0]+(SLANT[1][0]-SLANT[0][0])*(y-SLANT[0][1])/(SLANT[1][1]-SLANT[0][1]);
const glassAt=y=>{const yy=Math.min(y,WY1-.4),top=yy<SLANT[1][1]?[slantX(yy),yy]:[WX1,yy];return polygon([[WX0,yy],top,[WX1,Math.max(yy,SLANT[1][1])],[WX1,WY1],[WX0,WY1]],.6);};
const roll=D([[0,0],[.06,0],[.6,5,'smooth']],[[0,0],[.06,0,'hold'],[.3,5,'hit'],[.4,4.5],[.5,5]],[[0,5],[.06,5],[.56,0,'smooth']],[[0,5],[.06,5.2],[.32,0,'hit'],[.4,0]]);
const carWindow=toggle('car-window','Car window',
 'Forward: the glass rolls down 5 units inside the door window; Strong drops it fast with a stop and a small rebound. Reverse: the glass rolls back up.',[
 stroke('door-frame',doorD),fill('door-solid',holes(doorD,winD),'solid'),
 stroke('window',winD),
 fill('glass-interior',(t,s,r)=>glassAt(WY0+roll(t,s,r)),'tint'),fill('glass-solid',(t,s,r)=>glassAt(WY0+roll(t,s,r)),'solid'),
 ...tube('glass-edge',(t,s,r)=>{const y=WY0+roll(t,s,r);return `M${WX0} ${y} L${y<SLANT[1][1]?slantX(y):WX1} ${y}`;}),
 ...tube('handle','M17 22.2 L22 22.2')
],.6,.5,'closed','open',.56,.4);
carWindow.duotoneFill=true;
carWindow.constraints=[{type:'fixed',part:'car-window-door-frame'},{type:'fixed',part:'car-window-window'},{type:'fixed',part:'car-window-handle'}];

// ---------- TRUNK ----------
// A side-view car facing left. The body outline is open along the trunk lid, which is a separate stroke hinged at the rear window.
const TH=[22.5,14.2],TT=[27,17];
const trunkV=[TT,[27,20.4],[5,20.4],[5,16.8],[8,16.4],[11.5,11.2],[19,11.2],TH];
const trunkOutline=openPoly(trunkV,1.6),trunkFill=polygon(trunkV,1.6);
const lidA=D([[0,0],[.06,0],[.56,-70,'smooth']],[[0,0],[.06,4],[.3,-78],[.44,-66],[.56,-70]],[[0,-70],[.06,-70],[.56,0,'smooth']],[[0,-70],[.06,-72],[.3,0,'hit'],[.38,0]]);
const slam=F([[0,0],[.56,0]],[[0,0],[.3,0],[.38,.6,'hit'],[.5,-.15],[.6,0]]);
const trunk=toggle('trunk','Trunk',
 'Forward: the trunk lid lifts 70° about its hinge; Strong swings it past open and settles like a gas strut. Reverse: the lid closes; Strong slams it and the car dips.',[
 ...posed([fill('body-interior',trunkFill,'tint'),stroke('body-frame',trunkOutline),fill('body-solid',trunkFill,'solid'),
  ...wheel('wheel-rear',22.2,22.2,WR),...wheel('wheel-front',9.8,22.2,WR)],rigid((t,s,r)=>({dy:r?slam(t,s):0}))),
 ...posed(tube('lid',`M${TH} L${TT}`),rigid((t,s,r)=>({angle:lidA(t,s,r),dy:r?slam(t,s):0,pivot:TH})))
],.56,.56,'closed','open',.56,.6);
trunk.duotoneFill=true;
trunk.constraints=[{type:'rigid',part:'trunk-body-frame'},{type:'rigid',part:'trunk-lid'},{type:'rigid',part:'trunk-wheel-rear'}];

const ORDER=[car,electricCar,fuelPump,speedometer,parkingBrake,tirePressure,engine,seatbelt,headlights,wiper,steeringWheel,carKey,carLock,carBattery,gearShift,carNavigation,carWash,engineOil,carWindow,trunk];
export const icons=ORDER.map((icon,index)=>({...icon,number:986+index}));
