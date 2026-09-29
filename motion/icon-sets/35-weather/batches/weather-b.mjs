// Weather batch B: partly-cloudy, snow, thunderstorm, hail, fog.
import {make,F,P,tube,framed,rigid,posed,shown,circle,rotate,bothFill} from './helpers.mjs';
import {HIGH_CLOUD,placeCloud,pt} from './sky.mjs';

// ---------- PARTLY CLOUDY ----------
// Sun upper-left, cloud lower-right; only the five rays that face away from the cloud are drawn,
// so nothing has to be hidden behind a transparent outline. Cloud x 12.2–28.5, y 15.3–26.7.
// Sun r3.3 at (11.4,11.4): Bold gap to the cloud ≥1 through the whole gust; flared rays keep the 2-unit margin.
const SUN=[11.4,11.4];
const pcCloud=placeCloud(.72,8.74,9.4);
const gust=F([[0,0],[.12,-.2],[.46,0,'smooth']],[[0,0],[.06,.1,'launch'],[.24,-.4],[.4,-.1],[.54,-.2],[.7,0]]);
const shine=F([[0,0],[.14,.35],[.46,0,'smooth']],[[0,0],[.08,-.15,'launch'],[.26,.8],[.42,.35],[.6,0]]);
const turn=F([[0,0],[.46,0]],[[0,0],[.08,-4],[.3,16],[.46,-3],[.62,0]]);
// Rays push out along their own direction; Strong also turns the ray crown 16° about the sun and back.
const rayPose=a=>(x,y,t,s)=>{const e=shine(t,s),u=[Math.cos(a*Math.PI/180),Math.sin(a*Math.PI/180)],[rx,ry]=rotate(x+u[0]*e,y+u[1]*e,turn(t,s),...SUN);return[rx,ry];};
const partlyCloudy=make('partly-cloudy','Partly cloudy',
 'Light: the rays brighten outward while the cloud drifts slightly toward the sun and back. Strong: the ray crown flares 0.8 units and turns 16° with one return while the cloud edges in and settles.',[
 ...framed('sun',circle(...SUN,3.3)),
 ...[135,180,225,270,315].flatMap((a,i)=>posed(tube('ray-'+i,`M${pt(a,5.6,SUN)} L${pt(a,7.2,SUN)}`),rayPose(a))),
 ...posed(framed('cloud',pcCloud),rigid((t,s)=>({dx:gust(t,s)})))
],.46,.7);
partlyCloudy.duotoneFill=true;

// ---------- SNOW ----------
// Dots are 0.4-unit round-capped strokes, so their size follows the Outline weight preset.
// Each flake drifts down with a sideways sway, fades out, returns home while invisible, fades back in.
const FLAKES=[[11.2,20.6,0],[16.4,22.4,.09],[21.4,20.6,.18],[13.8,25.2,.05],[19,25.6,.14]];
const snowDrop=d=>P([[0,0],[d,0],[d+.5,2.2,'in'],[d+.52,2.2],[d+.53,0,'linear']],[[0,0],[.06+d,0],[.06+d+.42,3.0,'in'],[.06+d+.44,3.0],[.06+d+.45,0,'linear']]);
const snowShow=d=>P([[0,1],[d+.28,1],[d+.5,0,'in'],[d+.56,0],[d+.72,1,'out']],[[0,1],[.06+d+.2,1],[.06+d+.42,0,'in'],[.06+d+.48,0],[.06+d+.64,1,'out']]);
const sway=(t,s,d)=>{const u=Math.max(0,t-(s?.06+d:d));return (s?1.3:.7)*Math.sin(u*(s?15:11))*Math.exp(-u*1.2);};
const snow=make('snow','Snow',
 'Light: five flakes drift 2.2 units down with a small sway, fade and reappear under the cloud. Strong: they fall farther (3.2 units) with a wider, faster sway.',[
 ...framed('cloud',HIGH_CLOUD),
 ...FLAKES.flatMap(([x,y,d],i)=>{const drop=snowDrop(d);return posed(shown(tube('flake-'+i,`M${x-.2} ${y} L${x+.2} ${y}`),snowShow(d)),rigid((t,s)=>({dx:drop(t,s)>0?sway(t,s,d):0,dy:drop(t,s)})));})
],.9,.88);
snow.duotoneFill=true;

// ---------- THUNDERSTORM ----------
// Bolt paint starts ≥1 unit below the cloud's Bold paint. Its bottom leaves room for Strong's drop.
// v1 was a filled polygon: too small at 24 px and its counter closed at Bold. Now a single zigzag stroke,
// so its weight follows the Outline preset. Paint top clears the cloud by ≥1; Strong's drop keeps the margin.
const bolt='M18.4 20.5 L13.6 24.3 L18.4 24.3 L13.8 27.5';
const strike=F([[0,0],[.1,.5,'smooth'],[.4,0,'smooth']],[[0,0],[.06,-.3],[.14,1.1,'hit'],[.3,.2],[.44,0]]);
const flash=P([[0,1],[.08,.3],[.16,1],[.24,.4],[.36,1]],[[0,1],[.12,1],[.15,.15],[.2,1],[.25,.2],[.32,1],[.44,1]]);
const recoil=F([[0,0],[.4,0]],[[0,0],[.14,-.7,'hit'],[.3,.15],[.44,0]]);
const thunderstorm=make('thunderstorm','Thunderstorm',
 'Light: the bolt flickers twice and dips half a unit. Strong: the bolt strikes down 1.1 units with a hard double flash while the cloud recoils upward, then both settle.',[
 ...posed(framed('cloud',HIGH_CLOUD),rigid((t,s)=>({dy:recoil(t,s)}))),
 ...posed(shown(tube('bolt',bolt),flash),rigid((t,s)=>({dy:strike(t,s)})))
],.4,.44);
thunderstorm.duotoneFill=true;

// ---------- HAIL ----------
// Pellets fall onto a ground line, bounce once, fade and reappear under the cloud.
const PELLETS=[[11.4,0],[16.4,.1],[21.4,.05]];
// Pellet paint top clears the cloud's Bold paint by 1.16. At FALL the pellet's Bold paint meets the ground's.
const TOP=22,GROUND=28.3,FALL=GROUND-1.25-1.6-TOP;
const fallTo=()=>FALL;
const hailY=d=>P([[0,0],[d,0],[d+.26,fallTo(0),'in'],[d+.38,fallTo(0)-1,'out'],[d+.5,fallTo(0)-.2,'in'],[d+.52,fallTo(0)-.2],[d+.53,0,'linear']],
               [[0,0],[.04+d,0],[.04+d+.14,fallTo(1),'in'],[.04+d+.28,fallTo(1)-2.7,'out'],[.04+d+.42,fallTo(1)-.2,'in'],[.04+d+.44,fallTo(1)-.2],[.04+d+.45,0,'linear']]);
const hailShow=d=>P([[0,1],[d+.4,1],[d+.5,0,'in'],[d+.56,0],[d+.7,1,'out']],[[0,1],[.04+d+.32,1],[.04+d+.42,0,'in'],[.04+d+.48,0],[.04+d+.62,1,'out']]);
const hail=make('hail','Hail',
 'Light: three pellets drop to the ground line, rebound one unit, fade and reappear under the cloud. Strong: a harder drop in 0.14 s and a 2.7-unit rebound.',[
 ...framed('cloud',HIGH_CLOUD),
 ...tube('ground',`M8.4 ${GROUND} L23.6 ${GROUND}`),
 // v1 used ring pellets; at Bold the counters closed into dots-with-centres. Pellets are filled r1.6 marks.
 ...PELLETS.flatMap(([x,d],i)=>posed(shown(bothFill('pellet-'+i,circle(x,TOP,1.6)),hailShow(d)),rigid((t,s)=>({dy:hailY(d)(t,s)}))))
],.8,.76);
hail.duotoneFill=true;

// ---------- FOG ----------
// Fog layers below a cloud slide in opposite directions (parallax), then return.
const fogCloud=placeCloud(.74,4.05,-1.87);
// Layers 3.6 apart: Bold painted gap 1.1.
const LAYERS=[['M6 20.6 L22 20.6',1,0],['M10 24.2 L26 24.2',-1,.04],['M7 27.8 L19 27.8',1,.08]];
const slide=(dir,d)=>F([[0,0],[d,0],[d+.26,.8*dir],[d+.56,0,'smooth']],[[0,0],[d,0],[d+.06,-.35*dir,'launch'],[d+.28,2*dir],[d+.44,1.5*dir],[d+.66,0]]);
const fog=make('fog','Fog',
 'Light: the three fog layers drift 0.8 units in alternating directions and return. Strong: each layer is pulled back, slides 2 units in its direction and eases home.',[
 ...framed('cloud',fogCloud),
 ...LAYERS.flatMap(([d,dir,delay],i)=>posed(tube('layer-'+i,d),rigid((t,s)=>({dx:slide(dir,delay)(t,s)}))))
],.64,.74);
fog.duotoneFill=true;

export const icons=[partlyCloudy,snow,thunderstorm,hail,fog];
