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
// Drawing-specific contact and clearance checks on exported world geometry (Bold outline and
// Solid paint, every frame of every action). Distances subtract both parts' painted half-widths.
function curvePoints(node,t){const p=sample(node.points,t),out=[];let k=0,last,first;for(const cmd of node.commands){if(cmd===0||cmd===1){last=p.slice(k,k+2);k+=2;out.push(last);if(cmd===0)first=last;}else if(cmd===2){const a=last,b=p.slice(k,k+2),c=p.slice(k+2,k+4),d=p.slice(k+4,k+6);k+=6;for(let j=1;j<=32;j++){const u=j/32,v=1-u;out.push([0,1].map(i=>v*v*v*a[i]+3*v*v*u*b[i]+3*v*u*u*c[i]+u*u*u*d[i]));}last=d;}else if(cmd===3){out.push(first);last=first;}}return out;}
const lineDistance=(p,a,b)=>{const dx=b[0]-a[0],dy=b[1]-a[1],u=Math.max(0,Math.min(1,((p[0]-a[0])*dx+(p[1]-a[1])*dy)/(dx*dx+dy*dy||1)));return Math.hypot(p[0]-a[0]-u*dx,p[1]-a[1]-u*dy);};
const distance=(p,line)=>Math.min(...line.slice(1).map((b,i)=>lineDistance(p,line[i],b)));
const paintRadius=(n,t)=>n.type==='stroke'?sample(n.width,t)/2:0;
const boldNode=(sc,name)=>{const n=sc.nodes.find(n=>n.name.split('__')[0]===name+'-bold')||sc.nodes.find(n=>n.name.split('__')[0]===name);assert.ok(n,sc.name+' missing '+name);return n;};
const painted=(a,b,t)=>{const pa=curvePoints(a,t),pb=curvePoints(b,t);return Math.min(...pa.map(p=>distance(p,pb)),...pb.map(p=>distance(p,pa)))-paintRadius(a,t)-paintRadius(b,t);};
const bounds=(n,t)=>{const p=curvePoints(n,t),r=paintRadius(n,t);return [Math.min(...p.map(q=>q[0]))-r,Math.min(...p.map(q=>q[1]))-r,Math.max(...p.map(q=>q[0]))+r,Math.max(...p.map(q=>q[1]))+r];};
const visible=(n,t)=>sample(n.visibility,t)>.01;
// kind gap: painted clearance between a and b at every visible frame.
// kind touch: the first point of a sits on b's curve (within tol) whenever the action is at a state boundary.
// kind exclusive: a and b are never both partially or fully visible in the same frame.
// kind inside: a's painted box stays inside box by at least min.
// kind outside: the first point of a stays at least r - tol from b's sampled centre (a circle).
const RULES={
 'region':[{kind:'touch',a:'pin',b:'globe-frame',tol:.05,label:'pin tip seated on the globe rim at rest'},{kind:'outside',a:'pin',b:'globe-frame',r:8,tol:.05,label:'pin tip never sinks inside the globe'}],
 'auto-rotate':[{kind:'gap',a:'phone-frame',b:'arrow-a',min:1,label:'phone / arrow-a'},{kind:'gap',a:'phone-frame',b:'arrow-b',min:1,label:'phone / arrow-b'},{kind:'gap',a:'phone-frame',b:'head-a',min:1,label:'phone / head-a'},{kind:'gap',a:'phone-frame',b:'head-b',min:1,label:'phone / head-b'}],
 'mouse':[{kind:'gap',a:'wheel',b:'divider',min:.6,label:'wheel / divider'},{kind:'inside',a:'wheel',box:[10.75,5.75,21.25,13.25],min:.5,label:'wheel inside the top half at Bold'}],
 'currency':[{kind:'exclusive',a:'dollar-bar',b:'euro-c',label:'dollar and euro faces never show together'}],
 'sound-toggle':[{kind:'exclusive',a:'wave-1',b:'cross-0',label:'waves and cross never fade together'},{kind:'gap',a:'wave-0',b:'wave-1',min:1,label:'wave-0 / wave-1'}],
 'battery-saver':[{kind:'exclusive',a:'block',b:'leaf',label:'block and leaf never fade together'},{kind:'inside',a:'leaf',box:[5.25,11.25,23.75,20.75],min:.8,label:'leaf clear of the battery wall at Bold'},{kind:'inside',a:'block',box:[5.25,11.25,23.75,20.75],min:1,label:'charge block clear of the battery wall at Bold'}],
 'shortcut-key':[{kind:'inside',a:'cap-frame',box:[0,0,32,26.25],min:0,label:'cap never drops below the base skirt'},{kind:'inside',a:'bolt',box:[8.25,5.75,23.75,20.25],min:1,label:'bolt clear of the cap wall at Bold'}],
 'haptic-feedback':[{kind:'gap',a:'phone-frame',b:'arc-l',min:1,label:'phone / arc-l'},{kind:'gap',a:'phone-frame',b:'arc-r',min:1,label:'phone / arc-r'}],
 'mobile-data':[{kind:'gap',a:'down',b:'bar-0',min:1,label:'down arrow / bar-0'},{kind:'gap',a:'up',b:'down-head',min:.8,label:'up arrow / down head'},{kind:'gap',a:'bar-0',b:'bar-1',min:1,label:'bar-0 / bar-1'}],
 'pointer-speed':[{kind:'gap',a:'cursor-frame',b:'trail-0',min:.8,label:'cursor / trail-0'},{kind:'gap',a:'cursor-frame',b:'trail-1',min:.8,label:'cursor / trail-1'},{kind:'gap',a:'cursor-frame',b:'trail-2',min:.8,label:'cursor / trail-2'}],
 'trackpad':[{kind:'gap',a:'finger-0',b:'finger-1',min:.8,label:'finger-0 / finger-1'},{kind:'inside',a:'finger-1',box:[5.25,8.25,26.75,23.75],min:1,label:'fingers stay on the pad at Bold'},{kind:'gap',a:'finger-0',b:'divider',min:1,label:'fingers / divider'}],
 'reset-settings':[{kind:'gap',a:'knob-0',b:'ring',min:1,label:'knob-0 / ring'},{kind:'gap',a:'knob-1',b:'ring',min:1,label:'knob-1 / ring'},{kind:'gap',a:'knob-0',b:'knob-1',min:1,label:'knob-0 / knob-1'}],
 'auto-update':[{kind:'gap',a:'head',b:'ring',min:1,label:'arrow head / ring'},{kind:'gap',a:'shaft',b:'ring',min:1,label:'shaft / ring'}],
 'privacy-shield':[{kind:'inside',a:'eye',box:[7.25,5.75,24.75,26.25],min:1.5,label:'eye clear of the shield wall at Bold'}],
 'language':[{kind:'gap',a:'legs',b:'bubble-frame',min:1,label:'letter legs / bubble wall'},{kind:'gap',a:'bar',b:'bubble-frame',min:1,label:'letter bar / bubble wall'}],
 'appearance':[{kind:'gap',a:'handle',b:'band-frame',min:.5,label:'handle clear of the band'}],
};
report.clearance={rules:0,minimums:{},errors:[]};
for(const icon of set.icons)for(const action of icon.animations){
 const sc=readScene(action.name),rules=RULES[icon.name]||[];
 for(const rule of rules){
  const a=boldNode(sc,icon.name+'-'+rule.a),b=rule.b?boldNode(sc,icon.name+'-'+rule.b):null,key=icon.name+' '+rule.label;report.clearance.rules++;
  for(let f=0;f<=144;f++){
   const t=f/60;
   if(rule.kind==='gap'){if(!visible(a,t)||!visible(b,t))continue;const v=painted(a,b,t);report.clearance.minimums[key]=Math.min(report.clearance.minimums[key]??Infinity,v);if(v<rule.min-.015)report.clearance.errors.push(`${action.name} ${rule.label} frame ${f}: ${v.toFixed(3)} < ${rule.min}`);}
   else if(rule.kind==='touch'){if(t!==0&&f!==action.rest)continue;const tip=curvePoints(a,t)[0],v=distance(tip,curvePoints(b,t));report.clearance.minimums[key]=Math.max(report.clearance.minimums[key]??0,v);if(v>rule.tol)report.clearance.errors.push(`${action.name} ${rule.label} frame ${f}: ${v.toFixed(3)} > ${rule.tol}`);}
   else if(rule.kind==='exclusive'){const va=sample(a.visibility,t),vb=sample(b.visibility,t);if(va>.01&&vb>.01)report.clearance.errors.push(`${action.name} ${rule.label} frame ${f}: ${va.toFixed(2)} and ${vb.toFixed(2)}`);}
   else if(rule.kind==='outside'){if(!visible(a,t))continue;const tip=curvePoints(a,t)[0],pb=curvePoints(b,t),c=[0,1].map(i=>pb.reduce((v,q)=>v+q[i],0)/pb.length),v=Math.hypot(tip[0]-c[0],tip[1]-c[1]);report.clearance.minimums[key]=Math.min(report.clearance.minimums[key]??Infinity,v);if(v<rule.r-rule.tol)report.clearance.errors.push(`${action.name} ${rule.label} frame ${f}: ${v.toFixed(3)} < ${rule.r}`);}
   else if(rule.kind==='inside'){if(!visible(a,t))continue;const bb=bounds(a,t),[l,tp,r,bt]=rule.box,v=Math.min(bb[0]-l,bb[1]-tp,r-bb[2],bt-bb[3]);report.clearance.minimums[key]=Math.min(report.clearance.minimums[key]??Infinity,v);if(v<rule.min-.015)report.clearance.errors.push(`${action.name} ${rule.label} frame ${f}: ${v.toFixed(3)} < ${rule.min}`);}
   else throw Error('Unknown clearance rule '+rule.kind);
  }
 }
}
if(report.clearance.errors.length){report.result='FAIL';fs.writeFileSync(path.join(root,'evidence/directed-source-verification.json'),JSON.stringify(report,null,2)+'\n');throw Error('Clearance failures:\n'+report.clearance.errors.slice(0,20).join('\n'));}
fs.writeFileSync(path.join(root,'evidence/directed-source-verification.json'),JSON.stringify(report,null,2)+'\n');
console.log(`PASS ${set.id}: ${report.icons} icons, ${report.actions} native actions; provenance, styles, weights, topology, state boundaries, holds and physical constraints; ${report.clearance.rules} drawing clearance rules (${Object.keys(report.clearance.minimums).length} measured minimums).`);
