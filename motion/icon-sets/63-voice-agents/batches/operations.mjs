import {make,toggle,motion,state,gesture,bubble,panel,ring,dot,tube,framed,rigid,posed,circle,finalize} from './helpers.mjs';

const packetTravel=(t,s)=>motion(t,s,[[0,0],[.58,15.5],[.72,15.5],[.74,0,'linear']],[[0,0],[.06,0],[.32,15.5,'hit'],[.49,15.5],[.51,0,'linear']]);
const packetAlpha=(t,s)=>motion(t,s,[[0,1],[.58,1],[.68,0],[.8,0],[1.08,1]],[[0,1],[.32,1],[.44,0],[.57,0],[.84,1]]);
const buffer=make('speech-buffer','Speech buffer',
  'A queued speech chunk feeds the agent output, is consumed, and the queue refills. Light delivers the chunk over 0.58 s. Strong prepares then drives the full 15.5-unit path in 0.26 s. Refill occurs only while the packet is invisible.',[
  ...[5,13.5,22].flatMap((y,i)=>panel('buffer-slot-'+i,4.5,y,6,5.2,1.7)),
  ...tube('feed','M10.5 16.1 L17.5 16.1'),...bubble('speech-output',17.5,8.5,11,13.5,2.4),
  ...dot('speech-packet',7.5,16.1,1.1,{pose:rigid((t,s)=>({dx:packetTravel(t,s),pivot:[7.5,16.1]})),visibility:packetAlpha})
],1.08,.84);

const mask='M10 5.5 L20 5.5 C22.2 5.5 23 6.8 23 9 L23 17.5 C23 22.5 19.5 26 15 27 C10.5 26 7 22.5 7 17.5 L7 9 C7 6.8 7.8 5.5 10 5.5 Z';
const personaPose=rigid((t,s)=>({angle:gesture(t,s,5),pivot:[15,20]}));
const persona=make('agent-persona','Agent persona',
  'A character mask gives the voice agent its conversational persona. The face inclines toward the caller. Light tilts 5 degrees; Strong tilts 8 degrees after preparation, while the voice accent follows the same rigid head.',[
  ...posed([...ring('mask',mask),...tube('eye-left','M10.7 12 L12.7 12'),...tube('eye-right','M17.3 12 L19.3 12'),...tube('smile','M11.5 18 C13 20 17 20 18.5 18')],personaPose),
  ...tube('voice-accent','M27 11 C29 13 29 18 27 20',{pose:rigid((t,s)=>({dx:-Math.max(0,gesture(t,s,.6)),pivot:[27,16]}))})
],.9,.92);

const concurrent=make('concurrent-calls','Concurrent calls',
  'Three live call lanes carry utterances at the same time. Both endpoints remain fixed while independent packets traverse their lanes. Light moves 2 units; Strong drives 3.2 units with staggered responses.',[
  ...[6,15,24].flatMap((y,i)=>[
    ...bubble('caller-'+i,4.5,y-2.3,6.8,4.6,1.1),...bubble('agent-'+i,21.2,y-2.3,6.3,4.6,1.1),
    ...tube('packet-'+i,`M14 ${y} L17 ${y}`,{pose:rigid((t,s)=>({dx:(i%2?-1:1)*gesture(Math.max(0,t-i*.08),s,1.4),pivot:[15.5,y]}))})
  ])
],1.06,1.08);

const flagP=(t,s,r)=>state(t,s,r,.78,.5);
const flag='M21 5.3 C23.3 4.4 25.5 6.8 28 5.7 L28 11.8 C25.5 12.9 23.3 10.5 21 11.4 Z';
const escalation=toggle('escalation-flag','Escalation flag',
  'A call issue raises an escalation flag. The flag slides ten units up a fixed pole while the flagged conversation remains visible. Light raises it directly; Strong gathers low before one decisive lift. Reverse lowers the escalation.',[
  ...bubble('call-issue',4.5,18.3,12,8.3,1.5),...tube('issue','M10.5 21 L10.5 23.8'),
  ...tube('pole','M21 4.5 L21 28'),
  ...framed('flag',flag,flag,{pose:rigid((t,s,r)=>{const p=flagP(t,s,r);return {dy:10*(1-p)+(s&&!r?.7*Math.sin(Math.PI*p):0),pivot:[21,5.3]};})})
],'Normal priority','Escalated',.78,.5);

const detect=make('voicemail-detection','Voicemail detection',
  'An answering-machine greeting is sampled above a detection eye. The pupil searches across the greeting channel. Light scans 1.8 units; Strong scans 2.9 units with a faster recognition beat.',[
  ...bubble('greeting',5,4.5,22,11.2,1.8),
  ...[11,16,21].flatMap((x,i)=>tube('greeting-syllable-'+i,`M${x} ${10-(i===1?2.5:1)} L${x} ${10+(i===1?2.5:1)}`)),
  ...ring('detection-eye','M6 24 C10 18.7 22 18.7 26 24 C22 29.3 10 29.3 6 24 Z'),
  ...dot('detector-pupil',16,24,1.7,{pose:rigid((t,s)=>({dx:gesture(t,s,1.8),pivot:[16,24]}))})
],.9,.92);

export const icons=[buffer,persona,concurrent,escalation,detect].map(finalize);
