// Independent painted-bounds and attachment checks on exported world geometry.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url));
const meta=JSON.parse(fs.readFileSync(path.join(root,'../../../prototype/icons/20/assets/set.json')));
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
   if(sample(n.visibility,t)<.01)continue;const b=box(n,t),margin=Math.min(b[0],b[1],32-b[2],32-b[3]);report.minimumPaintedMargin=Math.min(report.minimumPaintedMargin,margin);if(margin<1.99)report.errors.push(`${action.name} ${n.name} frame${f}: margin ${margin}`);report.checks++;
  }
  const node=name=>pick(sc,icon.name+'-'+name);
  const gap=(name,a,b,minimum)=>{
   const na=node(a),nb=node(b);if(sample(na.visibility,t)<.05||sample(nb.visibility,t)<.05)return;
   const pa=curvePoints(na,t),pb=curvePoints(nb,t),r=(na.type==='stroke'?sample(na.width,t)/2:0)+(nb.type==='stroke'?sample(nb.width,t)/2:0);
   const g=Math.min(...pa.map(p=>distance(p,pb)),...pb.map(p=>distance(p,pa)))-r;
   report.gaps[name]=Math.min(report.gaps[name]??Infinity,g);if(g<minimum-.015)report.errors.push(`${action.name} ${name} frame${f}: gap ${g}`);report.checks++;
  };
  const metric=(name,g,minimum=1)=>{report.gaps[name]=Math.min(report.gaps[name]??Infinity,g);if(g<minimum-.015)report.errors.push(`${action.name} ${name} frame${f}: gap ${g}`);report.checks++;};
  const B=name=>box(node(name),t),pts=name=>curvePoints(node(name),t),vis=name=>sample(node(name).visibility,t);
  // Head-to-shaft overlap is intentional; unrelated stems and barbs are not.
  const joint=(label,a,b)=>metric(label,-Math.hypot(a[0]-b[0],a[1]-b[1]),-.0001);
  if(icon.name.startsWith('arrow-')&&!['arrow-turn-right','arrow-turn-left','arrow-u-turn'].includes(icon.name)){
   joint('shaft joins tip',ends(node('shaft'),t)[1],sample(node('head').points,t).slice(2,4));
   const shaft=sample(node('shaft').points,t),u=[shaft[2]-shaft[0],shaft[3]-shaft[1]],length=Math.hypot(...u),axis=u.map(v=>v/length),center=[shaft[0]+axis[0]*12,shaft[1]+axis[1]*12],poly=pts('head-solid');
   for(let j=0;j<32;j++){const a=2*Math.PI*j/32,p=[center[0]+1.3*Math.cos(a),center[1]+1.3*Math.sin(a)];metric('Solid shaft cap buried',inside(p,poly)?1:0,1);}
  }
  if(['arrows-horizontal','arrows-vertical'].includes(icon.name)){
   for(const [j,side]of ['left','right'].entries())joint('stretch joins '+side,ends(node('shaft'),t)[j],sample(node('head-'+side).points,t).slice(2,4));
   gap('opposed head clearance','head-left','head-right',1);gap('Solid opposed head clearance','head-left-solid','head-right-solid',1);
  }
  if(icon.name==='arrows-swap'){
   for(const side of ['top','bottom'])joint('swap joins '+side,ends(node(side+'-shaft'),t)[1],sample(node(side+'-head').points,t).slice(2,4));
   for(const a of ['shaft','head'])for(const b of ['shaft','head']){gap('swap lane '+a+' '+b,'top-'+a,'bottom-'+b,1);gap('Solid swap lane '+a+' '+b,'top-'+a+'-solid','bottom-'+b+'-solid',1);}
  }
  if(['arrow-turn-right','arrow-turn-left','arrow-u-turn'].includes(icon.name)){
   joint('outlet joins turn tip',ends(node('outlet'),t)[1],sample(node('head').points,t).slice(2,4));
   joint('stem joins outlet',ends(node('stem-arc'),t)[1],ends(node('outlet'),t)[0]);
   gap('entire head clears curved stem','head','stem-arc',1);
   gap('Solid entire head clears curved stem','head-solid','stem-arc-solid',1);
   const center=ends(node('outlet-solid'),t)[1],poly=pts('head-solid');
   for(let j=0;j<32;j++){const a=2*Math.PI*j/32,p=[center[0]+1.3*Math.cos(a),center[1]+1.3*Math.sin(a)];metric('Solid outlet cap buried',inside(p,poly)?1:0,1);}
  }
  if(['arrows-expand','arrows-collapse'].includes(icon.name)){
   for(let i=0;i<4;i++)joint('corner shaft-head joint',ends(node('corner-'+i+'-shaft'),t)[0],sample(node('corner-'+i+'-head').points,t).slice(2,4));
   for(let i=0;i<4;i++)for(let j=i+1;j<4;j++)for(const a of ['head','shaft'])for(const b of ['head','shaft']){gap('corner '+i+' '+a+' to '+j+' '+b,'corner-'+i+'-'+a,'corner-'+j+'-'+b,1);gap('Solid corner '+i+' '+a+' to '+j+' '+b,'corner-'+i+'-'+a+'-solid','corner-'+j+'-'+b+'-solid',1);}
  }



 }
 report.actions.push({name:action.name,frames:145});
}
report.result=report.errors.length?'FAIL':'PASS';
fs.writeFileSync(path.join(root,'evidence/clearance-verification.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({...report,actions:report.actions.length,errors:report.errors.slice(0,8)}));if(report.result==='FAIL')process.exitCode=1;
