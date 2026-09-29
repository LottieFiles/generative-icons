// Legal & compliance batch B: courthouse, gdpr-shield, audit, policy, terms-of-service.
import {make,toggle,dir,F,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,rotate,holes,annulus,bothFill,capsule,clamp01} from './helpers.mjs';

// ---------- COURTHOUSE ----------
// Pediment over three columns on a base line. Light: the pediment lifts 0.7 and lands. Strong: it is
// raised 1.8, held, and dropped; the columns take the shock (compress 7% from their feet) and rebound.
const pedD=polygon([[16,5.5],[27.5,12.5],[4.5,12.5]],[1.2,1.2,1.2]);
const liftP=F([[0,0],[.18,-.7,'out'],[.36,0,'in'],[.42,.15,'out'],[.5,0]],[[0,0],[.06,.3],[.24,-1.8],[.3,-1.8,'hold'],[.4,0,'hit'],[.46,.35],[.56,0]]);
const squashC=F([[0,1],[.5,1]],[[0,1],[.4,1],[.44,.93,'hit'],[.52,1.02],[.6,1]]);
const courthouse=make('courthouse','Courthouse',
 'Light: the pediment lifts 0.7 units and settles back onto the columns. Strong: it is raised 1.8 units, held and dropped; the columns compress under the impact and rebound.',[
 ...posed(framed('pediment',pedD),rigid((t,s)=>({dy:liftP(t,s)}))),
 ...[8.5,16,23.5].flatMap((x,i)=>posed(tube('column-'+i,`M${x} 15 L${x} 25`),(px,py,t,s)=>[px,25+(py-25)*squashC(t,s)])),
 ...tube('base','M4.5 27.8 L27.5 27.8')
],.5,.6);
courthouse.duotoneFill=true;

// ---------- GDPR SHIELD ----------
// A shield holding a ring of six stars (filled dots, r 1 at radius 4). The ring turns one step (60°)
// in Light; Strong winds back 12°, spins 120° with overshoot while the shield swells 5%.
const SH=[16,15.5],shieldD='M16 4 C20.5 6.6 24 7.2 26.5 7.2 L26.5 15 C26.5 21.6 22 26 16 27.5 C10 26 5.5 21.6 5.5 15 L5.5 7.2 C8 7.2 11.5 6.6 16 4 Z';
const spinR=F([[0,0],[.06,0],[.56,60,'smooth']],[[0,0],[.1,-12],[.34,126],[.48,118],[.58,120]]);
const swell=F([[0,1],[.56,1]],[[0,1],[.12,.97],[.3,1.05],[.48,1]]);
const dotAt=(i,t,s)=>{const a=(-90+i*60+spinR(t,s))*Math.PI/180;return [SH[0]+4*Math.cos(a),SH[1]+4*Math.sin(a)];};
const shieldPose=(x,y,t,s)=>{const k=swell(t,s);return [SH[0]+(x-SH[0])*k,SH[1]+(y-SH[1])*k];};
const gdprShield=make('gdpr-shield','GDPR shield',
 'Light: the ring of stars inside the shield turns one step. Strong: it winds back, spins two steps with an overshoot and the shield swells 5% as it lands.',[
 ...posed([fill('shield-interior',shieldD,'tint'),stroke('shield-frame',shieldD),fill('shield-solid',(t,s)=>holes(shieldD,...[0,1,2,3,4,5].map(i=>circle(...dotAt(i,t,s),1.15))),'solid'),
  ...[0,1,2,3,4,5].map(i=>fill('star-'+i,(t,s)=>circle(...dotAt(i,t,s),1),'outline'))],shieldPose)
],.56,.58);
gdprShield.duotoneFill=true;gdprShield.cycleDegrees=60;

// ---------- AUDIT ----------
// A ledger sheet with four entries beside a magnifier. The magnifier glides down the entries and
// back up. Strong drives it down 8 units in 0.22 s with a stop at the last entry, then snaps back.
const scan=F([[0,0],[.06,0],[.4,8,'smooth'],[.5,8],[.84,0,'smooth']],[[0,0],[.06,-.8],[.28,8,'hit'],[.36,8,'hold'],[.56,-.6],[.66,0]]);
const audit=make('audit','Audit',
 'Light: the magnifier glides down the ledger entries and back. Strong: it drives down 8 units, stops hard on the last entry and snaps back with a small overshoot.',[
 ...framed('sheet',rect(4,4,14,24,2),holes(rect(4,4,14,24,2),rect(5.3,5.3,11.4,21.4,.9))),
 ...[[9.5,8],[14,6],[18.5,8],[23,5.5]].flatMap(([y,w],i)=>tube('entry-'+i,`M7 ${y} L${7+w} ${y}`)),
 ...posed([...framed('lens',circle(23.8,12,3),annulus(23.8,12,3,2.6)),...tube('grip','M25.9 14.1 L27.6 15.8')],rigid((t,s)=>({dy:scan(t,s)})))
],.84,.66);
audit.duotoneFill=true;

// ---------- POLICY ----------
// A scroll: the top roll and page sides are fixed; the bottom roll unrolls 6 units down, extending
// the page and revealing a second clause. Stateful: rolled → unrolled, the reverse rolls it back up.
const roll=dir(F([[0,16.5],[.06,16.5],[.6,22.5,'smooth']],[[0,16.5],[.08,15.9],[.3,23.1,'hit'],[.42,22.3],[.52,22.5]]),F([[0,22.5],[.5,16.5,'smooth']],[[0,22.5],[.06,23],[.28,16.35,'hit'],[.4,16.7],[.5,16.5]]));
const policy=toggle('policy','Policy',
 'Light: the bottom roll unrolls 6 units, extending the page and revealing a second clause. Strong: it snaps down past its stop and settles. The unrolled policy is retained; the reverse rolls it up.',[
 ...framed('roll-top',rect(6,5,20,5,2.5)),
 ...tube('side-l',(t,s,r)=>`M8 10 L8 ${roll(t,s,r)}`),...tube('side-r',(t,s,r)=>`M24 10 L24 ${roll(t,s,r)}`),
 ...tube('clause-0','M11.5 13.8 L20.5 13.8'),
 ...shown(tube('clause-1','M11.5 18 L18 18'),(t,s,r)=>clamp01((roll(t,s,r)-20.6)/.8)),
 ...framed('roll-bottom',(t,s,r)=>rect(6,roll(t,s,r),20,5,2.5))
],.6,.52,'rolled','unrolled',.5,.5,{forward:'Unroll',reverse:'Roll up'});
policy.duotoneFill=true;

// ---------- TERMS OF SERVICE ----------
// A long page of clauses with a scrollbar. Reading to the end drives the thumb 11.5 units down the
// track; the read state is retained and the reverse scrolls back to the top.
const thumbY=dir(F([[0,0],[.06,0],[.6,11.5,'smooth']],[[0,0],[.08,-.8],[.26,11.5,'hit'],[.36,10.6],[.46,11.5]]),F([[0,11.5],[.5,0,'smooth']],[[0,11.5],[.08,12.3],[.26,0,'hit'],[.36,.9],[.46,0]]));
const PAGE=rect(5.5,4.5,17,23,2.2);
const termsOfService=toggle('terms-of-service','Terms of service',
 'Light: the scrollbar thumb glides to the bottom of the terms. Strong: it is flicked down, hits the end of the track and bounces back into place. The read state is retained; the reverse scrolls back up.',[
 ...framed('page',PAGE,holes(PAGE,rect(6.8,5.8,14.4,20.4,1))),
 ...[[9,10],[12.5,7.5],[16,10],[19.5,7.5],[23,6]].flatMap(([y,w],i)=>tube('clause-'+i,`M9 ${y} L${9+w} ${y}`)),
 ...tube('track','M26.3 8 L26.3 24'),
 ...posed(bothFill('thumb',capsule([26.3,8],[26.3,12.5],3)),rigid((t,s,r)=>({dy:thumbY(t,s,r)})))
],.6,.46,'unread','read',.5,.46,{forward:'Read to end',reverse:'Scroll up'});
termsOfService.duotoneFill=true;

export const icons=[courthouse,gdprShield,audit,policy,termsOfService];
