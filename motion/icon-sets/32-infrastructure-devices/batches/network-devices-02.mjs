// 10-infrastructure-devices / revision 02 — Ethernet, Load balancer, USB drive only.
// Router and Firewall unchanged (Codex retains). Same helpers as batch A. Draft, not live.
import {make,F,rigid,posed,tube,stroke,fill,framed,rect,circle,polygon,capsule,holes,line,geometricRearm} from './helpers.mjs';

// ---------- ETHERNET: RJ45 plug, neck widened 10..22, top y8, short contacts y12..13 ----
// Bold inner edges: neck top 9.25, neck sides 11.25/20.75. Contact paint: x14 -> 12.7..15.3, x18 -> 16.7..19.3, y 10.7..14.3.
// Gaps: top 1.45, sides 1.45, between contacts 1.4. No frame edge below the contacts (step is at y16 outside the neck).
const plugOut=polygon([[8,24],[8,16],[10,16],[10,8],[22,8],[22,16],[24,16],[24,24]],1.2);
const plugSolid=holes(
 polygon([[6.7,25.3],[6.7,14.7],[8.7,14.7],[8.7,6.7],[23.3,6.7],[23.3,14.7],[25.3,14.7],[25.3,25.3]],2.2),
 capsule([14,12],[14,13],2.6),capsule([18,12],[18,13],2.6)); // hole walls: 4.0 to neck sides, 4.0 to top
const plugDy=F([[0,0],[.25,-1.5,'smooth'],[.6,0,'smooth']],
               [[0,0],[.12,1],[.3,-4,'hit'],[.5,0]]);         // sweep: Solid top 6.7-4=2.7; cable cap 28.3+1=29.3
const ethernet=make('ethernet','Ethernet','The plug pushes up into its port; Light seats gently, Strong dips back then drives home with a decisive click.',
 posed([
  ...framed('plug',plugOut,plugSolid),
  stroke('contact-a','M14 12 L14 13'),stroke('contact-b','M18 12 L18 13'),
  ...line('cable',[16,24],[16,27]),
 ],rigid((t,s)=>({dy:plugDy(t,s)}))),.6,.5);
ethernet.duotoneFill=true;

// ---------- LOAD BALANCER: one hub, two target slots; packet rides the exact left route ----
// Hub 10..22 x 3.5..11.5 (Solid top 2.2). Packet r1.6 at (16,7.5): Bold inner 4.75/10.25 -> 1.15 each side.
// Targets 4..14 / 18..28 x 19.5..27.5 (Solid bottom 28.8). Gap between targets Bold 1.5, Solid 1.4.
// Route = branch-l centerline, continued to the target-l centre. Solid hub/targets carry r2.6 slot holes so the
// packet is visible in Solid at rest (hub) and after receipt (target-l); target-r keeps an empty slot.
const route=[[16,7.5],[16,13],[9,19.5],[9,23.5]];
const lens=route.slice(1).map((q,i)=>Math.hypot(q[0]-route[i][0],q[1]-route[i][1])),total=lens.reduce((a,b)=>a+b);
const along=p=>{let d=p*total,i=0;while(i<lens.length-1&&d>lens[i]){d-=lens[i];i++;}const a=route[i],b=route[i+1],u=d/lens[i];return[a[0]+(b[0]-a[0])*u-16,a[1]+(b[1]-a[1])*u-7.5];};
const prog=F([[0,0],[.08,0],[.8,1,'smooth'],[.9,1]],          // Light: one quiet ride, 19 units in .72s
             [[0,0],[.08,-.02],[.3,1,'hit'],[.7,1]]);          // Strong: lift .38 up in hub, fire 19 units in .22s, hit
const dip=F([[0,0]],[[0,0],[.3,0],[.37,.4,'hit'],[.55,0]]);    // target-l response starts from the contact
const loadBalancer=geometricRearm(make('load-balancer','Load balancer','The hub dispatches a packet down the left branch into a server slot; Strong coils in the hub and fires it faster, the target absorbing the hit.',[
 ...framed('hub',rect(10,3.5,12,8,2.2),holes(rect(8.7,2.2,14.6,10.6,3.5),circle(16,7.5,2.6))),
 ...line('branch-l',[16,11.5],[16,13]),...line('branch-l-diag',[16,13],[9,19.5]),
 ...line('branch-r',[16,11.5],[16,13]),...line('branch-r-diag',[16,13],[23,19.5]),
 ...posed(framed('target-l',rect(4,19.5,10,8,1.8),holes(rect(2.7,18.2,12.6,10.6,3.1),circle(9,23.5,2.6))),
          rigid((t,s)=>({dy:dip(t,s)}))),
 ...framed('target-r',rect(18,19.5,10,8,1.8),holes(rect(16.7,18.2,12.6,10.6,3.1),circle(23,23.5,2.6))),
 fill('packet',circle(16,7.5,1.6),'outline',{pose:rigid((t,s)=>{const[dx,dy]=along(prog(t,s));return{dx,dy};})}),
 fill('packet-solid',circle(16,7.5,1.6),'solid',{pose:rigid((t,s)=>{const[dx,dy]=along(prog(t,s));return{dx,dy};})}),
],.9,.7));
loadBalancer.duotoneFill=true;

// ---------- USB DRIVE: taller connector y10..22 with two round contact dots ----
// Connector 4..12 x 10..22, body 12..25 x 7.5..24.5, step 2.5 each side. Junction line at x13.
// Dots r1.1 at (7.6,13.6)/(7.6,18.4): Bold inner left 5.25 -> gap 1.25; top 11.25 -> 1.25; bottom 20.75 -> 1.25;
// between dots 2.6; to junction paint (11.75) 3.05. Solid: hole left 6.5 vs exterior 2.7 -> wall 3.8.
// Junction hole capsule 11..21 -> paint 9.7..22.3, 1.41 from the step corner (10.7,8.7).
const usbOut=polygon([[4,22],[4,10],[12,10],[12,7.5],[25,7.5],[25,24.5],[12,24.5],[12,22]],1.5);
const usbSolid=holes(polygon([[2.7,23.3],[2.7,8.7],[10.7,8.7],[10.7,6.2],[26.3,6.2],[26.3,25.8],[10.7,25.8],[10.7,23.3]],2.5),
 capsule([13,11],[13,21],2.6),circle(7.6,13.6,1.1),circle(7.6,18.4,1.1));
const usbDx=F([[0,0],[.3,1.2,'smooth'],[.7,0,'smooth']],
              [[0,0],[.12,-.6],[.28,3],[.52,-.3],[.68,0]]);  // sweep: Solid right 29.3, left 2.1
const usbDrive=make('usb-drive','USB drive','The flash drive is tugged out of its port and slides back; Strong seats inward first then pulls a full 3 units.',
 posed([
  ...framed('drive',usbOut,usbSolid),
  stroke('junction','M13 11 L13 21'),
  fill('contact-a',circle(7.6,13.6,1.1),'outline'),fill('contact-b',circle(7.6,18.4,1.1),'outline'),
 ],rigid((t,s)=>({dx:usbDx(t,s)}))),.7,.68);
usbDrive.duotoneFill=true;

export const icons=[ethernet,loadBalancer,usbDrive];
