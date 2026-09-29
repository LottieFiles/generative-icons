// Independent painted-bounds and attachment checks on exported world geometry.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url));
const meta=JSON.parse(fs.readFileSync(path.join(root,'../../../prototype/icons/18/assets/set.json')));
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
  // Only semantically separate paints require a gap. Attached handles, endpoint
  // keys and intentional blade contact are tested as attachments or bounded phases.
  const pair=(label,a,b)=>gap(icon.name+':'+label,a,b,1);
  if(icon.name==='timeline')for(const clip of ['clip-a','clip-b'])for(const part of ['playhead-head','playhead-guide-a','playhead-guide-b']){pair(clip+part,clip,part);pair('solid'+clip+part,clip+'-solid',part+'-solid');}
  if(['add-keyframe','remove-keyframe'].includes(icon.name))for(const rail of ['rail-left','rail-right']){pair('key/'+rail,'key-frame',rail);pair('solid-key/'+rail,'key-solid',rail+'-solid');}
  if(icon.name==='auto-keyframe'){pair('needle/body','needle','body');pair('key/body','key-tip','body');}
  if(icon.name==='graph-editor')for(const a of ['curve','key-0','key-1']){pair('axes/'+a,'axes',a);pair('solid-axes/'+a,'axes-solid',a+'-solid');}
  if(icon.name==='bezier-handles')for(const a of ['handle-0','handle-1']){pair('curve/'+a,'curve',a);pair('solid-curve/'+a,'curve-solid',a+'-solid');}
  if(icon.name==='layer-stack')for(let j=0;j<2;j++){pair('rows'+j,'layer-'+j+'-frame','layer-'+(j+1)+'-frame');pair('solid-rows'+j,'layer-'+j+'-solid','layer-'+(j+1)+'-solid');}
  if(icon.name==='parent-layers'){pair('layer-gap','parent-frame','child-frame');pair('solid-layer-gap','parent-solid','child-solid');
   // The flexible connector must remain joined to the moving rims.
   const w=ends(node('whip'),t),pb=box(node('parent-frame'),t),cb=box(node('child-frame'),t);
   if(Math.abs(w[0][0]-(cb[0]+1.25))>.01||Math.abs(w[1][1]-(pb[3]-1.25))>.01)report.errors.push(action.name+' detached pick whip frame'+f);report.checks++;
  }
  if(icon.name==='precompose'){
   for(const a of ['top','middle','bottom'])pair('frame/'+a,'frame',a+'-frame');
   for(const [a,b]of [['top','middle'],['middle','bottom']]){pair(a+'/'+b,a+'-frame',b+'-frame');pair('solid'+a+'/'+b,a+'-solid',b+'-solid');}
  }
  if(icon.name==='trim-layer'){pair('handle/bar','handle','bar-frame');pair('solid-handle/bar','handle-solid','bar-solid');pair('handle/baseline','handle','baseline');pair('bar/baseline','bar-frame','baseline');}
  if(icon.name==='split-layer'){
   // The seam begins joined. Separation is intentional contact until the split
   // is fully open. Once open, the downstroke must fit between both cut faces.
   if(action.direction==='forward'&&t>=(action.motionId==='strong'?.30:.26))for(const a of ['left','right']){pair('blade/'+a,'razor',a+'-frame');pair('solid-blade/'+a,'razor-solid',a+'-solid');}
  }
  if(icon.name==='time-stretch'){pair('key-pitch','key-0','key-1');pair('bar/baseline','bar-frame','baseline');pair('solid-bar/baseline','bar-solid','baseline-solid');for(const a of ['key-0','key-1'])pair(a+'/bar',a,'bar-frame');}
  if(icon.name==='onion-skin')for(const a of ['ghost-0','ghost-1']){pair('current/'+a,'current-frame',a);pair('solid-current/'+a,'current-solid',a+'-solid');}
  if(icon.name==='shape-morph')for(const a of ['source','target']){pair('hero/'+a,'hero-frame',a+'-frame');pair('solid-hero/'+a,'hero-solid',a+'-solid');}
  if(icon.name==='mask-path'){
   pair('mask/frame','mask','frame');for(let j=0;j<4;j++){pair('node'+j+'/frame','node-'+j,'frame');const b=box(node('node-'+j),t),center=[(b[0]+b[2])/2,(b[1]+b[3])/2],d=distance(center,curvePoints(node('mask'),t));if(d>.015)report.errors.push(action.name+' node off curve frame'+f);report.checks++;}
  }
  if(icon.name==='track-matte')for(let j=0;j<5;j++){pair('result'+j+'/rim','result-'+j,'matte-boundary');pair('solid-result'+j+'/rim','result-'+j+'-solid','matte-boundary-solid');if(j<4){pair('result-pitch'+j,'result-'+j,'result-'+(j+1));pair('solid-result-pitch'+j,'result-'+j+'-solid','result-'+(j+1)+'-solid');}}
  if(icon.name==='render-queue')for(let j=0;j<3;j++){pair('dot/track'+j,'dot-'+j,'track-'+j);pair('solid-dot/track'+j,'dot-'+j+'-solid','track-'+j+'-solid');}

 }
 report.actions.push({name:action.name,frames:145});
}
report.result=report.errors.length?'FAIL':'PASS';
fs.writeFileSync(path.join(root,'evidence/clearance-verification.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({...report,actions:report.actions.length,errors:report.errors.slice(0,8)}));if(report.result==='FAIL')process.exitCode=1;
