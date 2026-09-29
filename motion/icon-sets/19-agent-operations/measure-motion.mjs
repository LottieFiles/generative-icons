// Source timing evidence, distinct from perceptual/native render acceptance.
import fs from 'node:fs';
import {icons} from './definitions.mjs';
import svgpath from 'svgpath';
const specs={
 'agent-skills':['cartridge-frame','y',.1,.52,.16,.36],
 'tool-discovery':['lens','x',.08,.62,.12,.4],
 'mcp-tools':['plug-frame','x',.1,.55,.14,.36],
 'browser-agent':['pointer','x',.08,.55,.12,.3],
 'terminal-agent':['command','shape',.1,.52,.12,.3],
 'agent-sandbox':['chip-frame','y',.05,.55,.14,.34],
 'agent-permissions':['keyhole','angle',.05,.75,.14,.42],
 'human-input':['text','shape',.55,1.15,.44,.66],
 'agent-checkpoint':['flag','y',.05,.6,.15,.36],
 'resume-run':['runner','x',.05,.8,.1,.42],
 'retry-step':['loop','angle',0,.75,.1,.42],
 'execution-trace':['span-2','shape',.55,.95,.3,.56],
 'parallel-tools':['packet-0','x',.05,.65,.08,.36],
 'merge-results':['packet-1','x',.05,.85,.08,.5],
 'background-run':['front-frame','x',0,.6,.1,.36],
 'scheduled-agent':['hand','angle',.1,.72,.06,.36],
 'event-trigger':['pulse','x',.08,.62,.06,.32],
 'context-compaction':['jaw-top','y',.1,.64,.12,.32],
 'agent-artifact':['page-frame','y',.1,.64,.1,.3],
 'budget-limit':['token','x',.1,.7,.06,.34]
};
const points=(p,t,s)=>svgpath(typeof p.d==='function'?p.d(t,s,false,2):p.d).abs().unarc().unshort().segments.flatMap(v=>v.slice(1));
const at=(p,axis,t,s)=>{if(axis==='shape')return points(p,t,s);const tr=p.pose.transform(t,s,false);return [tr[axis==='angle'?'angle':axis==='x'?'dx':'dy']||0];};
const records=icons.map(icon=>{
 const [name,axis,ls,le,ss,se]=specs[icon.name],p=icon.parts.find(p=>p.name===icon.name+'-'+name);
 if(!p)throw Error(icon.name+' missing part');
 const profiles=[false,true].map(strong=>{
  const a=strong?ss:ls,b=strong?se:le,A=at(p,axis,a,strong),B=at(p,axis,b,strong),delta=B.map((v,i)=>v-A[i]),mag=Math.hypot(...delta),den=mag*mag;
  const rows=Array.from({length:Math.round((b-a)*1000)+1},(_,i)=>{const t=Math.min(b,a+i/1000),v=at(p,axis,t,strong),q=v.reduce((sum,x,j)=>sum+(x-A[j])*delta[j],0)/den;return {t,q};});
  const crossing=q=>rows.find(r=>r.q>=q)?.t;
  const opacity10=p.visibility?Array.from({length:1401},(_,i)=>i/1000).find(t=>p.visibility(t,strong,false)>=.1):0;
  return{profile:strong?'strong':'light',primaryDriveStart:a,primaryDriveEnd:b,driveDuration:+(b-a).toFixed(3),delta:+mag.toFixed(3),deltaUnits:axis==='angle'?'degrees':axis==='shape'?'path-coordinate vector norm; compare this icon only':'grid units',tenNinetyMs:Math.round((crossing(.9)-crossing(.1))*1000),visibleAt10PercentOpacity:opacity10};
 });
 return{id:icon.name,part:name,channel:axis,profiles,reading:'Measures the directed main drive; preparation, holds and reset are excluded. Native equal-time visual review decides acceptance.'};
});
fs.writeFileSync(new URL('./evidence/motion-measurements.json',import.meta.url),JSON.stringify({kind:'source numerical motion evidence',icons:records},null,2)+'\n');
console.table(records.map(r=>({id:r.id,lightRange:r.profiles[0].delta,strongRange:r.profiles[1].delta,light10to90:r.profiles[0].tenNinetyMs,strong10to90:r.profiles[1].tenNinetyMs})));
