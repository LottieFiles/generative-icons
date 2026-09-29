// Maps & navigation (42). Twenty icons authored here; assembly comes from the shared library.
// Designs, reviews and corrections: evidence/design-review.md.
import {make,F,P,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,bothFill,rotate,holes,capsule,mix,timePair} from '../library/ai-authoring.mjs';

// ---------- shared assembly (no artwork) ----------
const rad=a=>a*Math.PI/180;
const pt=(cx,cy,r,a)=>[cx+r*Math.cos(rad(a)),cy+r*Math.sin(rad(a))];
// Fixed-topology circular arc: always n cubic segments, so its span can animate.
const arcN=(cx,cy,r,a,b,n=4)=>{const p=t=>[cx+r*Math.cos(t),cy+r*Math.sin(t)];let s=rad(a),d='M'+p(s);for(let i=1;i<=n;i++){const e=rad(a+(b-a)*i/n),k=4/3*Math.tan((e-s)/4),u=p(s),v=p(e);d+=' C'+[u[0]-k*r*Math.sin(s),u[1]+k*r*Math.cos(s),v[0]+k*r*Math.sin(e),v[1]-k*r*Math.cos(e),...v];s=e;}return d;};
const L=d=>d.replace(/^M/,'L');
// Map pin: a disc of radius r centred (cx,cy) with two tangents meeting at the tip (cx,tipY).
const pinD=(cx,cy,r,tipY)=>{const alpha=Math.acos(r/(tipY-cy))*180/Math.PI,beta=90-alpha;return `M${cx} ${tipY} L${pt(cx,cy,r,beta)} ${L(arcN(cx,cy,r,beta,beta+2*alpha-360))} Z`;};
// Point at arc length s along polyline V; extrapolates past either end along the end segment.
function along(V,s){let acc=0;for(let i=0;i<V.length-1;i++){const a=V[i],b=V[i+1],len=Math.hypot(b[0]-a[0],b[1]-a[1]);if(s<=acc+len||i===V.length-2){const u=(s-acc)/len;return [a[0]+(b[0]-a[0])*u,a[1]+(b[1]-a[1])*u];}acc+=len;}}
const length=V=>V.slice(1).reduce((n,b,i)=>n+Math.hypot(b[0]-V[i][0],b[1]-V[i][1]),0);
const lerp=(a,b,u)=>[mix(a[0],b[0],u),mix(a[1],b[1],u)];
const clamp=x=>Math.max(0,Math.min(1,x));
// Stateful icon with an authored reverse action (same helper as sets 34 and 39).
function toggle(name,label,intent,parts,light,strong,initialState,finalState,reverseLight=light,reverseStrong=strong){
 return {...make(name,label,intent,parts,light,strong),stateful:true,initialState,finalState,reverseSuffix:'-reverse',forwardLabel:finalState,reverseLabel:initialState,reverseTimings:timePair(reverseLight,reverseStrong)};
}
// Forward and reverse drivers share one signature (t, strong, reverse).
const dir=(forward,reverse)=>(t,s,r)=>r?reverse(t,s):forward(t,s);

// ---------- 1. PIN CLUSTER ----------
// A large pin (r5 at y11.5) and two small pins (r2.6 at y18.5) at x6.3 / x25.7, low enough on the main pin's
// tapered flank to keep 1.4 clear at Bold. The flank leaves no room to slide them out, so they arrive in place
// from below: rising gently (Light) or shooting up past their places and landing (Strong). They fade rigidly.
const mainPin=pinD(16,11.5,5,23),sidePin=x=>pinD(x,18.5,2.6,24.5);
const sideDy=dir(F([[0,2],[.05,2],[.45,0,'out']],[[0,2],[.06,2,'hold'],[.2,-.5,'hit'],[.3,.15],[.38,0]]),
 F([[0,0],[.4,2,'in']],[[0,0],[.16,2],[.24,2]]));
const clusterShow=dir(P([[0,0],[.05,0],[.3,1,'out']],[[0,0],[.06,0],[.14,1,'out']]),P([[0,1],[.1,1],[.4,0,'in']],[[0,1],[.04,1],[.18,0,'in']]));
const mainHop=dir(F([[0,0],[.5,0]],[[0,0],[.2,0,'hold'],[.26,-.9],[.38,0,'hit'],[.44,-.15],[.5,0]]),F([[0,0],[.4,0]],[[0,0],[.3,0]]));
const pinCluster=toggle('pin-cluster','Pin cluster',
 'Light: two small pins rise into place beside the main pin and fade in. Strong: they shoot up past their places and land, and the main pin hops in response; reverse sinks them away.',[
 ...posed([...framed('pin',mainPin,holes(mainPin,circle(16,11.5,2))),stroke('pin-hole',circle(16,11.5,2))],rigid((t,s,r)=>({dy:mainHop(t,s,r)}))),
 ...posed(shown(framed('side-l',sidePin(6.3)),clusterShow),rigid((t,s,r)=>({dy:sideDy(t,s,r)}))),
 ...posed(shown(framed('side-r',sidePin(25.7)),clusterShow),rigid((t,s,r)=>({dy:sideDy(t,s,r)})))
],.5,.5,'clustered','expanded',.4,.3);

// ---------- 2. CURRENT LOCATION ----------
// A filled dot inside a ring (r7); a ripple ring leaves the ring, grows to r12 and fades. Strong pulses the dot
// and sends two ripples. Ripples are hidden while parked on the ring, so rest is exactly the plain drawing.
const ripple=(delay,ls,ss)=>{
 const wait=(...tag)=>delay?[[delay,7,...tag]]:[];
 const R=F([[0,7],...wait(),[delay+ls,12,'out'],[delay+ls+.001,7]],[[0,7],...wait('hold'),[delay+ss,12],[delay+ss+.001,7]]);
 const show=P([[0,0],...(delay?[[delay,0]]:[]),[delay+.02,.9,'out'],[delay+ls,0,'linear'],[delay+ls+.001,0]],[[0,0],...(delay?[[delay,0]]:[]),[delay+.02,1,'out'],[delay+ss,0,'linear'],[delay+ss+.001,0]]);
 return {R,show};
};
const rip0=ripple(0,.55,.4),rip1=ripple(.16,.55,.44);
const dotPulse=F([[0,1],[.6,1]],[[0,1],[.06,.85],[.16,1.3,'hit'],[.3,.95],[.42,1]]);
const currentLocation=make('current-location','Current location',
 'Light: one ripple leaves the ring, expands and fades. Strong: the dot pulses and two ripples race outward in quick succession.',[
 fill('field-interior',circle(16,16,7),'tint'),
 ...tube('ring',circle(16,16,7)),
 ...posed(bothFill('dot',circle(16,16,2.6)),(x,y,t,s)=>{const k=dotPulse(t,s);return [16+(x-16)*k,16+(y-16)*k];}),
 ...shown(tube('ripple-0',(t,s)=>circle(16,16,rip0.R(t,s))),(t,s)=>rip0.show(t,s)),
 ...shown(tube('ripple-1',(t,s)=>circle(16,16,rip1.R(t,s))),(t,s)=>s?rip1.show(t,s):0)
],.6,.62);

// ---------- 3. DIRECTIONS ----------
// The navigation pointer (notched arrowhead) turns about its centre as a heading updates.
const pointerD=polygon([[16,5],[25,24.5],[16,19.5],[7,24.5]],[1.2,1,.8,1]);
const heading=F([[0,0],[.05,0],[.35,28,'smooth'],[.8,0,'smooth']],[[0,0],[.06,-10],[.2,52,'hit'],[.36,-6],[.5,3],[.62,0]]);
const directions=make('directions','Directions',
 'Light: the pointer swings 28° to the new heading and eases back. Strong: it winds back, snaps 52° with a hard stop and damps to rest.',
 posed(framed('pointer',pointerD),rigid((t,s)=>({angle:heading(t,s),pivot:[16,16.5]}))),.8,.62);

// ---------- 4. TURN BY TURN ----------
// A right-turning road (outer r11, inner r1, 12 wide) and a small pointer that drives up the lane, carries speed
// through the bend and stops facing the exit. The arrived pointer is retained; preview reset rearms.
// The small pointer has no clear tint area at Bold; Duotone uses Outline in every action.
const CENTER=[[12,24],[12,17],...Array.from({length:24},(_,i)=>pt(17,17,5,180+90*(i+1)/24)),[24,12]];
const CENTER_LEN=length(CENTER);
const travel=F([[0,0],[.06,0],[.8,1,'smooth']],[[0,0],[.05,0,'hold'],[.18,.3,'pass'],[.42,1.03,'hit'],[.54,1]]);
const carPointer=polygon([[0,-3],[2.2,2.4],[0,1.3],[-2.2,2.4]],[.8,.7,.5,.7]);
const lanePose=(x,y,t,s,r)=>{const sd=travel(t,s,r)*CENTER_LEN,[px,py]=along(CENTER,sd),[ax,ay]=along(CENTER,sd-.05),[bx,by]=along(CENTER,sd+.05),h=Math.atan2(by-ay,bx-ax)*180/Math.PI+90,[rx,ry]=rotate(x,y,h,0,0);return [rx+px,ry+py];};
const turnByTurn=make('turn-by-turn','Turn by turn',
 'Light: the pointer drives up the lane, rounds the bend and stops facing the exit. Strong: it launches, carries speed through the bend and brakes with a short overrun. The arrived pointer is retained; preview reset rearms.',[
 ...tube('outer','M6 27.5 L6 17 C6 10.925 10.925 6 17 6 L27.5 6'),
 ...tube('inner','M18 27.5 L18 19 C18 18.45 18.45 18 19 18 L27.5 18'),
 ...posed([stroke('pointer-frame',carPointer),fill('pointer-solid',carPointer)],lanePose)
],.8,.56,['pointer']);

// ---------- 5. ROUTE RECALCULATE ----------
// A start dot, a destination pin and the route between them. Recalculating morphs the route from a low
// southern bow to a northern one; both ends stay attached. Strong whips past the new shape and the pin hops.
const ROUTE_A=[[9.2,24.3],[15,27],[21,23],[23.6,16.4]],ROUTE_B=[[9.2,24.3],[8,17],[15,14],[23.6,16.4]];
const reroute=dir(F([[0,0],[.05,0],[.55,1,'smooth']],[[0,0],[.05,0,'hold'],[.24,1.15,'hit'],[.4,.95],[.5,1]]),
 F([[0,1],[.5,0,'smooth']],[[0,1],[.2,-.12,'hit'],[.34,.04],[.44,0]]));
const pinHop=dir(F([[0,0],[.55,0]],[[0,0],[.2,0],[.3,-1.4],[.44,0,'hit'],[.52,-.3],[.6,0]]),F([[0,0],[.5,0]],[[0,0],[.16,0],[.24,-.8],[.36,0,'hit'],[.44,0]]));
const routeD=(t,s,r)=>{const p=reroute(t,s,r),c=ROUTE_A.map((a,i)=>lerp(a,ROUTE_B[i],p));return `M${c[0]} C${c[1]} ${c[2]} ${c[3]}`;};
const destPin=pinD(24,8.5,3.2,15.5);
const routeRecalculate=toggle('route-recalculate','Route recalculate',
 'Light: the route between the start dot and the destination pin bends smoothly onto a new course. Strong: it whips past the new course, settles, and the pin hops; reverse restores the original route.',[
 ...bothFill('start',circle(6.5,25,1.9)),
 ...tube('route',routeD),
 ...posed(framed('pin',destPin),rigid((t,s,r)=>({dy:pinHop(t,s,r)})))
],.6,.6,'original','rerouted',.5,.44);

// ---------- 6. TRAFFIC LIGHT ----------
// A tall housing with three lamps drawn as filled marks: a lit lamp is a disc (r2), an unlit one a dot (r0.9),
// so the three keep ≥2.5 clear at Bold. Solid keeps three real counters (r3.1) around the marks.
const LAMP_Y=[9.5,16,22.5];
const lampK=[
 dir(F([[0,1],[.08,1],[.3,0,'smooth']],[[0,1],[.04,1,'hold'],[.14,0]]),F([[0,0],[.3,0],[.52,1,'smooth']],[[0,0],[.3,0,'hold'],[.4,1.15,'hit'],[.5,1]])),
 dir(F([[0,0],[.12,0],[.3,1,'smooth'],[.36,1],[.55,0,'smooth']],[[0,0],[.05,0,'hold'],[.14,1.15,'hit'],[.22,1],[.3,1,'hold'],[.38,0]]),
     F([[0,0],[.1,0],[.28,1,'smooth'],[.34,1],[.52,0,'smooth']],[[0,0],[.05,0,'hold'],[.14,1.15,'hit'],[.22,1],[.3,1,'hold'],[.38,0]])),
 dir(F([[0,0],[.38,0],[.6,1,'smooth']],[[0,0],[.3,0,'hold'],[.4,1.2,'hit'],[.5,.95],[.58,1]]),F([[0,1],[.06,1],[.28,0,'smooth']],[[0,1],[.04,1,'hold'],[.14,0]]))
];
const housingD=rect(10.5,3.5,11,25,3);
const trafficLight=toggle('traffic-light','Traffic light',
 'Light: red fades to a dot as amber grows, then amber gives way to green. Strong: red drops, amber pops in and holds, and green lands with a small overshoot; reverse runs the signal back to red.',[
 ...framed('housing',housingD,holes(housingD,...LAMP_Y.map(y=>circle(16,y,3.1)))),
 ...LAMP_Y.flatMap((y,i)=>posed(bothFill('lamp-'+i,circle(16,y,1)),(x,py,t,s,r)=>{const R=mix(.9,2,lampK[i](t,s,r));return [16+(x-16)*R,y+(py-y)*R];}))
],.6,.6,'stop','go',.55,.5);

// ---------- 7. MAP LAYERS ----------
// A rhombus tile over two chevrons spaced 4.2 (3.76 perpendicular, 1.16 clear at Bold). The top tile lifts
// off the stack and lands; Strong hovers higher and the lower chevrons dip together on the landing.
const tileD=polygon([[16,7],[26,12],[16,17],[6,12]],1);
const lift=F([[0,0],[.05,0],[.3,-1.8,'out'],[.6,0,'smooth']],[[0,0],[.04,.3],[.18,-3.4],[.3,-3.1],[.44,0,'hit'],[.52,-.5],[.6,0]]);
const dip=F([[0,0],[.6,0]],[[0,0],[.44,0,'hold'],[.5,.45,'hit'],[.62,0]]);
const mapLayers=make('map-layers','Map layers',
 'Light: the top layer lifts 1.8 units off the stack and settles back. Strong: it springs up 3.4, hovers, drops with a hard landing and the lower layers dip under it.',[
 ...posed(framed('top',tileD),rigid((t,s)=>({dy:lift(t,s)}))),
 ...posed([...tube('layer-2','M6 16.2 L16 21.2 L26 16.2'),...tube('layer-3','M6 20.4 L16 25.4 L26 20.4')],rigid((t,s)=>({dy:dip(t,s)})))
],.6,.62);

// ---------- 8. SATELLITE ----------
// Body, two solar panels on short struts, and two downlink arcs (r4 / r7.6 about the body's bottom, 61°–119°
// so their end caps stay 1 below the body at Bold and 1.1 apart). The craft rolls about its centre as the arcs ping.
const roll=F([[0,0],[.05,0],[.3,4,'smooth'],[.7,0,'smooth']],[[0,0],[.06,-3],[.2,11,'hit'],[.36,-5],[.5,2.5],[.62,-.8],[.72,0]]);
const ping0=P([[0,0],[.1,1,'out'],[.55,0,'in']],[[0,0],[.08,1,'out'],[.3,0,'in'],[.36,1,'out'],[.6,0,'in']]);
const ping1=P([[0,0],[.15,0],[.28,1,'out'],[.7,0,'in']],[[0,0],[.06,0],[.16,1,'out'],[.34,0,'in'],[.44,1,'out'],[.7,0,'in']]);
const satPose=rigid((t,s)=>({angle:roll(t,s),pivot:[16,14]}));
const satellite=make('satellite','Satellite',
 'Light: the satellite rolls 4° and one downlink pulse fades out below it. Strong: it rolls 11° with a damped wobble while two pulses fire twice.',[
 ...posed([...framed('body',rect(12.5,10,7,8,1.5)),...framed('panel-l',rect(3.8,11.5,6.2,5,1)),...framed('panel-r',rect(22,11.5,6.2,5,1)),
  ...tube('strut-l','M10 14 L12.5 14'),...tube('strut-r','M19.5 14 L22 14'),
  ...shown(tube('signal-0',arcN(16,18,4,61,119,2)),ping0),...shown(tube('signal-1',arcN(16,18,7.6,61,119,2)),ping1)],satPose)
],.7,.72);

// ---------- 9. STREET VIEW ----------
// The pegman (filled head, capsule torso, two legs) standing on the street. Picking it up lifts it off the street;
// Strong lifts higher with a tilt and lands with a bounce. Legs start on the torso's edge and end in the street.
const pegLift=F([[0,0],[.05,0],[.3,-1.6,'out'],[.6,0,'smooth']],[[0,0],[.05,.3],[.2,-3.2],[.32,-3],[.46,0,'hit'],[.54,-.6],[.62,0]]);
const pegTilt=F([[0,0],[.6,0]],[[0,0],[.05,0],[.2,10],[.34,7],[.46,0,'hit'],[.54,-3],[.62,0]]);
const pegPose=rigid((t,s)=>({dy:pegLift(t,s),angle:pegTilt(t,s),pivot:[16,16.7]}));
const streetView=make('street-view','Street view',
 'Light: the pegman is lifted 1.6 units off the street and set back down. Strong: it is picked up 3.2 units with a 10° tilt, dropped, and bounces once on landing.',[
 ...posed([...bothFill('head',circle(16,9,2)),...framed('torso',rect(12.8,13.2,6.4,7,3)),...tube('leg-l','M14.2 20.2 L14.2 27.5'),...tube('leg-r','M17.8 20.2 L17.8 27.5')],pegPose),
 ...tube('street','M4.5 27.5 L27.5 27.5')
],.6,.62);

// ---------- 10. GEOFENCE ----------
// A pin inside a dashed fence (six 36° dashes at r10.5; 1.9 between dash ends and 1.2 to the pin at Bold).
// Enabling draws the dashes around clockwise from the top; Strong whips them all on at once as the fence snaps
// outward and settles. The pin stays put: its top is only 1.2 clear of the fence, so it never rises.
const DASH=6,SPAN=36;
const fenceR=dir(F([[0,10.5],[.6,10.5]],[[0,10.5],[.04,10.5,'hold'],[.22,11.5,'hit'],[.36,10.5]]),F([[0,10.5],[.55,10.5]],[[0,10.5],[.04,10.5,'hold'],[.14,11,'hit'],[.3,10.5]]));
const dashG=i=>dir(F([[0,0],[.04+i*.08,0],[.18+i*.08,1,'out']],[[0,0],[.04+i*.03,0,'hold'],[.12+i*.03,1]]),
 F([[0,1],[.02+(DASH-1-i)*.08,1],[.16+(DASH-1-i)*.08,0,'in']],[[0,1],[.02+(DASH-1-i)*.02,1,'hold'],[.1+(DASH-1-i)*.02,0]]));
const geoPin=pinD(16,13,3.8,21.5);
const geofence=toggle('geofence','Geofence',
 'Light: six fence dashes draw clockwise around the pin from the top. Strong: all dashes whip on together as the fence snaps outward and settles; reverse retracts the dashes.',[
 ...framed('pin',geoPin,holes(geoPin,circle(16,13,1.3))),
 ...Array.from({length:DASH},(_,i)=>{const g=dashG(i),a0=-90+i*60-SPAN/2;
  return shown(tube('dash-'+i,(t,s,r)=>arcN(16,16,fenceR(t,s,r),a0,a0+SPAN*Math.max(.02,g(t,s,r)),1)),(t,s,r)=>g(t,s,r)>.03?1:0);}).flat()
],.6,.4,'off','on',.6,.32);

// ---------- 11. DISTANCE ----------
// Two pins and a measuring tape beneath them: the tape extends from the left tick to the right pin and the far
// tick appears on arrival. Strong drives the tape fast and the far pin hops when it is reached. Result retained.
const tapeP=F([[0,0],[.05,0],[.6,1,'smooth']],[[0,0],[.05,0,'hold'],[.3,1.02,'hit'],[.4,1]]);
const farHop=F([[0,0],[.6,0]],[[0,0],[.3,0],[.37,-1],[.48,0,'hit'],[.54,-.2],[.6,0]]);
const pinL=pinD(8,12,3.2,19.2),pinR=pinD(24,12,3.2,19.2);
const distance=make('distance','Distance',
 'Light: a measuring tape extends from the left pin to the right pin and its far tick appears on arrival. Strong: the tape launches, overruns the far pin slightly and the pin hops. The measurement is retained; preview reset rearms.',[
 ...framed('pin-l',pinL,holes(pinL,circle(8,12,1.2))),
 ...posed(framed('pin-r',pinR,holes(pinR,circle(24,12,1.2))),rigid((t,s)=>({dy:farHop(t,s)}))),
 ...tube('tick-l','M8 22.8 L8 27.2'),
 ...shown(tube('tape',(t,s)=>`M8 25 L${8+16*Math.max(.03,tapeP(t,s))} 25`),(t,s)=>tapeP(t,s)>.03?1:0),
 ...shown(tube('tick-r','M24 22.8 L24 27.2'),(t,s)=>clamp((tapeP(t,s)-.9)/.08))
],.6,.6,['tape','tick-r']);

// ---------- 12. ELEVATION ----------
// A mountain profile with a marker that climbs the trail from the first slope to the summit and stays there.
// Strong changes pace on the climb and hops on the summit; reverse descends.
const TRAIL=[[4.5,26],[10,15],[15,20.5],[21.5,8.5],[27.5,26]];
const S0=4.47,S_PEAK=length(TRAIL.slice(0,4));
const climb=dir(F([[0,0],[.05,0],[.85,1,'smooth']],[[0,0],[.05,0,'hold'],[.25,.42,'pass'],[.5,1,'hit'],[.6,1]]),
 F([[0,1],[.8,0,'smooth']],[[0,1],[.04,1,'hold'],[.2,.62,'pass'],[.42,0,'hit'],[.5,0]]));
const summitHop=dir(F([[0,0],[.85,0]],[[0,0],[.5,0,'hold'],[.56,-1.2],[.66,0,'hit'],[.72,-.2],[.78,0]]),F([[0,0],[.8,0]],[[0,0],[.5,0]]));
const START=along(TRAIL,S0);
const markerPose=rigid((t,s,r)=>{const [x,y]=along(TRAIL,mix(S0,S_PEAK,climb(t,s,r)));return {dx:x-START[0],dy:y-START[1]+summitHop(t,s,r)};});
const elevation=toggle('elevation','Elevation',
 'Light: the marker climbs the trail from the lower slope to the summit. Strong: it climbs with a change of pace, arrives hard and hops once on the summit; reverse descends.',[
 ...framed('mountain',polygon(TRAIL,.9)),
 ...posed([fill('marker',circle(...START,2.1),'outline'),fill('marker-solid',circle(...START,2.6),'solid')],markerPose)
],.85,.78,'trailhead','summit',.8,.5);

// ---------- 13. PARKING ----------
// An open bay (U) and a top-view car that pulls in from above; parked it sits 1.5 clear of the bay's bottom at
// Bold. Strong drives in and brakes with a short overrun; reverse backs out.
const carD=rect(12,3.5,8,12,2.6);
const park=dir(F([[0,0],[.05,0],[.65,7.5,'smooth']],[[0,0],[.05,0,'hold'],[.3,7.9,'hit'],[.44,7.5]]),
 F([[0,7.5],[.6,0,'smooth']],[[0,7.5],[.06,7.9],[.28,0,'hit'],[.36,.4],[.44,0]]));
const parking=toggle('parking','Parking',
 'Light: the car pulls smoothly into the bay and stops. Strong: it drives in fast, brakes with a short overrun and settles; reverse backs it out.',[
 ...tube('bay','M8 9 L8 27 L24 27 L24 9'),
 // Windshield caps are buried in the car's side rails; Solid keeps a shorter counter with 1.3 walls.
 ...posed([...framed('car',carD,holes(carD,capsule([14.2,7.2],[17.8,7.2],1.8))),stroke('windshield','M12.4 7.2 L19.6 7.2')],rigid((t,s,r)=>({dy:park(t,s,r)})))
],.65,.5,'arriving','parked',.6,.44);

// ---------- 14. FUEL STATION ----------
// Pump body with a window, base, and a hose to a nozzle hooked on the right. Lifting the nozzle rotates it about
// its base; Strong also lifts it and the hose end follows before it clacks back onto the hook.
const NOZ=[24,15];
const nozTilt=F([[0,0],[.05,0],[.3,12,'smooth'],[.65,0,'smooth']],[[0,0],[.05,-2],[.22,18,'hit'],[.36,13],[.46,0,'hit'],[.54,4],[.62,0]]);
const nozLift=F([[0,0],[.65,0]],[[0,0],[.05,.2],[.2,-1.6],[.32,-1.4],[.46,0,'hit'],[.52,-.3],[.6,0]]);
const pumpBody=rect(5,6,12,21,2);
const fuelStation=make('fuel-station','Fuel station',
 'Light: the nozzle tips 12° off its hook and settles back. Strong: it is lifted 1.6 units and tipped 18°, the hose follows, and it clacks back onto the hook.',[
 ...framed('body',pumpBody,holes(pumpBody,rect(8.5,9.5,5,4.5,.8))),stroke('window',rect(8.5,9.5,5,4.5,.8)),
 ...tube('base','M3.5 27 L18.5 27'),
 ...tube('hose',(t,s)=>{const by=NOZ[1]+nozLift(t,s);return `M16.5 17 C21 17 24 ${by+2.5} 24 ${by}`;}),
 ...posed(tube('nozzle','M24 15 L24 10 L26 10'),rigid((t,s)=>({angle:nozTilt(t,s),dy:nozLift(t,s),pivot:NOZ})))
],.65,.62);

// ---------- 15. EV CHARGER ----------
// A charging post with a cable to a connector; plugging in lights the bolt on its face. Solid keeps the post as a
// ring so the bolt stays visible. The bolt fades in at 80% scale and never scales from zero.
// Post x9–22 leaves the bolt (x13.3–17.7, y8.8–21.5) ≥1.4 clear inside at Bold, ≥1 at its 110% pop.
const boltD=polygon([[16.6,8.8],[13.3,15.6],[15.6,15.6],[14.4,21.5],[17.7,14.4],[15.4,14.4]],.5);
const boltShow=dir(P([[0,0],[.04,0],[.3,1,'out']],[[0,0],[.04,0],[.12,1,'out'],[.2,1],[.26,.35],[.32,1]]),P([[0,1],[.35,0,'in']],[[0,1],[.06,.3],[.1,1],[.24,0,'in']]));
const boltK=dir(F([[0,.8],[.04,.8],[.3,1,'out']],[[0,.5],[.04,.5,'hold'],[.16,1.1,'hit'],[.28,.96],[.38,1]]),F([[0,1],[.35,.8,'in']],[[0,1],[.06,1.08],[.24,.5]]));
const evCharger=toggle('ev-charger','EV charger',
 'Light: the bolt on the charger face fades in and settles to full size. Strong: it pops in past full size, flickers once and holds; reverse fades it out.',[
 ...framed('post',rect(9,4.5,13,23,2.5),holes(rect(7.7,3.2,15.6,25.6,3.8),rect(10.3,5.8,10.4,20.4,1.2))),
 ...tube('base','M6.5 27.5 L24.5 27.5'),
 ...tube('cable','M22 11 C25.8 11 26.2 15 26.2 19.5'),...bothFill('connector',circle(26.2,21,1.5)),
 ...posed(shown(framed('bolt',boltD),boltShow),(x,y,t,s,r)=>{const k=boltK(t,s,r);return [15.5+(x-15.5)*k,15+(y-15)*k];})
],.32,.4,'idle','charging',.35,.3);

// ---------- 16. TRANSIT STOP ----------
// A sign on a post with a filled bus mark inside; the sign swings about the post top as a bus passes.
const signD=rect(7.5,4.6,17,12,2);
const swing=F([[0,0],[.05,0],[.3,4,'smooth'],[.7,0,'smooth']],[[0,0],[.06,-2],[.18,8,'hit'],[.34,-5],[.48,2.5],[.6,-1],[.7,0]]);
const busMark=[rect(11.5,8,9,4.8,1.2),circle(13.8,13.3,1),circle(18.2,13.3,1)];
const transitStop=make('transit-stop','Transit stop',
 'Light: the stop sign swings 4° on its post and settles. Strong: it is kicked 8° and rocks back and forth with a damped settle.',[
 ...posed([...framed('sign',signD,holes(rect(6.2,3.3,19.6,14.6,3.3),rect(8.8,5.9,14.4,9.4,.8))),
  ...busMark.flatMap((d,i)=>bothFill('bus-'+i,d))],rigid((t,s)=>({angle:swing(t,s),pivot:[16,16.5]}))),
 ...tube('post','M16 16.5 L16 27.5'),...tube('ground','M8 27.5 L24 27.5')
],.7,.7);

// ---------- 17. WALKING ROUTE ----------
// Four alternating footprints (4 × 6.4 capsules, tilted ±12°) climb the icon; the first is present and three more
// are stamped in order. Strong stamps each from 125% with a short settle. Result retained; preview reset rearms.
const printD=i=>{const x=i%2?19.8:12.2,y=24-i*5.3,a=rad(i%2?12:-12);return capsule([x-1.2*Math.sin(a),y+1.2*Math.cos(a)],[x+1.2*Math.sin(a),y-1.2*Math.cos(a)],4);};
const stampShow=i=>P([[0,0],[.05+.18*(i-1),0],[.2+.18*(i-1),1,'out']],[[0,0],[.03+.12*(i-1),0],[.09+.12*(i-1),1,'out']]);
const stampK=i=>F([[0,1],[.6,1]],[[0,1.25],[.03+.12*(i-1),1.25,'hold'],[.11+.12*(i-1),.96,'hit'],[.19+.12*(i-1),1]]);
const walkingRoute=make('walking-route','Walking route',
 'Light: three footprints appear one after another up the route. Strong: each is stamped down from 125% with a short settle, faster. The trail is retained; preview reset rearms.',[
 ...framed('print-0',printD(0)),
 ...[1,2,3].flatMap(i=>{const x=i%2?19.8:12.2,y=24-i*5.3,k=stampK(i);return posed(shown(framed('print-'+i,printD(i)),stampShow(i)),(px,py,t,s)=>{const q=k(t,s);return [x+(px-x)*q,y+(py-y)*q];});})
],.6,.5,['print-1','print-2','print-3']);

// ---------- 18. ROUNDABOUT ----------
// The roundabout road sign: three curved arrows (88° arcs on r6.5, 32° apart) chase each other around a filled
// central island. Turning one third of a revolution lands exactly on the same drawing (a declared 120° cycle).
// The arcs are open, so Duotone intentionally matches Outline. Strong winds back and whips past before settling.
const RB=[16,16];
const turn=F([[0,0],[.05,0],[.7,120,'smooth']],[[0,0],[.06,-12],[.34,128,'hit'],[.48,118],[.58,120]]);
const headD=a=>`M${pt(...RB,8.5,a-16)} L${pt(...RB,6.5,a)} L${pt(...RB,4.5,a-16)}`;
const roundabout=make('roundabout','Roundabout',
 'Light: the three arrows turn one third of the way round the island. Strong: they wind back, whip past the next position and settle on it.',[
 ...bothFill('island',circle(...RB,2.2)),
 ...posed([0,1,2].flatMap(i=>{const base=-90+120*i;return [...tube('arc-'+i,arcN(...RB,6.5,base-44,base+44,2)),...tube('head-'+i,headD(base+44))];}),rigid((t,s)=>({angle:turn(t,s),pivot:RB})))
],.7,.58);
roundabout.cycleDegrees=120;

// ---------- 19. OFFLINE MAP ----------
// A folded map (two folds) with a download arrow on its middle panel. Downloading drives the arrow into the map,
// then a check draws on in its place; Strong hits harder and the map jolts. Result retained; preview reset rearms.
const mapD=polygon([[4.5,7.6],[9.5,5],[22.5,7.6],[27.5,5],[27.5,24.4],[22.5,27],[9.5,24.4],[4.5,27]],.8);
const arrowDrop=F([[0,0],[.05,0],[.4,3,'in']],[[0,0],[.06,-.7],[.22,3.2,'hit'],[.3,3.2]]);
const arrowShow=P([[0,1],[.2,1],[.42,0,'in']],[[0,1],[.12,1],[.24,0,'in']]);
const checkP=F([[0,0],[.38,0],[.7,1,'smooth']],[[0,0],[.22,0,'hold'],[.42,1,'hit']]);
const jolt=F([[0,0],[.7,0]],[[0,0],[.22,0,'hold'],[.28,.5,'hit'],[.4,0]]);
const CA=[13.2,16.2],CB=[15.2,18.2],CC=[19,13.6];
const checkD=(t,s)=>{const c=Math.max(.02,checkP(t,s));return `M${CA} L${lerp(CA,CB,Math.min(1,c*2))} L${lerp(CB,CC,clamp(c*2-1))}`;};
const mapPose=rigid((t,s)=>({dy:jolt(t,s)}));
const offlineMap=make('offline-map','Offline map',
 'Light: the download arrow sinks into the map and fades as a check draws on. Strong: the arrow lifts, drives in hard, the map jolts and the check whips on. The saved state is retained; preview reset rearms.',[
 ...posed([fill('map-interior',mapD,'tint'),stroke('map-frame',mapD),{name:'map-solid',d:mapD,style:'solid',type:'stroke',width:2.6},
  ...tube('fold-l','M9.5 5 L9.5 24.4'),...tube('fold-r','M22.5 7.6 L22.5 27')],mapPose),
 ...posed(shown([...tube('stem','M16 10.5 L16 19'),...tube('head','M13.6 16.6 L16 19 L18.4 16.6')],arrowShow),rigid((t,s)=>({dy:arrowDrop(t,s)}))),
 ...shown(tube('check',checkD),(t,s)=>checkP(t,s)>.02?1:0)
],.7,.5,['stem','head','check']);

// ---------- 20. TOLL GATE ----------
// A barrier post on the ground with a 15.5-unit arm hinged at the post's top corner. Opening raises the arm 55°;
// Strong dips first and overshoots to 63°. Reverse lowers it; Strong drops it onto the stop with one rebound.
const armA=dir(F([[0,0],[.05,0],[.55,-55,'smooth']],[[0,0],[.06,4],[.26,-63,'hit'],[.4,-52],[.5,-55]]),
 F([[0,-55],[.5,0,'smooth']],[[0,-55],[.05,-58],[.24,0,'hit'],[.32,-6],[.42,0]]));
const tollGate=toggle('toll-gate','Toll gate',
 'Light: the barrier arm rises 55° about its hinge. Strong: it dips, swings up past the open position and settles; reverse lowers it, and Strong drops it onto the stop with one rebound.',[
 ...framed('post',rect(5,16,6,11.5,1.5)),
 ...tube('ground','M4 27.5 L28 27.5'),
 ...posed(tube('arm','M10 17.5 L25.5 17.5'),rigid((t,s,r)=>({angle:armA(t,s,r),pivot:[11,17.5]})))
],.55,.5,'closed','open',.5,.42);

const all=[pinCluster,currentLocation,directions,turnByTurn,routeRecalculate,trafficLight,mapLayers,satellite,streetView,geofence,distance,elevation,parking,fuelStation,evCharger,transitStop,walkingRoute,roundabout,offlineMap,tollGate];
for(const icon of all)icon.duotoneFill=icon.parts.some(p=>p.style==='tint');
// Design constraints checked by verify.mjs on exported world geometry, independently of the drivers.
const fixed={'pin-cluster':[],'current-location':['ring'],'turn-by-turn':['outer','inner'],'route-recalculate':['start'],'traffic-light':['housing-frame'],'map-layers':[],'street-view':['street'],'distance':['pin-l-frame','tick-l'],'elevation':['mountain-frame'],'parking':['bay'],'fuel-station':['body-frame','base'],'ev-charger':['post-frame','cable'],'transit-stop':['post','ground'],'walking-route':['print-0-frame'],'roundabout':['island'],'toll-gate':['post-frame','ground']};
const rigidParts={'pin-cluster':['pin-frame','side-l-frame','side-r-frame'],'directions':['pointer-frame'],'turn-by-turn':['pointer-frame'],'map-layers':['top-frame','layer-2'],'satellite':['body-frame','panel-l-frame','signal-1'],'street-view':['torso-frame','leg-l'],'parking':['car-frame'],'fuel-station':['nozzle'],'transit-stop':['sign-frame'],'roundabout':['head-0','arc-1'],'toll-gate':['arm']};
export const icons=all.map((icon,index)=>({...icon,number:666+index,constraints:[...(fixed[icon.name]||[]).map(p=>({type:'fixed',part:icon.name+'-'+p})),...(rigidParts[icon.name]||[]).map(p=>({type:'rigid',part:icon.name+'-'+p}))]}));
