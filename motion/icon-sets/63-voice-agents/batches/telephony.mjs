import {make,toggle,motion,state,gesture,bubble,panel,ring,disk,dot,tube,framed,rigid,posed,circle,receiver,checkPath,finalize} from './helpers.mjs';

const silence=make('silence-timeout','Silence timeout',
  'A quiet utterance is watched by a timeout clock. Its long hand sweeps the waiting interval and returns to the armed reading. Light covers 50 degrees; Strong covers 80 degrees with a clear terminal beat.',[
  ...bubble('silence',5,5,21,8,2),...tube('quiet','M10 9 L21 9'),
  ...ring('clock',circle(20.5,23,5.6)),
  ...tube('minute-hand','M20.5 23 L20.5 20.8',{pose:rigid((t,s)=>({angle:gesture(t,s,50),pivot:[20.5,23]}))}),
  ...tube('short-hand','M20.5 23 L19.1 23.8'),...tube('waiting-dot','M7 23 L10 23')
],.9,.92);

const keyPose=rigid((t,s)=>({dy:gesture(t,s,1.25),pivot:[10.5,16]}));
const tone=(x,y,n,t,s)=>{const a=.8+Math.max(0,gesture(t,s,.9)),w=8/n;let d=`M${x} ${y}`;for(let i=0;i<n;i++)d+=` C${x+i*w+w*.16} ${y-a} ${x+i*w+w*.34} ${y-a} ${x+i*w+w*.5} ${y} C${x+i*w+w*.66} ${y+a} ${x+i*w+w*.84} ${y+a} ${x+(i+1)*w} ${y}`;return d;};
const dtmf=make('dtmf-input','DTMF input',
  'One telephone key emits the two simultaneous frequencies of a keypad tone. Light depresses the key 1.25 units; Strong presses 2 units and opens both tone envelopes with one release.',[
  ...posed([...panel('key',5,8,11,15,2.4),...tube('digit-one','M8.5 13.2 L10.7 11.5 L10.7 19.5')],keyPose),
  ...tube('low-tone',(t,s)=>tone(20,11.5,1,t,s)),...tube('high-tone',(t,s)=>tone(20,21.2,2,t,s))
],.9,.92);

const holdP=(t,s,r)=>state(t,s,r,.68,.44);
const hold=toggle('on-hold','On hold',
  'The call is parked as pause bars extend above the receiver. Light lowers the receiver 1.5 units and extends the bars evenly. Strong prepares the receiver upward before a decisive park. Reverse resumes the call.',[
  ...framed('receiver',receiver,receiver,{pose:rigid((t,s,r)=>({dy:1.5*holdP(t,s,r)-(s&&!r?.6*Math.sin(Math.PI*holdP(t,s,r)):0),pivot:[16,21]}))}),
  ...[12.5,19.5].flatMap((x,i)=>tube('pause-'+i,(t,s,r)=>{const p=holdP(t,s,r);return `M${x} ${10-4*p} L${x} ${10+4*p}`;}))
],'Call active','Call held',.68,.44);

const consentP=(t,s,r)=>state(t,s,r,.78,.48);
const consent=toggle('recording-consent','Recording consent',
  'Caller permission enables call recording. The permission check is drawn while the recording dot becomes active. Light confirms in 0.78 s; Strong commits with one faster full check. Reverse withdraws consent.',[
  ...ring('record-ring',circle(10,16,6)),
  ...dot('record-dot',10,16,2,{visibility:(t,s,r)=>.18+.82*consentP(t,s,r)}),
  ...tube('permission',(t,s,r)=>checkPath(20,16.5,8,7,consentP(t,s,r)),{visibility:(t,s,r)=>Math.min(1,5*consentP(t,s,r))}),
  ...tube('permission-line','M20 24 L27 24')
],'Permission pending','Recording permitted',.78,.48);

const routeP=(t,s,r)=>state(t,s,r,.84,.5);
const topReceiverPose=rigid(()=>({dy:-13,pivot:[16,21]}));
const ivr=toggle('ivr-menu','IVR menu',
  'A phone menu routes the caller from its trunk into a chosen branch. Light traverses the junction evenly. Strong waits briefly at the decision and then drives into the selected option. Reverse returns to the menu.',[
  ...posed(framed('receiver',receiver,receiver),topReceiverPose),
  ...tube('trunk','M16 13.5 L16 18'),...tube('left-branch','M16 18 L9 18 L9 23'),...tube('right-branch','M16 18 L23 18 L23 23'),
  ...panel('option-one',5,23,8,5,1.8),...panel('option-two',19,23,8,5,1.8),
  ...dot('route-token',16,14,1.05,{pose:rigid((t,s,r)=>{const p=routeP(t,s,r);return {dx:7*Math.min(1,Math.max(0,(p-.22)/.44)),dy:p<.22?4*p/.22:p<.66?4:4+7.5*(p-.66)/.34,pivot:[16,14]};})})
],'Choose option','Option selected',.84,.5);

export const icons=[silence,dtmf,hold,consent,ivr].map(finalize);
