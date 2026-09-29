// A source measurement of each named main part, in real seconds. These values
// support the native equal-time review; they do not establish visual quality.
import fs from 'node:fs';
import crypto from 'node:crypto';
import svgpath from 'svgpath';
import {icons} from './definitions.mjs';
const primary=["bar-v", "stroke-a", "head", "dot-1", "shaft", "node-top-frame", "blade-a", "sheet-frame", "bar-v", "marquee", "bar-h", "tr-frame", "divider", "divider", "active", "key", "bar", "lid-frame", "ring", "bar"];
const sha=f=>crypto.createHash('sha256').update(fs.readFileSync(f)).digest('hex');
function points(p,t,s){
 const path=svgpath(typeof p.d==='function'?p.d(t,s,false,2):p.d).abs().unarc().unshort();
 const out=[];let last=[0,0];
 for(const [command,...a] of path.segments){
  if(command==='Z')continue;
  if(command==='H'){a.unshift(last[0]);a[0]=a[1];a[1]=last[1];}
  if(command==='V')a.unshift(last[0]);
  for(let k=0;k<a.length;k+=2)out.push(...(p.pose?p.pose(a[k],a[k+1],t,s,false):[a[k],a[k+1]]));
  last=a.slice(-2);
 }
 return out;
}
const rms=(a,b)=>Math.sqrt(a.reduce((n,v,i)=>n+(v-b[i])**2,0)/(a.length/2));
const records=icons.map((icon,i)=>{
 const name=icon.name+'-'+primary[i],part=icon.parts.find(p=>p.name===name);if(!part)throw Error(name);
 const metrics={};
 for(const strong of [false,true]){
  const first=points(part,0,strong),frames=Array.from({length:145},(_,f)=>({time:f/60,p:points(part,f/60,strong),alpha:part.visibility?.(f/60,strong,false)??1}));
  if(frames.some(f=>f.p.length!==first.length))throw Error('Topology changed '+name);
  const distances=frames.map(f=>rms(first,f.p)),speeds=frames.slice(1).map((f,j)=>rms(frames[j].p,f.p)*60);
  const peak=Math.max(...distances),firstChange=frames.find((f,j)=>distances[j]>.01||Math.abs(f.alpha-frames[0].alpha)>.01)?.time;
  const first10=frames.find((f,j)=>distances[j]>=peak*.1)?.time,first90=frames.find((f,j)=>distances[j]>=peak*.9)?.time;
  const transforms=part.pose?.transform?frames.map(f=>part.pose.transform(f.time,strong,false)):null;
  const profile=strong?'strong':'light';metrics[profile]={activeSeconds:icon.timings[profile].active,firstChangeSeconds:firstChange??null,peakRmsDisplacement:peak,travelRms:speeds.reduce((a,b)=>a+b,0)/60,peakRmsSpeed:Math.max(...speeds),first10To90Seconds:first90-first10,...(transforms?{rotationRange:Math.max(...transforms.map(t=>t.angle||0))-Math.min(...transforms.map(t=>t.angle||0)),horizontalRange:Math.max(...transforms.map(t=>t.dx||0))-Math.min(...transforms.map(t=>t.dx||0)),verticalRange:Math.max(...transforms.map(t=>t.dy||0))-Math.min(...transforms.map(t=>t.dy||0))}:{})};
 }
 return {icon:icon.name,mainPart:name,intent:icon.intent,metrics,strongToLightPeakSpeed:metrics.strong.peakRmsSpeed/metrics.light.peakRmsSpeed,strongToLightTravel:metrics.strong.travelRms/metrics.light.travelRms};
});
const report={scope:'Source geometry and rigid transforms at 60fps; grid units and seconds. RMS includes curve control points. Native normal-speed review required.',definitionsSha256:sha(new URL('./definitions.mjs',import.meta.url)),icons:icons.length,records};
fs.writeFileSync(new URL('./evidence/motion-measurements.json',import.meta.url),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({icons:records.length,measured:true,reviewAtNativeSpeed:records.filter(r=>r.strongToLightPeakSpeed<1.5&&r.strongToLightTravel<1.5).map(r=>({icon:r.icon,speed:r.strongToLightPeakSpeed,travel:r.strongToLightTravel}))}));
