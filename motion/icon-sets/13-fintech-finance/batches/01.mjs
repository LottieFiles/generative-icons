import {make,F as P,posed,shown,rigid,rect,circle,polygon,framed,fill,ink,ring,panel,disc,policy,reveal,arrowArc,mix} from './helpers.mjs';
const doorW=P([[0,7],[.07,7],[.58,4.2,'out']],[[0,7],[.1,7.45,'launch'],[.5,4.1],[.72,4.32],[.9,4.16],[1.04,4.2]]);
const bank=policy(make('bank','Bank','The entrance opens on its left hinge; the roof, columns and plinth stay fixed.',[
 ...framed('pediment',polygon([[16,4.7],[26.8,11],[5.2,11]],1),polygon([[16,3.6],[28,11.6],[4,11.6]],1.1)),
 ...ink('column-left','M7.5 14.5 L7.5 23'),...ink('column-right','M24.5 14.5 L24.5 23'),
 ...framed('door',(t,s)=>rect(12.5,14.5,doorW(t,s),8.5,1.5),(t,s)=>rect(11.35,13.35,doorW(t,s)+2.3,10.8,2.65)),
 ...ink('plinth','M5 26.5 L27 26.5')
],.65,1.12,['door']));
bank.constraints=[{type:'fixed',part:'bank-pediment-frame'},{type:'fixed',part:'bank-column-left'},{type:'fixed',part:'bank-plinth'}];
const post=P([[0,0],[.05,0],[.48,1,'out']],[[0,0],[.08,0],[.20,.55,'pass'],[.34,1]]);
const balance=P([[0,0],[.3,0],[.7,1,'out']],[[0,0],[.30,0],[.62,1]]);
const account=policy(make('bank-account','Bank account','A new ledger entry posts, then the balance line updates without reversing its value.',[
 ...panel('ledger',7,5,18,22,2.5),
 ...ink('balance',(t,s)=>`M11 10 L${mix(17,21,balance(t,s))} 10`),
 ...ink('entry-one','M11 14 L21 14'),...ink('entry-two','M11 18 L18 18'),
 ...reveal('entry-new',[11,22],[21,22],post)
],.78,1.16,['entry-new','balance']));
account.constraints=[{type:'fixed',part:'bank-account-ledger-frame'},{type:'opaque',part:'bank-account-ledger-solid'},{type:'monotonic',part:'bank-account-balance',measure:'width'}];
const transferTop=P([[0,0],[.3,1.6,'out'],[.62,0,'smooth']],[[0,0],[.1,-.3,'launch'],[.34,1.8],[.7,-.3],[.86,0]]);
const transferBottom=P([[0,0],[.1,0],[.4,-1.6,'out'],[.72,0,'smooth']],[[0,0],[.2,0],[.3,.3,'launch'],[.54,-1.8],[.96,.3],[1.13,0]]);
const transfer=policy(make('bank-transfer','Bank transfer','A transfer travels to the receiving account; the return channel answers after it.',[
 ...framed('account-left',rect(4.7,10.5,3.6,11,1.8),rect(3.4,9.2,6.2,13.6,3.1)),
 ...framed('account-right',rect(23.7,10.5,3.6,11,1.8),rect(22.4,9.2,6.2,13.6,3.1)),
 ...posed(ink('outgoing','M12.2 11.5 L18.4 11.5 L16.3 9.4 L18.4 11.5 L16.3 13.6'),rigid((t,s)=>({dx:transferTop(t,s)}))),
 ...posed(ink('incoming','M19.8 20.5 L13.6 20.5 L15.7 18.4 L13.6 20.5 L15.7 22.6'),rigid((t,s)=>({dx:transferBottom(t,s)})))
],.8,1.22));
transfer.constraints=[{type:'fixed',part:'bank-transfer-account-left-frame'},{type:'rigid',part:'bank-transfer-outgoing'},{type:'rigid',part:'bank-transfer-incoming'}];
const turn=P([[0,0],[.68,180,'smooth']],[[0,0],[.08,-7,'launch'],[.42,188],[.58,178.5],[.72,180]]);
const exchange=policy(make('currency-exchange','Currency exchange','Two exchange channels index half a turn around one fixed unit of value.',[
 ...disc('coin',16,16,3.5),...posed([...arrowArc('channel-a',16,16,9.5,198,336),...arrowArc('channel-b',16,16,9.5,18,156)],rigid((t,s)=>({angle:turn(t,s)})))
],.76,1.15));
exchange.cycleDegrees=180;exchange.constraints=[{type:'fixed',part:'currency-exchange-coin-frame'},{type:'rigid',part:'currency-exchange-channel-a-arc'}];
const saveY=P([[0,12.5],[.12,13.4],[.53,16,'in'],[.67,16]],[[0,12.5],[.1,12.5],[.24,12.9,'pass'],[.56,16.35],[.76,15.88],[1.08,16]]);
const saveA=P([[0,0],[.06,0],[.18,1,'out']],[[0,0],[.1,0],[.3,1]]);
const savings=policy(make('savings','Savings','One coin enters the jar and settles above the saved stack. The added coin remains.',[
 ...panel('jar',7.5,9,17,18.5,3),...ink('lid','M10 5 L22 5'),
 ...ink('coin-low','M12 24 L20 24'),...ink('coin-mid','M12 20 L20 20'),
 ...shown(posed(ink('coin-new','M12 0 L20 0'),rigid((t,s)=>({dy:saveY(t,s)}))),saveA)
],.75,1.17,['coin-new']));
savings.constraints=[{type:'fixed',part:'savings-jar-frame'},{type:'opaque',part:'savings-jar-solid'},{type:'rigid',part:'savings-coin-new'}];
export const icons=[bank,account,transfer,exchange,savings];
