import{make,toggle,tube,stroke,fill,bothFill,framed,circle,rect,capsule,polygon,arc,rigid,curve}from'./helpers.mjs';
// Arc-length lookup changes speed only: sampled positions remain on the exact cubic.
const cubic=(a,b,c,d,u)=>[0,1].map(i=>(1-u)**3*a[i]+3*(1-u)**2*u*b[i]+3*(1-u)*u*u*c[i]+u**3*d[i]);
function along(fn){const table=[[0,0]];let prev=fn(0),sum=0;for(let i=1;i<=300;i++){const p=fn(i/300);sum+=Math.hypot(p[0]-prev[0],p[1]-prev[1]);table.push([i/300,sum]);prev=p;}return p=>{const target=Math.max(0,Math.min(1,p))*sum;let i=table.findIndex(v=>v[1]>=target);if(i<=0)return fn(i===0?0:1);const a=table[i-1],b=table[i];return fn(a[0]+(b[0]-a[0])*(target-a[1])/(b[1]-a[1]));};}
const retryAngle=(t,s)=>curve(t,s,[[0,0],[.75,360,'out']],[[0,0],[.1,-20],[.42,372,'pass'],[.55,360]]);
const retryPose=rigid((t,s)=>({angle:retryAngle(t,s),pivot:[16,13]}));
const retry=make('retry-step','Retry step','A return arrow retries the marked step. Strong winds back, makes a fast full turn and settles once. The completed step stays; reset clears only its result.',[
 ...tube('track-a','M6.5 27.5 L11.5 27.5'),...tube('track-b','M20.5 27.5 L25.5 27.5'),...[6.5,16,25.5].flatMap((x,i)=>bothFill('node-'+i,circle(x,27.5,2.2),{visibility:(t,s)=>i===0?1:i===2?.35:curve(t,s,[[0,.35],[.45,.35],[.72,1,'out']],[[0,.35],[.32,.35],[.5,1]])})),stroke('loop',arc(16,13,6.5,-150,130)+' L7.61 14.44',{pose:retryPose}),stroke('head','M10.01 14.37 L7.61 14.44 L7.94 16.83',{pose:retryPose}),stroke('loop-solid',arc(16,13,6.5,-150,130)+' L9.14 15.73',{style:'solid',width:2.6,pose:retryPose}),fill('head-solid',polygon([[7.61,14.44],[10.41,14.34],[8,17.21]],.3),'solid',{pose:retryPose})
],.75,.55,['node-1']);retry.duotoneFill=false;retry.cycleDegrees=360;

const forkPoint=(i,p)=>{if(i===1)return[12.76175+(25.5-12.76175)*p,16];const y=i===0?7.5:24.5;return p<.5?cubic([9,16],[12.5,16],[12.5,y],[16,y],.55+.45*p*2):[16+9.5*(p-.5)*2,y];};
const forkRoutes=[0,1,2].map(i=>along(p=>forkPoint(i,p)));
const dispatch=(t,s,r)=>r?t<.12?1:0:curve(t,s,[[0,0],[.05,0],[.65,1]],[[0,0],[.08,0],[.36,1,'hit']]);
const packetAlpha=(t,s,r)=>r?curve(t,false,[[0,1],[.08,0]],[]):curve(t,s,[[0,0],[.02,0],[.08,1]],[[0,0],[.06,1]]);
const socketAlpha=(t,s,r)=>r?curve(t,false,[[0,1],[.1,1],[.24,.45]],[]):curve(t,s,[[0,.45],[.58,.45],[.78,1,'out']],[[0,.45],[.36,.45],[.46,1]]);
const parallel=toggle('parallel-tools','Parallel tools','Three separated packets run through concurrent tool lanes. Strong dispatches all three decisively and the sockets confirm arrival. Reset hides each packet inside its opaque socket before dimming the socket.',[
 ...tube('lane-middle','M4.5 16 L21 16'),...tube('lane-top','M9 16 C12.5 16 12.5 7.5 16 7.5 L21 7.5'),...tube('lane-bottom','M9 16 C12.5 16 12.5 24.5 16 24.5 L21 24.5'),fill('fork-solid',circle(9,16,1.7)),...[7.5,16,24.5].flatMap((y,i)=>bothFill('socket-'+i,rect(23.5,y-2,4,4,1.2),{visibility:socketAlpha})),...[0,1,2].flatMap(i=>bothFill('packet-'+i,circle(0,0,1.6),{pose:rigid((t,s,r)=>{const [dx,dy]=forkRoutes[i](dispatch(t,s,r));return{dx,dy};}),visibility:packetAlpha}))
],.8,.5,'Ready','Dispatched',.28,.28);parallel.previewReset=true;parallel.reverseSuffix='-reset';parallel.duotoneFill=false;

const gatherPoint=(i,q)=>{if(q>=1)return[18+6.7*(q-1),16];if(i===1)return[9.4+8.6*q,16];const y=i===0?7.5:24.5;return q<.2?[9.4+2.1*q/.2,y]:cubic([11.5,y],[14.8,y],[14.7,16],[18,16],(q-.2)/.8);};
const gatherRoutes=[0,1,2].map(i=>along(p=>gatherPoint(i,p)));
const gather=(t,s,r)=>r?t<.12?2:0:curve(t,s,[[0,0],[.05,0],[.5,1],[.85,2,'out']],[[0,0],[.08,0],[.32,1,'pass'],[.5,2,'hit']]);
const gatherAlpha=(i,t,s,r)=>r?i===1?curve(t,false,[[0,1],[.08,0]],[]):0:packetAlpha(t,s,false)*(i===1?1:curve(t,s,[[0,1],[.5,1],[.58,0]],[[0,1],[.32,1],[.38,0]]));
const outputAlpha=(t,s,r)=>r?curve(t,false,[[0,1],[.1,1],[.24,.4]],[]):curve(t,s,[[0,.4],[.75,.4],[.95,1,'out']],[[0,.4],[.48,.4],[.6,1]]);
const merge=toggle('merge-results','Merge results','Three tool results gather at a junction and arrive as one result. Strong carries the main action through the merge, then lands quickly. The output stays while preview reset removes its packet.',[
 ...[7.5,16,24.5].flatMap((y,i)=>bothFill('mark-'+i,rect(3.5,y-1.75,3.5,3.5,1.1))),stroke('route-middle','M9.4 16 L21 16'),stroke('route-middle-solid','M9.4 16 L22 16',{style:'solid',width:2.6}),...tube('route-top','M9.4 7.5 L11.5 7.5 C14.8 7.5 14.7 16 18 16'),...tube('route-bottom','M9.4 24.5 L11.5 24.5 C14.8 24.5 14.7 16 18 16'),...bothFill('junction',circle(18,16,1.8)),fill('output-interior',rect(21,11,7.4,10,2),'tint'),stroke('output-frame',rect(21,11,7.4,10,2),{visibility:outputAlpha}),fill('output-solid',rect(21,11,7.4,10,2),'solid',{visibility:outputAlpha}),...[0,1,2].flatMap(i=>bothFill('packet-'+i,circle(0,0,1.4),{pose:rigid((t,s,r)=>{const q=gather(t,s,r),[dx,dy]=q<=1?gatherRoutes[i](q):gatherPoint(i,q);return{dx,dy};}),visibility:(t,s,r)=>gatherAlpha(i,t,s,r)}))
],.95,.65,'Ready','Merged',.28,.28);merge.previewReset=true;merge.reverseSuffix='-reset';merge.duotoneFill=true;

const frontX=(t,s,r)=>r?curve(t,false,[[0,-1.5],[.28,0]],[]):curve(t,s,[[0,0],[.6,-1.5]],[[0,0],[.1,.4],[.36,-2.2,'pass'],[.56,-1.5]]);
const frontY=(t,s,r)=>r?curve(t,false,[[0,1.75],[.28,0]],[]):curve(t,s,[[0,0],[.6,1.75]],[[0,0],[.1,-.3],[.36,2.3,'pass'],[.56,1.75]]);
const back=(t,s,r,solid)=>`M11 ${13+frontY(t,s,r)+(solid?1.6:0)} L11 5.5 C11 4.4 11.9 3.5 13 3.5 L25 3.5 C26.1 3.5 27 4.4 27 5.5 L27 15.5 C27 16.6 26.1 17.5 25 17.5 L${21.5+frontX(t,s,r)-(solid?1.6:0)} 17.5`;
const background=toggle('background-run','Background run','The front pane moves aside while a task progresses in the back pane. Strong has a prepared slide with one damped response and a faster progress stroke. The panes stay opaque through reset.',[
 stroke('back',(t,s,r)=>back(t,s,r,false)),stroke('back-solid',(t,s,r)=>back(t,s,r,true),{style:'solid',width:2.6}),...framed('front',rect(5.5,13,16,12.5,2),undefined,{pose:rigid((t,s,r)=>({dx:frontX(t,s,r),dy:frontY(t,s,r)}))}),...tube('track','M15.5 8.2 L22.5 8.2'),...bothFill('runner',circle(15.5,8.2,1.8),{pose:rigid((t,s,r)=>({dx:r?t<.12?7:0:curve(t,s,[[0,0],[.3,0],[1.1,7,'out']],[[0,0],[.3,0],[.66,7]])})),visibility:(t,s,r)=>r?curve(t,false,[[0,1],[.08,0],[.15,0],[.25,1]],[]):1})
],1.1,.8,'Foreground','Running behind',.28,.28);background.previewReset=true;background.reverseSuffix='-reset';background.duotoneFill=true;
export const runtime={'retry-step':retry,'parallel-tools':parallel,'merge-results':merge,'background-run':background};
