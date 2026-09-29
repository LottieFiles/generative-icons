// Independent painted-bounds and attachment checks on exported world geometry.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url));
const meta=JSON.parse(fs.readFileSync(path.join(root,'../../../prototype/icons/15/assets/set.json')));
const sample=(keys,t)=>{if(t<=keys[0][0])return keys[0][1];if(t>=keys.at(-1)[0])return keys.at(-1)[1];const i=keys.findIndex(k=>k[0]>t),a=keys[i-1],b=keys[i],u=(t-a[0])/(b[0]-a[0]);return Array.isArray(a[1])?a[1].map((v,j)=>v+(b[1][j]-v)*u):a[1]+(b[1]-a[1])*u;};
function curvePoints(node,t){const p=sample(node.points,t),out=[];let k=0,last,first;for(const cmd of node.commands){if(cmd===0||cmd===1){last=p.slice(k,k+2);k+=2;out.push(last);if(cmd===0)first=last;}else if(cmd===2){const a=last,b=p.slice(k,k+2),c=p.slice(k+2,k+4),d=p.slice(k+4,k+6);k+=6;for(let j=1;j<=20;j++){const u=j/20,v=1-u;out.push([0,1].map(i=>v*v*v*a[i]+3*v*v*u*b[i]+3*v*u*u*c[i]+u*u*u*d[i]));}last=d;}else if(cmd===3){out.push(first);last=first;}else throw Error('Unknown path command');}return out;}
const box=(node,t)=>{const p=curvePoints(node,t),r=node.type==='stroke'?sample(node.width,t)/2:0;return [Math.min(...p.map(p=>p[0]))-r,Math.min(...p.map(p=>p[1]))-r,Math.max(...p.map(p=>p[0]))+r,Math.max(...p.map(p=>p[1]))+r];};
const pick=(sc,name)=>{const n=sc.nodes.find(n=>n.name.split('__')[0]===name+'-bold')||sc.nodes.find(n=>n.name.split('__')[0]===name);assert.ok(n,name);return n;};
const lineDistance=(p,a,b)=>{const dx=b[0]-a[0],dy=b[1]-a[1],u=Math.max(0,Math.min(1,((p[0]-a[0])*dx+(p[1]-a[1])*dy)/(dx*dx+dy*dy||1)));return Math.hypot(p[0]-a[0]-u*dx,p[1]-a[1]-u*dy);};
const distance=(p,line)=>Math.min(...line.slice(1).map((b,i)=>lineDistance(p,line[i],b)));
const ends=(n,t)=>{const p=curvePoints(n,t);return [p[0],p.at(-1)];};
const inside=(p,poly)=>{let c=false;for(let i=0,j=poly.length-1;i<poly.length;j=i++){let a=poly[i],b=poly[j];if((a[1]>p[1])!==(b[1]>p[1])&&p[0]<(b[0]-a[0])*(p[1]-a[1])/(b[1]-a[1])+a[0])c=!c;}return c;};
const report={result:'PASS',actions:[],minimumPaintedMargin:32,attachmentChecks:0,negativeSpaceChecks:0,paintContinuityChecks:0};
for(const icon of meta.icons.filter(i=>!process.argv[2]||process.argv[2].split(',').includes(i.name)))for(const action of icon.animations){
 const sc=JSON.parse(fs.readFileSync(path.join(root,'scenes',action.name+'.scene.json')));let checked=0;
 for(let f=0;f<=144;f++){
  const t=f/60;
  for(const n of sc.nodes.filter(n=>n.name.endsWith('__outline.bold')||n.name.endsWith('__solid'))){
   if(sample(n.visibility,t)<.01)continue;const b=box(n,t),margin=Math.min(b[0],b[1],32-b[2],32-b[3]);report.minimumPaintedMargin=Math.min(report.minimumPaintedMargin,margin);
   assert.ok(margin>=3-.025,`${action.name} ${n.name} at frame ${f}: painted margin ${margin}`);checked++;
  }

  const node=name=>pick(sc,icon.name+'-'+name),points=name=>curvePoints(node(name),t);
  const check=(ok,message,kind='attachmentChecks')=>{assert.ok(ok,`${action.name} frame ${f}: ${message}`);report[kind]++;};
  const attach=(part,housing,tolerance=.04)=>{for(const p of ends(node(part),t))check(distance(p,points(housing))<=tolerance,`${part} detached from ${housing}`);};
  const gap=(a,b,min)=>{const d=Math.min(...points(a).map(p=>distance(p,points(b))))-2.6;check(d>=min-.025,`${a}/${b} painted gap ${d}`,'negativeSpaceChecks');};
  const center=name=>{const b=box(node(name),t);return [(b[0]+b[2])/2,(b[1]+b[3])/2];};
  if(icon.name==='storefront'){gap('hem','facade',1);}
  if(icon.name==='shopping-bag')attach('handle','body');
  if(icon.name==='basket'){
   attach('handle','rim');attach('body','rim');
   for(const n of ['weave-left','weave-right']){const e=ends(node(n),t);check(distance(e[0],points('rim'))<.01&&distance(e[1],points('body'))<.01,`${n} detached`);}
   const h=points('handle'),y=Math.min(...h.map(p=>p[1]));check(14-y-2.6>=.7,'strap apex crowds rim','negativeSpaceChecks');
  }
  if(icon.name==='coupon')gap('body','stub',.9);
  if(icon.name==='discount'){gap('slash','dot-one',.7);gap('slash','dot-two',.7);}
  if(icon.name==='checkout'){
   for(const n of ['guide-left','guide-right']){const e=ends(node(n),t);check(distance(e[0],points('console'))<.01&&distance(e[1],points('drawer'))<.01,`${n} guide detached`);}
   gap('handle','drawer',1);
  }
  if(icon.name==='open-package')for(const n of ['flap-left','flap-right'])check(distance(ends(node(n),t)[0],points('box'))<.01,`${n} hinge detached`);
  if(icon.name==='packing-tape'){
   const c=center('rim'),e=ends(node('strip'),t);check(Math.hypot(e[1][0]-c[0],e[1][1]-c[1]-6)<.025,'tape does not meet roll tangent');check(Math.hypot(e[0][0]-5.5,e[0][1]-23)<.001,'free tape end moved');
   check(!inside(c,points('roll-tint')),'tape tint fills core','negativeSpaceChecks');
  }
  if(icon.name==='delivery-truck'){
   for(const [support,wheel,housing] of [['support-left','wheel-left','cargo'],['support-right','wheel-right','cab']]){const e=ends(node(support),t),c=center(wheel);check(distance(e[0],points(housing))<.01,`${support} body attachment`);check(Math.hypot(e[1][0]-c[0],e[1][1]-c[1]+2.5)<.001,`${support} wheel attachment`);check(c[1]-2.5+1.3-(box(node(housing),t)[3]+.05)>=1-.025,`${wheel} counter crowded`,'negativeSpaceChecks');}
  }
  if(icon.name==='order-tracking'){
   for(const n of ['stop-one','stop-two','stop-three'])gap('pin',n,1);
   for(const [rail,near,far] of [['track-one','stop-one','stop-two'],['track-two','stop-two','stop-three']])for(const [p,c] of [[ends(node(rail),t)[0],center(near)],[ends(node(rail),t)[1],center(far)]])check(Math.hypot(p[0]-c[0],p[1]-c[1])-1.3>=.7-.001,'rail invades stop counter','negativeSpaceChecks');
  }
  if(icon.name==='pickup-locker')gap('door','frame-frame',.85);
  if(icon.name==='warehouse'){
   attach('rail','jambs');const r=ends(node('rail'),t)[0][1];check(!inside([16,25],points('walls-tint'))&&!inside([16,r+1.4],points('curtain-tint')),'warehouse tint fills open aperture','negativeSpaceChecks');
  }
  if(icon.name==='inventory'){gap('box-left','box-right',.9);gap('moving-box','box-left',.9);gap('moving-box','box-right',.9);}
  if(icon.name==='return-package'){
   for(const n of ['arrow-body','arrow-head'])if(sample(node(n).visibility,t)>.01)gap(n,'parcel',2.5);
   if(sample(node('arrow-head').visibility,t)>.01){
    const p=sample(node('arrow-head').points,t),tip=p.slice(2,4),end=points('arrow-body').at(-1);check(Math.hypot(tip[0]-end[0],tip[1]-end[1])<.001,'return head detached from the drawn stem');
    const b=sample(node('arrow-body').points,t),stem=b.slice(-4,-2);check(Math.abs(stem[0]-end[0])<.001&&Math.abs(stem[1]-end[1])>=3.49,'return arrow reaches head without a straight stem');
   }
  }
  if(icon.name==='subscription-box')gap('hand','dial',.7);
  if(icon.name==='loyalty-points'){
   const p=points('cell-one'),r=(Math.max(...p.map(p=>p[0]))-Math.min(...p.map(p=>p[0])))/2,fillBox=box(node('earned-one'),t),fillRadius=(fillBox[2]-fillBox[0])/2;
   check(fillRadius-(r-.75)>=.099,'earned point meets rather than overlaps the thinnest ring paint','paintContinuityChecks');
  }

 }
 if(icon.name==='return-package'){
  const n=pick(sc,'return-package-parcel'),angles=n.points.map(([,p])=>Math.atan2(p[3]-p[1],p[2]-p[0])*180/Math.PI),angleRange=Math.max(...angles)-Math.min(...angles);
  if(action.direction==='forward')assert.ok(action.motionId==='strong'?angleRange>=9.9:angleRange<.001,action.name+' motion strength must include visible rigid rotation only in Strong');
  const b=pick(sc,'return-package-arrow-body'),a=pick(sc,'return-package-arrow-head'),end=sample(a.points,2.4),tail=sample(b.points,2.4),stem=tail.slice(-4,-2),tip=end.slice(2,4),barb=end.slice(4,6),clear=lineDistance(barb,stem,tip)-2.6;
  assert.ok(clear>=.45,action.name+' inner arrow barb crowds its straight stem');
  (report.returnPackageMotion??=[]).push({action:action.name,parcelRotationRangeDegrees:angleRange,headEndPaintedStemClearance:clear,bodyRedraw:Math.min(...b.visibility.map(k=>k[1]))===0});
 }
 report.actions.push({name:action.name,paintedBoundsChecked:checked});
}
fs.mkdirSync(path.join(root,'evidence'),{recursive:true});fs.writeFileSync(path.join(root,'evidence/clearance-verification.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({...report,actions:report.actions.length},null,2));
