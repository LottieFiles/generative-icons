import{make,tube,stroke,fill,bothFill,circle,annulus,capsule,rigid,curve}from'./helpers.mjs';
const E=(t,s)=>curve(t,s,[[0,14],[.1,14],[.22,18,'linear'],[.26,18,'hold'],[.38,22,'linear'],[.42,22,'hold'],[.52,24.5,'linear']],[[0,14],[.12,14],[.2,18.5,'pass'],[.24,18.5,'hold'],[.3,24.5]]);
const O=(t,s)=>curve(t,s,[[0,14],[.62,14],[.92,22,'out']],[[0,14],[.42,14],[.6,22]]);
const terminal=make('terminal-agent','Terminal agent','A command is entered, executed and produces an output line. Strong uses a short cursor cue, a faster command burst and a prompt response at Enter.',[
 ...tube('prompt','M5 9 L10 14 L5 19',{pose:rigid((t,s)=>({dx:s?curve(t,true,[],[[0,0],[.32,-1],[.42,0,'hit']]):0}))}),...tube('command',(t,s)=>`M14 14 L${E(t,s)} 14`,{visibility:(t,s)=>E(t,s)<14.4?0:1}),...bothFill('cursor',capsule([17.6,12.2],[17.6,15.8],2.6),{pose:rigid((t,s)=>({dx:E(t,s)-14})),visibility:(t,s)=>s?curve(t,true,[],[[0,1],[.05,0],[.11,1]]):1}),...tube('output',(t,s)=>`M14 22 L${O(t,s)} 22`,{visibility:(t,s)=>O(t,s)<14.4?0:1})
],.92,.6,['command','cursor','output']);terminal.duotoneFill=false;
const permissionAngle=(t,s)=>curve(t,s,[[0,0],[.05,0],[.75,90]],[[0,0],[.14,-10],[.42,97,'pass'],[.68,90]]);
const keyhole='M17.3 14.9 L17.3 20 C17.3 20.72 16.72 21.3 16 21.3 C15.28 21.3 14.7 20.72 14.7 20 L14.7 14.9 C14.2 14.5 13.7 13.8 13.7 13 C13.7 11.73 14.73 10.7 16 10.7 C17.27 10.7 18.3 11.73 18.3 13 C18.3 13.8 17.8 14.5 17.3 14.9 Z';
const permissions=make('agent-permissions','Agent permissions','The access cylinder turns to grant tool permission. Strong makes a wider preparation, a decisive quarter-turn and one short settle.',[
 fill('face-interior',circle(16,16,9.5),'tint'),stroke('face',circle(16,16,9.5)),fill('face-solid',annulus(16,16,9.5,2.6)),...bothFill('keyhole',keyhole,{pose:rigid((t,s)=>({angle:permissionAngle(t,s),pivot:[16,16]}))})
],.75,.68,['keyhole']);permissions.duotoneFill=true;
const resumeX=(t,s)=>curve(t,s,[[0,11.3],[.05,11.3],[.8,25]],[[0,11.3],[.1,11.3],[.42,25]]);
const resume=make('resume-run','Resume run','The run continues from a saved checkpoint and fills the remaining route. Strong departs faster while the checkpoint responds; completed progress stays retained.',[
 ...tube('done-track','M5 20 L11.3 20'),...bothFill('checkpoint',capsule([11.3,16.5],[11.3,23.5],2.6),{pose:rigid((t,s)=>({angle:s?curve(t,true,[],[[0,0],[.08,0],[.2,-7],[.36,3],[.52,0]]):0,pivot:[11.3,23.5]}))}),...[15,19.75,24.5].flatMap((x,i)=>tube('remaining-'+i,`M${x} 20 L${x+1} 20`)),...tube('progress',(t,s)=>`M11.3 20 L${resumeX(t,s)} 20`,{visibility:(t,s)=>resumeX(t,s)<11.6?0:1}),...bothFill('runner',circle(11.3,20,2.3),{pose:rigid((t,s)=>({dx:resumeX(t,s)-11.3}))})
],.8,.52,['progress','runner']);resume.duotoneFill=false;
const span=(i,t,s)=>curve(t,s,[[0,[10.5,15.5,19.5][i]],[[0,.25,.55][i],[10.5,15.5,19.5][i]],[[.4,.7,.95][i],[18,23,26.5][i],'out']],[[0,[10.5,15.5,19.5][i]],[[.06,.14,.3][i],[10.5,15.5,19.5][i]],[[.22,.4,.56][i],[18,23,26.5][i]]]);
const trace=make('execution-trace','Execution trace','Nested execution spans expand along a time axis. Strong produces a faster waterfall, with each child beginning before its parent finishes.',[
 ...tube('axis','M5 6 L5 25.5 L27 25.5'),...[0,1,2].flatMap(i=>tube('span-'+i,(t,s)=>`M${[9,14,18][i]} ${10+5.5*i} L${span(i,t,s)} ${10+5.5*i}`))
],.95,.56,['span']);trace.duotoneFill=false;
export const accepted={'terminal-agent':terminal,'agent-permissions':permissions,'resume-run':resume,'execution-trace':trace};
