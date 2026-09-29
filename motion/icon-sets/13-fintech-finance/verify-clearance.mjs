// Independent painted-bounds and attachment checks on exported world geometry.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url));
const meta=JSON.parse(fs.readFileSync(path.join(root,'../../../prototype/icons/13/assets/set.json')));
const sample=(keys,t)=>{if(t<=keys[0][0])return keys[0][1];if(t>=keys.at(-1)[0])return keys.at(-1)[1];const i=keys.findIndex(k=>k[0]>t),a=keys[i-1],b=keys[i],u=(t-a[0])/(b[0]-a[0]);return Array.isArray(a[1])?a[1].map((v,j)=>v+(b[1][j]-v)*u):a[1]+(b[1]-a[1])*u;};
function curvePoints(node,t){const p=sample(node.points,t),out=[];let k=0,last,first;for(const cmd of node.commands){if(cmd===0||cmd===1){last=p.slice(k,k+2);k+=2;out.push(last);if(cmd===0)first=last;}else if(cmd===2){const a=last,b=p.slice(k,k+2),c=p.slice(k+2,k+4),d=p.slice(k+4,k+6);k+=6;for(let j=1;j<=20;j++){const u=j/20,v=1-u;out.push([0,1].map(i=>v*v*v*a[i]+3*v*v*u*b[i]+3*v*u*u*c[i]+u*u*u*d[i]));}last=d;}else if(cmd===3){out.push(first);last=first;}else throw Error('Unknown path command');}return out;}
const box=(node,t)=>{const p=curvePoints(node,t),r=node.type==='stroke'?sample(node.width,t)/2:0;return [Math.min(...p.map(p=>p[0]))-r,Math.min(...p.map(p=>p[1]))-r,Math.max(...p.map(p=>p[0]))+r,Math.max(...p.map(p=>p[1]))+r];};
const pick=(sc,name)=>{const n=sc.nodes.find(n=>n.name.split('__')[0]===name+'-bold')||sc.nodes.find(n=>n.name.split('__')[0]===name);assert.ok(n,name);return n;};
const report={result:'PASS',actions:[],minimumPaintedMargin:32,minimumContactlessGap:32,minimumTransferGap:32,minimumDoorOpening:32,maximumHandleAttachmentError:0};
for(const icon of meta.icons.filter(i=>!process.argv[2]||process.argv[2].split(',').includes(i.name)))for(const action of icon.animations){
 const sc=JSON.parse(fs.readFileSync(path.join(root,'scenes',action.name+'.scene.json')));let checked=0;
 for(let f=0;f<=144;f++){
  const t=f/60;
  for(const n of sc.nodes.filter(n=>n.name.endsWith('__outline.bold')||n.name.endsWith('__solid'))){
   if(sample(n.visibility,t)<.01)continue;const b=box(n,t),margin=Math.min(b[0],b[1],32-b[2],32-b[3]);report.minimumPaintedMargin=Math.min(report.minimumPaintedMargin,margin);
   assert.ok(margin>=3-.025,`${action.name} ${n.name} at frame ${f}: painted margin ${margin}`);checked++;
  }
  if(icon.name==='contactless-payment'){
   const phone=box(pick(sc,'contactless-payment-phone-frame'),t),signal=box(pick(sc,'contactless-payment-signal-0'),t),gap=signal[0]-phone[2];report.minimumContactlessGap=Math.min(report.minimumContactlessGap,gap);assert.ok(gap>=1-.025,'Contactless painted phone/signal gap '+gap);
  }
  if(icon.name==='bank-transfer')for(const arrow of ['outgoing','incoming']){
   const a=box(pick(sc,'bank-transfer-'+arrow),t),l=box(pick(sc,'bank-transfer-account-left-frame'),t),r=box(pick(sc,'bank-transfer-account-right-frame'),t),gap=Math.min(a[0]-l[2],r[0]-a[2]);report.minimumTransferGap=Math.min(report.minimumTransferGap,gap);assert.ok(gap>=1-.025,'Transfer cap touches account '+gap);
  }
  if(icon.name==='bank'){
   const n=pick(sc,'bank-door-frame'),b=box(n,t),opening=b[2]-b[0]-5;report.minimumDoorOpening=Math.min(report.minimumDoorOpening,opening);assert.ok(opening>=1.5,'Bank door counter closed');
  }
  if(icon.name==='portfolio'){
   const h=sample(pick(sc,'portfolio-handle').points,t),body=box(pick(sc,'portfolio-case-frame'),t),edge=body[1]+1.25,error=Math.max(Math.abs(h[1]-edge),Math.abs(h.at(-1)-edge));report.maximumHandleAttachmentError=Math.max(report.maximumHandleAttachmentError,error);assert.ok(error<3e-5,'Handle detaches from case');
  }
 }
 report.actions.push({name:action.name,paintedBoundsChecked:checked});
}
fs.mkdirSync(path.join(root,'evidence'),{recursive:true});fs.writeFileSync(path.join(root,'evidence/clearance-verification.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({...report,actions:report.actions.length},null,2));
