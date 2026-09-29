// Weather batch A: rain, snowflake.
import {make,F,P,tube,framed,rigid,posed,shown} from './helpers.mjs';
import {HIGH_CLOUD,pt} from './sky.mjs';

// ---------- RAIN ----------
// Cause: the cloud wrings (Strong) → three streaks fall and fade → they recycle while invisible → rest.
// Streak paint starts 1.1 below the cloud's painted bottom at Bold.
const STREAKS=[[11.6,0],[16.5,.07],[21.4,.14]];
// Each streak falls, fades out, snaps home inside its zero-opacity window, then fades back in.
const rainFall=(d)=>P([[0,0],[d,0],[d+.28,2.6,'in'],[d+.3,2.6],[d+.31,0,'linear']],[[0,0],[.1+d*.8,0],[.1+d*.8+.2,4.4,'in'],[.1+d*.8+.22,4.4],[.1+d*.8+.23,0,'linear']]);
const rainShow=(d)=>P([[0,1],[d+.12,1],[d+.28,0,'in'],[d+.34,0],[d+.46,1,'out']],[[0,1],[.1+d*.8+.08,1],[.1+d*.8+.2,0,'in'],[.1+d*.8+.26,0],[.1+d*.8+.4,1,'out']]);
// The cloud lifts and returns without dipping below rest, keeping the streak gap ≥1.
const wring=F([[0,0],[.5,0]],[[0,0],[.1,1],[.36,0]]);
const cloudPose=rigid((t,s)=>({dy:-.6*wring(t,s)}));
const rain=make('rain','Rain',
 'Light: three streaks fall 2.6 units in a short stagger and fade, then reappear under the cloud. Strong: the cloud lifts as it wrings, the streaks shoot 4.4 units in a tight burst, and the cloud settles back.',[
 ...posed(framed('cloud',HIGH_CLOUD),cloudPose),
 ...STREAKS.flatMap(([x,d],i)=>{const fall=rainFall(d),show=rainShow(d);
   const pose=rigid((t,s)=>({dx:-.35*fall(t,s),dy:fall(t,s)}));
   return posed(shown(tube('streak-'+i,`M${x} 20.4 L${x-1} 23.4`),show),pose);})
],.62,.66);
rain.duotoneFill=true;

// ---------- SNOWFLAKE ----------
// Six-fold rigid flake. Light turns 60° (next equivalent pose); Strong winds back 10°, spins 126° and settles at 120°.
const turn=F([[0,0],[.08,0],[.56,60,'smooth']],[[0,0],[.09,-10],[.42,126],[.6,120]]);
const flakePose=rigid((t,s)=>({angle:turn(t,s),pivot:[16,16]}));
const arms=[0,60,120].map((a,i)=>tube('arm-'+i,`M${pt(a,10.5)} L${pt(a+180,10.5)}`));
// Branch vertex at r5.4, legs 2.9 at ±48°: Bold painted gaps ≈1.1 to the neighbour's leg and ≈1.3 to the arm tip.
// (v1 at r6.4/±42° fused each arm end into a blob at Bold.)
const branches=[0,60,120,180,240,300].map((a,i)=>{const p=pt(a,5.4),l=pt(a-48,2.9,p),r=pt(a+48,2.9,p);return tube('branch-'+i,`M${l} L${p} L${r}`);});
const snowflake=make('snowflake','Snowflake',
 'Light: the rigid flake turns 60° to its next equivalent pose. Strong: it winds back 10°, spins through 126° and settles on the 120° pose with one short return.',
 posed([...arms.flat(),...branches.flat()],flakePose),.56,.6);
snowflake.duotoneFill=false;snowflake.cycleDegrees=60;

export const icons=[rain,snowflake];
