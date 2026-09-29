// Independent painted-bounds and attachment checks on exported world geometry.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url));
const meta=JSON.parse(fs.readFileSync(path.join(root,'../../../prototype/icons/30/assets/set.json')));
const sample=(keys,t)=>{if(t<=keys[0][0])return keys[0][1];if(t>=keys.at(-1)[0])return keys.at(-1)[1];const i=keys.findIndex(k=>k[0]>t),a=keys[i-1],b=keys[i],u=(t-a[0])/(b[0]-a[0]);return Array.isArray(a[1])?a[1].map((v,j)=>v+(b[1][j]-v)*u):a[1]+(b[1]-a[1])*u;};
function curvePoints(node,t){const p=sample(node.points,t),out=[];let k=0,last,first;for(const cmd of node.commands){if(cmd===0||cmd===1){last=p.slice(k,k+2);k+=2;out.push(last);if(cmd===0)first=last;}else if(cmd===2){const a=last,b=p.slice(k,k+2),c=p.slice(k+2,k+4),d=p.slice(k+4,k+6);k+=6;if(a.every((v,j)=>v===b[j]&&v===c[j]&&v===d[j])){last=d;continue;}for(let j=1;j<=64;j++){const u=j/64,v=1-u;out.push([0,1].map(i=>v*v*v*a[i]+3*v*v*u*b[i]+3*v*u*u*c[i]+u*u*u*d[i]));}last=d;}else if(cmd===3){out.push(first);last=first;}else throw Error('Unknown path command');}return out;}
const box=(node,t)=>{const p=curvePoints(node,t),r=node.type==='stroke'?sample(node.width,t)/2:0;return [Math.min(...p.map(p=>p[0]))-r,Math.min(...p.map(p=>p[1]))-r,Math.max(...p.map(p=>p[0]))+r,Math.max(...p.map(p=>p[1]))+r];};
const pick=(sc,name)=>{const n=sc.nodes.find(n=>n.name.split('__')[0]===name+'-bold')||sc.nodes.find(n=>n.name.split('__')[0]===name);assert.ok(n,name);return n;};
const lineDistance=(p,a,b)=>{const dx=b[0]-a[0],dy=b[1]-a[1],u=Math.max(0,Math.min(1,((p[0]-a[0])*dx+(p[1]-a[1])*dy)/(dx*dx+dy*dy||1)));return Math.hypot(p[0]-a[0]-u*dx,p[1]-a[1]-u*dy);};
const distance=(p,line)=>Math.min(...line.slice(1).map((b,i)=>lineDistance(p,line[i],b)));
const ends=(n,t)=>{const p=curvePoints(n,t);return [p[0],p.at(-1)];};
const inside=(p,poly)=>{let c=false;for(let i=0,j=poly.length-1;i<poly.length;j=i++){let a=poly[i],b=poly[j];if((a[1]>p[1])!==(b[1]>p[1])&&p[0]<(b[0]-a[0])*(p[1]-a[1])/(b[1]-a[1])+a[0])c=!c;}return c;};
const segmentNode=(n,j)=>{const width=c=>c===2?6:c===0||c===1?2:0,offset=n.commands.slice(0,j).reduce((v,c)=>v+width(c),0);return {...n,commands:[0,n.commands[j]],points:n.points.map(([t,p])=>[t,p.slice(offset-2,offset+width(n.commands[j]))])};};
const contracts=JSON.parse(fs.readFileSync(path.join(root,'geometry-contract.json')));
const report={result:'PASS',actions:[],minimumPaintedMargin:32,gaps:{},checks:0,errors:[],exceptions:contracts.exceptions||{}};
for(const icon of meta.icons)for(const action of icon.animations){
 const sc=JSON.parse(fs.readFileSync(path.join(root,'scenes',action.name+'.scene.json')));
 const rules=contracts.icons[icon.name];assert(rules,icon.name+' missing supervisor geometry contract');
 for(let f=0;f<=144;f++){
  const t=f/60;
  for(const n of sc.nodes.filter(n=>n.name.endsWith('__outline.bold')||n.name.endsWith('__solid'))){
   if(sample(n.visibility,t)<.01)continue;
   const b=box(n,t),margin=Math.min(b[0],b[1],32-b[2],32-b[3]);report.minimumPaintedMargin=Math.min(report.minimumPaintedMargin,margin);report.checks++;
   if(margin<1.99)report.errors.push(`${action.name} ${n.name} frame ${f}: margin ${margin}`);
  }
  for(const rule of rules){
   if(rule.direction&&rule.direction!==action.direction||rule.profile&&rule.profile!==action.motionId)continue;
   const a=pick(sc,icon.name+'-'+rule.a);if(sample(a.visibility,t)<.01)continue;
   const metric=(value,min)=>{const key=icon.name+' '+rule.label;report.gaps[key]=Math.min(report.gaps[key]??Infinity,value);report.checks++;if(value<min-.015)report.errors.push(`${action.name} ${rule.label} frame ${f}: ${value} < ${min}`);};
   if(rule.kind==='gap'||rule.kind==='arrow-curve-gap'||rule.kind==='self-gap'){
    const b=pick(sc,icon.name+'-'+rule.b);if(sample(b.visibility,t)<.01)continue;
    const curved=rule.kind==='arrow-curve-gap'?{...b,commands:b.commands.slice(0,rule.curveCommands)}:b;
    const pa=curvePoints(rule.kind==='self-gap'?segmentNode(a,rule.ai):rule.aCommands?{...a,commands:a.commands.slice(0,rule.aCommands)}:a,t),pb=curvePoints(rule.kind==='self-gap'?segmentNode(b,rule.bi):rule.bCommands?{...curved,commands:curved.commands.slice(0,rule.bCommands)}:curved,t),radius=(a.type==='stroke'?sample(a.width,t)/2:0)+(b.type==='stroke'?sample(b.width,t)/2:0);
    metric(Math.min(...pa.map(p=>distance(p,pb)),...pb.map(p=>distance(p,pa)))-radius,rule.min??1);
   }else if(rule.kind==='opaque')metric(sample(a.visibility,t),1);
   else if(rule.kind==='inside'){
    const b=box(a,t),[left,top,right,bottom]=rule.box;metric(Math.min(b[0]-left,b[1]-top,right-b[2],bottom-b[3]),rule.min??0);
   }else if(rule.kind==='point-joint'){
    const b=pick(sc,icon.name+'-'+rule.b);if(sample(b.visibility,t)<.01)continue;
    const pa=sample(a.points,t).slice(rule.ai,rule.ai+2),pb=sample(b.points,t).slice(rule.bi,rule.bi+2);metric(-Math.hypot(pa[0]-pb[0],pa[1]-pb[1]),-.001);
   }else throw Error('Unknown geometry rule '+rule.kind);
  }
 }
 report.actions.push({name:action.name,frames:145});
}
report.result=report.errors.length?'FAIL':'PASS';fs.writeFileSync(path.join(root,'evidence/clearance-verification.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({...report,actions:report.actions.length,gaps:Object.keys(report.gaps).length,errors:report.errors.slice(0,20)}));if(report.result==='FAIL')process.exitCode=1;
