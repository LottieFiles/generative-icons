// Smart home batch C: window-blinds, robot-vacuum, washing-machine, refrigerator, oven.
import {make,F,P,tube,framed,rigid,posed,shown,stroke,fill,circle,rect,bothFill,annulus,holes} from './helpers.mjs';
import {arcN,pt,ringRect} from './sky.mjs';
const grown=(name,d)=>[stroke(name,d),{name:name+'-solid',d,style:'solid',type:'stroke',width:2.6}];

// ---------- WINDOW BLINDS ----------
// Venetian blind (v5): a head rail, five hanging slats and a side pull cord with a tassel; no boxed frame.
// Pulling the cord raises the blind one slat spacing: the top slat folds into the rail (fading below
// the rest position, never overlapping) while the tassel is drawn down. Rest: four slats still down. Slats stop 1 short of the cord and tassel at Bold.
// (v3/v4 in a square frame read as a document, then as a tablet with a home button.)
const SLATS=[9.5,13,16.5,20,23.5];
const lift=F([[0,0],[.06,0],[.56,1,'smooth']],[[0,0],[.06,0,'hold'],[.24,1.1],[.36,.95],[.46,1]]);
const slatY=(k,t,s)=>SLATS[k]-3.5*lift(t,s);
const slatV=(k,t,s)=>{const y=slatY(k,t,s);return y>=9.4?1:y<=8.8?0:(y-8.8)/.6;};
const cordEnd=(t,s)=>18+3.5*lift(t,s);
const windowBlinds=make('window-blinds','Window blinds',
 'Light: the cord is pulled and the blind rises one slat, its top slat folding into the head rail as the tassel comes down. Strong: it is yanked up past its stop, drops back and settles. The raised state is retained; preview reset rearms.',[
 ...tube('rail','M4.5 6 L27.5 6'),
 ...SLATS.flatMap((y,k)=>shown(posed(tube('slat-'+k,`M5.5 ${y} L22.8 ${y}`),rigid((t,s)=>({dy:slatY(k,t,s)-y}))),(t,s)=>slatV(k,t,s))),
 ...tube('cord',(t,s)=>`M26.5 6 L26.5 ${cordEnd(t,s)}`),
 ...posed(bothFill('tassel',circle(26.5,19.9,1.4)),rigid((t,s)=>({dy:cordEnd(t,s)-18})))
],.56,.46,['slat-0','slat-1','slat-2','slat-3','slat-4','cord','tassel','tassel-solid']);
windowBlinds.duotoneFill=false;

// ---------- ROBOT VACUUM ----------
// Top view: disc r9.4, a front bumper chord (ends on the rim) and a lidar ring. It drives right into three
// dust specks, which vanish as it reaches them, and reverses home: the floor stays clean. v1 used a
// bumper arc and centre dot; turning the arc under the dot drew a smiley face.
const VC=[13.2,16];
const drive=F([[0,0],[.06,0],[.36,2.6],[.72,0,'smooth']],[[0,0],[.06,-.5],[.22,3.6,'hit'],[.3,2.9],[.38,3.2],[.72,0]]);
const vacPose=rigid((t,s)=>({dx:drive(t,s)}));
const DUST=[[26.4,11.5],[26.8,16],[26.4,20.5]];
const robotVacuum=make('robot-vacuum','Robot vacuum',
 'Light: the vacuum drives 2.6 units right, the dust specks ahead of it vanish, and it reverses home. Strong: it backs up, charges 3.6 units into the dust, rebounds off it and reverses. The clean floor is retained; preview reset rearms.',[
 ...posed(framed('disc',circle(...VC,9.4),annulus(...VC,9.4,2.6)),vacPose),
 ...posed(tube('bumper','M18.2 8.04 L18.2 23.96'),vacPose),
 ...posed(tube('lidar',circle(10.8,16,2)),vacPose),
 ...DUST.flatMap(([x,y],i)=>shown(tube('dust-'+i,`M${x-.2} ${y} L${x+.2} ${y}`),P([[0,1],[.1+i*.02,1],[.18+i*.02,0,'in']],[[0,1],[.07+i*.01,1],[.11+i*.01,0,'in']])))
],.72,.72,['dust-0','dust-1','dust-2']);
robotVacuum.duotoneFill=true;

// ---------- WASHING MACHINE ----------
// Front loader: body, control line and knob, round door. The laundry swirl inside the door spins; it is
// two-fold symmetric, so a half turn is the next equivalent pose. Strong also shakes the machine.
const drum=F([[0,0],[.06,0],[.8,180,'smooth']],[[0,0],[.06,-20],[.62,560],[.8,540]]);
const shake=(t,s)=>{if(!s)return 0;const u=t-.16;return u<=0||u>=.5?0:.5*Math.sin(u*62)*(1-u/.5)**1.5;};
const machine=rigid((t,s)=>({dx:shake(t,s)}));
const washingMachine=make('washing-machine','Washing machine',
 'Light: the laundry swirl turns half a revolution inside the door. Strong: it rocks back, spins a turn and a half and settles, and the machine shudders during the spin.',[
 ...posed(framed('body',rect(5,4.5,22,23.5,3),ringRect(5,4.5,22,23.5,3)),machine),
 ...posed(tube('panel','M8.5 9 L13 9'),machine),...posed(tube('knob','M22.3 9 L22.7 9'),machine),
 ...posed(framed('door',circle(16,18.6,5.8),annulus(16,18.6,5.8,2.6)),machine),
 ...posed(tube('swirl','M13.7 18.6 C14.8 16.8 17.2 20.4 18.3 18.6'),rigid((t,s)=>({angle:drum(t,s),pivot:[16,18.6],dx:shake(t,s)})))
],.8,.8);
washingMachine.duotoneFill=true;washingMachine.cycleDegrees=180;

// ---------- REFRIGERATOR ----------
// Two-door fridge (cabinet x9.5–25.5). The lower door swings on its left hinge: its projected width is
// 16·cos θ, and past 90° it hangs open to the left of the cabinet, uncovering the whole interior with two
// shelves. Shelves are drawn only right of the door's edge; the handle (on the door's front) shows only
// while the front faces us. v1 squashed the door toward the hinge and hardly showed the interior.
const HINGE=9.5;
const swing=F([[0,0],[.06,0],[.34,105],[.5,105],[.8,0,'smooth']],[[0,0],[.04,-3],[.2,108,'hit'],[.28,98],[.36,106],[.5,106],[.66,0,'hit'],[.72,4],[.8,0]]);
const edgeX=(t,s)=>HINGE+16*Math.cos(swing(t,s)*Math.PI/180);
const doorD=(t,s)=>{const e=edgeX(t,s),a=Math.min(HINGE,e),w=Math.max(.02,Math.abs(e-HINGE));return rect(a,12,w,16.7,Math.min(2.5,w/2));};
// Always a ring (constant topology); a narrow door clamps its counter to a sliver inside the painted band.
const doorRing=(t,s)=>{const e=edgeX(t,s),a=Math.min(HINGE,e),w=Math.abs(e-HINGE),iw=Math.max(.2,w-2.6);return holes(rect(a-1.3,10.7,w+2.6,19.3,Math.min(3.8,(w+2.6)/2)),rect(a+1.3+Math.min(0,(w-2.6-iw)/2),13.3,iw,14.1,Math.min(1.2,iw/2)));};
const shelf=y=>(t,s)=>{const a=Math.min(21.6,Math.max(13,edgeX(t,s)+3.6));return `M${a} ${y} L22 ${y}`;};
const shelfV=(t,s)=>Math.max(13,edgeX(t,s)+3.6)<21.7?1:0;
const refrigerator=make('refrigerator','Refrigerator',
 'Light: the lower door swings open past square to hang at the left, showing the shelves inside, then closes. Strong: it is flung open against its stop and bounces, holds, then is slammed shut with a small rebound.',[
 ...tube('cabinet',rect(HINGE,3.3,16,25.4,3)),
 ...tube('divider',`M${HINGE} 12 L25.5 12`),
 ...tube('handle-top','M22 6.8 L22 8.5'),
 ...shown(tube('shelf-0',shelf(17.5)),shelfV),...shown(tube('shelf-1',shelf(22.5)),shelfV),
 fill('door-interior',doorD,'tint'),stroke('door-frame',doorD),{name:'door-solid',d:doorRing,style:'solid'},
 ...shown(tube('handle',(t,s)=>{const x=edgeX(t,s)-3.5;return `M${x} 15.5 L${x} 20`;}),(t,s)=>{const e=edgeX(t,s);return e>=17?1:e<=16?0:e-16;})
],.8,.8);
refrigerator.duotoneFill=true;

// ---------- OVEN ----------
// Oven with control dots, a display pill and a door window. Switching on lights the display, then
// two heat waves rise inside the window.
const pill=F([[0,0],[.06,0],[.24,1,'smooth']],[[0,0],[.04,0,'hold'],[.12,1]]);
const heatK=(i)=>F([[0,0],[.2+i*.08,0],[.56+i*.08,1,'smooth']],[[0,0],[.12+i*.04,0,'hold'],[.3+i*.04,1]]);
const shimmer=(t,s)=>{if(!s)return 1;const u=t-.3;return u<=0||u>=.4?1:1+.6*Math.sin(u*40)*(1-u/.4);};
const heatD=(x,i)=>(t,s)=>{const p=Math.max(.004,heatK(i)(t,s)),a=shimmer(t,s);return `M${x} 20.9 C${x+.9*a} ${20.9-1.27*p} ${x-.9*a} ${20.9-2.53*p} ${x} ${20.9-3.8*p}`;};
const oven=make('oven','Oven',
 'Light: the display lights and two heat waves rise inside the window in turn. Strong: the display snaps on, the waves shoot up and shimmer before they settle. The heated state is retained; preview reset rearms.',[
 ...framed('body',rect(4.5,4.5,23,23.7,3),ringRect(4.5,4.5,23,23.7,3)),
 ...[9,13,17].flatMap((x,i)=>tube('knob-'+i,`M${x-.2} 9 L${x+.2} 9`)),
 ...grown('display',(t,s)=>`M21 9 L${21+3*Math.max(.007,pill(t,s))} 9`),
 ...framed('window',rect(8.5,13.5,15,11,2),ringRect(8.5,13.5,15,11,2)),
 ...[13.5,18.5].flatMap((x,i)=>shown(grown('heat-'+i,heatD(x,i)),(t,s)=>heatK(i)(t,s)>.03?1:0))
],.72,.7,['display','heat-0','heat-1']);
oven.duotoneFill=true;

export const icons=[windowBlinds,robotVacuum,washingMachine,refrigerator,oven];
