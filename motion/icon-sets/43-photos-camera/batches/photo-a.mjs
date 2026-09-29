// Photos & camera batch A (hardware): flash-toggle, self-timer, switch-camera, tripod, aperture.
import {make,toggle,F,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,bothFill,rotate,holes,annulus} from './helpers.mjs';
import {arcN,pt} from './photo.mjs';

// ---------- FLASH TOGGLE ----------
// Off: a bolt alone. On: three rays shoot out from the bolt's tip and stay (the bolt is "flashing").
// Rays start 4.2 from the tip (1.7 painted clearance at Bold) and are 50° apart (1.05 clear at their roots).
const boltD=polygon([[18,10.4],[9.6,20.2],[15.2,20.2],[13.8,28.4],[22.6,16.4],[17,16.4]],[.9,.8,.6,.9,.8,.6]);
const TIP=[18,10.4],RAYS=[-140,-90,-40];
const rayOut=i=>F([[0,0],[.06+.04*i,0],[.4+.04*i,2,'out']],[[0,0],[.08+.03*i,0,'hold'],[.2+.03*i,2.7,'hit'],[.34+.03*i,2]]);
const rayBack=i=>F([[0,2],[.04+.03*i,2],[.34+.03*i,0,'in']],[[0,2],[.04+.02*i,2.3],[.22+.02*i,0,'in']]);
const rayLen=i=>{const o=rayOut(i),b=rayBack(i);return (t,s,r)=>r?b(t,s):o(t,s);};
const boltPop=F([[0,1],[.5,1]],[[0,1],[.06,.97],[.18,1.05,'hit'],[.3,.99],[.4,1]]);
const boltJolt=F([[0,0],[.5,0]],[[0,0],[.06,.2],[.18,-.6,'hit'],[.3,.1],[.4,0]]);
const boltPose=(x,y,t,s,r)=>{const k=r?1:boltPop(t,s),dy=r?0:boltJolt(t,s);return [16+(x-16)*k,17+(y-17)*k+dy];};
const rayD=i=>(t,s,r)=>{const a=RAYS[i]*Math.PI/180,len=Math.max(.01,rayLen(i)(t,s,r)),[ax,ay]=[TIP[0]+4.2*Math.cos(a),TIP[1]+4.2*Math.sin(a)];return `M${ax} ${ay} L${ax+len*Math.cos(a)} ${ay+len*Math.sin(a)}`;};
const flashToggle=toggle('flash-toggle','Flash toggle',
 'Off to on: three rays grow outward from the bolt\'s tip and stay. Strong: the bolt jolts and the rays shoot past their length before settling. Reverse pulls the rays back into the tip.',[
 ...posed(framed('bolt',boltD),boltPose),
 ...RAYS.flatMap((_,i)=>posed(shown(tube('ray-'+i,rayD(i)),(t,s,r)=>rayLen(i)(t,s,r)>.3?1:0),boltPose))
],.56,.5,'off','on',.44,.34);
flashToggle.duotoneFill=true;

// ---------- SELF TIMER ----------
// A camera whose lens ring is the countdown: the ring depletes clockwise from 12, then the lens dot
// flashes and the shot is taken. The fired camera (dot only) is retained; preview reset rearms the ring.
const camBody=polygon([[4.5,10],[10,10],[12.2,6.4],[19.8,6.4],[22,10],[27.5,10],[27.5,27.2],[4.5,27.2]],[2.2,1,1.2,1.2,1,2.2,2.6,2.6]);
const CC=[16,18.6];
const span=F([[0,360],[.08,360],[.66,0,'linear']],[[0,360],[.08,360,'hold'],[.44,0,'linear']]);
const flash=F([[0,1],[.66,1],[.76,1.8],[.94,1,'smooth']],[[0,1],[.44,1],[.5,2.3,'hit'],[.7,1]]);
const camJolt=F([[0,0],[.94,0]],[[0,0],[.44,0],[.48,.5,'hit'],[.62,0]]);
const camPose=rigid((t,s)=>({dy:camJolt(t,s)}));
const flashPose=(x,y,t,s)=>{const k=flash(t,s);return [CC[0]+(x-CC[0])*k,CC[1]+(y-CC[1])*k+camJolt(t,s)];};
const selfTimer=make('self-timer','Self-timer',
 'The lens ring counts down clockwise from 12, then the lens dot flashes as the shot fires. Strong counts faster, the flash is bigger and the camera jolts on the shot. The fired camera is retained; preview reset rearms the ring.',[
 ...posed(framed('body',camBody,holes(camBody,circle(...CC,6.5))),camPose),
 ...posed(shown(tube('ring',(t,s)=>arcN(...CC,4.4,-90,-90+Math.max(.05,span(t,s)),4)),(t,s)=>span(t,s)>1.5?1:0),camPose),
 ...posed(bothFill('dot',circle(...CC,1.7)),flashPose)
],.94,.7,['ring']);
selfTimer.duotoneFill=true;

// ---------- SWITCH CAMERA ----------
// Two curved arrows with filled heads cycle inside the camera body. A half turn lands each arrow on the
// other's start, so the icon returns to its own drawing (cycleDegrees 180).
const AR=4.8;
const arrowD=(a,b)=>arcN(...CC,AR,a,b,2);
const headD=(deg)=>{const p=pt(deg,AR,CC),T=[-Math.sin(deg*Math.PI/180),Math.cos(deg*Math.PI/180)],N=[T[1],-T[0]];const tip=[p[0]+3.1*T[0],p[1]+3.1*T[1]],bl=[p[0]-.3*T[0]+1.7*N[0],p[1]-.3*T[1]+1.7*N[1]],br=[p[0]-.3*T[0]-1.7*N[0],p[1]-.3*T[1]-1.7*N[1]];return polygon([tip,bl,br],[.35,.35,.35]);};
const spin=F([[0,0],[.06,0],[.62,180,'smooth']],[[0,0],[.08,-14],[.36,192],[.5,180]]);
const spinPose=rigid((t,s)=>({angle:spin(t,s),pivot:CC}));
const switchCamera=make('switch-camera','Switch camera',
 'Light: the two arrows turn a half circle inside the camera and land on each other\'s start. Strong: a short wind-back, then a fast half turn that overshoots and settles.',[
 ...framed('body',camBody,holes(camBody,circle(...CC,6.5))),
 ...posed([...tube('arrow-a',arrowD(-150,-60)),...bothFill('head-a',headD(-60)),...tube('arrow-b',arrowD(30,120)),...bothFill('head-b',headD(120))],spinPose)
],.62,.5);
switchCamera.duotoneFill=true;switchCamera.cycleDegrees=180;

// ---------- TRIPOD ----------
// A camera head on a column with three legs. The head tilts on its mount (16,11.5): Light a gentle tilt
// down and back; Strong winds up, tilts hard, and settles while the whole tripod dips on its feet.
const headD2=rect(10.5,5.5,11,6,1.8),MOUNT=[16,11.5];
const tilt=F([[0,0],[.06,0],[.3,-9],[.64,0,'smooth']],[[0,0],[.08,5],[.24,-20,'hit'],[.42,3],[.56,0]]);
const dip=F([[0,0],[.64,0]],[[0,0],[.22,0],[.28,.5,'hit'],[.42,0]]);
const headPose=rigid((t,s)=>({angle:tilt(t,s),pivot:MOUNT,dy:dip(t,s)}));
const legPose=rigid((t,s)=>({dy:dip(t,s)}));
const tripod=make('tripod','Tripod',
 'Light: the camera head tilts 9° on its mount and returns. Strong: it winds up 5°, tilts 20° with a hard stop, and settles as the tripod dips on its feet.',[
 ...posed([...framed('head',headD2,holes(headD2,circle(16,8.5,1.6))),fill('lens',circle(16,8.5,1.5),'outline')],headPose),
 ...posed([...tube('column','M16 11.5 L16 28'),...tube('leg-l','M16 16.5 L7.5 28'),...tube('leg-r','M16 16.5 L24.5 28')],legPose)
],.64,.56);
tripod.duotoneFill=true;

// ---------- APERTURE ----------
// Six blades tangent to an inner circle of radius rho, each running from the ring to the next blade.
// Open (rho 5.6) stops down to rho 2.6 and holds; reverse opens it again.
const R=11.5;
const rhoF=F([[0,5.6],[.06,5.6],[.56,2.6,'smooth']],[[0,5.6],[.08,6.4],[.3,2.2,'hit'],[.44,2.6]]);
const rhoB=F([[0,2.6],[.5,5.6,'smooth']],[[0,2.6],[.08,2.2],[.3,6.2,'hit'],[.44,5.6]]);
const rho=(t,s,r)=>r?rhoB(t,s):rhoF(t,s);
const bladeLine=(i,p)=>{const th=(-90+60*i)*Math.PI/180,o=[16+R*Math.cos(th),16+R*Math.sin(th)],beta=Math.asin(p/R),n=[-Math.cos(th),-Math.sin(th)];const d=[n[0]*Math.cos(beta)-n[1]*Math.sin(beta),n[0]*Math.sin(beta)+n[1]*Math.cos(beta)];return {o,d};};
const bladeD=i=>(t,s,r)=>{const p=rho(t,s,r),a=bladeLine(i,p),b=bladeLine(i+1,p);
 const den=a.d[0]*b.d[1]-a.d[1]*b.d[0],u=((b.o[0]-a.o[0])*b.d[1]-(b.o[1]-a.o[1])*b.d[0])/den;
 const start=[a.o[0]+.4*a.d[0],a.o[1]+.4*a.d[1]],end=[a.o[0]+u*a.d[0],a.o[1]+u*a.d[1]];return `M${start} L${end}`;};
const aperture=toggle('aperture','Aperture',
 'Open to stopped down: the six blades sweep in and close the iris to a small opening, then hold. Strong opens a touch wider first, snaps shut and settles. Reverse opens the blades again.',[
 ...framed('ring',circle(16,16,R),annulus(16,16,R,2.6)),
 ...[0,1,2,3,4,5].flatMap(i=>tube('blade-'+i,bladeD(i)))
],.56,.44,'open','stopped',.5,.44);
aperture.duotoneFill=true;

export const icons=[flashToggle,selfTimer,switchCamera,tripod,aperture];
