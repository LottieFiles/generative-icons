// Independent all-frame contacts and paint gaps from exported sampled world coordinates.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url));
const set=JSON.parse(fs.readFileSync(path.join(root,'../../../prototype/icons/45/assets/set.json')));
const sample=(keys,t)=>{if(t<=keys[0][0])return keys[0][1];let j=keys.findIndex(k=>k[0]>=t);if(j<0)return keys.at(-1)[1];const[a,x]=keys[j-1],[b,y]=keys[j],u=(t-a)/(b-a);return Array.isArray(x)?x.map((v,i)=>v+(y[i]-v)*u):x+(y-x)*u;};
const pick=(sc,n)=>{const p=sc.nodes.find(p=>p.name.split('__')[0]===n);assert(p,'Missing '+n);return p;};
const report={result:'PASS',checks:0,frames:0,minimumWavePaintGap:Infinity,maximumPenContactError:0,maximumFoldContactError:0};
for(const icon of set.icons)for(const a of icon.animations){
 const sc=JSON.parse(fs.readFileSync(path.join(root,'scenes',a.name+'.scene.json')));
 for(let f=0;f<=144;f++){
  const t=f/60;report.frames++;
  if(icon.icon==='edit-message')for(const suffix of ['-bold','-solid']){
   const ink=sample(pick(sc,'edit-message-ink'+suffix).points,t),pen=sample(pick(sc,'edit-message-pen'+suffix).points,t);
   const d=Math.hypot(ink.at(-2)-pen[0],ink.at(-1)-pen[1]);report.maximumPenContactError=Math.max(report.maximumPenContactError,d);assert(d<.00003,a.name+' pen detached from ink');report.checks++;
  }
  if(icon.icon==='sticker')for(const suffix of ['-bold','-solid']){
   const paper=sample(pick(sc,'sticker-paper'+suffix).points,t),fold=sample(pick(sc,'sticker-fold'+suffix).points,t);
   // Paper's diagonal endpoints are (27,e) and (e,27), after its first cubic.
   const top=paper.slice(10,12),bottom=paper.slice(12,14);
   const d=Math.max(Math.hypot(fold[0]-bottom[0],fold[1]-bottom[1]),Math.hypot(fold.at(-2)-top[0],fold.at(-1)-top[1]));
   report.maximumFoldContactError=Math.max(report.maximumFoldContactError,d);assert(d<.00003,a.name+' fold detached from paper diagonal');report.checks++;
  }
  if(icon.icon==='voice-note')for(const suffix of ['-bold','-solid'])for(let j=0;j<3;j++){
   const n=pick(sc,'voice-note-wave-'+j+suffix),p=sample(n.points,t),radius=sample(n.width,t)/2;
   const gap=Math.min(Math.min(p[1],p[3])-radius-9.3,22.7-Math.max(p[1],p[3])-radius,26.7-p[0]-radius);
   report.minimumWavePaintGap=Math.min(report.minimumWavePaintGap,gap);assert(gap>=1.15,a.name+' waveform crowds clip border: '+gap);report.checks++;
  }
 }
}
fs.writeFileSync(path.join(root,'evidence/clearance-verification.json'),JSON.stringify(report,null,2)+'\n');
console.log(`PASS chat geometry: ${report.checks} checks; pen contact, sticker fold contacts, waveform painted clearance.`);
