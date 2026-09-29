import svgpath from 'svgpath';
// Smart home batch B: ceiling-fan, air-conditioner, smart-speaker, video-doorbell, security-camera.
import {make,F,P,tube,framed,rigid,posed,shown,stroke,fill,circle,rect,bothFill,annulus} from './helpers.mjs';
import {arcN,pt,ringRect} from './sky.mjs';

// ---------- CEILING FAN ----------
// Side elevation (v3): ceiling line, downrod, motor housing, two long blades and a light bowl. Spinning is
// shown by foreshortening: each blade's reach from the motor is 8.2·|cos φ| (hidden when edge-on), so the
// blades shrink and flip back as φ turns. Strong spins faster and the fan wobbles on its rod.
const phi=F([[0,0],[.06,0],[.7,180,'smooth']],[[0,0],[.06,-12],[.62,552],[.76,540]]);
const reach=(t,s)=>8.2*Math.abs(Math.cos(phi(t,s)*Math.PI/180));
const wob=(t,s)=>{if(!s)return 0;const u=t-.1;return u<=0||u>=.66?0:3*Math.sin(u*24)*(1-u/.66)**1.5;};
const fanPose=rigid((t,s)=>({angle:wob(t,s),pivot:[16,7]}));
const ceilingFan=make('ceiling-fan','Ceiling fan',
 'Light: the blades turn half a revolution, foreshortening to edge-on and swinging back out. Strong: they wind back, spin a turn and a half, and the fan wobbles on its downrod as it spins.',[
 ...tube('ceiling','M11 7 L21 7'),
 ...posed([...tube('rod','M16 7 L16 13'),...framed('motor',rect(12,13,8,5,2.3)),...framed('bowl',`M13 18 L19 18 C19 19.66 17.66 21 16 21 C14.34 21 13 19.66 13 18 Z`)],fanPose),
 ...shown(posed(tube('blade-l',(t,s)=>`M12 14.6 L${12-Math.max(.02,reach(t,s))} 14.6`),fanPose),(t,s)=>reach(t,s)>.4?1:0),
 ...shown(posed(tube('blade-r',(t,s)=>`M20 14.6 L${20+Math.max(.02,reach(t,s))} 14.6`),fanPose),(t,s)=>reach(t,s)>.4?1:0)
],.7,.76);
ceilingFan.duotoneFill=true;

// ---------- AIR CONDITIONER ----------
// Wall unit with a vent line; three wavy airflow strokes below fall, fade and recycle while invisible.
const FLOWS=[[10,0],[16,.07],[22,.14]];
const flowY=d=>P([[0,0],[d,0],[d+.32,2.2,'in'],[d+.34,2.2],[d+.35,0,'linear']],[[0,0],[.08+d*.6,0],[.08+d*.6+.2,3.2,'in'],[.08+d*.6+.22,3.2],[.08+d*.6+.23,0,'linear']]);
const flowV=d=>P([[0,1],[d+.16,1],[d+.32,0,'in'],[d+.38,0],[d+.52,1,'out']],[[0,1],[.08+d*.6+.08,1],[.08+d*.6+.2,0,'in'],[.08+d*.6+.26,0],[.08+d*.6+.4,1,'out']]);
const louver=F([[0,0],[.66,0]],[[0,0],[.04,.2],[.12,-.15],[.2,0]]);
const airConditioner=make('air-conditioner','Air conditioner',
 'Light: three airflow waves drift 2.2 units down from the vent in a stagger, fade and return. Strong: the vent kicks open and the waves blast 3.2 units down in a tight burst.',[
 ...framed('unit',rect(4.5,6.5,23,11,3),ringRect(4.5,6.5,23,11,3)),
 ...posed(tube('vent','M8.5 13.8 L23.5 13.8'),rigid((t,s)=>({dy:louver(t,s)}))),
 ...FLOWS.flatMap(([x,d],i)=>posed(shown(tube('flow-'+i,`M${x} 21 C${x+1.1} 22 ${x-1.1} 23.5 ${x} 24.5`),flowV(d)),rigid((t,s)=>({dy:flowY(d)(t,s)}))))
],.66,.6);
airConditioner.duotoneFill=true;

// ---------- SMART SPEAKER ----------
// Squat cylinder in side view (v3): an elliptical top edge that is the light ring, a curved base, and two
// sound arcs rising above it. Deliberately wider than tall so it cannot be confused with the doorbell.
const TOP='M6 17.5 C6 16.06 10.48 14.9 16 14.9 C21.52 14.9 26 16.06 26 17.5 C26 18.94 21.52 20.1 16 20.1 C10.48 20.1 6 18.94 6 17.5 Z';
const WALL='M6 17.5 L6 24 C6 26.3 10.5 27.8 16 27.8 C21.5 27.8 26 26.3 26 24 L26 17.5';
const BODY='M6 17.5 C6 18.94 10.48 20.1 16 20.1 C21.52 20.1 26 18.94 26 17.5 L26 24 C26 26.3 21.5 27.8 16 27.8 C10.5 27.8 6 26.3 6 24 Z';
const wave=F([[0,0],[.1,.8],[.42,0,'smooth']],[[0,0],[.16,1.4],[.3,.4],[.42,.9],[.56,0]]);
const bob=F([[0,1],[.5,1]],[[0,1],[.06,.94],[.18,1.03],[.3,1]]);
const flick=P([[0,1],[.08,.3],[.2,1]],[[0,1],[.05,.2],[.12,1],[.2,.25],[.3,1]]);
const squat=(x,y,t,s)=>{const k=bob(t,s);return[16+(x-16)*(2-k)**.5,27.8+(y-27.8)*k];};
const smartSpeaker=make('smart-speaker','Smart speaker',
 'Light: the light ring on top flickers and the two sound arcs above swell 0.8 units and return. Strong: the speaker squats and springs as it speaks, the ring flashes twice and the arcs push out 1.4 units with a second pulse.',[
 ...posed(shown([stroke('ring',TOP),fill('ring-interior',TOP,'tint'),{name:'ring-solid',d:TOP,style:'solid',type:'stroke',width:2.6}],flick),squat),
 ...posed([stroke('wall',WALL),fill('body-solid',BODY,'solid')],squat),
 ...[5.4,9].flatMap((r,k)=>tube('arc-'+k,(t,s)=>arcN(16,16,r+wave(t,s),240,300,2)))
],.42,.56);
smartSpeaker.duotoneFill=true;

// ---------- VIDEO DOORBELL ----------
// Tall narrow pill (v3): a camera lens (ring with a centre dot) at the top and a large ring button at the
// bottom. A press scales the button; Strong presses deeper and the doorbell rocks on its base as it rings.
const press=F([[0,1],[.08,.84],[.3,1,'smooth']],[[0,1],[.14,.7,'hit'],[.26,1.02],[.36,1]]);
const ring=F([[0,0],[.3,0]],[[0,0],[.14,0],[.22,4],[.32,-3],[.42,1.5],[.52,0]]);
const bellPose=rigid((t,s)=>({angle:ring(t,s),pivot:[16,28.2]}));
const videoDoorbell=make('video-doorbell','Video doorbell',
 'Light: the ring button presses in to 84% and springs back. Strong: a deeper press to 70% with a snap back, and the doorbell rocks 4° on its base as it rings.',[
 ...posed(framed('body',rect(9.7,3.8,12.6,24.4,6.3),ringRect(9.7,3.8,12.6,24.4,6.3)),bellPose),
 ...posed(tube('lens',circle(16,10.1,2.5)),bellPose),
 ...posed(bothFill('pupil',circle(16,10.1,.55)),bellPose),
 ...posed(framed('button',circle(16,21.6,3.2),annulus(16,21.6,3.2,2.6)),(x,y,t,s)=>{const k=press(t,s);return bellPose(16+(x-16)*k,21.6+(y-21.6)*k,t,s);})
],.3,.52);
videoDoorbell.duotoneFill=true;

// ---------- SECURITY CAMERA ----------
// Bullet camera (v3) angled 14° down toward its front, a lens disc at the front end, a round joint under the
// rear, and a short straight arm to a small wall plate. The body tilts about the joint. (v1/v2 read as "4".)
const JOINT=[18,16.5];
const camBody=svgpath(rect(6,7.5,14,9,3)).rotate(-14,...JOINT).toString();
const lensAt=(()=>{const a=-14*Math.PI/180,dx=10-18,dy=12-16.5;return [18+dx*Math.cos(a)-dy*Math.sin(a),16.5+dx*Math.sin(a)+dy*Math.cos(a)];})();
const tilt=F([[0,0],[.08,-6],[.3,4],[.5,0,'smooth']],[[0,0],[.08,3],[.22,-10],[.34,-10],[.52,9],[.66,-3],[.78,0]]);
const camPose=rigid((t,s)=>({angle:tilt(t,s),pivot:JOINT}));
const securityCamera=make('security-camera','Security camera',
 'Light: the camera tilts 6° down, scans up 4° and returns. Strong: it dips 10°, holds on the view, sweeps 9° up and settles with one small return.',[
 ...tube('plate','M26.5 16.6 L26.5 24.4'),
 ...tube('arm',`M${JOINT} L26.5 20.5`),
 ...posed(framed('body',camBody,ringRect(6,7.5,14,9,3)),camPose),
 ...posed(bothFill('lens',circle(...lensAt,1.6)),camPose),
 ...bothFill('joint',circle(...JOINT,1.4))
],.5,.78);
for(const p of securityCamera.parts.filter(p=>p.name==='security-camera-body-solid'))p.d=svgpath(p.d).rotate(-14,...JOINT).toString();
securityCamera.duotoneFill=true;

export const icons=[ceilingFan,airConditioner,smartSpeaker,videoDoorbell,securityCamera];
