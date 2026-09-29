// Drawing-specific checks on exported world coordinates, independent of drivers.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url));
const set=JSON.parse(fs.readFileSync(path.resolve(root,'../../../prototype/icons/62/assets/set.json')));
const sample=(keys,t)=>{if(t<=keys[0][0])return keys[0][1];const i=keys.findIndex(k=>k[0]>=t);if(i<0)return keys.at(-1)[1];const [ta,a]=keys[i-1],[tb,b]=keys[i],u=(t-ta)/(tb-ta);return Array.isArray(a)?a.map((v,j)=>v+(b[j]-v)*u):a+(b-a)*u;};
const pick=(sc,id,p)=>{const n=sc.nodes.find(n=>n.name.split('__')[0]===id+'-'+p+'-bold')||sc.nodes.find(n=>n.name.split('__')[0]===id+'-'+p);assert(n,`${id}: missing ${p}`);return n;};
const points=(n,t)=>{const p=sample(n.points,t);let k=0,last,first;const out=[];for(const c of n.commands){if(c===0||c===1){last=p.slice(k,k+2);k+=2;out.push(last);if(c===0)first=last;}else if(c===2){const a=last,b=p.slice(k,k+2),d=p.slice(k+2,k+4),e=p.slice(k+4,k+6);k+=6;for(let i=1;i<=40;i++){const u=i/40,v=1-u;out.push([0,1].map(j=>v**3*a[j]+3*v*v*u*b[j]+3*v*u*u*d[j]+u**3*e[j]));}last=e;}else if(c===3){last=first;out.push(first);}else throw Error('Unknown command');}return out;};
const bounds=(n,t,paint=true)=>{const p=points(n,t),r=paint&&n.type==='stroke'?sample(n.width,t)/2:0;return [Math.min(...p.map(p=>p[0]))-r,Math.min(...p.map(p=>p[1]))-r,Math.max(...p.map(p=>p[0]))+r,Math.max(...p.map(p=>p[1]))+r];};
const distance=(a,b)=>Math.hypot(a[0]-b[0],a[1]-b[1]);
const report={result:'PASS',checks:0,minima:{},actions:[]};
const min=(key,value,limit)=>{report.checks++;report.minima[key]=Math.min(report.minima[key]??Infinity,value);assert(value>=limit-1e-4,`${key}: ${value} < ${limit}`);};
for(const icon of set.icons)for(const action of icon.animations){
 const sc=JSON.parse(fs.readFileSync(path.join(root,'scenes',action.name+'.scene.json'))),id=icon.icon;
 for(let f=0;f<=144;f++){
  const t=f/60;
  // Exact join: each node centre follows its actual curve endpoint in both directions.
  if(id==='join-paths')for(const side of ['left','right']){
   const line=points(pick(sc,id,side),t),b=bounds(pick(sc,id,side+'-node-frame'),t,false),center=[(b[0]+b[2])/2,(b[1]+b[3])/2];
   min('join endpoint contact',-distance(center,side==='left'?line.at(-1):line[0]),-.0001);
  }
  if(id==='offset-path'){
   const inner=bounds(pick(sc,id,'source-frame'),t),outer=bounds(pick(sc,id,'offset'),t,false),r=1.3;
   min('offset painted channel',Math.min(inner[0]-outer[0]-r,inner[1]-outer[1]-r,outer[2]-r-inner[2],outer[3]-r-inner[3]),2.5);
  }
  if(id==='auto-layout')for(let j=0;j<3;j++){
   const b=bounds(pick(sc,id,'child-'+j+'-frame'),t);
   min('layout child padding',Math.min(b[0]-5.3,b[1]-6.3,26.7-b[2],25.7-b[3]),1);
  }
  if(id==='component-instance'){
   const link=points(pick(sc,id,'link'),t),cell=bounds(pick(sc,id,'instance-frame'),t,false);
   min('instance connector contact',-distance(link.at(-1),[cell[0],(cell[1]+cell[3])/2]),-.0001);
  }
  if(id==='pin-constraints'){
   const b=bounds(pick(sc,id,'child-frame'),t,false),top=points(pick(sc,id,'pin-top'),t),left=points(pick(sc,id,'pin-left'),t);
   min('top pin contact',-Math.abs(top.at(-1)[1]-b[1]),-.0001);
   min('left pin contact',-Math.abs(left.at(-1)[0]-b[0]),-.0001);
  }
  if(id==='blend-steps'){
   const a=bounds(pick(sc,id,'circle-frame'),t),b=bounds(pick(sc,id,'intermediate-frame'),t),c=bounds(pick(sc,id,'square-frame'),t);
   min('blend left spacing',b[0]-a[2],1.2);min('blend right spacing',c[0]-b[2],1.2);
  }
  if(id==='export-slice'){
   const tile=bounds(pick(sc,id,'slice-frame'),t),shaft=bounds(pick(sc,id,'arrow'),t),head=bounds(pick(sc,id,'arrow-head'),t);
   min('export arrow clearance',tile[1]-Math.max(shaft[3],head[3]),1.0);
  }
  if(id==='gradient-stops'){
   const stops=[0,1,2].map(j=>bounds(pick(sc,id,'stop-'+j+'-frame'),t));
   min('gradient stop spacing',Math.min(stops[1][0]-stops[0][2],stops[2][0]-stops[1][2]),1.0);
  }
 }
 report.actions.push(action.name);
}
fs.writeFileSync(path.join(root,'evidence/geometry-verification.json'),JSON.stringify(report,null,2)+'\n');
console.log(`PASS set 62 geometry: ${report.checks} drawing-specific contact and clearance checks across ${report.actions.length} actions.`);
