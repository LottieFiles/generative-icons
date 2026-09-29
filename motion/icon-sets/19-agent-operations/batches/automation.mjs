import{make,tube,stroke,fill,bothFill,circle,annulus,capsule,polygon,holes,rigid,curve}from'./helpers.mjs';
const handAngle=(t,s)=>curve(t,s,[[0,0],[.1,0],[.72,150]],[[0,0],[.06,0],[.14,18,'pass'],[.36,150]]);
const scheduled=make('scheduled-agent','Scheduled agent','The timer advances to its marked time, then releases a run. Strong drives the hand to the same scheduled time faster; the release follows arrival and the data never overshoots.',[
 fill('dial-interior',circle(12,16,8.5),'tint'),stroke('dial',circle(12,16,8.5)),fill('dial-solid',annulus(12,16,8.5,2.6)),stroke('tick','M16.25 23.36 L17.25 25.09'),fill('tick-solid',capsule([16.45,23.707],[17.25,25.09],2.6)),...bothFill('hub',circle(12,16,1.3)),...tube('hand','M12 16 L12 11.2',{pose:rigid((t,s)=>({angle:handAngle(t,s),pivot:[12,16]}))}),...tube('release','M24.5 12.5 L27.5 16 L24.5 19.5',{pose:rigid((t,s)=>({dx:curve(t,s,[[0,1],[.72,1],[.92,0,'out']],[[0,1],[.36,1],[.52,0]])})),visibility:(t,s)=>curve(t,s,[[0,0],[.72,0],[.84,1]],[[0,0],[.36,0],[.46,1]])})
],.92,.52,['hand','release']);scheduled.duotoneFill=true;
const event=make('event-trigger','Event trigger','A signal pulse reaches a listener and triggers its output. Strong carries the pulse faster with a decisive arrival. The wire remains fixed and the completed trigger stays.',[
 ...tube('wire','M4 22 L28 22'),...bothFill('listener',circle(26.5,22,1.8)),...tube('pulse','M8 22 L8 13 L14 13 L14 22',{pose:rigid((t,s)=>({dx:curve(t,s,[[0,0],[.08,0],[.62,10.5]],[[0,0],[.06,0],[.14,1.2,'pass'],[.32,10.5]])}))}),...tube('output','M26.5 5 L26.5 9.4',{pose:rigid((t,s)=>({dy:curve(t,s,[[0,.6],[.62,.6],[.8,0,'out']],[[0,.6],[.32,.6],[.46,0]])})),visibility:(t,s)=>curve(t,s,[[0,0],[.62,0],[.74,1]],[[0,0],[.32,0],[.42,1]])})
],.8,.46,['pulse','output']);event.duotoneFill=false;
const jaw=(t,s)=>curve(t,s,[[0,0],[.1,0],[.64,3.5]],[[0,0],[.12,-1],[.32,3.5,'hit']]);
const textY=(t,s)=>curve(t,s,[[0,0],[.16,0],[.64,1.3]],[[0,0],[.2,0],[.34,1.3]]);
const compaction=make('context-compaction','Context compaction','The context closes to a compact readable group. Strong opens the jaws briefly before a fast compression; the text stays ordered without stretching or overshoot.',[
 ...tube('jaw-top','M6 8 L6 5 L26 5 L26 8',{pose:rigid((t,s)=>({dy:jaw(t,s)}))}),...tube('jaw-bottom','M6 24 L6 27 L26 27 L26 24',{pose:rigid((t,s)=>({dy:-jaw(t,s)}))}),...tube('line-top','M9.8 11 L20 11',{pose:rigid((t,s)=>({dy:textY(t,s)}))}),...tube('line-middle','M9.8 16 L22.5 16'),...tube('line-bottom','M9.8 21 L18 21',{pose:rigid((t,s)=>({dy:-textY(t,s)}))})
],.64,.34,['jaw','line-top','line-bottom']);compaction.duotoneFill=false;
const page=polygon([[9.5,9.5],[18.5,9.5],[22.5,13.5],[22.5,23],[9.5,23]],1.2);
const pagePose=rigid((t,s)=>({dy:curve(t,s,[[0,0],[.1,0],[.64,-5.5,'out']],[[0,0],[.1,.6,'launch'],[.3,-6],[.5,-5.5]])}));
const artifact=make('agent-artifact','Agent artifact','A finished artifact rises out of the output tray. Strong presses down before a larger, faster lift and a short settle. The page stays rigid and its open tray has no invented fill.',[
 ...tube('tray','M5 20 L5 25 C5 26.4 6.6 27.5 8 27.5 L24 27.5 C25.4 27.5 27 26.4 27 25 L27 20'),fill('page-interior',page,'tint',{pose:pagePose}),stroke('page-frame',page,{pose:pagePose}),fill('page-solid',holes(page,capsule([13.25,14.8],[18.75,14.8],1.6),capsule([13.25,18.6],[16.75,18.6],1.6)),'solid',{pose:pagePose}),stroke('page-text-0','M13.25 14.8 L18.75 14.8',{pose:pagePose}),stroke('page-text-1','M13.25 18.6 L16.75 18.6',{pose:pagePose})
],.64,.5,['page']);artifact.duotoneFill=true;
const budget=make('budget-limit','Budget limit','Usage advances along a rail and stops at its limit. Strong has a faster decisive run but never crosses the limit or refunds the value. The result remains until preview reset.',[
 ...tube('rail','M4 24 L22.5 24 C23.33 24 24 23.33 24 22.5 L24 13.5 C24 12.67 23.33 12 22.5 12 L20 12'),...bothFill('token',circle(8,21,2.2),{pose:rigid((t,s)=>({dx:curve(t,s,[[0,0],[.1,0],[.7,11.4]],[[0,0],[.06,0],[.14,.8,'pass'],[.34,11.4,'hit']])}))})
],.7,.34,['token']);budget.duotoneFill=false;
export const automation={'scheduled-agent':scheduled,'event-trigger':event,'context-compaction':compaction,'agent-artifact':artifact,'budget-limit':budget};
