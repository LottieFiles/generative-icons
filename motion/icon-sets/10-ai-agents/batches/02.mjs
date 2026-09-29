// Claude Code design; Codex integration and verification.
import {make,F as P,posed,segment,framed,line,tube,bothFill,fill,stroke,circle,rect,capsule,rigid,rotate,rearmTime} from '../../library/ai-authoring.mjs';
const rot=(p,a,c)=>rotate(p[0],p[1],a,c[0],c[1]);
const tubeSplit=(n,dO,dS)=>[stroke(n,dO),{name:n+'-solid',d:dS,style:'solid',type:'stroke',width:2.6}];

// 1 agent-handoff: sender leans, bead rides the bus, receiver nods. Bead retained.
const aA=P([[0,0]],[[0,0],[.08,-1.5],[.24,5],[.52,-.8],[.64,0]]);
const aB=P([[0,0]],[[0,0],[.42,0],[.52,-4],[.66,1],[.78,0]]);
const beadX=P([[0,0],[.45,6,'io']],[[0,0],[.14,0],[.42,6,'hit']]);
const unit=(n,x,ang)=>posed([...framed(n+'-body',rect(x,17,6,10,2.5)),...line(n+'-mast',[x+3,17],[x+3,12]),...bothFill(n+'-tip',circle(x+3,10.5,2))],rigid((t,s)=>({angle:ang(t,s),pivot:[x+3,27]})));
const handoff=make('agent-handoff','Agent handoff','The sender unit leans and pushes the payload bead along the bus; the receiver nods on receipt and keeps the payload.',[
 ...unit('a',4,aA),...unit('b',22,aB),
 ...tube('bus',(t,s)=>segment(rot([10,22],aA(t,s),[7,27]),rot([22,22],aB(t,s),[25,27]))),
 ...posed(bothFill('bead',circle(13,22,2.5)),rigid((t,s)=>({dx:beadX(t,s)})))
],.6,1,['bead']);

// 2 approval-step: token waits, gate bar lifts, token proceeds. Gate and token retained.
const gateY=P([[0,23.5],[.25,12,'out']],[[0,23.5],[.08,23.5],[.14,24.2],[.34,11.3],[.46,12.3],[.56,12]]);
const tokX=P([[0,0],[.18,0],[.55,14,'io']],[[0,0],[.1,.45],[.22,.4],[.50,14.5],[.62,14]]);
const approval=make('approval-step','Approval step','A request token waits at a closed gate; approval lifts the bar and the token passes through. Open gate and passed token stay.',[
 ...line('beam',[6,8],[26,8]),
 ...tube('gate',(t,s)=>`M16 8 L16 ${gateY(t,s)}`),
 ...posed(framed('token',circle(9,20,3.25)),rigid((t,s)=>({dx:tokX(t,s)})))
],.6,1,['gate','token']);

// 3 guardrails: disc drifts, rail flexes only after exact contact, disc returns. Symmetric.
const gx=P([[0,0],[.26,6,'in'],[.6,0,'out']],[[0,0],[.2,-6.25,'hit'],[.5,6.25,'hit'],[.76,-1.2],[.9,0]]);
const rail=(n,x,sg)=>{const f=(t,s,o)=>{const c=x+sg*Math.max(0,sg*gx(t,s)-6.5+o)*4/3;return `M${x} 5 C${c} 16 ${c} 16 ${x} 27`};return tubeSplit(n,(t,s,r,w=2)=>f(t,s,w),(t,s)=>f(t,s,1.3))};
const guardrails=make('guardrails','Guardrails','An agent disc drifts off lane, meets the rail exactly, the rail gives, and the disc is steered back to a clean lane.',[
 ...rail('rail-l',5.5,-1),...rail('rail-r',26.5,1),
 ...line('lane-top',[16,5],[16,8.5]),...line('lane-bottom',[16,23.5],[16,27]),
 ...posed(framed('disc',circle(16,16,4)),rigid((t,s)=>({dx:gx(t,s)})))
],.6,1);
Object.assign(guardrails,{symmetricCycle:true});

// 4 model-router: arm on hub pivot sweeps to the top port and stays set.
const armA=P([[0,0],[.4,-33,'out']],[[0,0],[.1,7],[.36,-37],[.48,-31.5],[.58,-33]]);
const port=(n,a)=>bothFill(n,circle(12+13*Math.cos(a*Math.PI/180),16+13*Math.sin(a*Math.PI/180),2.25));
const router=make('model-router','Model router','The switch arm pivots on its hub from the middle port to the selected top port and holds that setting.',[
 ...line('input',[4,16],[7.75,16]),...framed('hub',circle(12,16,3.5)),
 ...port('port-top',-33),...port('port-mid',0),...port('port-bottom',33),
 ...posed([stroke('arm',segment([16.25,16],[23.25,16])),fill('arm-solid',capsule([12,16],[23.25,16],2.6),'solid')],rigid((t,s)=>({angle:armA(t,s),pivot:[12,16]})))
],.55,.9,['arm']);

// 5 model-training: ball follows the bowl's exact inner offset to the minimum. Minimum retained.
const BL=[[5.5,6],[5.5,21],[10,26],[16,26]];
const seat=(q,o)=>{const u=1-Math.abs(q),v=1-u,[a,b,c,d]=BL,x=v*v*v*a[0]+3*v*v*u*b[0]+3*v*u*u*c[0]+u*u*u*d[0],y=v*v*v*a[1]+3*v*v*u*b[1]+3*v*u*u*c[1]+u*u*u*d[1],tx=3*v*v*(b[0]-a[0])+6*v*u*(c[0]-b[0])+3*u*u*(d[0]-c[0]),ty=3*v*v*(b[1]-a[1])+6*v*u*(c[1]-b[1])+3*u*u*(d[1]-c[1]),l=Math.hypot(tx,ty),px=x+ty/l*o,py=y-tx/l*o;return[q>0?32-px:px,py]};
const bq=P([[0,-.7],[.55,0,'io']],[[0,-.7],[.1,-.78],[.4,0,'pass'],[.56,.45],[.72,-.2],[.86,.08],[1,0]]);
const ballD=o=>(t,s,r,w=2)=>{const [x,y]=seat(bq(t,s),3.5+(o??w/2));return circle(x,y,3.5)};
const training=make('model-training','Model training','The ball rolls down the loss bowl on its inner contour and settles at the minimum; Strong overshoots the far slope and searches back. The minimum stays.',[
 ...tube('bowl','M5.5 6 C5.5 21 10 26 16 26 C22 26 26.5 21 26.5 6'),
 fill('ball',ballD(null),'outline'),fill('ball-solid',ballD(1.3),'solid')
],.65,1,['ball']);
Object.assign(training,{duotoneFill:false});
// Preserve the selected native stroke weight while resetting the ball.
for (const part of training.parts.filter(p=>p.name==='model-training-ball'||p.name==='model-training-ball-solid')) {
 const shape=ballD(part.style==='solid'?1.3:null);
 part.d=(t,s,r,w)=>shape(rearmTime(t,s,r,2.4),s,false,w);
}

export const icons=[handoff,approval,guardrails,router,training];
