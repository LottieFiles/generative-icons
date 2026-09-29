// Independent painted-bounds and attachment checks on exported world geometry.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url));
const meta=JSON.parse(fs.readFileSync(path.join(root,'../../../prototype/icons/14/assets/set.json')));
const sample=(keys,t)=>{if(t<=keys[0][0])return keys[0][1];if(t>=keys.at(-1)[0])return keys.at(-1)[1];const i=keys.findIndex(k=>k[0]>t),a=keys[i-1],b=keys[i],u=(t-a[0])/(b[0]-a[0]);return Array.isArray(a[1])?a[1].map((v,j)=>v+(b[1][j]-v)*u):a[1]+(b[1]-a[1])*u;};
function curvePoints(node,t){const p=sample(node.points,t),out=[];let k=0,last,first;for(const cmd of node.commands){if(cmd===0||cmd===1){last=p.slice(k,k+2);k+=2;out.push(last);if(cmd===0)first=last;}else if(cmd===2){const a=last,b=p.slice(k,k+2),c=p.slice(k+2,k+4),d=p.slice(k+4,k+6);k+=6;for(let j=1;j<=20;j++){const u=j/20,v=1-u;out.push([0,1].map(i=>v*v*v*a[i]+3*v*v*u*b[i]+3*v*u*u*c[i]+u*u*u*d[i]));}last=d;}else if(cmd===3){out.push(first);last=first;}else throw Error('Unknown path command');}return out;}
const box=(node,t)=>{const p=curvePoints(node,t),r=node.type==='stroke'?sample(node.width,t)/2:0;return [Math.min(...p.map(p=>p[0]))-r,Math.min(...p.map(p=>p[1]))-r,Math.max(...p.map(p=>p[0]))+r,Math.max(...p.map(p=>p[1]))+r];};
const pick=(sc,name)=>{const n=sc.nodes.find(n=>n.name.split('__')[0]===name+'-bold')||sc.nodes.find(n=>n.name.split('__')[0]===name);assert.ok(n,name);return n;};
const lineDistance=(p,a,b)=>{const dx=b[0]-a[0],dy=b[1]-a[1],u=Math.max(0,Math.min(1,((p[0]-a[0])*dx+(p[1]-a[1])*dy)/(dx*dx+dy*dy||1)));return Math.hypot(p[0]-a[0]-u*dx,p[1]-a[1]-u*dy);};
const distance=(p,line)=>Math.min(...line.slice(1).map((b,i)=>lineDistance(p,line[i],b)));
const ends=(n,t)=>{const p=curvePoints(n,t);return [p[0],p.at(-1)];};
const inside=(p,poly)=>{let c=false;for(let i=0,j=poly.length-1;i<poly.length;j=i++){let a=poly[i],b=poly[j];if((a[1]>p[1])!==(b[1]>p[1])&&p[0]<(b[0]-a[0])*(p[1]-a[1])/(b[1]-a[1])+a[0])c=!c;}return c;};
const report={result:'PASS',actions:[],minimumPaintedMargin:32,attachmentChecks:0,negativeSpaceChecks:0};
for(const icon of meta.icons.filter(i=>!process.argv[2]||process.argv[2].split(',').includes(i.name)))for(const action of icon.animations){
 const sc=JSON.parse(fs.readFileSync(path.join(root,'scenes',action.name+'.scene.json')));let checked=0;
 for(let f=0;f<=144;f++){
  const t=f/60;
  for(const n of sc.nodes.filter(n=>n.name.endsWith('__outline.bold')||n.name.endsWith('__solid'))){
   if(sample(n.visibility,t)<.01)continue;const b=box(n,t),margin=Math.min(b[0],b[1],32-b[2],32-b[3]);report.minimumPaintedMargin=Math.min(report.minimumPaintedMargin,margin);
   assert.ok(margin>=3-.025,`${action.name} ${n.name} at frame ${f}: painted margin ${margin}`);checked++;
  }
  const node=name=>pick(sc,icon.name+'-'+name),points=name=>curvePoints(node(name),t);
  const attach=(part,housing,tolerance=.04)=>{for(const p of ends(node(part),t)){assert.ok(distance(p,points(housing))<=tolerance,`${action.name} ${part} attachment at frame ${f}`);report.attachmentChecks++;}};
  if(icon.name==='suitcase')attach('handle','case');
  if(icon.name==='backpack')attach('loop','body');
  if(icon.name==='bus')for(const n of ['tire-left','tire-right']){const p=ends(node(n),t);assert.ok(distance(p[0],points('body'))<.03,`${action.name} suspension body attachment`);assert.ok(Math.abs(p[1][1]-27.5)<.0001,`${action.name} fixed tire contact`);report.attachmentChecks+=2;}
  if(icon.name==='bed')for(const n of ['pillow','blanket'])attach(n,'mattress');
  if(icon.name==='tent'){for(const p of ends(node('door'),t)){assert.ok(Math.abs(p[1]-26)<.0001,`${action.name} tent hem detached`);report.attachmentChecks++;}assert.ok(!inside([16,23],points('canvas-tint')),`${action.name} tint fills doorway`);report.negativeSpaceChecks++;}
  if(icon.name==='beach-umbrella'){assert.ok(distance(ends(node('pole'),t)[1],points('sand'))<.0001,`${action.name} umbrella foot moved`);report.attachmentChecks++;}
  if(icon.name==='route'&&sample(node('path').visibility,t)>.01){const b=box(node('traveller'),t),head=points('path').at(-1);assert.ok(Math.hypot((b[0]+b[2])/2-head[0],(b[1]+b[3])/2-head[1])<.001,`${action.name} traveller does not follow path head`);report.attachmentChecks++;}
  if(icon.name==='bicycle'){const p=ends(node('crank'),t);for(const c of [[8,22],[24,22]]){const gap=lineDistance(c,p[0],p[1])-3.2-2.6;assert.ok(gap>=.7,`${action.name} crank touches wheel: ${gap}`);report.negativeSpaceChecks++;}}
  if(icon.name==='signpost'){const a=ends(node('post-between'),t),b=ends(node('post-lower'),t);assert.ok(distance(a[0],points('upper-board'))<.03&&distance(a[1],points('lower-board'))<.03&&distance(b[0],points('lower-board'))<.03,`${action.name} exposed post attachment`);report.attachmentChecks+=3;const board=points('lower-board'),foot=points('foot');const gap=Math.min(...board.map(p=>distance(p,foot)))-2.6;assert.ok(gap>=.7,`${action.name} board crowds base: ${gap}`);report.negativeSpaceChecks++;}
 }
 report.actions.push({name:action.name,paintedBoundsChecked:checked});
}
fs.mkdirSync(path.join(root,'evidence'),{recursive:true});fs.writeFileSync(path.join(root,'evidence/clearance-verification.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({...report,actions:report.actions.length},null,2));
