// motion/icon-sets/23-get-help/batches/get-help.mjs — Claude revision 03 after supervisor decision
import {make,stroke,fill,tube,framed,circle,rect,polygon,holes,capsule,arc,annulus,rotate,rigid,posed,shown,F,geometricRearm} from './helpers.mjs';

// ---------- 1. support-agent: head + headset nods as one rigid assembly; mic follows its hinge ----------
const NECK=[16,19.5],HINGE=[23.4,13];
const nod=F([[0,0],[.1,0],[.45,7],[.9,0,'out']],[[0,0],[.06,0],[.16,4],[.34,-16],[.6,2],[.85,0]]);
const mic=F([[0,0]],[[0,0],[.34,0,'hit'],[.46,-7],[.62,2],[.8,0]]);
const headPose=rigid((t,s)=>({angle:nod(t,s),pivot:NECK}));
const boomPose=rigid((t,s)=>{const angle=nod(t,s)+mic(t,s),a=rotate(...HINGE,nod(t,s),...NECK),b=rotate(...HINGE,angle,...NECK);return{angle,pivot:NECK,dx:a[0]-b[0],dy:a[1]-b[1]};});
const supportAgent={...make('support-agent','Support agent','A support representative wearing a headset tilts their head to listen; Strong makes a decisive tilt with a small preparation and the boom mic swinging on its hinge.',[
 ...tube('shoulders','M6 27 C6 22.5 10 20 16 20 C22 20 26 22.5 26 27'),
 ...posed(framed('head',circle(16,11.5,3.6)),headPose),
 ...posed(tube('band',arc(16,11.5,7.6,193.2,346.8)),headPose),
 ...posed(tube('ear-left','M8.6 9.76 L8.6 13'),headPose),
 ...posed(tube('ear-right','M23.4 9.76 L23.4 13'),headPose),
 ...posed(tube('boom','M23.4 13 C23.4 14.7 22.8 16.3 22.1 16.3'),boomPose),
],.95,.9),duotoneFill:true};

// ---------- 2. help-center: lifebuoy index turn; Solid is four closed ring sectors with transparent index slots ----------
const RAD=Math.PI/180,pt=(r,a)=>`${16+r*Math.cos(a*RAD)} ${16+r*Math.sin(a*RAD)}`;
const sector=k=>{const c=-45+90*k,o=Math.asin(.9/12.3)/RAD,i=Math.asin(.9/3.2)/RAD,P=(r,a)=>[16+r*Math.cos(a*RAD),16+r*Math.sin(a*RAD)],span=(r,a,b,n)=>Array.from({length:n+1},(_,j)=>P(r,a+(b-a)*j/n));const outer=span(12.3,c-45+o,c+45-o,24),inner=span(3.2,c+45-i,c-45+i,4),v=[...outer,...inner];return polygon(v,v.map((_,j)=>j===0||j===outer.length-1||j===outer.length||j===v.length-1?.35:0));};
const turn=F([[0,0],[.1,0],[.7,90]],[[0,0],[.05,0],[.15,-12],[.44,186],[.72,180],[.9,180]]);
const ringPose=rigid((t,s)=>({angle:turn(t,s),pivot:[16,16]}));
const helpCenter={...make('help-center','Help center','A lifebuoy turns one index; Light a quiet quarter turn, Strong a wound-up half turn with one damped settle.',posed([
 fill('ring-interior',annulus(16,16,7.75,6.5),'tint'),
 stroke('outer',circle(16,16,11)),stroke('inner',circle(16,16,4.5)),
 stroke('grip-n','M16 5 L16 11.5'),stroke('grip-s','M16 20.5 L16 27'),stroke('grip-w','M5 16 L11.5 16'),stroke('grip-e','M20.5 16 L27 16'),
 ...[0,1,2,3].map(k=>fill('sector-'+k+'-solid',sector(k),'solid')),
],ringPose),.75,.95),duotoneFill:true,cycleDegrees:90};

// ---------- 3. support-ticket: closed body and closed stub; stub tears off to one common torn state ----------
const tearX=F([[0,0],[.1,0],[.55,5,'out']],[[0,0],[.05,0],[.13,0,'launch'],[.3,5.5],[.5,5],[.7,5]]);
const tearY=F([[0,0]],[[0,0],[.05,0],[.13,-.6],[.3,0],[.5,0]]);
const tearA=F([[0,0],[.1,0],[.55,6,'out']],[[0,0],[.05,0],[.13,0,'launch'],[.3,8],[.5,6],[.7,6]]);
const stubPose=rigid((t,s)=>({dx:tearX(t,s),dy:tearY(t,s),angle:tearA(t,s),pivot:[19.5,16]}));
const supportTicket=geometricRearm({...make('support-ticket','Support ticket','A ticket stub tears away from the submitted record and stays torn; Strong lifts first, rips faster and overshoots before settling on the same torn state.',[
 ...framed('body',rect(4,9,13,14,2),holes(rect(2.7,7.7,15.6,16.6,3.3),capsule([8,14],[13.5,14],2),capsule([8,18],[12,18],2))),
 stroke('line-1','M8 14 L13.5 14'),stroke('line-2','M8 18 L12 18'),
 ...posed(framed('stub',rect(17,9,5,14,2),rect(15.7,7.7,7.6,16.6,3.3)),stubPose),
],.6,.75),duotoneFill:true});

// ---------- 4. live-support: customer bubble fixed; agent reply bubble with typing dots hops toward it ----------
const hop=F([[0,0],[.1,0],[.32,-1.5],[.75,0,'out']],[[0,0],[.05,0],[.15,.8],[.34,-4],[.62,.6],[.85,0]]);
const agentPose=rigid((t,s)=>({dy:hop(t,s)}));
const customer=polygon([[3.5,5],[13.5,5],[13.5,14],[8,14],[3.5,17.5]],[2.5,2.5,2.5,1.2,.6]);
const customerSolid=polygon([[2.2,3.7],[14.8,3.7],[14.8,15.3],[8.5,15.3],[2.2,20]],[3.5,3.5,3.5,1.5,.8]);
const agent=polygon([[17.5,12],[28.5,12],[28.5,27],[24,23],[17.5,23]],[2.5,2.5,.6,1.2,2.5]);
const agentSolid=polygon([[16.2,10.7],[29.8,10.7],[29.8,29.4],[23.5,24.3],[16.2,24.3]],[3.5,3.5,.8,1.5,3.5]);
const dots=[20.5,23,25.5];
const liveSupport={...make('live-support','Live support','A live chat: the agent reply bubble with typing dots pushes up toward the customer message and settles; Strong dips first and hops higher.',[
 ...framed('customer',customer,holes(customerSolid,capsule([7.2,9.5],[9.8,9.5],2))),
 stroke('customer-line','M7.2 9.5 L9.8 9.5'),
 ...posed(framed('agent',agent,holes(agentSolid,...dots.map(x=>circle(x,17.5,.75)))),agentPose),
 ...posed(dots.flatMap((x,i)=>[fill('dot-'+i,circle(x,17.5,.72),'outline')]),agentPose),
],.8,.9),duotoneFill:true};

// ---------- 5. callback: closed handset receiver; a return arrow draws back onto it, head grows after arrival ----------
const HANDSET='M7 19 L9 19 C11.5 19 12.5 23.5 16 23.5 C19.5 23.5 20.5 19 23 19 L25 19 C26.5 19 27.5 20 27.5 21.5 L27.5 24.5 C27.5 26 26.5 27 25 27 L7 27 C5.5 27 4.5 26 4.5 24.5 L4.5 21.5 C4.5 20 5.5 19 7 19 Z';
const BEZ=[[[25,14],[25,7],[20,4.5],[14.5,4.5]],[[14.5,4.5],[11,4.5],[8.5,6.5],[8,9]],[[8,9],[8,9],[8,14.5],[8,14.5]]];
const lerp=(a,b,u)=>[a[0]+(b[0]-a[0])*u,a[1]+(b[1]-a[1])*u],J=p=>p.join(' ');
const cut=([a,b,c,d],u)=>{const ab=lerp(a,b,u),bc=lerp(b,c,u),cd=lerp(c,d,u),abc=lerp(ab,bc,u),bcd=lerp(bc,cd,u);return [a,ab,abc,lerp(abc,bcd,u)];};
const stemPath=u=>{let end=BEZ[0][0],d='M'+J(end);for(let i=0;i<3;i++){const l=Math.max(0,Math.min(1,3*u-i)),q=l>=1?BEZ[i]:l<=0?[end,end,end,end]:cut(BEZ[i],l);end=q[3];d+=' C'+J(q[1])+' '+J(q[2])+' '+J(q[3]);}return d;};
const prog=F([[0,0],[.1,0],[.7,1,'out']],[[0,0],[.05,0],[.36,1]]);
const barb=F([[0,0],[.7,0],[.85,3.4,'out']],[[0,0],[.36,0],[.46,3.4]]);
const rock=F([[0,0],[.75,0],[.92,-2],[1.1,0,'out']],[[0,0],[.4,0,'hit'],[.52,-5],[.7,1.5],[.9,0]]);
const handsetPose=rigid((t,s)=>({angle:rock(t,s),pivot:[16,25]}));
const callback=geometricRearm({...make('callback','Callback','A return arrow draws back from the right and lands on the handset; the head grows at arrival and the receiver rocks in response. The drawn arrow is retained.',[
 ...posed(framed('handset',HANDSET),handsetPose),
 ...shown(tube('stem',(t,s)=>stemPath(prog(t,s))),(t,s)=>prog(t,s)>0?1:0),
 ...shown(tube('head',(t,s)=>{const k=barb(t,s)*.7071;return `M${8-k} ${14.5-k} L8 14.5 L${8+k} ${14.5-k}`;}),(t,s)=>barb(t,s)>0?1:0),
],1.1,.95),duotoneFill:true});

export const icons=[supportAgent,helpCenter,supportTicket,liveSupport,callback];
