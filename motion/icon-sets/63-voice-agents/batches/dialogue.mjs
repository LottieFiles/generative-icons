import {make,toggle,motion,state,gesture,bubble,panel,disk,dot,tube,rigid,posed,circle,finalize} from './helpers.mjs';

// A speaking token moves between two utterances, not between voice personas.
const turn=toggle('turn-taking','Turn taking',
  'The caller yields the floor and the token moves into the agent utterance. Light crosses in 0.72 s. Strong prepares, travels in 0.34 s along a wider bowed route, and lands. Reverse returns the floor to the caller.',[
  ...bubble('caller',5,4.8,15,8,2),...bubble('agent',12,20.2,15,7,1.3),
  ...dot('floor-token',12.5,8.8,1.25,{pose:rigid((t,s,r)=>{const p=state(t,s,r,.72,.46);return {dx:7*p+(s?2:0)*Math.sin(Math.PI*p),dy:15*p,pivot:[12.5,8.8]};})})
],'Caller turn','Agent turn',.72,.46);

const interrupt=(t,s,r)=>state(t,s,r,.72,.48);
const barge=toggle('barge-in','Barge in',
  'Caller speech enters beneath the speaking agent. The incoming arrow reaches the agent bubble and its output bars stop. Strong briefly raises the output before cutting it off. Reverse releases the interruption and restores speech.',[
  ...bubble('agent',5,4.5,22,13,1.3),...bubble('caller',5,22.8,9,4.5,.7),
  ...[10.5,16,21.5].flatMap((x,i)=>tube('output-'+i,(t,s,r)=>{const p=interrupt(t,s,r),prep=s&&!r?motion(t,true,[],[[0,0],[.08,.5],[.22,0]]):0,h=(i===1?2.3:1.2)*(1-p)+.12+prep;return `M${x} ${11-h} L${x} ${11+h}`;})),
  ...tube('incoming',(t,s,r)=>{const p=interrupt(t,s,r);return `M19 28 L23.5 28 L23.5 ${21.5-4*p}`;}),
  ...tube('interrupt-head',(t,s,r)=>{const y=21.5-4*interrupt(t,s,r);return `M20.8 ${y+2.7} L23.5 ${y} L26.2 ${y+2.7}`;})
],'Agent speaking','Caller interrupted',.72,.48);

const endP=(t,s,r)=>state(t,s,r,.78,.46);
const end=toggle('end-of-turn-detection','End of turn detection',
  'A trailing utterance reaches its end boundary. The final syllable contracts while the end gate closes. Light settles over 0.78 s; Strong briefly gathers the syllable and closes the gate decisively. Reverse opens a new turn.',[
  ...bubble('utterance',5,5.5,22,16,2.5),
  ...tube('speech-start','M10 12 L10 15'),
  ...tube('last-syllable',(t,s,r)=>{const p=endP(t,s,r),h=3.6*(1-p)+.2;return `M15 ${13.5-h} L15 ${13.5+h}`;}),
  ...tube('end-gate',(t,s,r)=>{const p=endP(t,s,r);return `M22 ${13.5-4*p} L22 ${13.5+4*p}`;},{visibility:(t,s,r)=>.3+.7*endP(t,s,r)}),
  ...tube('deadline','M13 27 L24 27')
],'Utterance open','Turn ended',.78,.46);

const ackPose=rigid((t,s)=>({dy:-gesture(t,s,1.9),pivot:[24,24]}));
const ack=make('backchannel','Backchannel',
  'The main utterance remains steady while a small acknowledgement nods upward. Light nods 1.9 units once; Strong makes a quicker 3-unit acknowledgement with a small settling response.',[
  ...bubble('utterance',4.7,5.2,17,13,2.3),
  ...tube('phrase','M9.5 11.7 L16.7 11.7'),
  ...posed([...bubble('acknowledgement',20,23,7.3,4.7,1.2),...dot('ack-dot',23.7,25.2,.7)],ackPose)
],.9,.92);

const diarize=make('speaker-diarization','Speaker diarization',
  'Two identified speakers keep separate utterance lanes. Their speech segments advance independently without changing lanes. Strong doubles the useful segment travel and separates the response timing.',[
  ...disk('speaker-a',6.6,8.5,2.4),...panel('speaker-b',4.2,20.7,4.8,4.8,1.2),
  ...tube('lane-a','M12 8.5 L27 8.5'),...tube('lane-b','M12 23.1 L27 23.1'),
  ...[8.5,23.1].flatMap((y,i)=>[-1,0,1].flatMap((side,j)=>{
    const x=(i?23:16)+side*2.4,h=side?1.2:3;
    return tube('utterance-'+i+'-'+j,`M${x} ${y-h} L${x} ${y+h}`,{pose:rigid((t,s)=>({dx:(i?-1:1)*gesture(Math.max(0,t-i*.07),s,1.8),pivot:[i?23:16,y]}))});
  }))
],.97,.99);

export const icons=[turn,barge,end,ack,diarize].map(finalize);
