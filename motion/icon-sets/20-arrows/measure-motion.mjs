// Source primary travel and timing; native perceptual review is separate.
import fs from'node:fs';
import svgpath from'svgpath';
import{icons}from'./definitions.mjs';
const reports=icons.map(icon=>{
 let part,channel,range;
 if(icon.name.startsWith('chevron-')){part='vee';channel='morph apex';range=[0,.22,.1,.26];}
 else if(['arrows-horizontal','arrows-vertical'].includes(icon.name)){part='head-right';channel='head translation';range=[0,.28,.12,.32];}
 else if(icon.name==='arrows-swap'){part='top-head';channel='lane translation';range=[0,.26,.12,.32];}
 else if(icon.name.startsWith('arrow-turn-')){part='head';channel='outlet translation';range=[.18,.58,.16,.4];}
 else if(icon.name==='arrow-u-turn'){part='head';channel='outlet translation';range=[.18,.6,.18,.44];}
 else if(icon.name.startsWith('arrows-')){part='corner-0-head';channel='diagonal translation';range=[0,.22,.14,icon.name==='arrows-expand'?.4:.38];}
 else{part='head';channel='axial translation';range=[0,.18,.1,.28];}
 const p=icon.parts.find(p=>p.name===icon.name+'-'+part);
 const at=(t,s)=>{if(typeof p.d==='function'){const d=svgpath(p.d(t,s,false,2)).abs().segments;return d[1].slice(-2);}const tr=p.pose.transform(t,s,false);return[tr.dx||0,tr.dy||0];};
 const profiles=[false,true].map(strong=>{
  const a=range[strong?2:0],b=range[strong?3:1],A=at(a,strong),B=at(b,strong),delta=B.map((v,j)=>v-A[j]),mag=Math.hypot(...delta),den=mag*mag;
  const rows=Array.from({length:Math.round((b-a)*1000)+1},(_,i)=>{const t=Math.min(b,a+i/1000),v=at(t,strong),q=v.reduce((sum,x,j)=>sum+(x-A[j])*delta[j],0)/den;return{t,q};});
  const crossing=q=>rows.find(r=>r.q>=q)?.t,start=at(0,strong),onset=Array.from({length:1401},(_,i)=>i/1000).find(t=>{const v=at(t,strong);return Math.hypot(...v.map((x,j)=>x-start[j]))>.02;});
  return{profile:strong?'strong':'light',primaryDriveStart:a,primaryDriveEnd:b,delta:+mag.toFixed(3),deltaUnits:'grid units',tenNinetyMs:Math.round((crossing(.9)-crossing(.1))*1000),firstChangeAbovePoint02:onset};
 });return{id:icon.name,part,channel,profiles,reading:'Real-time primary drive; preparation, response and still rest are reviewed separately in native playback.'};
});
fs.writeFileSync(new URL('./evidence/motion-measurements.json',import.meta.url),JSON.stringify({kind:'source numerical motion evidence',icons:reports},null,2)+'\n');
console.table(reports.map(r=>({id:r.id,lightRange:r.profiles[0].delta,strongRange:r.profiles[1].delta,light10to90:r.profiles[0].tenNinetyMs,strong10to90:r.profiles[1].tenNinetyMs})));
