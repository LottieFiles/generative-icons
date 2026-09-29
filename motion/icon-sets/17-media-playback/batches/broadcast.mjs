import svgpath from 'svgpath';
import {make,toggle,continuous,tube,stroke,fill,bothFill,framed,rect,polygon,rigid,curve,arc,circle,holes,TAU} from './helpers.mjs';
const headphones=make('headphones','Headphones','The ear cups close around the listener and release. Strong makes a clear inward swing with a small preparation and one damped response.',[
 stroke('band','M6.5 16 C6.5 10.75 10.75 6.5 16 6.5 C21.25 6.5 25.5 10.75 25.5 16'),{...stroke('band-solid','M6.5 17 C6.5 11.2 10.75 6.5 16 6.5 C21.25 6.5 25.5 11.2 25.5 17'),style:'solid',width:2.6},
 ...[0,1].flatMap(i=>framed('cup-'+i,rect(i?23:4,16,5,9,2.2),undefined,{pose:rigid((t,s)=>({angle:(i?-1:1)*curve(t,s,[[0,0],[.42,-8,'out'],[.90,0]],[[0,0],[.10,3],[.30,-22,'launch'],[.58,2],[.78,-1],[.95,0]]),pivot:[i?25.5:6.5,16]}))}))
],.90,.95);headphones.duotoneFill=true;
const periodic=(t,base,amplitude)=>base+amplitude*(1-Math.cos(TAU*t/2.4));
const podcast=continuous('podcast','Podcast','A central voice sends two clear signal rings outward and back. Strong expands the same rings more than three times farther without crowding their gaps.',[
 ...bothFill('source',circle(16,14,2.6)),...tube('stem','M16 14 L16 26'),...tube('base','M11 26 L21 26'),
 ...[0,1].flatMap(i=>tube('ring-'+i,(t,s)=>arc(16,14,periodic(t,5.5,s?.8:.25)+i*3.6,132,408),{visibility:(t,s)=>s&&i?.8+.2*Math.cos(TAU*(t-.3)/2.4):1}))
]);
const live=continuous('live-broadcast','Live broadcast','A fixed broadcast source sends an even signal to both sides. Strong has nearly three times the outward travel while the wave spacing remains clear.',[
 ...bothFill('source',circle(16,10,2.4)),...tube('mast','M16 11.5 L16 27'),
 ...[0,1,2,3].flatMap(i=>tube('wave-'+i,(t,s)=>arc(16,10,periodic(t,5.5,s?1:.35)+Math.floor(i/2)*3.6,i%2?-30:150,i%2?30:210),{visibility:(t,s)=>s&&i>1?.8+.2*Math.cos(TAU*(t-.3)/2.4):1}))
]);
const castWidth=(t,s,r)=>r?(t<.16?8:0):curve(t,s,[[0,0],[.15,0],[.65,8,'out']],[[0,0],[.18,0],[.42,8.4,'launch'],[.56,7.85],[.66,8]]);
const castA=(t,s,r)=>{if(r)return curve(t,false,[[0,1],[.14,0,'out']],[]);if(castWidth(t,s,r)<1.5)return 0;return curve(t,s,[[0,0],[.15,0],[.28,1,'out']],[[0,0],[.18,0],[.26,1]]);};
const cast=toggle('cast','Cast','A signal reaches the display and reveals its content. Strong sends the signal faster, then opens the content tile with a decisive wipe and a small settle.',[
 ...tube('frame','M5 10.5 L5 8.7 C5 7.21 6.21 6 7.7 6 L24.3 6 C25.79 6 27 7.21 27 8.7 L27 19.3 C27 20.79 25.79 22 24.3 22 L16.5 22'),...bothFill('source',circle(5,22,1.4)),
 ...[0,1].flatMap(i=>tube('wave-'+i,arc(5,22,i?7.8:4.2,270,360),{visibility:(t,s,r)=>r?1:curve(t,s,[[0,1],[.10+i*.10,.3,'out'],[.35+i*.15,1,'out']],[[0,1],[.06+i*.06,.2],[.22+i*.1,1]])})),
 ...framed('tile',(t,s,r)=>rect(15,9.5,castWidth(t,s,r),6,Math.min(1.5,castWidth(t,s,r)/2)),undefined,{visibility:castA})
],.65,.66,'Idle','Casting',.30,.30);Object.assign(cast,{duotoneFill:true,previewReset:true,reverseSuffix:'-reset',reverseLabel:'Reset preview'});
const shareY=(t,s,r)=>r?curve(t,s,[[0,-2.4],[.30,0,'out']],[[0,-2.4],[.22,.45,'hit'],[.38,0]]):curve(t,s,[[0,0],[.50,-2.4,'out']],[[0,0],[.10,.5],[.36,-3.2,'launch'],[.55,-2.05],[.70,-2.5],[.82,-2.4]]);
const shareArrow=polygon([[16,11],[19.9,14.9],[18.5,16.3],[17,14.8],[17,17.5],[15,17.5],[15,14.8],[13.5,16.3],[12.1,14.9]],.6);
const share=toggle('screen-share','Screen share','The sharing arrow rises inside a stable display. Strong dips briefly, rises farther and settles at the same active position.',[
 fill('screen-interior',rect(5,5,22,16,2.7),'tint'),stroke('screen-frame',rect(5,5,22,16,2.7)),fill('screen-solid',(t,s,r)=>holes(rect(5,5,22,16,2.7),svgpath(shareArrow).translate(0,shareY(t,s,r)).toString())),
 stroke('stand','M16 21 L16 26'),{...stroke('stand-solid','M16 20 L16 26'),style:'solid',width:2.6},...tube('foot','M11 26 L21 26'),
 ...['M16 17 L16 12.7','M13 15.5 L16 12.5 L19 15.5'].map((d,i)=>stroke('arrow-'+i,d,{pose:rigid((t,s,r)=>({dy:shareY(t,s,r)}))}))
],.50,.82,'Idle','Sharing',.34,.42);share.duotoneFill=true;
export const broadcast={headphones,podcast,'live-broadcast':live,cast,'screen-share':share};
