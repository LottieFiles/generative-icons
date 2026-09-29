// 10-infrastructure-devices / revision 03 — Load balancer only. Draft, not live.
import {make,F,rigid,posed,framed,fill,line,rect,circle,holes,geometricRearm,track} from './helpers.mjs';

// Hub 10..22 x 3.5..11.5 (Solid shell 8.7..23.3 x 2.2..12.8). Targets 4..14 / 18..28 x 19.5..27.5.
// Packet r1.4; slot counters r2.8 in every Solid surface (2.5 wall to the shell top).
// Bold clearance, packet at rest in hub: inner top 4.75 vs packet top 6.1 -> 1.35. Solid slot: 1.40.
// Strong prep lifts .008 of the 19.05-unit route = .152 up: Bold 1.20, Solid 1.25 (both >= 1.15).
// Reception in target-l: Bold inner top 20.75 vs packet top 22.1 -> 1.35; sides 2.35; Solid slot 1.40.
// The dip moves slot and packet together, so reception gaps are unchanged at the .4 peak.
const route=[[16,7.5],[16,13],[9,19.5],[9,23.5]];
const lens=route.slice(1).map((q,i)=>Math.hypot(q[0]-route[i][0],q[1]-route[i][1])),total=lens.reduce((a,b)=>a+b);
const along=p=>{let d=p*total,i=0;while(i<lens.length-1&&d>lens[i]){d-=lens[i];i++;}
 const a=route[i],b=route[i+1],u=d/lens[i];return[a[0]+(b[0]-a[0])*u-16,a[1]+(b[1]-a[1])*u-7.5];};
const prog=F([[0,0],[.08,0],[.8,1,'smooth'],[.9,1]],           // Light: one quiet 19-unit ride, .08 to .8s
             [[0,0],[.08,-.008],[.3,1,'hit'],[.7,1]]);          // Strong: .15 lift, fire 19 units in .22s, hit, hold
const dip=F([[0,0]],[[0,0],[.3,0],[.37,.4,'hit'],[.55,0]]);    // receiver response begins at the contact
const seated=rigid((t,s)=>({dy:dip(t,s)}));
const carried=rigid((t,s)=>{const[dx,dy]=along(prog(t,s));return{dx,dy:dy+dip(t,s)};}); // dip is 0 until seated

const loadBalancer=geometricRearm(make('load-balancer','Load balancer',
 'The hub dispatches a packet down the shared trunk and left branch into a server slot; Strong lifts it briefly, fires it faster, and the receiving server absorbs the hit with the packet seated.',[
 ...framed('hub',rect(10,3.5,12,8,2.2),holes(rect(8.7,2.2,14.6,10.6,3.5),circle(16,7.5,2.8))),
 ...line('trunk',[16,11.5],[16,13]),
 ...line('branch-l',[16,13],[9,19.5]),
 ...line('branch-r',[16,13],[23,19.5]),
 ...posed(framed('target-l',rect(4,19.5,10,8,1.8),holes(rect(2.7,18.2,12.6,10.6,3.1),circle(9,23.5,2.8))),seated),
 ...framed('target-r',rect(18,19.5,10,8,1.8),holes(rect(16.7,18.2,12.6,10.6,3.1),circle(23,23.5,2.8))),
 fill('packet',circle(16,7.5,1.4),'outline',{pose:carried}),
 fill('packet-solid',circle(16,7.5,1.4),'solid',{pose:carried}),
],.9,.7));
loadBalancer.duotoneFill=true;

// Codex integration: a preview reset must not replay the forward receiver impact.
// The general rearm keeps the returned target still, so remove only its dip from the carried reset pose.
for(const part of loadBalancer.parts.filter(p=>p.name==='load-balancer-packet'||p.name==='load-balancer-packet-solid')){
 const sourcePose=part.pose;
 part.pose=rigid((t,strong,reverse)=>{
  const pose=sourcePose.transform(t,strong,reverse);
  if(!reverse)return pose;
  const replayTime=track([[0,loadBalancer.timings[strong?'strong':'light'].active],[.3,0,'smooth']],t);
  return {...pose,dy:pose.dy-dip(replayTime,strong)};
 });
}
export const icons=[loadBalancer];
