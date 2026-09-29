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

// ---------- Drawing-specific contact and clearance checks (Onboarding & tips) ----------
// Independent of the authored drivers: sampled curve points from the exported world geometry,
// painted radii from the exported stroke widths (Bold outline and Solid tubes), every frame.
const lin=(keys,t)=>{if(t<=keys[0][0])return keys[0][1];if(t>=keys.at(-1)[0])return keys.at(-1)[1];const i=keys.findIndex(k=>k[0]>t),a=keys[i-1],b=keys[i],u=(t-a[0])/(b[0]-a[0]);return Array.isArray(a[1])?a[1].map((v,j)=>v+(b[1][j]-v)*u):a[1]+(b[1]-a[1])*u;};
function curvePoints(node,t){const p=lin(node.points,t),out=[];let k=0,last,first;for(const cmd of node.commands){if(cmd===0||cmd===1){last=p.slice(k,k+2);k+=2;out.push(last);if(cmd===0)first=last;}else if(cmd===2){const a=last,b=p.slice(k,k+2),c=p.slice(k+2,k+4),d=p.slice(k+4,k+6);k+=6;if(a.every((v,j)=>v===b[j]&&v===c[j]&&v===d[j])){last=d;continue;}for(let j=1;j<=48;j++){const u=j/48,v=1-u;out.push([0,1].map(i=>v*v*v*a[i]+3*v*v*u*b[i]+3*v*u*u*c[i]+u*u*u*d[i]));}last=d;}else if(cmd===3){out.push(first);last=first;}}return out;}
const radius=(n,t)=>n.type==='stroke'?lin(n.width,t)/2:0;
const segDist=(p,a,b)=>{const dx=b[0]-a[0],dy=b[1]-a[1],u=Math.max(0,Math.min(1,((p[0]-a[0])*dx+(p[1]-a[1])*dy)/(dx*dx+dy*dy||1)));return Math.hypot(p[0]-a[0]-u*dx,p[1]-a[1]-u*dy);};
const polyDist=(p,line)=>{let m=Infinity;for(let i=1;i<line.length;i++)m=Math.min(m,segDist(p,line[i-1],line[i]));return line.length===1?Math.hypot(p[0]-line[0][0],p[1]-line[0][1]):m;};
const painted=(sc,base)=>{const n=sc.nodes.find(n=>n.name.split('__')[0]===base+'-bold')||sc.nodes.find(n=>n.name.split('__')[0]===base);assert.ok(n,sc.name+' missing part '+base);return n;};
const gapBetween=(a,b,t)=>{const pa=curvePoints(a,t),pb=curvePoints(b,t);return Math.min(...pa.map(p=>polyDist(p,pb)),...pb.map(p=>polyDist(p,pa)))-radius(a,t)-radius(b,t);};
const bbox=(n,t)=>{const p=curvePoints(n,t),r=radius(n,t);return[Math.min(...p.map(q=>q[0]))-r,Math.min(...p.map(q=>q[1]))-r,Math.max(...p.map(q=>q[0]))+r,Math.max(...p.map(q=>q[1]))+r];};
const centre=(n,t)=>{const b=bbox(n,t);return[(b[0]+b[2])/2,(b[1]+b[3])/2];};
// Each rule names two painted parts (Bold outline or Solid) and a minimum painted gap in grid units.
// A negative minimum allows a deliberate momentary contact. fromEnd checks only the settled forward result;
// minVisibility skips frames where a part is still faint (confetti pieces separate while fading in).
const pair=(a,b,min,extra={})=>[{kind:'gap',a,b,min,...extra},{kind:'gap',a:a+'-solid',b:b+'-solid',min,...extra}];
const clearanceRules={
 'lightbulb':[...['ray-up','ray-left','ray-right'].flatMap(r=>[{kind:'gap',a:r,b:'glass-frame',min:1},{kind:'gap',a:r+'-solid',b:'glass-solid',min:1}]),{kind:'gap',a:'foot',b:'glass-frame',min:1},{kind:'gap',a:'foot-solid',b:'glass-solid',min:1.6}],
 'product-tour':[{kind:'on-route',a:'marker',route:[[7,22],[16,10],[25,22]],tolerance:.05,profile:'light'},{kind:'on-route',a:'marker',route:[[7,22],[16,10],[25,22]],tolerance:.05,fromEnd:true},{kind:'hop',a:'marker',route:[[7,22],[16,10],[25,22]],min:-2.25,profile:'strong'},...pair('stop-a','route-ab',1),...pair('stop-b','route-ab',1),...pair('stop-b','route-bc',1),...pair('stop-c','route-bc',1)],
 'tooltip':[...pair('target','bubble-frame',1).slice(0,1),{kind:'gap',a:'target-solid',b:'bubble-solid',min:1},{kind:'gap',a:'line-0',b:'bubble-frame',min:1},{kind:'gap',a:'line-1',b:'bubble-frame',min:1},{kind:'gap',a:'line-0',b:'line-1',min:1}],
 'hotspot':[...pair('dot','ring',1)],
 'new-badge':[{kind:'gap',a:'star',b:'seal',min:.9}],
 'welcome-wave':[...pair('finger-0','finger-1',1),...pair('finger-1','finger-2',1),...pair('finger-2','finger-3',1),...pair('thumb','finger-0',1)],
 'progress-checklist':[...[0,1,2].flatMap(i=>[...pair('check-'+i,'row-'+i,1),{kind:'gap',a:'check-'+i,b:'card-frame',min:.9},{kind:'gap',a:'row-'+i,b:'card-frame',min:1}]),...pair('row-0','row-1',1),...pair('row-1','row-2',1)],
 'skip-step':[...pair('chevron-1','bar',-.35),...pair('chevron-0','chevron-1',1),...pair('chevron-0','bar',1)],
 'next-step':[{kind:'gap',a:'stem',b:'button',min:1},{kind:'gap',a:'barb',b:'button',min:1},{kind:'gap',a:'barb-solid',b:'button-solid',min:1}],
 'spark-wand':[{kind:'gap',a:'wand',b:'sparkle',min:1},{kind:'gap',a:'wand-solid',b:'sparkle-solid',min:1},...pair('spark-0','wand',1),...pair('spark-1','wand',1),{kind:'gap',a:'spark-1',b:'sparkle',min:1},{kind:'gap',a:'spark-0',b:'sparkle',min:1}],
 'hotkey-hint':[{kind:'gap',a:'face-frame',b:'skirt',min:1},{kind:'gap',a:'face-solid',b:'skirt-solid',min:1},{kind:'gap',a:'legend',b:'face-frame',min:1}],
 'spotlight':[{kind:'gap',a:'spot',b:'page',min:1}],
 'coachmark':[{kind:'gap',a:'card-frame',b:'button',min:1},{kind:'gap',a:'card-solid',b:'button-solid',min:1},{kind:'gap',a:'line-0',b:'card-frame',min:1},{kind:'gap',a:'line-1',b:'card-frame',min:1}],
 'getting-started':[...['trail-c','trail-l','trail-r'].flatMap(tr=>[{kind:'gap',a:tr,b:'body',min:1},{kind:'gap',a:tr+'-solid',b:'body-solid',min:1}]),...pair('trail-l','trail-c',1),...pair('trail-c','trail-r',1),{kind:'gap',a:'window',b:'body',min:1}],
 'tip-of-the-day':[{kind:'gap',a:'sparkle',b:'header',min:1},{kind:'gap',a:'sparkle',b:'leaf-frame',min:1},{kind:'gap',a:'sparkle-solid',b:'leaf-solid',min:1}],
 'welcome-gift':[{kind:'gap',a:'lid-frame',b:'box',min:1,fromEnd:true},{kind:'gap',a:'lid-solid',b:'box-solid',min:1,fromEnd:true},...pair('bow-l','box',1),...pair('bow-r','box',1)],
 'first-run':[{kind:'gap',a:'pointer',b:'button-frame',min:-.6},{kind:'gap',a:'pointer-solid',b:'button-solid',min:-.6},{kind:'gap',a:'label',b:'button-frame',min:1}],
 'dismiss-tip':[{kind:'gap',a:'close-a',b:'card-frame',min:.8},{kind:'gap',a:'close-b',b:'card-frame',min:.8},{kind:'gap',a:'line-0',b:'close-a',min:1},{kind:'gap',a:'line-0',b:'line-1',min:1}],
 'confetti':[...pair('strip-2','dot-1',1,{minVisibility:.5}),...pair('ribbon-0','dot-0',1,{minVisibility:.5}),...pair('strip-3','dot-0',1,{minVisibility:.5}),...pair('ribbon-1','strip-2',1,{minVisibility:.5}),...pair('strip-0','dot-1',1,{minVisibility:.5})],
 'sparkle':[...pair('small','big',1).slice(0,1),{kind:'gap',a:'small-solid',b:'big-solid',min:1}],
};
const clearance={result:'PASS',checks:0,minimumPaintedMargin:32,gaps:{},errors:[]};
for(const icon of set.icons){
 const rules=clearanceRules[icon.name];assert.ok(rules,icon.name+' has no drawing-specific clearance rules');
 for(const action of icon.animations){
  const sc=readScene(action.name),lastFrame=Math.round(action.active*60);
  for(let f=0;f<=144;f++){
   const t=f/60;
   for(const n of sc.nodes.filter(n=>n.name.endsWith('__outline.bold')||n.name.endsWith('__solid'))){
    if(lin(n.visibility,t)<.01)continue;
    const b=bbox(n,t),margin=Math.min(b[0],b[1],32-b[2],32-b[3]);clearance.minimumPaintedMargin=Math.min(clearance.minimumPaintedMargin,margin);clearance.checks++;
    if(margin<1.99)clearance.errors.push(`${action.name} ${n.name} frame ${f}: painted margin ${margin.toFixed(3)}`);
   }
   for(const rule of rules){
    if(rule.fromEnd&&!(action.direction==='forward'&&f>=lastFrame))continue;
    if(rule.profile&&rule.profile!==action.motionId)continue;
    const faint=rule.minVisibility??.01,a=painted(sc,icon.name+'-'+rule.a);if(lin(a.visibility,t)<faint)continue;
    const key=icon.name+' '+rule.kind+' '+rule.a+(rule.b?' / '+rule.b:'');
    let value,min;
    if(rule.kind==='gap'){const b=painted(sc,icon.name+'-'+rule.b);if(lin(b.visibility,t)<faint)continue;value=gapBetween(a,b,t);min=rule.min;}
    else if(rule.kind==='on-route'){const c=centre(a,t);value=-polyDist(c,rule.route);min=-rule.tolerance;}
    else if(rule.kind==='hop'){const c=centre(a,t);value=-polyDist(c,rule.route);min=rule.min;}
    else throw Error('Unknown clearance rule '+rule.kind);
    clearance.gaps[key]=Math.min(clearance.gaps[key]??Infinity,value);clearance.checks++;
    if(value<min-.015)clearance.errors.push(`${action.name} ${key} frame ${f}: ${value.toFixed(3)} < ${min}`);
   }
  }
 }
}
clearance.result=clearance.errors.length?'FAIL':'PASS';
report.clearance=clearance;
fs.writeFileSync(path.join(root,'evidence/clearance-verification.json'),JSON.stringify(clearance,null,2)+'\n');
if(clearance.errors.length){console.log('FAIL clearance: '+[...new Set(clearance.errors)].slice(0,30).join('\n  '));process.exitCode=1;}
fs.writeFileSync(path.join(root,'evidence/directed-source-verification.json'),JSON.stringify(report,null,2)+'\n');
if(!clearance.errors.length)console.log(`PASS ${set.id}: ${report.icons} icons, ${report.actions} native actions; provenance, styles, weights, topology, state boundaries, holds, physical constraints and ${Object.keys(clearance.gaps).length} drawing-specific clearance rules (min painted margin ${clearance.minimumPaintedMargin.toFixed(2)}).`);
