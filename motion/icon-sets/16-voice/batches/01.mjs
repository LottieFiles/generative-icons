import{continuous,make,tube,stroke,fill,framed,bothFill,circle,rect,hole,rigid,posed,TAU,cycle,mod,curve}from'./helpers.mjs';
import{geometry as G}from'./geometry.mjs';
import{activity}from'./activity.mjs';
const g=G['voice-listening'];
const earPose=rigid((t,s)=>({angle:s?-2+2*Math.cos(TAU*cycle(t)):0,pivot:[21,26]}));
const listening=continuous('voice-listening','Listening','Sound arrives at an ear in two separate lanes. Strong draws the signal inward with more travel and a small receiving lean.',[
 ...posed(tube('ear-helix',g['ear-helix']),earPose),...posed(tube('ear-fold',g['ear-fold']),earPose),
 ...['near','far'].flatMap((which,i)=>tube('arc-'+which,g['arc-'+which],{pose:rigid((t,s)=>({dx:(i?(s?3.2:1.5):(s?2:1))*mod(cycle(t)+i*.5)})),visibility:t=>{const u=mod(cycle(t)+i*.5);return u<.07||u>.93?0:Math.sin(Math.PI*(u-.07)/.86)**2;}}))
]);
// Four cubic wave spans with fixed endpoints. Sampling the same centerline
// builds a real transparent band in Solid; there is no background-colored mask.
function waveNodes(t,s){return [0,1,2,3,4].map(i=>[9+3.5*i,13.5+(i===0||i===4?0:(s?3.2:2.2)*Math.sin(TAU*(s?2:1)*cycle(t)-i*Math.PI/2))]);}
function wavePath(t,s){const p=waveNodes(t,s);return 'M'+p[0]+p.slice(1).map((v,i)=>' C'+[p[i][0]+1.2,p[i][1],v[0]-1.2,v[1],...v]).join('');}
function waveBand(t,s){
 const q=waveNodes(t,s),upper=[],lower=[];let firstN,lastN;
 for(let j=0;j<=32;j++){const span=Math.min(3,Math.floor(j/8)),u=(j-span*8)/8,v=1-u,a=q[span],b=[a[0]+1.2,a[1]],d=q[span+1],c=[d[0]-1.2,d[1]],p=[0,1].map(i=>v*v*v*a[i]+3*v*v*u*b[i]+3*v*u*u*c[i]+u*u*u*d[i]),dv=[0,1].map(i=>3*v*v*(b[i]-a[i])+6*v*u*(c[i]-b[i])+3*u*u*(d[i]-c[i])),len=Math.hypot(...dv),n=[-dv[1]/len,dv[0]/len];upper.push([p[0]+1.3*n[0],p[1]+1.3*n[1]]);lower.push([p[0]-1.3*n[0],p[1]-1.3*n[1]]);if(j===0)firstN=n;if(j===32)lastN=n;}
 const a=upper[0],b=upper.at(-1),c=lower.at(-1),d=lower[0];
 return 'M'+a+upper.slice(1).map(p=>' L'+p).join('')+' C'+[b[0]+1.734,b[1],c[0]+1.734,c[1],...c]+lower.slice(0,-1).reverse().map(p=>' L'+p).join('')+' C'+[d[0]-1.734,d[1],a[0]-1.734,a[1],...a]+' Z';
}
const bubble=G['voice-speaking'].bubble;
const speaking=continuous('voice-speaking','Speaking','A spoken waveform travels through a fixed speech bubble. Strong doubles its cadence and opens its wave height.',[
 fill('bubble-interior',bubble,'tint'),stroke('bubble-frame',bubble),stroke('wave',wavePath),fill('bubble-solid',(t,s)=>hole(bubble,waveBand(t,s)))
],true);
speaking.constraints=[{type:'fixed',part:'voice-speaking-bubble-frame'},{type:'inside',part:'voice-speaking-wave',box:[9,10.3,23,16.7]}];
const rec=G['voice-recording'];
const recording=continuous('voice-recording','Voice recording','A fixed microphone records a live level. Strong has a taller, faster level response; the record dot stays visible.',[
 ...framed('capsule',rec.capsule),...tube('cradle',rec.cradle),...tube('stem',rec.stem),...tube('foot',rec.foot),
 ...tube('level',(t,s)=>{const w=TAU*cycle(t),l=s?1.9+.9*Math.sin(2*w)+.2*Math.sin(6*w):1.2+.5*Math.sin(w)+.2*Math.sin(3*w);return `M20 ${9-l} L20 ${9+l}`;}),
 ...tube('rec-ring',circle(24.5,22,4)),...bothFill('rec-dot',circle(24.5,22,1.2),{visibility:(t,s)=>s?.45+.55*Math.cos(TAU*cycle(t))**2:.7+.3*Math.cos(TAU*cycle(t))})
],true);
recording.constraints=['capsule-frame','cradle','stem','foot','rec-ring'].map(p=>({type:'fixed',part:'voice-recording-'+p}));
const key=G['push-to-talk'];
const press=(t,s)=>curve(t,s,[[0,0],[.14,1,'out'],[.30,1],[.52,0]],[[0,0],[.06,-.4],[.16,2.5,'hit'],[.38,2.5,'hold'],[.52,-.4],[.72,0]]);
const keyPose=rigid((t,s)=>({dy:press(t,s)}));
const talk=make('push-to-talk','Push to talk','Press the talk key; a short signal leaves both sides. Strong has a deeper, quicker key press and one controlled release.',[
 ...posed([fill('cap-interior',key.cap,'tint'),stroke('cap-frame',key.cap),fill('cap-solid',hole(key.cap,rect(13.4,8.9,5.2,8.2,2.6))),fill('mic',key.mic,'outline')],keyPose),
 ...tube('key-side',key['key-side']),
 ...['left','right'].flatMap((side,i)=>tube('arc-'+side,key['arc-'+side],{pose:rigid((t,s)=>({dx:(i?1:-1)*curve(t,s,[[0,0],[.14,0],[.46,.3]],[[0,0],[.16,0],[.58,.5]])})),visibility:(t,s)=>curve(t,s,[[0,0],[.14,0],[.24,1],[.32,1],[.46,0]],[[0,0],[.16,0],[.24,1],[.42,1],[.58,0]])}))
],.52,.72);
talk.duotoneFill=true;talk.constraints=[{type:'fixed',part:'push-to-talk-key-side'}];
export const icons=[listening,speaking,activity,recording,talk];
