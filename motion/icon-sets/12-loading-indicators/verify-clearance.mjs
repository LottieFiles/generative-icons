import fs from'node:fs';import path from'node:path';import{fileURLToPath}from'node:url';import assert from'node:assert/strict';
const root=path.dirname(fileURLToPath(import.meta.url));
const read=n=>JSON.parse(fs.readFileSync(path.join(root,'scenes',n+'.scene.json')));
const sample=(ks,t)=>{if(t<=ks[0][0])return ks[0][1];const i=ks.findIndex(k=>k[0]>=t);if(i<0)return ks.at(-1)[1];const[a,x]=ks[i-1],[b,y]=ks[i],u=(t-a)/(b-a);return Array.isArray(x)?x.map((v,j)=>v+(y[j]-v)*u):x+(y-x)*u;};
const report={result:'PASS',paintWidth:2.5,movingPointPairs:[],rippleResetGuards:[],stackPaintGap:Infinity};
for(const icon of['wave-dots','infinity-run','bead-relay','bead-track'])for(const strong of[false,true]){
 const name=icon+(strong?'-strong':''),sc=read(name),nodes=sc.nodes.filter(n=>n.name.includes('-regular__')&&n.transform&&n.basePoints.length===4);let minimum=Infinity;
 for(let f=0;f<=144;f++){const t=f/60,active=nodes.filter(n=>sample(n.visibility,t)>0);for(let i=0;i<active.length;i++)for(let j=i+1;j<active.length;j++){const a=sample(active[i].points,t),b=sample(active[j].points,t),gap=Math.hypot(a[0]-b[0],a[1]-b[1])-2.5;minimum=Math.min(minimum,gap);assert.ok(gap>.15,name+' has colliding Bold beads');}}
 report.movingPointPairs.push({name,minimumBoldGap:minimum});
}
for(const strong of[false,true]){
 const name='ripple-rings'+(strong?'-strong':''),sc=read(name);let resets=0;
 for(const n of sc.nodes.filter(n=>n.name.includes('-ring-')&&n.name.includes('-regular__'))){let previous=sample(n.points,0);for(let f=1;f<=144;f++){const p=sample(n.points,f/60),jump=Math.max(...p.map((x,i)=>Math.abs(x-previous[i])));if(jump>3){for(const k of[-1,0,1]){const frame=(f+k+144)%144;assert.equal(sample(n.visibility,frame/60),0,name+' visible radius recycle');}resets++;}previous=p;}}
 report.rippleResetGuards.push({name,resets});
}
// Three horizontal stack centerlines must never fuse, including at equal y.
for(const strong of[false,true]){
 const sc=read('stack-cycle'+(strong?'-strong':'')),nodes=sc.nodes.filter(n=>n.name.includes('-regular__'));
 for(let f=0;f<=144;f++)for(let i=0;i<nodes.length;i++)for(let j=i+1;j<nodes.length;j++){
  const a=sample(nodes[i].points,f/60),b=sample(nodes[j].points,f/60);
  const dx=Math.max(0,Math.max(Math.min(a[0],a[2]),Math.min(b[0],b[2]))-Math.min(Math.max(a[0],a[2]),Math.max(b[0],b[2]))),dy=Math.abs(a[1]-b[1]);
  const gap=Math.hypot(dx,dy)-2.5;assert.ok(gap>.5,'Stack strokes fuse in Bold');report.stackPaintGap=Math.min(report.stackPaintGap,gap);
 }
}
fs.writeFileSync(path.join(root,'evidence/clearance-verification.json'),JSON.stringify(report,null,2)+'\n');console.log('PASS moving-point clearance, hidden ripple recycling, and rigid stack separation.');
