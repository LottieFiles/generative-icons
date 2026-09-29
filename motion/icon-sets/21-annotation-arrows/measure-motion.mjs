// Numerical source timing is separate from native, normal-speed visual review.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import {icons} from './definitions.mjs';
import {compile} from './batches/reveal.mjs';
const records=icons.map(icon=>{
 const parts=icon.parts.filter(p=>p.style==='outline'&&p.type==='stroke'&&!p.name.includes('head')&&!p.name.includes('frame'));
 const length=(t,strong)=>parts.reduce((sum,p)=>sum+compile(typeof p.d==='function'?p.d(t,strong,false,2):p.d).total,0);
 const profiles=[false,true].map(strong=>{
  const full=length(2.4,strong),rows=Array.from({length:361},(_,i)=>({t:i/240,length:length(i/240,strong)}));
  const at=q=>rows.find(r=>r.length>=full*q)?.t;
  return{profile:strong?'strong':'light',completedRouteLength:+full.toFixed(3),tenPercentSeconds:at(.1),ninetyPercentSeconds:at(.9),tenNinetyMs:Math.round((at(.9)-at(.1))*1000),activeSeconds:icon.timings[strong?'strong':'light'].active};
 });
 assert(profiles[1].tenNinetyMs<profiles[0].tenNinetyMs*.82,icon.name+' Strong draw must have a materially faster primary drive');
 return{id:icon.name,channel:'sum of source route arc lengths, before head finish',sampleRate:240,profiles,strongDriveRatio:+(profiles[1].tenNinetyMs/profiles[0].tenNinetyMs).toFixed(3)};
});
fs.writeFileSync(new URL('./evidence/motion-measurements.json',import.meta.url),JSON.stringify({kind:'source numerical motion evidence',icons:records},null,2)+'\n');
console.table(records.map(r=>({id:r.id,light10to90:r.profiles[0].tenNinetyMs,strong10to90:r.profiles[1].tenNinetyMs,strongRatio:r.strongDriveRatio})));
