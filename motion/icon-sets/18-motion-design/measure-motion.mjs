// Source timing evidence, distinct from perceptual/native render acceptance.
import fs from 'node:fs';
import {icons} from './definitions.mjs';
import svgpath from 'svgpath';
const specs={
 timeline:['playhead-head','x',0,.5,0,.32],
 'add-keyframe':['key-frame','y',.3,.7,.2,.44],
 'remove-keyframe':['key-frame','y',.1,.5,.15,.38],
 'auto-keyframe':['needle','angle',.1,1,.12,.92],
 'keyframe-easing':['key-type-frame','shape',0,.55,.08,.4],
 'graph-editor':['curve','shape',0,.6,.08,.5],
 'bezier-handles':['handle-bar','angle',0,.45,.12,.5],
 'motion-path':['traveler','x',0,.8,.1,.7],
 'anchor-point':['cross-v','x',0,.45,.1,.4],
 'layer-stack':['layer-0-frame','x',0,.35,.1,.38],
 'parent-layers':['parent-frame','x',.08,.4,.1,.36],
 precompose:['top-frame','x',0,.5,.08,.32],
 'trim-layer':['handle','x',0,.55,.1,.36],
 'split-layer':['razor','y',.14,.3,.16,.26],
 'time-stretch':['bar-frame','shape',0,.6,.1,.4],
 'onion-skin':['ghost-0','x',.12,.54,.04,.36],
 'shape-morph':['hero-frame','shape',.04,.62,.06,.42],
 'mask-path':['node-2','x',.04,.34,.1,.3],
 'track-matte':['matte-boundary','x',.04,.44,.12,.4],
 'render-queue':['progress-2','shape',.5,1,.22,.62]
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
