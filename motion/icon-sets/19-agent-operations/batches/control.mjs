import{make,toggle,tube,stroke,fill,bothFill,framed,circle,rect,capsule,rigid,curve}from'./helpers.mjs';
const chipY=(t,s,r)=>r?curve(t,s,[[0,12.5],[.2,12.5],[.48,0]],[[0,12.5],[.16,12.5],[.4,0]]):curve(t,s,[[0,0],[.05,0],[.55,12.5,'out']],[[0,0],[.14,-.5],[.34,12.9,'hit'],[.5,12.5]]);
const lidAngle=(t,s,r)=>r?curve(t,s,[[0,90],[.2,0]],[[0,90],[.16,0]]):curve(t,s,[[0,0],[.4,0],[.95,90]],[[0,0],[.28,0],[.52,94],[.72,90]]);
const sandbox=toggle('agent-sandbox','Sandbox','A process enters an isolated container and the hatches close. Strong drops the process faster and gives the latch one short response. Unseal opens the doors before raising the process.',[
 ...tube('box','M7 14 L7 24.5 C7 26.16 8.34 27.5 10 27.5 L22 27.5 C23.66 27.5 25 26.16 25 24.5 L25 14'),...framed('chip','M13 6 L19 6 C19.9 6 21 7.1 21 8 L21 9 C21 9.9 19.9 11 19 11 L13 11 C12.1 11 11 9.9 11 9 L11 8 C11 7.1 12.1 6 13 6 Z',undefined,{pose:rigid((t,s,r)=>({dy:chipY(t,s,r)}))}),...[7,25].flatMap((x,i)=>tube('door-'+i,`M${x} 14 L${x} 5`,{pose:rigid((t,s,r)=>({angle:lidAngle(t,s,r)*(i?-1:1),pivot:[x,14]}))}))
],.95,.72,'Open','Sealed',.48,.4);sandbox.duotoneFill=true;
const lean=(t,s)=>curve(t,s,[[0,0],[.3,0],[.55,3],[1.1,0]],[[0,0],[.2,0],[.32,-3],[.46,10,'pass'],[.7,7],[.9,0]]);
const typed=(t,s)=>curve(t,s,[[0,0],[.55,0],[1.15,9]],[[0,0],[.44,0],[.66,9]]);
const caretX=(t,s)=>curve(t,s,[[0,0],[.45,0],[.55,3.7,'out'],[1.15,12.7]],[[0,0],[.36,0],[.44,3.7,'launch'],[.66,12.7]]);
const personPose=rigid((t,s)=>({angle:lean(t,s),pivot:[9,15]}));
const shoulders='M4.5 15 C4.5 12.3 6.5 11.5 9 11.5 C11.5 11.5 13.5 12.3 13.5 15';
const human=make('human-input','Human input','The run waits for input, a person leans toward the field and enters guidance. Strong types the full line faster with a clearer lean; the completed text stays.',[
 ...bothFill('head',circle(9,6.5,2.2),{pose:personPose}),stroke('shoulders',shoulders,{pose:personPose}),fill('shoulders-solid',shoulders+' Z','solid',{pose:personPose}),fill('field-interior',rect(5.5,19.5,21,8.5,2.5),'tint'),...tube('field',rect(5.5,19.5,21,8.5,2.5)),...tube('text',(t,s)=>`M9.5 23.75 L${9.5+typed(t,s)} 23.75`,{visibility:(t,s)=>typed(t,s)<.3?0:1}),...bothFill('caret',capsule([9.5,22.9],[9.5,24.6],2),{pose:rigid((t,s)=>({dx:caretX(t,s)})),visibility:(t,s)=>curve(t,s,[[0,1],[.15,0],[.3,1]],[[0,1],[.12,0],[.24,1]])})
],1.15,.9,['text','caret']);human.duotoneFill=true;
const flagY=(t,s,r)=>r?curve(t,s,[[0,8],[.24,0]],[[0,8],[.26,0]]):curve(t,s,[[0,0],[.05,0],[.6,8]],[[0,0],[.15,-.5],[.36,8,'hit']]);
const flutter=(t,s,r)=>r?curve(t,s,[[0,0],[.1,.3],[.24,0]],[[0,0],[.1,.4],[.26,0]]):curve(t,s,[[0,0],[.55,0],[.75,.4],[1.05,0]],[[0,0],[.36,0],[.46,1.4],[.62,-.5],[.82,0]]);
const flag=(t,s,r,tint=false)=>{const f=flutter(t,s,r),x=.06*Math.abs(f);return(tint?'M16 5.2':'M16 14 L16 5.2')+` C16 4.4 16.7 4 17.4 4.3 L${22.2-x} ${6.3-f} C${23-x} ${6.6-f} ${23-x} ${7.4-f} ${22.2-x} ${7.7-f} L17.4 9.7 C16.7 10 16 9.6 16 8.8`+(tint?' L16 5.2 Z':'');};
const flagSolid=(t,s,r)=>{const f=flutter(t,s,r),x=.06*Math.abs(f);return`M14.7 14 L14.7 5.2 C14.7 3.9 15.9 3 17.2 3.4 L${22.8-x} ${5.7-f} C${24.1-x} ${6.2-f} ${24.1-x} ${7.8-f} ${22.8-x} ${8.3-f} L18 10.2 C17.6 10.4 17.3 10.8 17.3 11.3 L17.3 14 C17.3 14.72 16.72 15.3 16 15.3 C15.28 15.3 14.7 14.72 14.7 14 Z`;};
const flagPose=rigid((t,s,r)=>({dy:flagY(t,s,r)}));
const checkpoint=toggle('agent-checkpoint','Agent checkpoint','A flag plants at the current execution point and its cloth responds to the landing. Strong plants it faster and gives the attached cloth a larger response. Remove lifts the flag opaquely.',[
 ...tube('track','M5 23 L27 23'),...bothFill('node',circle(16,23,2.4)),fill('cloth',(t,s,r)=>flag(t,s,r,true),'tint',{pose:flagPose}),stroke('flag',(t,s,r)=>flag(t,s,r,false),{pose:flagPose}),fill('flag-solid',flagSolid,'solid',{pose:flagPose})
],1.05,.82,'Prepared','Checkpoint saved',.24,.26);checkpoint.duotoneFill=true;
export const control={'agent-sandbox':sandbox,'human-input':human,'agent-checkpoint':checkpoint};
