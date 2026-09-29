// 10-infrastructure-devices / batch A — Claude first-design draft. Not live.
// Pivots/transforms are native rigid moves; all gestures return to the exact rest transform.
import {make,F,rigid,posed,shown,tube,stroke,fill,bothFill,framed,rect,circle,polygon,capsule,holes,line} from './helpers.mjs';

// ---------- ETHERNET: RJ45 plug (contacts up) seats into the port -----------------------
// Fixed: nothing (the port is off-canvas above). Moving: plug body + 2 contacts + cable, one rigid dy.
const plugOut=polygon([[8,24],[8,16],[11,16],[11,9],[21,9],[21,16],[24,16],[24,24]],1.2);
const plugSolid=holes(
 polygon([[6.7,25.3],[6.7,14.7],[9.7,14.7],[9.7,7.7],[22.3,7.7],[22.3,14.7],[25.3,14.7],[25.3,25.3]],2.2),
 capsule([14,11.5],[14,13.5],2.6),capsule([18,11.5],[18,13.5],2.6)); // real contact counters, 1.4 apart
const plugDy=F([[0,0],[.25,-1.5,'smooth'],[.6,0,'smooth']],           // Light: quiet 1.5 seat
               [[0,0],[.12,1],[.3,-4,'hit'],[.5,0]]);                // Strong: dip 1, drive 4 up, seat, ease home
const ethernet=make('ethernet','Ethernet','The plug pushes up into its port; Light seats gently, Strong dips back then drives home with a decisive click.',
 posed([
  ...framed('plug',plugOut,plugSolid),
  stroke('contact-a','M14 11.5 L14 13.5'),stroke('contact-b','M18 11.5 L18 13.5'), // outline only; Solid uses holes
  ...line('cable',[16,24],[16,27]),                                              // cap buried in body surface
 ],rigid((t,s)=>({dy:plugDy(t,s)}))),.6,.5);
ethernet.duotoneFill=true;

// ---------- ROUTER: flat gateway box, two antennas hinge at the body top ----------------
// Fixed: body, status dot. Moving: antenna-l pivot [10,18], antenna-r pivot [22,18] (mirrored).
const antL=F([[0,0],[.3,-8,'smooth'],[.7,0,'smooth']],
             [[0,0],[.12,5],[.32,-24],[.55,4],[.7,0]]);               // inward prep, wide outward swing, one settle
const router=make('router','Router','Both antennas hinge outward from the gateway box and settle back; Strong preps inward and swings 24 degrees.',[
 fill('body-tint',rect(5,18,22,7,2),'tint'),stroke('body-frame',rect(5,18,22,7,2)),
 fill('body-solid',holes(rect(3.7,16.7,24.6,9.6,3.3),circle(16,21.5,1.3)),'solid'),
 fill('status',circle(16,21.5,1.1),'outline'),                                   // filled dot in Outline, counter in Solid
 ...posed(line('antenna-l',[10,18],[10,8]),rigid((t,s)=>({angle:antL(t,s),pivot:[10,18]}))),
 ...posed(line('antenna-r',[22,18],[22,8]),rigid((t,s)=>({angle:-antL(t,s),pivot:[22,18]}))),
],.7,.7);
router.duotoneFill=true;

// ---------- LOAD BALANCER: hub dispatches one packet down a branch to a server target ---
// Fixed: hub, 2 branches, 2 targets. Moving: packet (static disk, rigid position keys), fades at arrival, rearms hidden.
const dirL=[-5.85,12.36],dirR=[5.85,12.36];  // hub (16,7) -> left target dock (10.15,19.36) / right dock (21.85,19.36)
const prog=F([[0,0],[.5,1,'smooth'],[.62,1],[.75,0,'linear'],[.9,0]],
             [[0,0],[.12,-.07],[.32,1,'hit'],[.42,1,'hold'],[.55,0,'hold'],[.7,0]]);
const pktVis=F([[0,1],[.5,1],[.62,0,'smooth'],[.75,0],[.9,1,'smooth']],
               [[0,1],[.32,1],[.42,0],[.55,0],[.7,1]]);
const loadBalancer=make('load-balancer','Load balancer','A packet leaves the hub and is routed to a server target; Light glides down the left branch, Strong coils in the hub then fires down the right branch.',[
 ...framed('hub',rect(12,4,8,6,1.8),rect(10.7,2.7,10.6,8.6,3)),
 ...line('branch-l',[16,10],[8.5,22]),...line('branch-r',[16,10],[23.5,22]),  // caps buried in hub/target fills
 ...framed('target-l',rect(5,22,7,6,1.5),rect(3.7,20.7,9.6,8.6,2.6)),
 ...framed('target-r',rect(20,22,7,6,1.5),rect(18.7,20.7,9.6,8.6,2.6)),
 ...shown(posed(bothFill('packet',circle(16,7,2.2)),rigid((t,s)=>{const p=prog(t,s),v=s?dirR:dirL;return{dx:p*v[0],dy:p*v[1]};})),(t,s)=>pktVis(t,s)),
],.9,.7);
loadBalancer.duotoneFill=true;

// ---------- FIREWALL: incoming arrow is stopped by the brick boundary -------------------
// Fixed: wall (frame, tint, 3 mortar lines; Solid = 4 tiles with 2.6 mortar gaps). Moving: arrow shaft + head, one rigid dx.
const arrowDx=F([[0,0],[.28,1.2,'smooth'],[.65,0,'smooth']],
                [[0,0],[.12,-.6],[.26,2.3,'hit'],[.5,0]]);           // rest tip x9; peak tip 11.3 (+cap 12.6) vs wall 13.7
const firewall=make('firewall','Firewall','Incoming traffic pushes at the wall and is turned back; Strong winds up, strikes the boundary hard and rebounds.',[
 fill('wall-tint',rect(15,5,13,22,2),'tint'),stroke('wall-frame',rect(15,5,13,22,2)),
 stroke('mortar-h','M15 16 L28 16'),stroke('mortar-top','M21.5 5 L21.5 16'),stroke('mortar-bottom','M19.5 16 L19.5 27'),
 fill('tile-tl',rect(13.7,3.7,6.5,11,1.5),'solid'),fill('tile-tr',rect(22.8,3.7,6.5,11,1.5),'solid'),
 fill('tile-bl',rect(13.7,17.3,4.5,11,1.5),'solid'),fill('tile-br',rect(20.8,17.3,8.5,11,1.5),'solid'),
 ...posed([...tube('arrow-shaft','M4 16 L9 16'),...tube('arrow-head','M6.5 13.5 L9 16 L6.5 18.5')],
          rigid((t,s)=>({dx:arrowDx(t,s)}))),
],.65,.5);
firewall.duotoneFill=true;

// ---------- USB DRIVE: the drive is pulled out (eject) and slides back ------------------
// Fixed: nothing on canvas. Moving: whole drive (body+connector silhouette, junction line, 2 contacts), one rigid dx.
const usbOut=polygon([[4,21],[4,11],[12,11],[12,8],[25,8],[25,24],[12,24],[12,21]],1.5);
const usbSolid=holes(polygon([[2.7,22.3],[2.7,9.7],[10.7,9.7],[10.7,6.7],[26.3,6.7],[26.3,25.3],[10.7,25.3],[10.7,22.3]],2.5),
 capsule([13.2,11],[13.2,21],2.6),capsule([6.5,14],[9.5,14],2.6),capsule([6.5,18],[9.5,18],2.6));
const usbDx=F([[0,0],[.3,1.2,'smooth'],[.7,0,'smooth']],
              [[0,0],[.12,-.6],[.28,3],[.52,-.3],[.68,0]]);         // push in, yank 3 right, one settle
const usbDrive=make('usb-drive','USB drive','The flash drive is tugged out of its port and slides back; Strong seats inward first then pulls a full 3 units.',
 posed([
  ...framed('drive',usbOut,usbSolid),
  stroke('junction','M13.2 11 L13.2 21'),stroke('contact-a','M6.5 14 L9.5 14'),stroke('contact-b','M6.5 18 L9.5 18'),
 ],rigid((t,s)=>({dx:usbDx(t,s)}))),.7,.68);
usbDrive.duotoneFill=true;

export const icons=[ethernet,router,loadBalancer,firewall,usbDrive];
