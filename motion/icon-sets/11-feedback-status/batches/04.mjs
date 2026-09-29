import {make,F as P,posed,shown,ink,fill,stroke,bothFill,rigid,rect,circle,arc,hole,holes,framed,mix,policy,tint,timePair,track,polygon} from './helpers.mjs';
import {flow} from '../../library/direction.mjs';

const closeL=(t,s,r)=>r?(s?flow([[0,1],[.06,1,'launch'],[.4,-.12],[.52,.04],[.62,0]],t):track([[0,1],[.5,0,'out']],t)):P([[0,0],[.07,0],[.54,1,'out']],[[0,0],[.1,-.7,'launch'],[.42,1,'hit'],[.54,.95],[.64,1]])(t,s);
const closeR=(t,s,r)=>r?(s?flow([[0,1],[.16,1],[.22,1,'launch'],[.56,-.1],[.68,.03],[.78,0]],t):track([[0,1],[.1,1],[.55,0,'out']],t)):P([[0,0],[.16,0],[.62,1,'out']],[[0,0],[.14,0],[.2,-.5,'launch'],[.56,1,'hit'],[.7,.96],[.82,1]])(t,s);
const left=(t,s,r)=>-1.5*(1-closeL(t,s,r)),right=(t,s,r)=>1.5*(1-closeR(t,s,r));
const reconnect=policy(make('reconnect','Reconnect','The two plugs pull clear, align, and seat; disconnect is an explicit reverse action.',[
 ...ink('cable-left',(t,s,r)=>`M3.2 16 L${8+left(t,s,r)} 16`),
 ...ink('cable-right',(t,s,r)=>`M${24+right(t,s,r)} 16 L28.8 16`),
 ...posed(framed('plug-left',rect(8,10,6,12,2),rect(6.7,8.7,8.6,14.6,3.2)),rigid((t,s,r)=>({dx:left(t,s,r)}))),
 ...posed(framed('plug-right',rect(18,10,6,12,2),rect(16.7,8.7,8.6,14.6,3.2)),rigid((t,s,r)=>({dx:right(t,s,r)}))),
 ...posed(ink('pin-upper','M14 13 L18 13'),rigid((t,s,r)=>({dx:left(t,s,r)}))),
 ...posed(ink('pin-lower','M14 19 L18 19'),rigid((t,s,r)=>({dx:left(t,s,r)})))
],.7,1.06),true);
Object.assign(reconnect,{stateful:true,initialState:'disconnected',finalState:'connected',reverseSuffix:'-disconnect',forwardLabel:'Reconnect',reverseLabel:'Disconnect',reverseTimings:timePair(.6,.82)});
reconnect.constraints=[{type:'rigid',part:'reconnect-plug-left-frame'},{type:'rigid',part:'reconnect-plug-right-frame'}];

const cloud='M9 25 C5.4 25 3.5 21.4 3.5 17 C3.5 14 5.6 11.4 8.8 11 C9.6 6.8 12.3 4.8 16 4.8 C20.4 4.8 23.3 7.6 23.8 11.4 C27 11.9 28.5 14.1 28.5 17 C28.5 21.4 26.1 25 23 25 Z';
const upload=P([[0,0],[.1,0],[.32,-1,'out'],[.59,0,'smooth']],[[0,0],[.08,.7,'launch'],[.34,-1.95],[.56,.4],[.72,0]]);
const download=P([[0,0],[.3,0],[.55,1,'out'],[.83,0,'smooth']],[[0,0],[.34,0],[.42,-.7,'launch'],[.68,1.9],[.9,-.4],[1.06,0]]);
const cloudSync=policy(make('cloud-sync','Cloud sync','Upload leads; download answers. Both arrows return to their own anchors inside a fixed cloud.',[
 tint('surface',cloud),...ink('cloud',cloud),
 ...posed(ink('upload','M9.7 16.4 L12 14.1 L14.3 16.4 L12 14.1 L12 21'),rigid((t,s)=>({dy:upload(t,s)}))),
 ...posed(ink('download','M17.7 17.7 L20 20 L22.3 17.7 L20 20 L20 11'),rigid((t,s)=>({dy:download(t,s)})))
],.91,1.4),true);
cloudSync.constraints=[{type:'fixed',part:'cloud-sync-cloud'},{type:'rigid',part:'cloud-sync-upload'},{type:'rigid',part:'cloud-sync-download'}];

const disk=polygon([[7,5],[23,5],[27,9],[27,25],[25,27],[7,27],[5,25],[5,7]],1.3);
const head=P([[0,0],[.18,.55,'in'],[.36,0,'out']],[[0,0],[.1,-.5,'launch'],[.3,1.1,'hit'],[.44,-.15],[.56,0]]);
const tick=P([[0,0],[.25,0],[.7,1,'out']],[[0,0],[.5,0],[.6,.35,'pass'],[.78,1]]);
const saved=policy(make('saved','Saved','The write head seats, then a two-stroke check records the saved result.',[
 tint('surface',disk),stroke('body',disk),
 fill('body-solid',holes(disk,rect(7.6,14,16.8,11.4,1.5),rect(9,7.1,14,6,1.4))),
 {name:'body-solid-edge',d:disk,style:'solid',type:'stroke',width:2.6},
 ...posed(ink('head','M11 9.3 L20 9.3'),rigid((t,s)=>({dy:head(t,s)}))),
 ...ink('check',(t,s)=>{const p=tick(t,s),a=Math.min(1,p/.35),b=Math.max(0,(p-.35)/.65);return `M11 19 L${mix(11,14.5,a)} ${mix(19,22.5,a)} L${b?mix(14.5,22,b):mix(11,14.5,a)} ${b?mix(22.5,16.5,b):mix(19,22.5,a)}`;},{visibility:(t,s)=>tick(t,s)>.0001?1:0})
],.8,.9,['check']),true);
saved.constraints=[{type:'fixed',part:'saved-body'},{type:'rigid',part:'saved-head'}];

const bubble='M7 8 L18 8 C18 13 21 16 26 16 L26 23 C26 24.1 25.1 25 24 25 L13 25 L8 27.5 C7.5 27.8 7 27.5 7 27 L7 25 C5.3 25 4 23.7 4 22 L4 11 C4 9.3 5.3 8 7 8 Z';
const badge=P([[0,0],[.12,0],[.58,1,'out']],[[0,0],[.34,0],[.62,1.1],[.78,.96],[.9,1]]);
const message=i=>P([[0,0],[.05+i*.08,0],[.32+i*.1,1,'out']],[[0,0],[.08+i*.12,0],[.22+i*.12,1]]);
const notification=policy(make('notification-dot','Notification dot','A message arrives first; its unread badge then grows into the reserved corner.',[
 tint('surface',bubble),stroke('bubble',bubble),fill('bubble-solid',hole(bubble,rect(8,17,15,7,1.5))),{name:'bubble-solid-edge',d:bubble,style:'solid',type:'stroke',width:2.6},
 ...[0,1].flatMap(i=>ink('message-'+i,(t,s)=>`M10 ${19+i*3} L${mix(10,i?17:20,message(i)(t,s))} ${19+i*3}`,{visibility:(t,s)=>message(i)(t,s)>.0001?1:0})),
 ...shown(framed('badge',(t,s)=>circle(25,7,Math.max(.05,3.3*badge(t,s))),(t,s)=>circle(25,7,Math.max(.05,3.3*badge(t,s))+1.05)),(t,s)=>badge(t,s)>.0001?1:0)
],.68,1.25,['message-0','message-1','badge']),true);
notification.constraints=[{type:'fixed',part:'notification-dot-bubble'}];

const browser=rect(5,5,22,22,2.3);
const remaining=P([[0,1],[.08,1],[.6,0,'in']],[[0,1],[.1,1],[.26,.55,'pass'],[.42,0]]);
const expired=P([[0,0],[.62,0],[.93,1,'out']],[[0,0],[.48,0],[.58,.5,'pass'],[.68,1]]);
const session=policy(make('session-expired','Session expired','The remaining session time drains, then a cross records the expired state.',[
 tint('surface',browser),stroke('browser',browser),
 fill('browser-solid',holes(rect(3.7,3.7,24.6,24.6,3.6),rect(7.5,12.5,17,12,1.5),circle(9,8.3,.9),circle(13,8.3,.9))),
 stroke('header','M5 11 L27 11'),fill('header-dot-a',circle(9,8.3,.9),'outline'),fill('header-dot-b',circle(13,8.3,.9),'outline'),
 ...ink('remaining',(t,s)=>`M10 18.5 L${10+12*remaining(t,s)} 18.5`,{visibility:(t,s)=>remaining(t,s)>.0001?1:0}),
 ...ink('cross-first',(t,s)=>{const p=Math.min(1,expired(t,s)*2);return `M12.5 15.5 L${mix(12.5,19.5,p)} ${mix(15.5,22.5,p)}`;},{visibility:(t,s)=>expired(t,s)>.0001?1:0}),
 ...ink('cross-second',(t,s)=>{const p=Math.max(0,(expired(t,s)-.5)*2);return `M12.5 22.5 L${mix(12.5,19.5,p)} ${mix(22.5,15.5,p)}`;},{visibility:(t,s)=>expired(t,s)>.5001?1:0})
],1.02,.8,['remaining','cross-first','cross-second']),true);
session.constraints=[{type:'fixed',part:'session-expired-browser'},{type:'monotonic',part:'session-expired-remaining',measure:'width',direction:-1}];
export const icons=[reconnect,cloudSync,saved,notification,session];
