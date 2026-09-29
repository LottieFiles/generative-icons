// scheduling-tools-a.mjs — alarm-clock (revision 03: Strong body rock composed with bell strike)
import {make,F,tube,circle,arc,fill,posed,rigid,stroke} from './helpers.mjs';

// ── Drivers ──────────────────────────────────────────────────────────────────
// Bell strike about the true rim pivots. Light is unchanged from revision 02.
const ring=F([[0,0],[.1,-6],[.42,0,'smooth']],
             [[0,0],[.07,5],[.17,-20,'hit'],[.29,13],[.41,-7],[.53,3],[.68,0]]);
// Body rock (degrees, Lottie sign: + = top leans right). Light is exactly 0.
// Small opposite prep, one hard tip synchronized with the strike, one undershoot, rest at .66.
const rock=F([[0,0],[.45,0]],
             [[0,0],[.07,-1.3],[.17,5.5,'hit'],[.33,-1.7],[.48,.5],[.66,0]]);

// ── Rigid composition (body ∘ local), constant native anchors ────────────────
const FOOT_L=[10.6,27.6],FOOT_R=[21.4,27.6],MID=[16,27.6],RAD=Math.PI/180;
// The clock tips on whichever foot is on the lean side; at θ=0 both give zero offset,
// so the switch is position-continuous.
const ground=th=>th>0?FOOT_R:th<0?FOOT_L:MID;
// Part with native anchor P: first local rotation φ about P, then body rotation θ
// about the grounded foot B.  R_B(θ)·R_P(φ) = pivot P, angle θ+φ, offset (R(θ)-I)(P-B).
const carry=(th,P,phi=0)=>{
 const [bx,by]=ground(th),c=Math.cos(th*RAD),s=Math.sin(th*RAD),ux=P[0]-bx,uy=P[1]-by;
 return {pivot:P,angle:th+phi,dx:ux*c-uy*s-ux,dy:ux*s+uy*c-uy};
};
const body=rigid((t,s)=>carry(rock(t,s),MID));                       // whole assembly, anchor MID
const bellPose=(pivot,sign)=>rigid((t,s)=>carry(rock(t,s),pivot,sign*ring(t,s)));

// ── Geometry (unchanged from revision 02) ────────────────────────────────────
const bellD=(cx,cy,a,b)=>arc(cx,cy,2.2,a,b)+' Z';
const bell=(n,d,pivot,sign)=>posed(
 [stroke(n,d),{name:n+'-solid',d,style:'solid',type:'stroke',width:2.6},fill(n+'-solid-core',d,'solid')],
 bellPose(pivot,sign));                                                // posed exactly once per part

export const alarmClock=make('alarm-clock','Alarm clock',
 'Light: one restrained outward strike of both bells. Strong: a small inward wind-up, a hard outward strike, and the whole clock tips 5.5° on its foot with the hit, undershoots once on the other foot and settles while the bells ring down.',[
 ...posed(tube('body',circle(16,17.5,8.5)),body),
 ...posed([fill('dial',circle(16,17.5,8.5),'tint')],body),
 ...posed(tube('minute','M16 17.5 L16 13'),body),
 ...posed(tube('hour','M16 17.5 L19.3 17.5'),body),
 ...posed(tube('leg-l','M12.2 25.2 L10.6 27.6'),body),
 ...posed(tube('leg-r','M19.8 25.2 L21.4 27.6'),body),
 ...bell('bell-l',bellD(9.57,11.07,135,315),[10,11.5],1),
 ...bell('bell-r',bellD(22.43,11.07,225,405),[22,11.5],-1)
],.45,.7);
alarmClock.duotoneFill=true;
export const icons=[alarmClock];
