// Independent painted-bounds and attachment checks on exported world geometry.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url));
const meta=JSON.parse(fs.readFileSync(path.join(root,'../../../prototype/icons/21/assets/set.json')));
const sample=(keys,t)=>{if(t<=keys[0][0])return keys[0][1];if(t>=keys.at(-1)[0])return keys.at(-1)[1];const i=keys.findIndex(k=>k[0]>t),a=keys[i-1],b=keys[i],u=(t-a[0])/(b[0]-a[0]);return Array.isArray(a[1])?a[1].map((v,j)=>v+(b[1][j]-v)*u):a[1]+(b[1]-a[1])*u;};
function curvePoints(node,t){const p=sample(node.points,t),out=[];let k=0,last,first;for(const cmd of node.commands){if(cmd===0||cmd===1){last=p.slice(k,k+2);k+=2;out.push(last);if(cmd===0)first=last;}else if(cmd===2){const a=last,b=p.slice(k,k+2),c=p.slice(k+2,k+4),d=p.slice(k+4,k+6);k+=6;for(let j=1;j<=20;j++){const u=j/20,v=1-u;out.push([0,1].map(i=>v*v*v*a[i]+3*v*v*u*b[i]+3*v*u*u*c[i]+u*u*u*d[i]));}last=d;}else if(cmd===3){out.push(first);last=first;}else throw Error('Unknown path command');}return out;}
const box=(node,t)=>{const p=curvePoints(node,t),r=node.type==='stroke'?sample(node.width,t)/2:0;return [Math.min(...p.map(p=>p[0]))-r,Math.min(...p.map(p=>p[1]))-r,Math.max(...p.map(p=>p[0]))+r,Math.max(...p.map(p=>p[1]))+r];};
const pick=(sc,name)=>{const n=sc.nodes.find(n=>n.name.split('__')[0]===name+'-bold')||sc.nodes.find(n=>n.name.split('__')[0]===name);assert.ok(n,name);return n;};
const lineDistance=(p,a,b)=>{const dx=b[0]-a[0],dy=b[1]-a[1],u=Math.max(0,Math.min(1,((p[0]-a[0])*dx+(p[1]-a[1])*dy)/(dx*dx+dy*dy||1)));return Math.hypot(p[0]-a[0]-u*dx,p[1]-a[1]-u*dy);};
const distance=(p,line)=>Math.min(...line.slice(1).map((b,i)=>lineDistance(p,line[i],b)));
const ends=(n,t)=>{const p=curvePoints(n,t);return [p[0],p.at(-1)];};
const inside=(p,poly)=>{let c=false;for(let i=0,j=poly.length-1;i<poly.length;j=i++){let a=poly[i],b=poly[j];if((a[1]>p[1])!==(b[1]>p[1])&&p[0]<(b[0]-a[0])*(p[1]-a[1])/(b[1]-a[1])+a[0])c=!c;}return c;};
function spans(node,t){
 const p=sample(node.points,t),out=[];let k=0,last;
 for(const cmd of node.commands){
  if(cmd===0){last=p.slice(k,k+2);k+=2;}
  else if(cmd===1){const end=p.slice(k,k+2);out.push([last,end]);last=end;k+=2;}
  else if(cmd===2){const a=last,b=p.slice(k,k+2),c=p.slice(k+2,k+4),d=p.slice(k+4,k+6);k+=6;const q=Array.from({length:65},(_,j)=>{const u=j/64,v=1-u;return[0,1].map(i=>v*v*v*a[i]+3*v*v*u*b[i]+3*v*u*u*c[i]+u*u*u*d[i]);});out.push(q);last=d;}
 }
 return out;
}
const report={result:'PASS',actions:[],minimumPaintedMargin:32,gaps:{},checks:0,errors:[],exceptions:['Opaque source-route and route-head tip contacts','Branch junction contacts within 4 grid units','Curled pointer has one intentional stem crossover; its counter remains protected']};
for(const icon of meta.icons)for(const action of icon.animations){
 const sc=JSON.parse(fs.readFileSync(path.join(root,'scenes',action.name+'.scene.json')));
 for(let f=0;f<=144;f++){
  const t=f/60;
  for(const n of sc.nodes.filter(n=>n.name.endsWith('__outline.bold')||n.name.endsWith('__solid'))){
   if(sample(n.visibility,t)<.01)continue;
   const b=box(n,t),margin=Math.min(b[0],b[1],32-b[2],32-b[3]);report.minimumPaintedMargin=Math.min(report.minimumPaintedMargin,margin);report.checks++;
   if(margin<1.99)report.errors.push(`${action.name} ${n.name} f${f}: margin ${margin}`);
  }
  for(const solid of [false,true]){
   const node=name=>pick(sc,icon.name+'-'+name+(solid?'-solid':'')),vis=n=>sample(n.visibility,t)>.01;
   const metric=(label,value,minimum=1)=>{const key=icon.name+' '+(solid?'Solid ':'Bold ')+label;report.gaps[key]=Math.min(report.gaps[key]??Infinity,value);report.checks++;if(value<minimum-.015)report.errors.push(`${action.name} ${key} f${f}: ${value}`);};
   const headGap=(head,body,skipStart=0,skipEnd=1)=>{
    const h=node(head),b=node(body);if(!vis(h)||!vis(b))return;
    const portions=spans(b,t).slice(skipStart,skipEnd?-skipEnd:undefined).filter(q=>Math.hypot(q[0][0]-q.at(-1)[0],q[0][1]-q.at(-1)[1])>.02);
    if(!portions.length)return;
    const hp=curvePoints(h,t),bp=portions.flat();const d=Math.min(...hp.map(p=>distance(p,bp)),...bp.map(p=>distance(p,hp)))-sample(h.width,t)/2-sample(b.width,t)/2;
    metric(head+' clears '+body+' curved section',d);
   };
   const attach=(head,body,tip)=>{
    const h=node(head),b=node(body);if(!vis(h))return;
    const end=ends(b,t)[1];metric(head+' waits for full '+body,-Math.hypot(end[0]-tip[0],end[1]-tip[1]),-.001);
    metric(head+' has visible '+body,vis(b)?1:0,1);
   };
   const branchWait=(child,parent,junction)=>{
    const ch=node(child),pa=node(parent);if(!vis(ch))return;
    const e=ends(pa,t)[1];metric(child+' waits for '+parent,-Math.hypot(e[0]-junction[0],e[1]-junction[1]),-.001);
    metric(child+' parent visible',vis(pa)?1:0,1);
   };
   const id=icon.name;
   const tips={'annotation-curve-right':[26,9],'annotation-curve-left':[6,9],'annotation-s-curve':[26,9],'annotation-swoop-up':[25.5,8],'annotation-swoop-down':[25.5,24],'annotation-loop':[26.95,17.13],'annotation-elbow':[25,8],'annotation-callout':[25.5,25.5]};
   if(tips[id]){headGap('head','route');attach('head','route',tips[id]);}
   if(id==='annotation-loop'){
    const n=node('route');if(vis(n)){
     const qs=spans(n,t).slice(1);const center=[13,14],d=Math.min(...qs.flat().map(p=>Math.hypot(p[0]-center[0],p[1]-center[1])))-sample(n.width,t)/2;
     metric('loop counter radius',d,3.7);
     const anchor=[17.65,23.94],d2=Math.min(...qs.flat().map(p=>Math.hypot(p[0]-anchor[0],p[1]-anchor[1])))-2-sample(n.width,t)/2;
     // Ignore collapsed future segments while the initial stem is still writing.
     if(Math.hypot(qs[0][0][0]-qs[0].at(-1)[0],qs[0][0][1]-qs[0].at(-1)[1])>.02)metric('return stroke clears anchor',d2);
    }
   }
   if(id==='annotation-bracket')for(const c of ['upper','lower'])branchWait(c,'leader',[11,16]);
   if(id==='annotation-split')for(const side of ['up','down']){branchWait('branch-'+side,'trunk',[12,16]);attach('head-'+side,'branch-'+side,[26,side==='up'?9.5:22.5]);headGap('head-'+side,'branch-'+side);headGap('head-'+side,'branch-'+(side==='up'?'down':'up'),0,0);}
   if(id==='annotation-merge'){
    for(const side of ['up','down']){branchWait('trunk','branch-'+side,[20,16]);headGap('head','branch-'+side,0,0);}
    attach('head','trunk',[26,16]);
   }
   if(id==='annotation-double-ended'){
    headGap('head-source','body',1,1);headGap('head-target','body',1,1);attach('head-target','body',[25.5,22]);
   }
  }
 }
 report.actions.push({name:action.name,frames:145});
}
report.result=report.errors.length?'FAIL':'PASS';fs.writeFileSync(path.join(root,'evidence/clearance-verification.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({...report,gaps:Object.fromEntries(Object.entries(report.gaps).filter(([k])=>k.includes('clears')||k.includes('radius'))),actions:report.actions.length,errors:report.errors.slice(0,15)}));if(report.result==='FAIL')process.exitCode=1;
