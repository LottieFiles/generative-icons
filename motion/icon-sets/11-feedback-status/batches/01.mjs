import {make,F as P,posed,shown,ink,fill,stroke,bothFill,rigid,rect,circle,arc,hole,framed,mix,ring,policy,tint,revealLine} from './helpers.mjs';

// Three vanes share a rigid rotor. Strong carries momentum through a larger clockwise sweep.
const spin=P([[0,0],[.12,0],[.9,120,'smooth']],[[0,0],[.1,-10,'launch'],[.7,252],[.86,236],[1.12,240]]);
const spinner=policy(make('loading-spinner','Loading spinner','Three vanes pass the signal clockwise; Strong carries momentum through a larger clockwise sweep.',[0,120,240].flatMap((a,i)=>posed(ink('vane-'+i,arc(16,16,10,a-90,a-20)),rigid((t,s)=>({angle:spin(t,s)})))),.96,1.18),false);
spinner.cycleDegrees=120;
spinner.constraints=[{type:'rigid',part:'loading-spinner-vane-0'}];

// A relay of rigid circles: no stroke scaling and no confetti.
const hop=(i)=>P([[0,0],[.05+i*.1,0],[.2+i*.1,-2.2,'out'],[.4+i*.1,0,'smooth']],[[0,0],[.05+i*.13,0],[.1+i*.13,.6,'launch'],[.28+i*.13,-5.2],[.46+i*.13,0,'hit'],[.58+i*.13,-1.4],[.7+i*.13,0,'hit'],[.78+i*.13,-.35],[.86+i*.13,0]]);
const dots=policy(make('loading-dots','Loading dots','A three-dot relay: each landing cues the next dot; Strong adds a small return from contact.',[7.2,16,24.8].flatMap((x,i)=>posed(framed('dot-'+i,circle(x,16,2),circle(x,16,3.1)),rigid((t,s)=>({dy:hop(i)(t,s)})))),.66,1.12),true);
dots.constraints=[0,1,2].map(i=>({type:'rigid',part:'loading-dots-dot-'+i+'-frame'}));

const level=i=>P([[0,i===0?1:.22],[.12+i*.13,i===0?1:.22],[.29+i*.13,1,'smooth']],[[0,i===0?1:.22],[.02+Math.max(0,i-1)*.14,i===0?1:.22],[.2+Math.max(0,i-1)*.14,1]]);
const progress=policy(make('progress-ring','Progress ring','Four fixed sectors confirm progress clockwise and keep the completed value.',[0,1,2,3].flatMap(i=>shown(ink('sector-'+i,arc(16,16,10,-80+i*90,-10+i*90)),level(i))),.76,.54,['sector-1','sector-2','sector-3']),false);
progress.constraints=[{type:'fixed',part:'progress-ring-sector-0'},{type:'monotonic',part:'progress-ring-sector-3',measure:'alpha'}];

const rail=P([[0,0],[.18,0],[.7,1,'smooth']],[[0,0],[.14,0],[.22,.28,'pass'],[.32,.6,'pass'],[.44,1]]);
const steps=policy(make('progress-steps','Progress steps','The progress line advances between three milestones; Strong advances continuously through the fixed checkpoints.',[
 ...[6.5,16,25.5].flatMap((x,i)=>framed('step-'+i,circle(x,12,2.6),circle(x,12,3.7))),
 ...ink('rail','M5 23 L27 23',{visibility:()=>.2}),
 ...revealLine('advance',[5,23],[27,23],rail)
],.76,.6,['advance']),true);
steps.constraints=[{type:'fixed',part:'progress-steps-step-0-frame'},{type:'monotonic',part:'progress-steps-advance',measure:'width'}];

const glass='M8 5 L24 5 C24 10 21.4 12.7 18.4 14.6 C17.2 15.4 17.2 16.6 18.4 17.4 C21.4 19.3 24 22 24 27 L8 27 C8 22 10.6 19.3 13.6 17.4 C14.8 16.6 14.8 15.4 13.6 14.6 C10.6 12.7 8 10 8 5 Z';
const sand=P([[0,0],[.12,0],[.83,1,'smooth']],[[0,0],[.16,0],[.5,.42,'pass'],[1.25,1]]);
const grainY=P([[0,15.6],[.13,15.6],[.75,22.5,'linear']],[[0,15.6],[.16,15.6],[.5,22.5,'hit'],[.58,15.6],[.66,15.6],[1.05,22.5,'hit']]);
const grainAlpha=P([[0,0],[.14,1],[.64,1],[.76,0]],[[0,0],[.16,1],[.46,1],[.49,0],[.64,0],[.68,1],[.98,1],[1.05,0]]);
const hourglass=policy(make('hourglass','Hourglass','Sand drains through a fixed neck into the lower chamber; Strong makes the transfer readable in two stages.',[
 tint('glass-interior',glass),...ink('glass',glass),
 ...bothFill('sand-upper',(t,s)=>{const p=sand(t,s),y=mix(8,12.2,p),w=mix(5.1,1.0,p);return `M${16-w} ${y} L${16+w} ${y} L16 13.5 Z`;},{visibility:(t,s)=>1-sand(t,s)}),
 ...bothFill('sand-lower',(t,s)=>`M10.6 24.5 L21.4 24.5 L16 ${mix(24.3,19.6,sand(t,s))} Z`),
 ...shown(posed(ink('grain','M16 15.6 L16 16.6'),rigid((t,s)=>({dy:grainY(t,s)-15.6}))),grainAlpha)
],.9,1.43,['sand-upper','sand-lower','grain']),true);
hourglass.constraints=[{type:'fixed',part:'hourglass-glass'},{type:'monotonic',part:'hourglass-sand-lower',measure:'height'}];
export const icons=[spinner,dots,progress,steps,hourglass];
