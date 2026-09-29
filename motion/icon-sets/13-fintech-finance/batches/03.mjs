import {make,F as P,posed,shown,rigid,rect,circle,polygon,framed,fill,stroke,ink,ring,panel,disc,policy,reveal,hole,capsule,tint,sector,mix} from './helpers.mjs';
const receiptD=polygon([[7.5,5],[24.5,5],[24.5,27],[21.7,25.3],[18.8,27],[16,25.3],[13.2,27],[10.3,25.3],[7.5,27]],.7);
const feed=P([[0,0],[.7,0]],[[0,0],[.08,-.2,'launch'],[.24,.9],[.5,-.1],[.62,0]]);
const print=i=>P([[0,0],[.08+i*.13,0],[.4+i*.13,1,'out']],[[0,0],[.3+i*.1,0],[.46+i*.1,1]]);
const receipt=policy(make('receipt','Receipt','The paper feeds; item lines print in sequence and the final total stays visible.',[
 ...posed([tint('paper-interior',receiptD),stroke('paper',receiptD),fill('paper-solid',hole(receiptD,rect(9,7,14,17,1.3))),{name:'paper-solid-edge',d:receiptD,type:'stroke',width:2.6,style:'solid'},
 ...[0,1,2].flatMap(i=>reveal('printed-'+i,[11.5,10+i*4.5],[i===2?17.5:20.5,10+i*4.5],print(i)))],rigid((t,s)=>({dy:feed(t,s)})))
],.76,.8,['printed-0','printed-1','printed-2']));
receipt.constraints=[{type:'opaque',part:'receipt-paper-solid'},{type:'rigid',part:'receipt-paper'}];
const sheet=polygon([[8,4.7],[19,4.7],[24,9.7],[24,27.5],[8,27.5]],.8);
const stampY=P([[0,-1.4],[.1,-1.4],[.56,0,'in']],[[0,-2.5],[.1,-2.5],[.2,-2.9,'launch'],[.48,.4,'hit'],[.64,-.3],[.8,.1],[1.07,0]]);
const stampA=P([[0,0],[.18,1,'out']],[[0,0],[.12,0],[.34,1]]);
const invoice=policy(make('invoice','Invoice','A seal approaches the invoice, stamps once, and stays recorded on the page.',[
 tint('paper-interior',sheet),stroke('paper',sheet),fill('paper-solid',hole(sheet,rect(9.6,10.8,12.8,15.4,1.2))),{name:'paper-solid-edge',d:sheet,type:'stroke',width:2.6,style:'solid'},
 ...ink('fold','M19 5.1 L19 8.8 C19 9.5 19.5 10 20.2 10 L23.6 10'),...ink('heading','M12 13.5 L20 13.5'),
 ...shown(posed(disc('seal',16,21.5,2),rigid((t,s)=>({dy:stampY(t,s)}))),stampA)
],.68,1.16,['seal']));
invoice.constraints=[{type:'fixed',part:'invoice-paper'},{type:'opaque',part:'invoice-paper-solid'},{type:'rigid',part:'invoice-seal-frame'}];
const refundX=P([[0,0],[.33,-1,'out'],[.69,0,'smooth']],[[0,0],[.1,.55,'launch'],[.4,-2.4],[.82,.25],[1.13,0]]);
const refundCoin=P([[0,0],[.16,-.4,'out'],[.46,0,'smooth']],[[0,0],[.12,-1.3],[.3,.2],[.5,0]]);
const refund=policy(make('refund','Refund','A returned payment follows the leftward return channel; the coin gives a small acknowledgement.',[
 ...posed(disc('coin',21,9.3,3.5),rigid((t,s)=>({dy:refundCoin(t,s)}))),
 ...posed(ink('return','M21 16 L21 18 C21 21.3 18.3 24 15 24 L8 24 L11 21 L8 24 L11 27'),rigid((t,s)=>({dx:refundX(t,s)})))
],.78,1.22));
refund.constraints=[{type:'rigid',part:'refund-return'},{type:'rigid',part:'refund-coin-frame'}];
const paidY=P([[0,0],[.08,0],[.65,2,'in']],[[0,0],[.12,-.75,'launch'],[.5,2.4,'hit'],[.68,1.7],[.84,2.1],[1.08,2]]);
const billD=rect(7.5,6.5,17,12,2.2);
const bill=policy(make('bill-payment','Bill payment','The complete bill seats above the payment tray; its outline and value travel as one rigid object.',[
 ...ink('tray','M4.5 22 L4.5 23 C4.5 24.7 5.8 26 7.5 26 L24.5 26 C26.2 26 27.5 24.7 27.5 23 L27.5 22'),
 ...posed([tint('bill-interior',billD),stroke('bill',billD),fill('bill-solid',hole(rect(6.2,5.2,19.6,14.6,3.5),capsule([13,12.5],[19,12.5],2.6))),stroke('bill-value','M13 12.5 L19 12.5')],rigid((t,s)=>({dy:paidY(t,s)})))
],.74,1.17,['bill']));
bill.constraints=[{type:'fixed',part:'bill-payment-tray'},{type:'rigid',part:'bill-payment-bill'}];
const allocation=P([[0,0],[.32,.65,'out'],[.7,0,'smooth']],[[0,0],[.1,-.3,'launch'],[.4,1.6],[.82,-.15],[1.13,0]]);
const budget=policy(make('budget','Budget','One allocation separates from the budget, pauses for inspection, then returns to its exact share.',[
 ...framed('committed',sector(13.5,18.5,8,0,270),sector(13.5,18.5,9.1,0,270)),
 ...posed(framed('allocation',sector(17.3,14.7,8,270,360),sector(17.3,14.7,9.1,270,360)),rigid((t,s)=>({dx:allocation(t,s),dy:-allocation(t,s)})))
],.78,1.22));
budget.constraints=[{type:'fixed',part:'budget-committed-frame'},{type:'rigid',part:'budget-allocation-frame'}];
export const icons=[receipt,invoice,refund,bill,budget];
