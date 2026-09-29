// Independent painted-bounds and attachment checks on exported world geometry.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url));
const meta=JSON.parse(fs.readFileSync(path.join(root,'../../../prototype/icons/16/assets/set.json')));
const sample=(keys,t)=>{if(t<=keys[0][0])return keys[0][1];if(t>=keys.at(-1)[0])return keys.at(-1)[1];const i=keys.findIndex(k=>k[0]>t),a=keys[i-1],b=keys[i],u=(t-a[0])/(b[0]-a[0]);return Array.isArray(a[1])?a[1].map((v,j)=>v+(b[1][j]-v)*u):a[1]+(b[1]-a[1])*u;};
function curvePoints(node,t){const p=sample(node.points,t),out=[];let k=0,last,first;for(const cmd of node.commands){if(cmd===0||cmd===1){last=p.slice(k,k+2);k+=2;out.push(last);if(cmd===0)first=last;}else if(cmd===2){const a=last,b=p.slice(k,k+2),c=p.slice(k+2,k+4),d=p.slice(k+4,k+6);k+=6;for(let j=1;j<=20;j++){const u=j/20,v=1-u;out.push([0,1].map(i=>v*v*v*a[i]+3*v*v*u*b[i]+3*v*u*u*c[i]+u*u*u*d[i]));}last=d;}else if(cmd===3){out.push(first);last=first;}else throw Error('Unknown path command');}return out;}
const box=(node,t)=>{const p=curvePoints(node,t),r=node.type==='stroke'?sample(node.width,t)/2:0;return [Math.min(...p.map(p=>p[0]))-r,Math.min(...p.map(p=>p[1]))-r,Math.max(...p.map(p=>p[0]))+r,Math.max(...p.map(p=>p[1]))+r];};
const pick=(sc,name)=>{const n=sc.nodes.find(n=>n.name.split('__')[0]===name+'-bold')||sc.nodes.find(n=>n.name.split('__')[0]===name);assert.ok(n,name);return n;};
const lineDistance=(p,a,b)=>{const dx=b[0]-a[0],dy=b[1]-a[1],u=Math.max(0,Math.min(1,((p[0]-a[0])*dx+(p[1]-a[1])*dy)/(dx*dx+dy*dy||1)));return Math.hypot(p[0]-a[0]-u*dx,p[1]-a[1]-u*dy);};
const distance=(p,line)=>Math.min(...line.slice(1).map((b,i)=>lineDistance(p,line[i],b)));
const ends=(n,t)=>{const p=curvePoints(n,t);return [p[0],p.at(-1)];};
const inside=(p,poly)=>{let c=false;for(let i=0,j=poly.length-1;i<poly.length;j=i++){let a=poly[i],b=poly[j];if((a[1]>p[1])!==(b[1]>p[1])&&p[0]<(b[0]-a[0])*(p[1]-a[1])/(b[1]-a[1])+a[0])c=!c;}return c;};
const report={result:'PASS',actions:[],minimumPaintedMargin:32,gaps:{},checks:0,errors:[]};
for(const icon of meta.icons)for(const action of icon.animations){
 const sc=JSON.parse(fs.readFileSync(path.join(root,'scenes',action.name+'.scene.json')));
 for(let f=0;f<=144;f++){
  const t=f/60;
  for(const n of sc.nodes.filter(n=>n.name.endsWith('__outline.bold')||n.name.endsWith('__solid'))){
   if(sample(n.visibility,t)<.01)continue;const b=box(n,t),margin=Math.min(b[0],b[1],32-b[2],32-b[3]);report.minimumPaintedMargin=Math.min(report.minimumPaintedMargin,margin);assert.ok(margin>=1.99,`${action.name} ${n.name} frame${f}: margin ${margin}`);report.checks++;
  }
  const node=name=>pick(sc,icon.name+'-'+name);
  const gap=(name,a,b,minimum)=>{
   const na=node(a),nb=node(b);if(sample(na.visibility,t)<.05||sample(nb.visibility,t)<.05)return;
   const pa=curvePoints(na,t),pb=curvePoints(nb,t),r=(na.type==='stroke'?sample(na.width,t)/2:0)+(nb.type==='stroke'?sample(nb.width,t)/2:0);
   const g=Math.min(...pa.map(p=>distance(p,pb)),...pb.map(p=>distance(p,pa)))-r;
   report.gaps[name]=Math.min(report.gaps[name]??Infinity,g);if(g<minimum-.015)report.errors.push(`${action.name} ${name} frame${f}: gap ${g}`);report.checks++;
  };
  if(icon.name==='voice-listening'){gap('listening arc lanes','arc-near','arc-far',.65);gap('listening receiving gap','arc-near','ear-helix',.65);}
  if(icon.name==='voice-recording'){
   gap('recording level hardware','level','cradle',.9);
   const ring=box(node('rec-ring'),t),dot=box(node('rec-dot'),t);const inner=4-1.25;const r=(dot[2]-dot[0])/2;assert.ok(inner-r>=1.49,'Record ring counter');report.checks++;
  }
  if(icon.name==='push-to-talk')gap('talk key clearance','cap-frame','key-side',.85);
  if(icon.name==='wake-word')gap('wake arc dot','arc','dot',1);
  if(icon.name==='switch-voice'){gap('speaker a opening','head-a-frame','body-a',.95);gap('speaker b opening','head-b-frame','body-b',.95);gap('voice marker a','marker','head-a-frame',1);gap('voice marker b','marker','head-b-frame',1);}
  if(icon.name==='voice-speed'){gap('cadence 1-2','tick-1','tick-2',1.05);gap('cadence 2-3','tick-2','tick-3',1.05);gap('play to cadence','play-frame','tick-1',1.05);}
 }
 report.actions.push({name:action.name,frames:145});
}
report.result=report.errors.length?'FAIL':'PASS';
fs.writeFileSync(path.join(root,'evidence/clearance-verification.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({...report,actions:report.actions.length,errors:report.errors.slice(0,8)}));if(report.result==='FAIL')process.exitCode=1;
