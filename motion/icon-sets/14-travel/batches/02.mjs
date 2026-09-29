import {make,P,F,posed,rigid,rect,circle,polygon,fill,stroke,ink,plainRing,surface,policy,capsule,bothFill,annulus} from './helpers.mjs';
const crank=F([[0,0],[.7,360,'smooth']],[[0,0],[.10,-30,'launch'],[.60,360]]);
const train=policy(make('train','Train','The fixed carriage drives one complete connecting-rod cycle. Strong takes up a short amount of slack before accelerating.',[
 ...surface('carriage',rect(5.5,6,21,13.5,2.5),[rect(9,9.5,4.5,5,1),rect(18.5,9.5,4.5,5,1)]),
 stroke('window-left',rect(9,9.5,4.5,5,1)),stroke('window-right',rect(18.5,9.5,4.5,5,1)),
 ...plainRing('wheel-left',10,23,3.5),...plainRing('wheel-right',22,23,3.5),
 ...posed(ink('connecting-rod','M11 23 L23 23'),rigid((t,s)=>({dx:Math.cos(crank(t,s)*Math.PI/180)-1,dy:Math.sin(crank(t,s)*Math.PI/180)}))),
 ...ink('rail','M5 27.5 L27 27.5')
],.78,.95));
train.constraints=[{type:'fixed',part:'train-carriage'},{type:'rigid',part:'train-connecting-rod'}];
const suspension=F([[0,0],[.3,1,'out'],[.65,0,'smooth']],[[0,0],[.14,-.5],[.48,1.8],[.88,-.4],[1.15,0]]);
const bus=policy(make('bus','Bus','The bus kneels on its suspension and rises. Its tire contacts stay fixed while the exposed supports follow the body.',[
 ...ink('tire-left',(t,s)=>`M10 ${25+suspension(t,s)} L10 27.5`),...ink('tire-right',(t,s)=>`M22 ${25+suspension(t,s)} L22 27.5`),
 ...posed([...surface('body',rect(7,5,18,20,3),[rect(10.5,8.5,11,6.5,1.5),circle(11,20,1.2),circle(21,20,1.2)]),stroke('windshield',rect(10.5,8.5,11,6.5,1.5)),stroke('lamp-left','M11 20 L11.01 20'),stroke('lamp-right','M21 20 L21.01 20'),...ink('mirror-left','M7 13 L4.5 13'),...ink('mirror-right','M25 13 L27.5 13')],rigid((t,s)=>({dy:suspension(t,s)})))
],.73,1.23));
bus.constraints=[{type:'rigid',part:'bus-body'}];
const handover=F([[0,0],[.4,.5,'out']],[[0,0],[.14,-.55],[.5,.9],[.72,.35],[.85,.5]]);
const acknowledge=F([[0,0],[.7,0]],[[0,0],[.45,0],[.62,.7],[.9,-.15],[1.1,0]]);
const carD=polygon([[5.5,21],[5.5,18],[9,18],[11.5,13.5],[20.5,13.5],[23,18],[26.5,18],[26.5,21]],1);
const car=policy(make('car-rental','Car rental','A separate rental key lowers into a held handover position; the car acknowledges with a small suspension settle.',[
 ...posed([...surface('body',carD),...plainRing('wheel-left',10.5,24.5,2.3),...plainRing('wheel-right',21.5,24.5,2.3)],rigid((t,s)=>({dy:acknowledge(t,s)}))),
 ...posed([...plainRing('key-bow',9,7.5,2.5),...ink('key-shaft','M11.5 7.5 L21 7.5'),...ink('key-tooth','M19 7.5 L19 9.5')],rigid((t,s)=>({dy:handover(t,s)})))
],.65,1.15,['key']));
car.constraints=[{type:'rigid',part:'car-rental-body'}];
const sea=P([[0,0],[.75,1,'smooth']],[[0,0],[1.2,1,'smooth']]);
function wave(t,s){const ph=sea(t,s)*Math.PI*2;const f=x=>26.8+.6*Math.sin((x-5)*Math.PI/6-ph),v=x=>.6*Math.PI/6*Math.cos((x-5)*Math.PI/6-ph);let d=`M5 ${f(5)}`;for(let i=0;i<8;i++){let x=5+i*2.75,z=x+2.75;d+=` C${x+2.75/3} ${f(x)+v(x)*2.75/3} ${z-2.75/3} ${f(z)-v(z)*2.75/3} ${z} ${f(z)}`;}return d;}
const roll=F([[0,0],[.32,3,'out'],[.75,0,'smooth']],[[0,0],[.18,-3],[.55,6],[.95,-2.5],[1.28,0]]);
const heave=F([[0,0],[.32,-.3,'out'],[.75,0,'smooth']],[[0,0],[.18,.4],[.55,-.9],[.95,.3],[1.28,0]]);
const ferry=policy(make('ferry','Ferry','The ferry pitches over a passing wave; the wave travels in one direction and completes the same cycle in both modes.',[
 ...posed([...surface('vessel',polygon([[5,17],[10,17],[10,9],[22,9],[22,17],[27,17],[24,22],[8,22]],1.2),[circle(14,13,1.1),circle(18,13,1.1)]),stroke('porthole-left','M14 13 L14.01 13'),stroke('porthole-right','M18 13 L18.01 13')],rigid((t,s)=>({angle:roll(t,s),dy:heave(t,s),pivot:[16,19]}))),...ink('wave',wave)
],.83,1.28));
ferry.constraints=[{type:'rigid',part:'ferry-vessel'}];
const pedal=F([[0,0],[.65,180,'smooth']],[[0,0],[.13,-35,'launch'],[.82,360]]);
const bicycle=policy(make('bicycle','Bicycle','The pedals turn within a stationary diamond frame. Strong takes up the chain before a complete crank revolution.',[
 ...plainRing('wheel-left',8,22,3.2),...plainRing('wheel-right',24,22,3.2),
 ...ink('rear-frame','M13 11 L8 22 L16 18.5'),...ink('front-frame','M13 11 L21 12 L16 18.5 Z'),...ink('fork','M21 12 L24 22'),...ink('seat-post','M13 11 L12.8 9.5'),...ink('saddle','M10.5 9.5 L14 9.5'),...ink('handlebar','M21 12 L23.5 9.5'),
 ...posed(ink('crank','M13.8 18.5 L18.2 18.5'),rigid((t,s)=>({angle:pedal(t,s),pivot:[16,18.5]})))
],.73,.9),false);
bicycle.cycleDegrees=180;bicycle.constraints=[{type:'fixed',part:'bicycle-front-frame'},{type:'rigid',part:'bicycle-crank'}];
export const icons=[train,bus,car,ferry,bicycle];
