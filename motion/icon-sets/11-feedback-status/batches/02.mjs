import {make,F as P,posed,ink,fill,stroke,rigid,rect,circle,polygon,holes,capsule,rotate,ring,frame,policy,tint,mix} from './helpers.mjs';

const advance=P([[0,0],[.08,0],[.5,4,'out']],[[0,0],[.1,-1.2,'launch'],[.32,4,'hit'],[.45,3.6],[.60,4]]);
const follow=i=>P([[0,0],[.2+i*.06,0],[.65+i*.05,-2,'out']],[[0,0],[.3+i*.1,0],[.36+i*.1,.4,'launch'],[.62+i*.12,-2.4],[.76+i*.12,-1.85],[.9+i*.12,-2]]);
const queued=policy(make('queued','Queued','The leading ticket advances to its stop; the waiting tickets follow in order.',[
 ...ink('stop','M28.5 21 L28.5 27'),
 ...posed(frame('ticket-back',12,6,14,4,1.6),rigid((t,s)=>({dx:follow(1)(t,s)}))),
 ...posed(frame('ticket-middle',10,14,14,4,1.6),rigid((t,s)=>({dx:follow(0)(t,s)}))),
 ...posed(frame('ticket-front',6,22,14,4,1.6),rigid((t,s)=>({dx:advance(t,s)})))
],.76,1.08,['ticket-back','ticket-middle','ticket-front']),true);
queued.constraints=[{type:'fixed',part:'queued-stop'},{type:'rigid',part:'queued-ticket-front-frame'}];

const brakeL=P([[0,0],[.08,0],[.55,-2,'out']],[[0,0],[.1,.55,'launch'],[.36,-2.35],[.48,-1.9],[.6,-2]]);
const brakeR=P([[0,0],[.14,0],[.62,2,'out']],[[0,0],[.16,0],[.22,-.5,'launch'],[.5,2.35],[.62,1.9],[.74,2]]);
const paused=policy(make('paused','Paused','Two brake bars separate and lock into place; Strong seats the left bar before the right.',[
 ...ring('rim',16,16,11),
 ...posed(ink('brake-left','M14 11 L14 21'),rigid((t,s)=>({dx:brakeL(t,s)}))),
 ...posed(ink('brake-right','M18 11 L18 21'),rigid((t,s)=>({dx:brakeR(t,s)})))
],.68,1.0,['brake-left','brake-right']),true);
paused.constraints=[{type:'fixed',part:'paused-rim-frame'},{type:'rigid',part:'paused-brake-left'}];

const cancelAngle=P([[0,0],[.5,0]],[[0,0],[.14,-16],[.36,3],[.5,0]]);
const strike=P([[0,0],[.12,0],[.53,1,'out']],[[0,0],[.3,0],[.52,1]]);
const cancelled=policy(make('cancelled','Cancelled','A second diagonal closes the cancellation mark; the completed cross stays visible.',[
 ...ring('rim',16,16,11),
 ...posed(ink('first','M12 20 L20 12'),rigid((t,s)=>({angle:cancelAngle(t,s)}))),
 ...ink('strike',(t,s)=>`M12 12 L${mix(12,20,strike(t,s))} ${mix(12,20,strike(t,s))}`,{visibility:(t,s)=>strike(t,s)>.0001?1:0})
],.6,.78,['strike']),true);
cancelled.constraints=[{type:'fixed',part:'cancelled-rim-frame'}];

const oct=polygon([[11,5],[21,5],[27,11],[27,21],[21,27],[11,27],[5,21],[5,11]],1.4);
const octSolid=polygon([[10.5,3.9],[21.5,3.9],[28.1,10.5],[28.1,21.5],[21.5,28.1],[10.5,28.1],[3.9,21.5],[3.9,10.5]],1.9);
const impact=P([[0,0],[.2,.9,'in'],[.48,0,'out']],[[0,0],[.12,-1.3,'launch'],[.32,1.8,'hit'],[.5,-.35],[.62,.12],[.72,0]]);
const receive=P([[0,0],[.22,0],[.32,.3],[.52,0]],[[0,0],[.32,0,'hit'],[.42,.9],[.56,-.2],[.68,0]]);
const error=policy(make('error','Error','A rigid exclamation stem strikes downward; the dot receives the contact and settles.',[
 tint('surface',oct),stroke('boundary',oct),
 stroke('stem','M16 10 L16 16.3',{pose:rigid((t,s)=>({dy:impact(t,s)}))}),
 fill('dot',(t,s)=>circle(16,22+receive(t,s),1.2),'outline'),
 fill('body-solid',(t,s)=>holes(octSolid,capsule([16,10+impact(t,s)],[16,16.3+impact(t,s)],2.6),circle(16,22+receive(t,s),1.35)))
],.6,.9),true);
error.constraints=[{type:'fixed',part:'error-boundary'},{type:'rigid',part:'error-stem'}];

const tri=polygon([[16,4.2],[28,26],[4,26]],2.3),triSolid=polygon([[16,2.7],[29.6,27],[2.4,27]],3.0);
const caution=P([[0,0],[.18,8,'out'],[.5,0,'smooth']],[[0,0],[.12,-14,'launch'],[.34,12],[.56,-4],[.72,1.2],[.86,0]]);
const warning=policy(make('warning','Warning','The warning stays anchored while its exclamation checks left and right, then aligns.',[
 tint('surface',tri),stroke('boundary',tri),
 stroke('stem','M16 12 L16 17',{pose:rigid((t,s)=>({angle:caution(t,s),pivot:[16,21.8]}))}),
 fill('dot',circle(16,21.8,1.1),'outline'),
 fill('body-solid',(t,s)=>holes(triSolid,capsule(rotate(16,12,caution(t,s),16,21.8),rotate(16,17,caution(t,s),16,21.8),2.6),circle(16,21.8,1.35)))
],.58,.98),true);
warning.constraints=[{type:'fixed',part:'warning-boundary'},{type:'rigid',part:'warning-stem'}];
export const icons=[queued,paused,cancelled,error,warning];
