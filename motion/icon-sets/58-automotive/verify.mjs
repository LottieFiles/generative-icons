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
fs.writeFileSync(path.join(root,'evidence/directed-source-verification.json'),JSON.stringify(report,null,2)+'\n');
console.log(`PASS ${set.id}: ${report.icons} icons, ${report.actions} native actions; provenance, styles, weights, topology, state boundaries, holds and physical constraints.`);

// ---------- Automotive drawing contracts ----------
// Independent painted-clearance, contact and joint checks on the exported world
// geometry (Bold outline and Solid nodes), sampled every frame of every action.
{
 const curvePoints=(node,t)=>{const p=sample(node.points,t),out=[];let k=0,last,first;for(const cmd of node.commands){if(cmd===0||cmd===1){last=p.slice(k,k+2);k+=2;out.push(last);if(cmd===0)first=last;}else if(cmd===2){const a=last,b=p.slice(k,k+2),c=p.slice(k+2,k+4),d=p.slice(k+4,k+6);k+=6;if(!a.every((v,j)=>v===b[j]&&v===c[j]&&v===d[j]))for(let j=1;j<=48;j++){const u=j/48,v=1-u;out.push([0,1].map(i=>v*v*v*a[i]+3*v*v*u*b[i]+3*v*u*u*c[i]+u*u*u*d[i]));}last=d;}else if(cmd===3){out.push(first);last=first;}}return out;};
 const radius=(node,t)=>node.type==='stroke'?sample(node.width,t)/2:0;
 const segDist=(p,a,b)=>{const dx=b[0]-a[0],dy=b[1]-a[1],u=Math.max(0,Math.min(1,((p[0]-a[0])*dx+(p[1]-a[1])*dy)/(dx*dx+dy*dy||1)));return Math.hypot(p[0]-a[0]-u*dx,p[1]-a[1]-u*dy);};
 const polyDist=(p,line)=>Math.min(...line.slice(1).map((b,i)=>segDist(p,line[i],b)));
 const gapBetween=(a,b,t)=>{const pa=curvePoints(a,t),pb=curvePoints(b,t);return Math.min(...pa.map(p=>polyDist(p,pb)),...pb.map(p=>polyDist(p,pa)))-radius(a,t)-radius(b,t);};
 const node=(sc,name,style)=>{const n=sc.nodes.find(n=>n.name===`${name}-bold__color.primary__outline.bold`)||sc.nodes.find(n=>n.name===`${name}__color.primary__${style||'solid'}`);assert.ok(n,sc.name+' missing '+name);return n;};
 const visible=(n,t)=>sample(n.visibility,t)>.05;
 const endpoints=(n,t)=>{const p=curvePoints(n,t);return [p[0],p.at(-1)];};
 // kind: gap (painted clearance ≥ min), contact (curves within max), joint (endpoint of a on b's curve),
 // inside (all of a's painted extent within box).
 const contracts={
  'car':[{kind:'contact',a:'car-wheel-rear',b:'car-body-frame',max:.3},{kind:'contact',a:'car-wheel-front',b:'car-body-frame',max:.3}],
  'electric-car':[{kind:'gap',a:'electric-car-bolt',b:'electric-car-body-frame',min:.5}],
  'fuel-pump':[{kind:'joint',a:'fuel-pump-hose',end:1,b:'fuel-pump-nozzle'},{kind:'gap',a:'fuel-pump-nozzle',b:'fuel-pump-body-frame',min:1},{kind:'joint',a:'fuel-pump-hose',end:0,b:'fuel-pump-body-frame'}],
  'speedometer':[{kind:'gap',a:'speedometer-needle',b:'speedometer-dial',min:1},{kind:'gap',a:'speedometer-needle',b:'speedometer-tick-1',min:.9},{kind:'gap',a:'speedometer-needle',b:'speedometer-tick-0',min:.9},{kind:'gap',a:'speedometer-needle',b:'speedometer-tick-2',min:.9}],
  'parking-brake':[{kind:'gap',a:'parking-brake-bracket-0',b:'parking-brake-ring',min:1},{kind:'gap',a:'parking-brake-bracket-1',b:'parking-brake-ring',min:1},{kind:'gap',a:'parking-brake-p',b:'parking-brake-ring',min:1}],
  'tire-pressure':[{kind:'gap',a:'tire-pressure-mark',b:'tire-pressure-tire',min:1},{kind:'gap',a:'tire-pressure-dot',b:'tire-pressure-tire',min:1}],
  'engine':[{kind:'inside',a:'engine-block-frame',box:[3.4,7.4,28.6,25.2]}],
  'seatbelt':[{kind:'gap',a:'seatbelt-head',b:'seatbelt-torso-frame',min:1}],
  'headlights':[{kind:'gap',a:'headlights-beam-0',b:'headlights-lamp-frame',min:1},{kind:'gap',a:'headlights-beam-0',b:'headlights-beam-1',min:1},{kind:'gap',a:'headlights-beam-1',b:'headlights-beam-2',min:1}],
  'wiper':[{kind:'gap',a:'wiper-arm',b:'wiper-shield-frame',min:1,skipStart:true},{kind:'gap',a:'wiper-arm',b:'wiper-shield-solid',min:.8,solid:true,skipStart:true}],
  'steering-wheel':[{kind:'contact',a:'steering-wheel-spoke-0',b:'steering-wheel-rim',max:.01},{kind:'buried',a:'steering-wheel-spoke-0',end:0,b:'steering-wheel-hub'}],
  'car-key':[{kind:'gap',a:'car-key-signal-0',b:'car-key-head-frame',min:1},{kind:'gap',a:'car-key-signal-0',b:'car-key-signal-1',min:1}],
  'car-lock':[{kind:'gap',a:'car-lock-pad',b:'car-lock-body-frame',min:1},{kind:'gap',a:'car-lock-shackle',b:'car-lock-body-frame',min:1},{kind:'buried',a:'car-lock-shackle',end:1,b:'car-lock-pad'},{kind:'buried',a:'car-lock-shackle',end:0,b:'car-lock-pad',from:'complete'}],
  'car-battery':[{kind:'gap',a:'car-battery-plus',b:'car-battery-box-frame',min:1.2},{kind:'gap',a:'car-battery-minus',b:'car-battery-box-frame',min:1.2},{kind:'gap',a:'car-battery-plus',b:'car-battery-minus',min:2}],
  'gear-shift':[{kind:'gap',a:'gear-shift-knob',b:'gear-shift-slot-1',min:2}],
  'car-navigation':[{kind:'gap',a:'car-navigation-route',b:'car-navigation-screen-solid',min:.8,solid:true},{kind:'gap',a:'car-navigation-route',b:'car-navigation-screen-frame',min:1},{kind:'contact',a:'car-navigation-route',b:'car-navigation-pin',max:1.2,from:'complete'}],
  'car-wash':[{kind:'gap',a:'car-wash-drop-0',b:'car-wash-roof',min:.5},{kind:'gap',a:'car-wash-drop-1',b:'car-wash-roof',min:.5},{kind:'gap',a:'car-wash-drop-2',b:'car-wash-roof',min:.5}],
  'engine-oil':[{kind:'gap',a:'engine-oil-drop',b:'engine-oil-spout',min:.8},{kind:'joint',a:'engine-oil-handle',end:0,b:'engine-oil-can-frame'},{kind:'joint',a:'engine-oil-handle',end:1,b:'engine-oil-can-frame'},{kind:'joint',a:'engine-oil-spout',end:0,b:'engine-oil-can-frame'}],
  'car-window':[{kind:'gap',a:'car-window-window',b:'car-window-door-frame',min:1},{kind:'gap',a:'car-window-handle',b:'car-window-window',min:1},{kind:'inside',a:'car-window-glass-edge',box:[7.5,7.4,24.2,18.9]},{kind:'gap',a:'car-window-glass-edge',b:'car-window-handle',min:2}],
  'trunk':[{kind:'joint',a:'trunk-lid',end:0,b:'trunk-body-frame'},{kind:'joint',a:'trunk-lid',end:1,b:'trunk-body-frame',from:'closed'},{kind:'contact',a:'trunk-wheel-rear',b:'trunk-body-frame',max:.3}]
 };
 const clearance={result:'PASS',minimumPaintedMargin:32,checks:0,gaps:{},errors:[]};
 for(const icon of set.icons)for(const action of icon.animations){
  const sc=readScene(action.name),rules=contracts[icon.name];assert.ok(rules,icon.name+' has no drawing contract');
  for(let f=0;f<=144;f++){
   const t=f/60;
   for(const n of sc.nodes.filter(n=>n.name.endsWith('__outline.bold')||n.name.endsWith('__solid'))){
    if(!visible(n,t))continue;
    const p=curvePoints(n,t),r=radius(n,t);const m=Math.min(...p.map(q=>Math.min(q[0],q[1],32-q[0],32-q[1])))-r;
    clearance.minimumPaintedMargin=Math.min(clearance.minimumPaintedMargin,m);clearance.checks++;
    if(m<1.99)clearance.errors.push(`${action.name} ${n.name} frame ${f}: painted margin ${m.toFixed(3)}`);
   }
   for(const rule of rules){
    const a=node(sc,rule.a,rule.solid?'solid':undefined),b=rule.b?node(sc,rule.b,rule.solid?'solid':undefined):null;
    if(!visible(a,t)||(b&&!visible(b,t)))continue;
    const key=icon.name+' '+rule.kind+' '+rule.a.slice(icon.name.length+1)+(b?' / '+rule.b.slice(icon.name.length+1):'');
    const record=(value,ok)=>{clearance.gaps[key]=Math.min(clearance.gaps[key]??Infinity,value);clearance.checks++;if(!ok)clearance.errors.push(`${action.name} ${key} frame ${f}: ${value.toFixed(3)}`);};
    if(rule.kind==='gap'){
     // A stroke that is deliberately attached at one end is measured without that end's cap.
     let A=a;if(rule.skipEnd||rule.skipStart){const pts=curvePoints(a,t);const trimmed=rule.skipStart?pts.slice(Math.ceil(pts.length*.12)):pts.slice(0,Math.floor(pts.length*.88));A={...a,points:[[0,trimmed.flat()]],commands:trimmed.map((_,i)=>i?1:0)};}
     const g=gapBetween(A,b,t);record(g,g>=rule.min-.015);
    }else if(rule.kind==='contact'){
     if(rule.from==='complete'&&!(action.direction==='forward'?f===144:f===0))continue;
     const pa=curvePoints(a,t),pb=curvePoints(b,t);const d=Math.min(...pa.map(p=>polyDist(p,pb)));record(d,d<=rule.max+.015);
    }else if(rule.kind==='joint'){
     // A closed-only joint is checked at the closed state (frame 0 forward, final frame reverse).
     if(rule.from==='closed'&&!(action.direction==='forward'?f===0:f===144))continue;
     const e=endpoints(a,t)[rule.end],pb=curvePoints(b,t);const d=polyDist(e,pb);record(d,d<=(rule.tolerance??.02));
    }else if(rule.kind==='buried'){
     if(rule.from==='complete'&&!(action.direction==='forward'?f===144:f===0))continue;
     const e=endpoints(a,t)[rule.end],poly=curvePoints(b,t);let c=false;for(let i=0,j=poly.length-1;i<poly.length;j=i++){const p=poly[i],q=poly[j];if((p[1]>e[1])!==(q[1]>e[1])&&e[0]<(q[0]-p[0])*(e[1]-p[1])/(q[1]-p[1])+p[0])c=!c;}
     const d=polyDist(e,poly);record(c?d:-d,c&&d>=.25);
    }else if(rule.kind==='inside'){
     const p=curvePoints(a,t),r=radius(a,t),[l,tp,rt,bt]=rule.box;const m=Math.min(...p.map(q=>Math.min(q[0]-l,q[1]-tp,rt-q[0],bt-q[1])))-r;record(m,m>=-.015);
    }else throw Error('Unknown drawing rule '+rule.kind);
   }
  }
 }
 clearance.result=clearance.errors.length?'FAIL':'PASS';
 fs.writeFileSync(path.join(root,'evidence/clearance-verification.json'),JSON.stringify(clearance,null,2)+'\n');
 if(clearance.errors.length){console.log(clearance.errors.slice(0,30).join('\n'));throw Error(`FAIL drawing contracts: ${clearance.errors.length} violations`);}
 console.log(`PASS ${set.id} drawing contracts: ${clearance.checks} samples, minimum painted margin ${clearance.minimumPaintedMargin.toFixed(3)}, ${Object.keys(clearance.gaps).length} contact/clearance rules.`);
}
