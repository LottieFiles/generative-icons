import {make,F as P,posed,shown,ink,fill,stroke,bothFill,rigid,rect,circle,framed,mix,ring,policy,tint,revealLine} from './helpers.mjs';

const infoWrite=P([[0,0],[.08,0],[.55,1,'out']],[[0,0],[.3,0],[.44,.78,'pass'],[.56,1]]);
const infoDot=P([[0,0],[.1,-.5],[.35,0]],[[0,0],[.06,.3,'launch'],[.2,-1.5],[.38,.4],[.5,0]]);
const info=policy(make('info','Info','The dot cues a written information stem and its short foot; the completed letter remains.',[
 ...ring('rim',16,16,11),
 ...posed(bothFill('dot',circle(16,10,1.25)),rigid((t,s)=>({dy:infoDot(t,s)}))),
 ...ink('letter',(t,s)=>{const p=infoWrite(t,s),y=mix(15,21,Math.min(1,p/.78)),x=mix(16,19,Math.max(0,(p-.78)/.22));return `M14 15 L16 15 L16 ${y} C16 ${y} ${x} ${y} ${x} ${y}`;},{visibility:(t,s)=>infoWrite(t,s)>.0001?1:0})
],.62,.72,['letter']),true);
info.constraints=[{type:'fixed',part:'info-rim-frame'}];

const queryLift=P([[0,0],[.2,-.7,'out'],[.55,0,'smooth']],[[0,0],[.08,.3,'launch'],[.3,-1.5],[.58,.25],[.8,0]]);
const queryLook=P([[0,0],[.55,0]],[[0,0],[.3,1.2],[.58,-.25],[.8,0]]);
const queryDot=P([[0,0],[.12,0],[.3,-.4,'out'],[.62,0,'smooth']],[[0,0],[.2,0],[.26,.2,'launch'],[.44,-1.6],[.7,.3],[.92,0]]);
const help=policy(make('help','Help','The question hook lifts to ask; its dot follows, then both return to their own anchors.',[
 ...ring('rim',16,16,11),
 ...posed(ink('hook','M12 12 C12 7.8 20 7.8 20 12.5 C20 15.2 16 15.4 16 18.5'),rigid((t,s)=>({dy:queryLift(t,s),dx:queryLook(t,s)}))),
 ...posed(bothFill('dot',circle(16,22,1.2)),rigid((t,s)=>({dy:queryDot(t,s)})))
],.68,1.07),true);
help.constraints=[{type:'fixed',part:'help-rim-frame'},{type:'rigid',part:'help-hook'}];

const scan=P([[0,0],[.06,0],[.54,6.6,'smooth']],[[0,0],[.1,-1,'launch'],[.6,7.2],[.72,6.4],[.82,6.6]]);
const scanAlpha=P([[0,1],[.48,1],[.64,0,'out']],[[0,1],[.76,1],[.9,0]]);
const emptyMark=P([[0,0],[.58,0],[.84,1,'out']],[[0,0],[.84,0],[1.08,1]]);
const noResults=policy(make('no-results','No results','A scan crosses the lens once, then an empty-result dash records the answer.',[
 ...ring('lens',13.5,13.5,8.5),...ink('handle','M19.6 19.6 L26.2 26.2'),
 ...shown(posed(ink('scan','M10.2 10.8 L10.2 16.2'),rigid((t,s)=>({dx:scan(t,s)}))),scanAlpha),
 ...ink('empty',(t,s)=>`M${mix(13.5,10.4,emptyMark(t,s))} 13.5 L${mix(13.5,16.6,emptyMark(t,s))} 13.5`,{visibility:(t,s)=>emptyMark(t,s)>.0001?1:0})
],.92,1.26,['scan','empty']),true);
noResults.constraints=[{type:'fixed',part:'no-results-lens-frame'},{type:'rigid',part:'no-results-scan'}];

const tray='M5 11 L9.5 11 C11.5 11 11 16.5 13.5 16.5 L18.5 16.5 C21 16.5 20.5 11 22.5 11 L27 11 L27 23 C27 24.1 26.1 25 25 25 L7 25 C5.9 25 5 24.1 5 23 Z';
const paperLift=P([[0,0],[.08,0],[.48,-.8,'out']],[[0,0],[.1,.7,'launch'],[.45,-1.3],[.56,-.7],[.64,-.8]]);
const paperTurn=P([[0,0],[.6,0]],[[0,0],[.12,0],[.5,-7],[.64,-6]]);
const paperAlpha=P([[0,1],[.23,1],[.6,0,'out']],[[0,1],[.3,1],[.72,0]]);
const emptyState=policy(make('empty-state','Empty state','The last card lifts clear of the tray and disappears; the empty container stays visible.',[
 tint('tray-interior',tray),...ink('tray',tray),fill('tray-solid-surface',tray),
 ...shown(posed(framed('paper',rect(12,5.5,8,4,1),rect(10.9,4.4,10.2,6.2,1.8)),rigid((t,s)=>({dy:paperLift(t,s),angle:paperTurn(t,s),pivot:[16,7.5]}))),paperAlpha)
],.67,.88,['paper']),true);
emptyState.constraints=[{type:'fixed',part:'empty-state-tray'},{type:'rigid',part:'empty-state-paper-frame'}];

const waves=i=>P([[0,1],[.15+i*.1,1],[.4+i*.08,.42,'out'],[.65+i*.08,1,'out']],[[0,1],[.1+i*.18,1],[.3+i*.18,.22],[.64+i*.1,1]]);
const slash=P([[0,0],[.3,0],[.77,1,'out']],[[0,0],[.5,0],[.74,1]]);
const offline=policy(make('offline','Offline','The signal recedes from outside in as a diagonal records the offline state; the final bands retain clear contrast.',[
 ...shown(ink('wave-outer-left','M4.5 12 C5.2 11.3 6.1 10.7 6.8 10.7'),waves(0)),
 ...shown(ink('wave-outer-right','M12.5 7.2 C17.8 5.7 23.3 7.5 27.5 12'),waves(0)),
 ...shown(ink('wave-inner-left','M8.8 17 C9.5 16.1 10.5 15.5 11.1 15.3'),waves(1)),
 ...shown(ink('wave-inner-right','M18.3 13.7 C20.3 14.1 22.1 15.5 23.2 17'),waves(1)),
 ...bothFill('source',circle(16,24.5,1.6)),
 ...revealLine('slash',[6,5],[26,27],slash)
],.84,.9,['slash']),false);
offline.constraints=[{type:'fixed',part:'offline-wave-outer-right'},{type:'monotonic',part:'offline-slash',measure:'height'}];
export const icons=[info,help,noResults,emptyState,offline];
