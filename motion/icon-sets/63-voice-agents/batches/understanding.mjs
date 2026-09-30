import {make,toggle,motion,state,gesture,bubble,panel,ring,dot,tube,rigid,circle,checkPath,finalize} from './helpers.mjs';

const typing=(t,s,r)=>state(t,s,r,.86,.5);
const transcript=toggle('live-transcript','Live transcript',
  'A live utterance appends text under two committed transcript lines. The caret leads the new line and the text remains. Light writes evenly; Strong accelerates through the full line after a short input beat. Reverse removes this transcript segment.',[
  ...bubble('transcript',5.5,4.5,21,21,2),
  ...tube('line-one','M10 10 L22 10'),...tube('line-two','M10 15.3 L18.5 15.3'),
  ...tube('new-line',(t,s,r)=>`M10 20.6 L${10+9*typing(t,s,r)} 20.6`,{visibility:(t,s,r)=>typing(t,s,r)>.005?1:0}),
  ...tube('caret','M13.5 18.9 L13.5 22.3',{pose:rigid((t,s,r)=>({dx:9*typing(t,s,r),pivot:[13.5,20.6]}))})
],'Transcribing','Transcript captured',.86,.5);

const summaryP=(t,s,r)=>state(t,s,r,.9,.54);
const summary=toggle('call-summary','Call summary',
  'Two call utterances converge into a compact summary. The branch junction stays attached while summary rows form. Light condenses steadily; Strong gathers both inputs and writes the result with a faster main stroke. Reverse clears the summary.',[
  ...bubble('utterance-a',4.8,4.7,8.5,6.8,1.5),...bubble('utterance-b',18.7,4.7,8.5,6.8,1.5),
  ...tube('branch-a','M9 14.3 L9 16.5 L16 18.5'),...tube('branch-b','M23 14.3 L23 16.5 L16 18.5'),
  ...panel('summary',6.5,21,19,7,2),
  ...tube('summary-line',(t,s,r)=>`M11 24.5 L${11+10*summaryP(t,s,r)} 24.5`,{visibility:(t,s,r)=>summaryP(t,s,r)>.005?1:0}),
  ...[9,23].flatMap((x,i)=>dot('utterance-token-'+i,x,8.1,1.05,{pose:rigid((t,s,r)=>({dx:(i?-1:1)*(s?1:.5)*Math.sin(Math.PI*summaryP(t,s,r)),pivot:[x,8.1]}))}))
],'Call details','Summary ready',.9,.54);

const mood=(t,s)=>gesture(t,s,1);
const sentiment=make('sentiment-meter','Sentiment meter',
  'The caller expression lifts as the sentiment indicator moves toward positive. Light makes a small smile and shifts the marker 3.2 units; Strong opens the smile and shifts it 5.1 units before returning to the neutral reading.',[
  ...ring('face',circle(16,11,6.4)),...dot('eye-left',13.5,9.8,.65),...dot('eye-right',18.5,9.8,.65),
  ...tube('mouth',(t,s)=>`M13.5 13 C14.5 ${13+mood(t,s)} 17.5 ${13+mood(t,s)} 18.5 13`),
  ...panel('sentiment-scale',5,22,22,5.4,2.7),
  ...dot('sentiment-marker',16,24.7,1.05,{pose:rigid((t,s)=>({dx:3.2*mood(t,s),pivot:[16,24.7]}))})
],.9,.92);

const confirmP=(t,s,r)=>state(t,s,r,.82,.48);
const confirm=toggle('intent-confirmation','Intent confirmation',
  'The agent asks the caller to confirm an interpreted intent. The question resolves into a full check in the same utterance. Light traces the check calmly; Strong makes one decisive stroke. Reverse restores the question.',[
  ...bubble('confirmation',5,5,22,19,3),
  ...tube('question','M12.8 11 C12.8 7.8 19.2 7.8 19.2 11 C19.2 13.2 16 13.2 16 15.5',{visibility:(t,s,r)=>Math.max(0,1-confirmP(t,s,r)*3)}),
  ...dot('question-dot',16,19,1,{visibility:(t,s,r)=>Math.max(0,1-confirmP(t,s,r)*3)}),
  ...tube('confirmed',(t,s,r)=>checkPath(10,15,12,8,confirmP(t,s,r)),{visibility:(t,s,r)=>Math.min(1,confirmP(t,s,r)*3)})
],'Confirm intent','Intent confirmed',.82,.48);

const scanPose=rigid((t,s)=>({dx:motion(t,s,[[0,0],[.48,10],[.62,10],[1,0]],[[0,0],[.06,-.7],[.3,10,'hit'],[.46,10],[.72,0,'hit']]),pivot:[14,27]}));
const language=make('language-detection','Language detection',
  'A detector scans the language of an utterance. Light sweeps the full ten-unit sampling rail in 0.48 s; Strong prepares then traverses in 0.24 s. The language glyph and speech card stay fixed.',[
  ...bubble('language',5,4.5,22,17.5,2),
  ...tube('letter','M10 17 L16 8.5 L22 17'),...tube('letter-crossbar','M11.6 14.7 L20.4 14.7'),
  ...tube('sampling-rail','M13 27 L27 27'),...tube('detector','M14 25.5 L14 28.4',{pose:scanPose})
],1,.72);

export const icons=[transcript,summary,sentiment,confirm,language].map(finalize);
