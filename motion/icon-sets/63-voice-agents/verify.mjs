import {verifyFilledEllipsis} from '../library/verify-filled-ellipsis.mjs';
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import crypto from "node:crypto";
import assert from "node:assert/strict";
const root = path.dirname(fileURLToPath(import.meta.url));
const out = path.resolve(root, `../../../prototype/icons/${path.basename(root).slice(0,2)}/assets`);
const hash = (value) => crypto.createHash("sha256").update(value).digest("hex");
const set = JSON.parse(fs.readFileSync(path.join(out, "set.json")));
for (const icon of set.icons.flatMap(icon => icon.animations)) {
  const proof = JSON.parse(
    fs.readFileSync(
      path.join(root, `native/${icon.name}.json.provenance.json`),
    ),
  );
  for (const [file, expected] of [
    [`native/${icon.name}.json`, proof.sha256],
    [`scenes/${icon.name}.engine.js`, proof.scriptSha256],
    [`scenes/${icon.name}.scene.json`, proof.sceneSha256],
  ])
    assert.equal(hash(fs.readFileSync(path.join(root, file))), expected);
  assert.equal(
    hash(fs.readFileSync(path.join(root, "build.mjs"))),
    icon.sourceSha256,
  );
  assert.equal(hash(fs.readFileSync(path.join(root,"definitions.mjs"))),icon.definitionsSha256);
  assert.equal(hash(fs.readFileSync(path.join(root,"build.mjs"))),icon.setBuilderSha256,"Stale set builder");
  for(const [name,expected] of Object.entries(icon.authoringSources||{})) assert.equal(hash(fs.readFileSync(path.join(root,name))),expected,"Stale authoring source "+name);
  assert.equal(
    hash(fs.readFileSync(path.join(out, `${icon.name}.lottie`))),
    icon.sha256,
  );
  const raw = JSON.parse(
    fs.readFileSync(path.join(root, `native/${icon.name}.json`)),
  );
  const data = JSON.parse(fs.readFileSync(path.join(out, `${icon.name}.json`)));
  assert.equal(data.w, 32);
  assert.equal(data.h, 32);
  assert.equal(data.fr, 60);
  assert.equal(data.op, 144);
  assert.ok(data.layers.every((layer) => layer.ty === 4));
  assert.deepEqual(icon.styles, ["outline", "solid", "duotone"]);
  assert.deepEqual(icon.strokeWidth.presets, [1.5, 2, 2.5]);
  const strokeWidths = new Set();
  for (const layer of data.layers) for (const paint of layer.shapes.filter(s => ["st", "fl"].includes(s.ty))) {
    assert.ok(data.slots[paint.c.sid], "Unbound color");
    assert.ok(data.slots[paint.o.sid], "Unbound style");
    if (paint.ty === "st") { if(paint.o.sid.startsWith("style.outline"))strokeWidths.add(paint.w.k); assert.equal(paint.lc, 2); assert.equal(paint.lj, 2); }
  }
  if(icon.icon==='more-options'){assert.equal(strokeWidths.size,0);verifyFilledEllipsis(data,icon.name);}else assert.deepEqual([...strokeWidths].sort(), [1.5, 2, 2.5]);
  for (const theme of ["dark", "light"]) {
    const rules = JSON.parse(fs.readFileSync(path.join(out, `${icon.name}.theme-${theme}.json`))).rules;
    assert.deepEqual(rules.map(r => r.id).sort(), Object.keys(data.slots).sort(), "Theme drops token registry");
  }

  assert.deepEqual(
    data.layers.map((layer) => layer.ks),
    // JSON.stringify canonically writes the exporter's -0 as 0. They are
    // the same angle; every other field must remain exactly equal.
    JSON.parse(JSON.stringify(raw.layers.map((layer) => layer.ks))),
    "Packaging changed motion",
  );
  assert.deepEqual(
    data.layers.map((layer) =>
      layer.shapes.filter((shape) => shape.ty === "sh"),
    ),
    raw.layers.map((layer) =>
      layer.shapes.filter((shape) => shape.ty === "sh"),
    ),
    "Packaging changed path animation",
  );
  assert.ok(
    data.markers.some(
      (marker) => marker.cm === "rest" && marker.tm === icon.rest,
    ),
  );
  assert.ok(data.markers.some((marker) => marker.cm === "action"));
  const scene = JSON.parse(
    fs.readFileSync(path.join(root, `scenes/${icon.name}.scene.json`)),
  );
  for (const node of scene.nodes) {
    const points = node.points;
    if(!icon.stateful && !icon.cycleDegrees && node.visibility[0][1]>0 && node.visibility.at(-1)[1]>0) assert.deepEqual(points[0][1], points.at(-1)[1], icon.name+": reset changes visible geometry");
    const restPoints=points.filter(k=>k[0]>=icon.rest/60);
    if(!icon.continuous && node.visibility.at(-1)[1]>0)for(const k of restPoints)assert.deepEqual(k[1],points.at(-1)[1],icon.name+": moving after rest");
    if(!icon.continuous)for(const k of node.visibility.filter(k=>k[0]>=icon.rest/60))assert.equal(k[1],node.visibility.at(-1)[1],icon.name+": visibility moving after rest");
    assert.ok(
      points.every((point) => point[1].length === points[0][1].length),
      "Morph topology changed",
    );
    for (const [, point] of points)
      for (const coordinate of point) assert.ok(Number.isFinite(coordinate));
  }
  for(const [name,expected]of Object.entries(icon.directionSources||{}))assert.equal(hash(fs.readFileSync(path.join(root,"../library",name))),expected,"Stale direction source "+name);
}

// Strong and Light share complete start and final states, including visibility.
const readScene = name => JSON.parse(fs.readFileSync(path.join(root,`scenes/${name}.scene.json`)));
const ends=scene=>scene.nodes.map(n=>({name:n.name,commands:n.commands,first:n.points[0][1],last:n.points.at(-1)[1],alphaFirst:n.visibility[0][1],alphaLast:n.visibility.at(-1)[1]}));
for(const icon of set.icons){
  assert.deepEqual(icon.motions.map(m=>m.motionId),['light','strong']);
  const [light,strong]=icon.motions.map(m=>readScene(m.name));
  if(!icon.continuous&&!icon.cycleDegrees)for(let i=0;i<light.nodes.length;i++){const a=ends(light)[i],b=ends(strong)[i];assert.equal(a.alphaFirst,b.alphaFirst,icon.name+' initial visibility');assert.equal(a.alphaLast,b.alphaLast,icon.name+' final visibility');if(a.alphaFirst>0)assert.deepEqual(a.first,b.first,icon.name+' initial drawing');if(a.alphaLast>0)assert.deepEqual(a.last,b.last,icon.name+' final drawing');}
  for(const motion of icon.motions){
    const scene=readScene(motion.name);
    if(icon.stateful){
      assert.ok(motion.reverse,'Missing explicit reverse');
      const reverse=readScene(motion.reverse.name);
      for(let i=0;i<scene.nodes.length;i++){
        if(scene.nodes[i].visibility[0][1]>0)assert.deepEqual(scene.nodes[i].points[0][1],reverse.nodes[i].points.at(-1)[1],icon.name+' reverse final geometry');
        if(scene.nodes[i].visibility.at(-1)[1]>0)assert.deepEqual(scene.nodes[i].points.at(-1)[1],reverse.nodes[i].points[0][1],icon.name+' reverse initial geometry');
        assert.equal(scene.nodes[i].visibility[0][1],reverse.nodes[i].visibility.at(-1)[1],'Reverse visibility mismatch');
      }
    }
  }

}

// Independent constraints use exported world points, not the authored drivers.
const sample=(keys,t)=>{if(t<=keys[0][0])return keys[0][1];const i=keys.findIndex(k=>k[0]>=t);if(i<0)return keys.at(-1)[1];const[ta,a]=keys[i-1],[tb,b]=keys[i],u=(t-ta)/(tb-ta);return Array.isArray(a)?a.map((v,j)=>v+(b[j]-v)*u):a+(b-a)*u;};
const close=(a,b,label,tol=3e-5)=>assert.ok(Math.abs(a-b)<tol,`${label}: ${a} != ${b}`);
const pick=(scene,name)=>{const n=scene.nodes.find(n=>n.name.startsWith(name+'__')||n.name.startsWith(name+'-regular__'));assert.ok(n,'Missing '+scene.name+'/'+name);return n;};
const xy=(p,axis)=>p.filter((_,i)=>i%2===axis),extent=(p,axis)=>Math.max(...xy(p,axis))-Math.min(...xy(p,axis));
const still=(sc,name)=>{const n=pick(sc,name);for(const k of n.points)assert.deepEqual(k[1],n.points[0][1],sc.name+' fixed '+name);};
const increasing=(node,measure,label,sign=1)=>{let previous=-Infinity;for(const[,p]of node.points){const v=sign*measure(p);assert.ok(v>=previous-3e-5,label+' runs backward');previous=v;}};
const report={result:'PASS',icons:set.icons.length,actions:set.icons.reduce((a,i)=>a+i.animations.length,0),nativeTransformNodes:0,nativeTransformMaxCoordinateError:0,checks:[]};
for(const meta of set.icons.flatMap(i=>i.motions)){
 const sc=readScene(meta.name),n=meta.icon;
 const fixed=(meta.constraints||[]).filter(r=>r.type==='fixed').map(r=>r.part);
 // The supervisor records design constraints independently of driver code.
 // New AI sets use these to test fixed shells, rigid pieces, semantic progress,
 // and clipped content regions on the evaluated native scene data.
 for(const rule of meta.constraints||[]){
  if(rule.profile && rule.profile!==meta.motionId)continue;
  const node=pick(sc,rule.part),label=meta.name+': '+rule.type+' '+rule.part;
  if(rule.type==='fixed')still(sc,rule.part);
  else if(rule.type==='opaque'){for(const k of node.visibility)assert.equal(k[1],1,label+' fades');}
  else if(rule.type==='hidden'){for(let f=0;f<=144;f++){const t=f/60;if(t>=rule.from&&t<=rule.to)assert.equal(sample(node.visibility,t),0,label+' leaves a visible cap');}}
  else if(rule.type==='rigid'){
   const base=node.points[0][1],distances=[];
   for(let i=2;i<base.length;i+=2)distances.push(Math.hypot(base[i]-base[0],base[i+1]-base[1]));
   for(const[,p]of node.points)for(let i=2;i<p.length;i+=2)close(Math.hypot(p[i]-p[0],p[i+1]-p[1]),distances[i/2-1],label,5e-5);
  }else if(rule.type==='monotonic'){
   if(rule.measure==='alpha')increasing({points:node.visibility.map(([t,v])=>[t,[v]])},p=>p[0],label,rule.direction||1);
   else {const measures={width:p=>extent(p,0),height:p=>extent(p,1),x:p=>xy(p,0).reduce((a,b)=>a+b,0)/(p.length/2),y:p=>xy(p,1).reduce((a,b)=>a+b,0)/(p.length/2)};assert.ok(measures[rule.measure],label+' unknown measure');increasing(node,measures[rule.measure],label,rule.direction||1);}
  }else if(rule.type==='inside'){
   for(let f=0;f<=144;f++)if(sample(node.visibility,f/60)>0){const p=sample(node.points,f/60),[left,top,right,bottom]=rule.box;assert.ok(Math.min(...xy(p,0))>=left-3e-5&&Math.max(...xy(p,0))<=right+3e-5&&Math.min(...xy(p,1))>=top-3e-5&&Math.max(...xy(p,1))<=bottom+3e-5,label+' crosses its content region');}
  }else throw Error('Unknown design constraint '+rule.type);
 }
 // Confirm every native rigid transform against independently sampled world points.
 const data=JSON.parse(fs.readFileSync(path.join(out,meta.name+'.json'))),property=(p,f)=>p.a?sample(p.k.map(k=>[k.t,k.s]),f):p.k,scalar=v=>Array.isArray(v)?v[0]:v;
 for(const node of sc.nodes.filter(n=>n.transform)){
  const layer=data.layers.find(l=>l.nm===node.name.split('__')[0]);report.nativeTransformNodes++;
  assert.ok(layer.shapes.filter(s=>s.ty==='sh').every(s=>s.ks.a===0),'Rigid path was resampled');
  for(let f=0;f<=144;f++){const r=scalar(property(layer.ks.r,f))*Math.PI/180,pos=property(layer.ks.p,f),anchor=property(layer.ks.a,f),expected=sample(node.points,f/60);for(let i=0;i<node.basePoints.length;i+=2){const x=node.basePoints[i]-anchor[0],y=node.basePoints[i+1]-anchor[1],actual=[pos[0]+x*Math.cos(r)-y*Math.sin(r),pos[1]+x*Math.sin(r)+y*Math.cos(r)];for(let j=0;j<2;j++){report.nativeTransformMaxCoordinateError=Math.max(report.nativeTransformMaxCoordinateError,Math.abs(actual[j]-expected[i+j]));close(actual[j],expected[i+j],'Native world geometry');}}}
 }
 report.checks.push({name:meta.name,fixedParts:fixed,topology:true,stateBoundaries:true,hold:!meta.continuous,continuous:!!meta.continuous,designConstraints:meta.constraints||[]});
}
// Independently evaluate complete-cycle positions and velocities at the seam.
report.loopChecks=[];
for(const meta of set.icons.filter(i=>i.continuous).flatMap(i=>i.animations)){
 const sc=readScene(meta.name);let maxSeamPositionError=0,maxVelocityRatio=0;
 for(const n of sc.nodes){
  for(const keys of [n.points,n.visibility]){
   // A recycling part may change geometry only inside an explicit invisible
   // guard interval spanning the wrap. Never exempt its opacity seam.
   const hiddenSeam=keys===n.points&&[0,1/60,143/60,2.4].every(t=>sample(n.visibility,t)===0);
   if(hiddenSeam)continue;
   const vec=v=>Array.isArray(v)?v:[v],samples=Array.from({length:145},(_,f)=>vec(sample(keys,f/60)));
   for(let c=0;c<samples[0].length;c++){
    const a=samples[0][c],b=samples[144][c];maxSeamPositionError=Math.max(maxSeamPositionError,Math.abs(a-b));close(a,b,meta.name+' loop position',.00003);
    const velocities=samples.slice(1).map((p,i)=>p[c]-samples[i][c]);
    const peak=Math.max(...velocities.map(Math.abs));const mismatch=Math.abs(velocities[0]-velocities.at(-1));
    const acceleration=Math.max(...velocities.slice(1).map((v,i)=>Math.abs(v-velocities[i])));
    assert.ok(mismatch<=acceleration*1.06+.0001,meta.name+' loop acceleration spike '+n.name);
    if(peak>.0001)maxVelocityRatio=Math.max(maxVelocityRatio,mismatch/peak);
   }
  }
 }
 report.loopChecks.push({name:meta.name,maxSeamPositionError,maxVelocityRatio});
}
// Drawing contracts, independently measured on exported world geometry in both
// directions and profiles. These add to the scaffold's complete native gates.
report.drawingChecks={frames:0,contacts:0,clearances:0,minimumTranscriptGap:32,minimumToneGap:32};
const paintNode=(sc,name,style)=>{
 const suffix=style==='solid'?'-solid__color.primary__solid':'-bold__color.primary__outline.bold';
 const node=sc.nodes.find(n=>n.name===name+suffix);
 assert.ok(node,'Missing drawing contract part '+name+suffix);return node;
};
const coords=(node,t)=>sample(node.points,t);
const bounds=(node,t)=>{const p=coords(node,t),r=node.type==='stroke'?sample(node.width,t)/2:0;return [Math.min(...xy(p,0))-r,Math.min(...xy(p,1))-r,Math.max(...xy(p,0))+r,Math.max(...xy(p,1))+r];};
const curveSamples=(node,t)=>{
 const values=coords(node,t),points=[];let at=0,last,first;
 for(const command of node.commands){
  if(command===0||command===1){last=values.slice(at,at+2);at+=2;points.push(last);if(command===0)first=last;}
  else if(command===2){const a=last,b=values.slice(at,at+2),c=values.slice(at+2,at+4),d=values.slice(at+4,at+6);at+=6;
   for(let step=1;step<=32;step++){const u=step/32,v=1-u;points.push([0,1].map(k=>v*v*v*a[k]+3*v*v*u*b[k]+3*v*u*u*c[k]+u*u*u*d[k]));}last=d;
  }else if(command===3){points.push(first);last=first;}
 }
 return points;
};
for(const icon of set.icons)for(const action of icon.animations){
 const sc=readScene(action.name),name=icon.name;
 for(const style of ['bold','solid'])for(let f=0;f<=144;f++){
  const t=f/60,part=p=>paintNode(sc,name+'-'+p,style),p=p=>coords(part(p),t),b=p=>bounds(part(p),t);
  report.drawingChecks.frames++;
  if(name==='live-transcript'){
   if(sample(part('new-line').visibility,t)>.01){
    const gap=b('caret')[0]-b('new-line')[2];
    assert.ok(gap>=.89,action.name+' caret collides with committed text');
    report.drawingChecks.minimumTranscriptGap=Math.min(report.drawingChecks.minimumTranscriptGap,gap);
    report.drawingChecks.clearances++;
   }
   assert.ok(b('caret')[2]<=25.21,action.name+' caret crosses transcript rim');
  }
  if(name==='dtmf-input'){
   const gap=Math.min(b('low-tone')[0],b('high-tone')[0])-b('key')[2];
   assert.ok(gap>=1.39,action.name+' dual tones touch the pressed key');
   report.drawingChecks.minimumToneGap=Math.min(report.drawingChecks.minimumToneGap,gap);
   report.drawingChecks.clearances++;
  }
  if(name==='ivr-menu'){
   const trunk=p('trunk').slice(-2);
   for(const [branch,option,x]of [['left-branch','option-one',9],['right-branch','option-two',23]]){
    const route=p(branch),frame=p(option);
    close(route[0],trunk[0],action.name+' routing junction x');
    close(route[1],trunk[1],action.name+' routing junction y');
    close(route.at(-2),x,action.name+' option contact x');
    close(route.at(-1),Math.min(...xy(frame,1)),action.name+' branch misses option rim');
    report.drawingChecks.contacts+=2;
   }
  }
  if(name==='barge-in'){
   const stem=p('incoming').slice(-2),head=p('interrupt-head').slice(2,4);
   close(stem[0],head[0],action.name+' interruption arrow joint x');
   close(stem[1],head[1],action.name+' interruption arrow joint y');
   const shaft=p('incoming'),barb=p('interrupt-head');
   const barbGap=shaft[1]-Math.max(barb[1],barb[5])-(sample(part('incoming').width,t)+sample(part('interrupt-head').width,t))/2;
   assert.ok(barbGap>=1.19,action.name+' interruption barb closes against the arrow bend');
   report.drawingChecks.clearances++;
   for(const index of [0,1,2]){
    const bar=b('output-'+index);
    assert.ok(bar[1]>=6.8&&bar[3]<=15.2,action.name+' output touches speaking bubble rim');
    report.drawingChecks.clearances++;
   }
   report.drawingChecks.contacts++;
  }
  if(name==='sentiment-meter'||name==='silence-timeout'){
   const shell=part(name==='sentiment-meter'?'face':'clock'),points=coords(shell,t);
   const cx=(Math.min(...xy(points,0))+Math.max(...xy(points,0)))/2,cy=(Math.min(...xy(points,1))+Math.max(...xy(points,1)))/2;
   const radius=(Math.max(...xy(points,0))-Math.min(...xy(points,0)))/2;
   for(const detail of name==='sentiment-meter'?['mouth']:['minute-hand','short-hand']){
    const node=part(detail),paint=sample(node.width,t)/2+sample(shell.width,t)/2;
    const gap=radius-Math.max(...curveSamples(node,t).map(([x,y])=>Math.hypot(x-cx,y-cy)))-paint;
    assert.ok(gap>=(name==='sentiment-meter'?.5:.79),action.name+' '+detail+' touches the enclosing rim');
    report.drawingChecks.clearances++;
   }
  }
 }
}
fs.writeFileSync(path.join(root,'evidence/directed-source-verification.json'),JSON.stringify(report,null,2)+'\n');
console.log(`PASS ${set.id}: ${report.icons} icons, ${report.actions} native actions; provenance, styles, weights, topology, state boundaries, holds and physical constraints.`);
