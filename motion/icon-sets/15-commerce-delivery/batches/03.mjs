import {make,F as P,posed,rigid,rect,circle,polygon,fill,stroke,ink,surface,policy,capsule,held,bidir,holes,trace,tracedInk,bothFill} from './helpers.mjs';
const drawer=bidir([[0,0],[.03,0],[.7,2,'out']],[[0,0],[.04,0],[.16,-.25,'launch'],[.42,3.3],[.72,1.75],[1.05,2]],[[0,2],[.65,0,'smooth']],[[0,2],[.12,2.5,'out'],[.4,0,'in']]);
const tillY=P([[0,0]],[[0,0],[.36,0],[.48,.3,'hit'],[.76,0]]);
const consoleD='M7 17 L9.7 12.5 C9.86 12.18 10.18 12 10.5 12 L21.5 12 C21.82 12 22.14 12.18 22.3 12.5 L25 17';
const drawerD='M7 17 L25 17 L25 22.9 C25 23.728 24.328 24.4 23.5 24.4 L8.5 24.4 C7.672 24.4 7 23.728 7 22.9 Z';
const checkout=policy(held(make('checkout','Checkout','Light opens the drawer smoothly. Strong releases it with a short preparation, a larger opening and a controlled settle. Exposed guides keep the drawer joined to its console.',[
 ...posed([...surface('display',rect(9,4.5,14,4.5,1.5)),...ink('neck','M16 9 L16 12'),...ink('console',consoleD),fill('console-face',consoleD+' Z','solid')],rigid((t,s,r)=>({dy:r?0:tillY(t,s)}))),
 ...ink('guide-left',(t,s,r)=>`M7 ${17+(r?0:tillY(t,s))} L7 ${17+drawer(t,s,r)}`),...ink('guide-right',(t,s,r)=>`M25 ${17+(r?0:tillY(t,s))} L25 ${17+drawer(t,s,r)}`),
 ...posed([...surface('drawer',drawerD,[capsule([13,20.6],[19,20.6],2.6)]),stroke('handle','M13 20.6 L19 20.6')],rigid((t,s,r)=>({dy:drawer(t,s,r)})))
],.78,1.33),'closed','open',.73,.88));checkout.constraints=[{type:'rigid',part:'checkout-console'},{type:'rigid',part:'checkout-drawer'}];
const confirm=bidir([[0,0],[.03,0],[.7,1,'out']],[[0,0],[.04,0],[.18,.375,'pass'],[.4,1]],[[0,1],[.45,0,'smooth']],[[0,1],[.55,0,'smooth']]);
const paperY=P([[0,0]],[[0,0],[.36,0],[.46,.45,'hit'],[.72,0]]);
const order=policy(held(make('order-confirmed','Order confirmed','A check draws onto the order sheet. Strong sweeps through the corner decisively, then holds the completed confirmation.',posed([
 ...surface('sheet',polygon([[8,5],[19.5,5],[24,9.5],[24,26],[8,26]],.9),[rect(10,10.5,12,13,1.2)]),...tracedInk('check',[[12,17],[15,20],[20.5,13.5]],confirm)
],rigid((t,s,r)=>({dy:r?0:paperY(t,s)}))),.78,.95),'pending','confirmed',.53,.63));order.constraints=[{type:'rigid',part:'order-confirmed-sheet'}];
const flapL=bidir([[0,0],[.03,0],[.65,110,'out']],[[0,0],[.03,0],[.13,-3,'launch'],[.39,116],[.6,108],[.8,110]],[[0,110],[.65,0,'smooth']],[[0,110],[.06,110],[.7,-2],[.9,0]]);
const flapR=bidir([[0,0],[.097,0],[.717,110,'out']],[[0,0],[.12,0],[.22,-3,'launch'],[.48,116],[.69,108],[.89,110]],[[0,110],[.067,110],[.717,0,'smooth']],[[0,110],[.16,110],[.8,-2],[1,0]]);
const boxU='M8 14 L8 24 C8 25.105 8.895 26 10 26 L22 26 C23.105 26 24 25.105 24 24 L24 14';
const openPackage=policy(held(make('open-package','Open package','The two lid edges split and swing around fixed box corners. Strong opens them in sequence and settles each flap.',[
 ...ink('box',boxU),fill('box-face',boxU+' Z','solid'),...posed(ink('flap-left','M8 14 L16 14'),rigid((t,s,r)=>({angle:-flapL(t,s,r),pivot:[8,14]}))),...posed(ink('flap-right','M24 14 L16 14'),rigid((t,s,r)=>({angle:flapR(t,s,r),pivot:[24,14]})))
],.80,.95),'closed','open',.80,1.0),false);openPackage.constraints=[{type:'fixed',part:'open-package-box'},{type:'rigid',part:'open-package-flap-left'},{type:'rigid',part:'open-package-flap-right'}];
const rollX=bidir([[0,0],[.03,0],[.7,5,'out']],[[0,0],[.06,0],[.2,-.8,'in'],[.54,7.2,'out'],[.82,4.6,'smooth'],[1.1,5,'smooth']],[[0,5],[.65,0,'smooth']],[[0,5],[.12,5.7,'out'],[.45,0,'in']]);
const tape=policy(held(make('packing-tape','Packing tape','Light dispenses tape smoothly. Strong takes up the pull, drives the roll farther and settles. The strip follows the roll tangent from its fixed free end.',[
 ...posed([stroke('rim',circle(11.5,17,6)),stroke('core',circle(11.5,17,2.5)),fill('roll-tint',holes(circle(11.5,17,6),circle(11.5,17,2.5)),'tint'),fill('roll-solid',holes(circle(11.5,17,7.3),circle(11.5,17,1.2)),'solid')],rigid((t,s,r)=>({dx:rollX(t,s,r),angle:rollX(t,s,r)/6*180/Math.PI,pivot:[11.5,17]}))),...ink('strip',(t,s,r)=>`M5.5 23 L${11.5+rollX(t,s,r)} 23`)
],.78,1.33),'ready','dispensed',.73,.88));tape.constraints=[{type:'rigid',part:'packing-tape-rim'}];
// Return package v2: straight stem before the splayed head; the arrow cue leads the rigid parcel.
const returnX=bidir([[0,0],[.03,0],[.7,-5,'smooth']],[[0,0],[.08,0],[.42,1.4,'launch'],[.82,-5.9],[1.18,-5]],[[0,-5],[.65,0,'smooth']],[[0,-5],[.45,.8,'out'],[.8,0,'smooth']]);
const returnTilt=bidir([[0,0]],[[0,0],[.08,0],[.42,4,'launch'],[.66,-6],[1.06,0]],[[0,0]],[[0,0],[.25,5,'out'],[.7,0,'smooth']]);
const arrowDraw=P([[0,1]],[[0,1],[.06,1],[.18,0],[.24,0],[.46,1]]);
const arrowHead=P([[0,1]],[[0,1],[.06,0],[.46,0],[.56,1.04],[.66,1]]);
const returnCurve=[[22.5,11],[22.5,3],[10,3],[10,7.5]];
const returnLerp=(a,b,u)=>a.map((v,i)=>v+(b[i]-v)*u);
function splitReturn(u){const c=returnCurve,a=returnLerp(c[0],c[1],u),b=returnLerp(c[1],c[2],u),d=returnLerp(c[2],c[3],u),e=returnLerp(a,b,u),f=returnLerp(b,d,u);return [c[0],a,e,returnLerp(e,f,u)];}
// Arc-length lookup keeps the curve and straight stem on one continuous reveal clock.
const returnLengths=[0];let returnLast=returnCurve[0];
for(let i=1;i<=128;i++){const p=splitReturn(i/128)[3];returnLengths.push(returnLengths.at(-1)+Math.hypot(p[0]-returnLast[0],p[1]-returnLast[1]));returnLast=p;}
function returnBody(progress){const curveLength=returnLengths.at(-1),distance=Math.max(0,Math.min(1,progress))*(curveLength+3.5);let u=1;
 if(distance<curveLength){let i=returnLengths.findIndex(v=>v>=distance);i=Math.max(1,i);u=(i-1+(distance-returnLengths[i-1])/(returnLengths[i]-returnLengths[i-1]))/128;}
 const c=splitReturn(u),end=distance<=curveLength?c[3]:[10,7.5+distance-curveLength];return `M${c[0]} C${c.slice(1).join(' ')} L${end}`;
}
const returnPackage=policy(held(make('return-package','Return package','A clear return arrow leads the parcel home. Strong redraws the curl, winds the parcel back, then sends it left with a rigid tilt and a controlled settle. Light is one quiet slide.',[
 ...posed(surface('parcel',rect(10.5,17,13,10,2)),rigid((t,s,r)=>({dx:returnX(t,s,r),angle:returnTilt(t,s,r),pivot:[17,22]}))),
 ...ink('arrow-body',(t,s,r)=>returnBody(r?1:arrowDraw(t,s)),{visibility:(t,s,r)=>r||arrowDraw(t,s)>.00001?1:0}),
 ...ink('arrow-head',(t,s,r)=>{const q=r?1:arrowHead(t,s);return `M${10-3.06*q} ${11-2.57*q} L10 11 L${10+3.06*q} ${11-2.57*q}`;},{visibility:(t,s,r)=>r||arrowHead(t,s)>.00001?1:0})
],.78,1.32),'outbound','returned',.73,.88));returnPackage.revision='return-package-v2-user-arrow-and-strong';returnPackage.constraints=[{type:'rigid',part:'return-package-parcel'},{type:'opaque',part:'return-package-parcel'}];
export const icons=[checkout,order,openPackage,tape,returnPackage];
