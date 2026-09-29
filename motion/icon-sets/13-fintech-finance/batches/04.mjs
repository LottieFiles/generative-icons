import {make,F as P,posed,shown,rigid,rect,circle,fill,stroke,ink,ring,panel,disc,framed,policy,bothFill,mix,track} from './helpers.mjs';
const historyStep=P([[0,0],[.08,0],[.69,1,'out']],[[0,0],[.08,0],[.20,.55,'pass'],[.40,1]]);
const historyAlpha=(i,t,s)=>{const p=historyStep(t,s);return i===0?track([[0,1],[.15,1],[.65,0,'out']],p):i===3?track([[0,0],[.4,0],[.85,1,'out']],p):1;};
const history=policy(make('transaction-history','Transaction history','The ledger advances one entry. A new transaction stays at the bottom while the oldest leaves the timeline.',[
 ...ink('timeline','M7 8 L7 24'),...[9,16,23].flatMap((y,i)=>bothFill('date-'+i,circle(7,y,1.8))),
 ...[5,10,7,11].flatMap((length,i)=>shown(posed(ink('entry-'+i,`M13 ${9+7*i} L${13+length} ${9+7*i}`),rigid((t,s)=>({dy:-7*historyStep(t,s)}))),(t,s)=>historyAlpha(i,t,s)))
],.78,1.17,['entry-0','entry-1','entry-2','entry-3']),false);
history.constraints=[{type:'fixed',part:'transaction-history-timeline'},{type:'rigid',part:'transaction-history-entry-2'},{type:'monotonic',part:'transaction-history-entry-2',measure:'y',direction:-1}];
const growStem=P([[0,0],[.06,0],[.54,1,'out']],[[0,0],[.06,0],[.26,1]]);
const growLeft=P([[0,0],[.2,0],[.65,1,'out']],[[0,0],[.18,0],[.42,1]]);
const growRight=P([[0,0],[.3,0],[.75,1,'out']],[[0,0],[.32,0],[.55,1]]);
const branch=(name,anchor,controlA,controlB,end,p)=>ink(name,(t,s)=>{const q=p(t,s),at=v=>[mix(anchor[0],v[0],q),mix(anchor[1],v[1],q)];return `M${anchor} C${at(controlA)} ${at(controlB)} ${at(end)}`;},{visibility:(t,s)=>p(t,s)>1e-5?1:0});
const investment=policy(make('investment','Investment','A stem grows from the invested coin; two branches extend in order and retain their growth.',[
 ...disc('principal',16,23,4.5),
 ...ink('stem',(t,s)=>`M16 17.25 L16 ${mix(17.25,10.8,growStem(t,s))}`,{visibility:(t,s)=>growStem(t,s)>1e-5?1:0}),
 ...shown(posed(framed('branch-left','M16 14.5 C10.3 15.1 7.2 12 7.5 7.3 C13.2 7 16.3 10.3 16 14.5 Z'),rigid((t,s)=>({angle:-25*(1-growLeft(t,s)),pivot:[16,14.5]}))),growLeft),
 ...shown(posed(framed('branch-right','M16 10.8 C21.5 11.4 24.3 9 24 4.8 C18.5 4.2 15.7 7.5 16 10.8 Z'),rigid((t,s)=>({angle:25*(1-growRight(t,s)),pivot:[16,10.8]}))),growRight)
],.83,1.22,['stem','branch-left','branch-right']));
investment.constraints=[{type:'fixed',part:'investment-principal-frame'},{type:'monotonic',part:'investment-stem',measure:'height'},{type:'rigid',part:'investment-branch-left-frame'},{type:'rigid',part:'investment-branch-right-frame'}];
const carry=P([[0,0],[.27,-.55,'out'],[.4,-.55],[.72,0,'smooth']],[[0,0],[.1,.3,'launch'],[.42,-1.8],[.82,.2],[1.18,0]]);
const grip=P([[0,0],[.18,-.3,'out'],[.52,0,'smooth']],[[0,0],[.12,-1.4],[.5,-.5],[.86,.15],[1.12,0]]);
const portfolio=policy(make('portfolio','Portfolio','The handle takes the load before the case lifts; the whole case returns with every attachment intact.',[
 ...posed([
 ...ink('handle',(t,s)=>{const y=7.5+grip(t,s);return `M12 12 L12 ${y+2.5} C12 ${y+1.12} 13.12 ${y} 14.5 ${y} L17.5 ${y} C18.88 ${y} 20 ${y+1.12} 20 ${y+2.5} L20 12`; }),
 ...panel('case',5,12,22,15,3),...ink('seam','M5 18.5 L27 18.5'),...ink('latch','M16 17.3 L16 20.5')
 ],rigid((t,s)=>({dy:carry(t,s)})))
],.8,1.26));
portfolio.constraints=[{type:'rigid',part:'portfolio-case-frame'}];
const pressA=P([[0,0],[.7,0]],[[0,0],[.1,.65,'hit'],[.26,0]]);
const pressB=P([[0,0],[.16,.6,'in'],[.3,0,'out']],[[0,0],[.3,0],[.4,.65,'hit'],[.56,0]]);
const resultA=i=>P([[0,0],[.26+i*.05,0],[.61+i*.05,1,'out']],[[0,0],[.5+i*.04,0],[.66+i*.04,1]]);
const resultY=i=>P([[0,.7],[.26+i*.05,.7],[.61+i*.05,0,'out']],[[0,1.4],[.5+i*.04,1.4],[.66+i*.04,0]]);
const calculator=policy(make('calculator','Calculator','A key press computes a result; Strong enters a value first, then confirms it. The display stays posted.',[
 ...panel('body',6,4.5,20,23,2.5),...ink('display-divider','M10 14 L22 14'),
 ...[0,1,2,3,4,5].flatMap(i=>posed(ink('key-'+i,`M${11.4+(i%3)*4.5} ${18.5+Math.floor(i/3)*4.5} L${11.6+(i%3)*4.5} ${18.5+Math.floor(i/3)*4.5}`),rigid((t,s)=>({dy:i===0?pressA(t,s):i===5?pressB(t,s):0})))),
 ...[0,1,2].flatMap(i=>shown(posed(bothFill('result-'+i,circle(12.5+i*3.5,9.5,.85)),rigid((t,s)=>({dy:resultY(i)(t,s)}))),resultA(i)))
],.79,.9,['result-0','result-1','result-2']));
calculator.constraints=[{type:'fixed',part:'calculator-body-frame'},{type:'opaque',part:'calculator-body-solid'},{type:'rigid',part:'calculator-key-5'}];
const wheel=P([[0,0],[.7,90,'smooth']],[[0,0],[.08,-7,'launch'],[.40,96],[.58,88.5],[.75,90]]);
const vault=policy(make('vault','Vault','The vault wheel takes up slack, indexes a quarter turn, and locks on its next symmetric position.',[
 ...ring('door',16,16,11.5),...bothFill('hub',circle(16,16,2.7)),
 ...posed([0,1,2,3].flatMap(i=>{const a=i*Math.PI/2,at=r=>[16+r*Math.cos(a),16+r*Math.sin(a)];return ink('spoke-'+i,`M${at(2.7)} L${at(7.5)}`);}),rigid((t,s)=>({angle:wheel(t,s)})))
],.79,1.22));
vault.cycleDegrees=90;vault.constraints=[{type:'fixed',part:'vault-door-frame'},{type:'fixed',part:'vault-hub'},{type:'rigid',part:'vault-spoke-0'}];
export const icons=[history,investment,portfolio,calculator,vault];
