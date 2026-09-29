// Independent painted-bounds and attachment checks on exported world geometry.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url));
const meta=JSON.parse(fs.readFileSync(path.join(root,'../../../prototype/icons/19/assets/set.json')));
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
  // Names describe independent paint measurements. Intentional mechanical joints
  // are checked by their attachment, not misclassified as detached components.
  if(icon.name==='agent-skills'){
   gap('skill upper-to-dock','cartridge-frame','dock-frame',1);
   metric('skill tab clears dock counter',B('dock-frame')[1]+2.5-B('tab')[3],0);
  }
  if(icon.name==='tool-discovery')for(let i=0;i<3;i++)gap('lens-to-candidate-'+i,'lens','candidate-'+i,1);
  if(icon.name==='mcp-tools'){
   gap('plug prong pitch','prong-0','prong-1',1);
   for(let i=0;i<2;i++)metric('prong stops outside housing counter',B('housing-frame')[0]+2.5-B('prong-'+i)[2],0);
  }
  if(icon.name==='browser-agent'){
   gap('browser pointer-to-result','pointer','result',1);
   gap('browser pointer-to-window','pointer','window',1);
   gap('browser pointer-to-tabbar','pointer','tabbar',1);
  }
  if(icon.name==='terminal-agent'){
   gap('command-to-cursor','command','cursor',1);gap('command-to-output','command','output',1);
  }
  if(icon.name==='agent-sandbox'){
   gap('process-to-box','chip-frame','box',1);for(let i=0;i<2;i++)gap('process-to-door-'+i,'chip-frame','door-'+i,1);
   for(let i=0;i<2;i++)metric('hinge remains fixed',-Math.hypot(pts('door-'+i)[0][0]-[7,25][i],pts('door-'+i)[0][1]-14),-.001);
  }
  if(icon.name==='agent-permissions')gap('permission key-to-rim','keyhole','face',1);
  if(icon.name==='human-input'){
   gap('person head-to-shoulder','head','shoulders',1);gap('person-to-field','shoulders','field',1);gap('input-text-to-field','text','field',1);gap('caret-to-field','caret','field',1);gap('caret-to-text','caret','text',1);
  }
  if(icon.name==='agent-checkpoint'){
   // Pole planting is intentional contact. The cloth remains attached to x16
   // while the pole's bottom approaches the existing node.
   const p=pts('flag');metric('flag hinge attachment',-Math.abs(p[0][0]-p[1][0]),-.001);
  }
  if(icon.name==='resume-run')for(const i of[0,1])gap('remaining route spacing-'+i,'remaining-'+i,'remaining-'+(i+1),1);
  if(icon.name==='retry-step'){
   gap('retry loop-to-nodes','loop','node-1',1);gap('retry head-to-nodes','head','node-1',1);
   gap('retry middle-node-left-gap','track-a','node-1',1);gap('retry middle-node-right-gap','track-b','node-1',1);
   const loop=pts('loop'),head=pts('head');metric('retry inner barb-to-arc',distance(head[0],loop.slice(0,-1))-2.5,1);
  }
  if(icon.name==='execution-trace'){
   for(let i=0;i<3;i++)gap('trace-axis-span-'+i,'axis','span-'+i,1);
   gap('trace-span-pitch-a','span-0','span-1',1);gap('trace-span-pitch-b','span-1','span-2',1);
  }
  if(icon.name==='parallel-tools'){
   for(let i=0;i<2;i++)gap('parallel packet spacing-'+i,'packet-'+i,'packet-'+(i+1),1);
   for(const [i,l]of['lane-top','lane-middle','lane-bottom'].entries())gap('parallel port approach-'+i,l,'socket-'+i,1);
   if(action.name.includes('-reset'))for(let i=0;i<3;i++)if(vis('packet-'+i)>.01)metric('parallel reset socket opaque',vis('socket-'+i),1);
  }
  if(icon.name==='merge-results'){
   for(const[i,l]of['route-top','route-middle','route-bottom'].entries())gap('merge input gap-'+i,'mark-'+i,l,1);
   if(vis('packet-1')>.01&&Math.abs((B('packet-1')[0]+B('packet-1')[2])/2-24.7)<.001)gap('merged result counter','packet-1','output-frame',1);
   if(action.name.includes('-reset')&&vis('packet-1')>.01)metric('merge reset output opaque',vis('output-frame'),1);
  }
  if(icon.name==='background-run'){
   gap('background runner-to-frame','runner','back',1);gap('background runner-to-front','runner','front-frame',1);
   const p=pts('back'),b=B('front-frame');metric('back left cap on front edge',-Math.abs(p[0][1]-(b[1]+1.25)),-.001);metric('back right cap on front edge',-Math.abs(p.at(-1)[0]-(b[2]-1.25)),-.001);
  }
  if(icon.name==='scheduled-agent'){
   gap('scheduled hand rim','hand','dial',1);gap('scheduled release rim','release','dial',1);
  }
  if(icon.name==='event-trigger'){gap('trigger ray listener','output','listener',1);gap('trigger ray pulse','output','pulse',1);}
  if(icon.name==='context-compaction'){
   gap('compact line pitch-a','line-top','line-middle',1);gap('compact line pitch-b','line-middle','line-bottom',1);
   for(const j of['jaw-top','jaw-bottom'])for(const l of['line-top','line-middle','line-bottom'])gap('compact '+j+' '+l,j,l,1);
  }
  if(icon.name==='agent-artifact'){
   gap('artifact page-to-tray','page-frame','tray',1);for(let i=0;i<2;i++)gap('artifact text-to-page-'+i,'page-text-'+i,'page-frame',1);gap('artifact text pitch','page-text-0','page-text-1',1);
  }
  if(icon.name==='budget-limit')metric('usage stops before wall',24-1.3-B('token')[2],1);
  // Independent Solid silhouettes: do not infer their clearances from Bold.
  const S=n=>node(n.endsWith('-frame')?n.replace(/-frame$/,'-solid'):n+'-solid');
  const gs=(name,a,b,min=1)=>{const A=S(a),C=S(b);if(sample(A.visibility,t)<.05||sample(C.visibility,t)<.05)return;const pa=curvePoints(A,t),pc=curvePoints(C,t),w=(A.type==='stroke'?sample(A.width,t)/2:0)+(C.type==='stroke'?sample(C.width,t)/2:0);metric('Solid '+name,Math.min(...pa.map(p=>distance(p,pc)),...pc.map(p=>distance(p,pa)))-w,min);};
  if(icon.name==='mcp-tools')gs('prong pitch','prong-0','prong-1');
  if(icon.name==='terminal-agent')gs('cursor-command','cursor','command');
  if(icon.name==='agent-sandbox'){gs('process-box','chip-frame','box');for(let i=0;i<2;i++)gs('process-door-'+i,'chip-frame','door-'+i);}
  if(icon.name==='human-input'){gs('head-shoulders','head','shoulders');gs('person-field','shoulders','field');gs('caret-field','caret','field');gs('text-field','text','field');gs('caret-text','caret','text');}
  if(icon.name==='retry-step'){gs('loop-nodes','loop','node-1');gs('head-nodes','head','node-1');gs('track-left','track-a','node-1');gs('track-right','track-b','node-1');}
  if(icon.name==='execution-trace')for(let i=0;i<3;i++)gs('span-axis-'+i,'span-'+i,'axis');
  if(icon.name==='parallel-tools'){for(let i=0;i<2;i++)gs('packet-pitch-'+i,'packet-'+i,'packet-'+(i+1));for(const[i,n]of['lane-top','lane-middle','lane-bottom'].entries())gs('port-gap-'+i,n,'socket-'+i);}
  if(icon.name==='merge-results')for(const[i,n]of['route-top','route-middle','route-bottom'].entries())gs('input-gap-'+i,'mark-'+i,n);
  if(icon.name==='background-run'){gs('runner-back','runner','back');gs('runner-front','runner','front-frame');}
  if(icon.name==='event-trigger'){gs('ray-pulse','output','pulse');gs('ray-listener','output','listener');}
  if(icon.name==='context-compaction'){gs('text-pitch-a','line-top','line-middle');gs('text-pitch-b','line-middle','line-bottom');for(const j of['jaw-top','jaw-bottom'])for(const l of['line-top','line-middle','line-bottom'])gs(j+'-'+l,j,l);}


 }
 report.actions.push({name:action.name,frames:145});
}
report.result=report.errors.length?'FAIL':'PASS';
fs.writeFileSync(path.join(root,'evidence/clearance-verification.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({...report,actions:report.actions.length,errors:report.errors.slice(0,8)}));if(report.result==='FAIL')process.exitCode=1;
