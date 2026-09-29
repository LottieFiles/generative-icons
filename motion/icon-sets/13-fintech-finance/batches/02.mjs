import {make,F as P,posed,shown,rigid,rect,circle,hole,framed,fill,ink,panel,disc,policy,arc,mix,ring} from './helpers.mjs';
const coinY=P([[0,8.3],[.08,8.3],[.57,16,'in'],[.67,16]],[[0,8.3],[.12,7.9,'launch'],[.24,8.9,'pass'],[.5,16.4,'hit'],[.68,15.4],[.84,16.15],[1.08,16]]);
const deposit=policy(make('deposit','Deposit','A coin drops into a fixed receiving tray and stays deposited.',[
 ...ink('tray','M6 13 L6 20 C6 21.7 7.3 23 9 23 L23 23 C24.7 23 26 21.7 26 20 L26 13'),
 ...posed(disc('coin',16,0,3.5),rigid((t,s)=>({dy:coinY(t,s)})))
],.75,1.17,['coin']));
deposit.constraints=[{type:'fixed',part:'deposit-tray'},{type:'rigid',part:'deposit-coin-frame'}];
const noteH=P([[0,6],[.1,6],[.66,12,'out']],[[0,6],[.08,5.75,'launch'],[.24,8.6,'pass'],[.42,12]]);
const notePrint=P([[0,0],[.3,0],[.72,1,'out']],[[0,0],[.36,0],[.62,1]]);
const withdraw=policy(make('withdraw','Withdraw','The ATM feeds a note in two measured steps, then the note value becomes visible.',[
 ...panel('slot',5,5,22,5,2.3),
 ...framed('note',(t,s)=>rect(9,13.8,14,noteH(t,s),1.8),(t,s)=>hole(rect(7.7,12.5,16.6,noteH(t,s)+2.6,3.1),rect(10.3,15.1,11.4,noteH(t,s)-2.6,.5))),
 ...shown(ink('note-mark','M13 20 L19 20'),notePrint)
],.8,1.17,['note','note-mark']));
withdraw.constraints=[{type:'fixed',part:'withdraw-slot-frame'},{type:'opaque',part:'withdraw-slot-solid'}];
const keyY=P([[0,0],[.12,.45,'in'],[.25,0,'out']],[[0,0],[.1,.9,'hit'],[.3,-.12],[.42,0]]);
const approved=P([[0,0],[.23,0],[.69,1,'out']],[[0,0],[.20,0],[.28,.38,'pass'],[.44,1]]);
const terminal=policy(make('payment-terminal','Payment terminal','A physical key press starts approval; the check completes and remains on the terminal.',[
 ...panel('housing',6.5,4.5,19,23,3.2),
 ...ink('screen-divider','M10.5 19 L21.5 19'),
 ...ink('key-left','M10.7 23.3 L11.4 23.3'),...posed(ink('key-center','M15.65 23.3 L16.35 23.3'),rigid((t,s)=>({dy:keyY(t,s)}))),...ink('key-right','M20.6 23.3 L21.3 23.3'),
 ...ink('approval',(t,s)=>{const p=approved(t,s),a=Math.min(1,p/.38),b=Math.max(0,(p-.38)/.62);return `M11.5 12 L${mix(11.5,14.5,a)} ${mix(12,15,a)} L${b?mix(14.5,20.5,b):mix(11.5,14.5,a)} ${b?mix(15,9,b):mix(12,15,a)}`;},{visibility:(t,s)=>approved(t,s)>1e-5?1:0})
],.78,.7,['approval']));
terminal.constraints=[{type:'fixed',part:'payment-terminal-housing-frame'},{type:'opaque',part:'payment-terminal-housing-solid'},{type:'rigid',part:'payment-terminal-key-center'}];
const tapX=P([[0,0],[.28,.8,'out'],[.46,.8],[.74,0,'smooth']],[[0,0],[.1,-.45,'launch'],[.34,1.4],[.72,-.2],[.9,0]]);
const tapAngle=P([[0,0],[.28,0],[.74,0]],[[0,0],[.1,-6,'launch'],[.34,2],[.72,-.5],[.9,0]]);
const signal=i=>P([[0,1],[.14+i*.08,1],[.31+i*.08,.32,'out'],[.55+i*.08,1,'out']],[[0,1],[.18+i*.1,1],[.3+i*.1,.15],[.5+i*.1,1],[.6+i*.1,.75],[.74+i*.1,1]]);
const contactless=policy(make('contactless-payment','Contactless payment','The phone approaches; three signal bands acknowledge the tap from near to far.',[
 ...posed(panel('phone',5.5,7,7,18,2.3),rigid((t,s)=>({dx:tapX(t,s),angle:tapAngle(t,s),pivot:[9.5,16]}))),
 ...[0,1,2].flatMap(i=>shown(ink('signal-'+i,arc(14.1,16,5.4+i*3.7,-48,48)),signal(i)))
],.83,1.22));
contactless.constraints=[{type:'rigid',part:'contactless-payment-phone-frame'},{type:'fixed',part:'contactless-payment-signal-2'}];
const phase=P([[0,0],[.69,360,'smooth']],[[0,0],[.08,-6,'launch'],[.45,366],[.62,359],[.80,360]]);
// A scheduled payment is a dated calendar with one cycling hand, not a second exchange ring.
const recurring=policy(make('recurring-payment','Recurring payment','A calendar stays fixed while the payment clock completes its scheduled cycle.',[
 ...panel('calendar',5.5,8,21,19,2.5),...ink('binding-left','M11 4.5 L11 10'),...ink('binding-right','M21 4.5 L21 10'),
 ...ink('header','M5.5 13 L26.5 13'),
 ...posed(ink('clock-hand','M16 16.7 L16 20 L18.8 20'),rigid((t,s)=>({angle:phase(t,s),pivot:[16,20]})))
],.78,1.22));
recurring.cycleDegrees=360;recurring.constraints=[{type:'fixed',part:'recurring-payment-calendar-frame'},{type:'rigid',part:'recurring-payment-clock-hand'}];
export const icons=[deposit,withdraw,terminal,contactless,recurring];
