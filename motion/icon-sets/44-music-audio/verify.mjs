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

// ---------------------------------------------------------------------------
// Music & audio drawing-specific contact and clearance checks on exported world
// geometry (Bold outline and Solid paint, every frame of every action).
// Gaps subtract the painted half-width of both parts; margins are painted bounds.
{
 const lerp=(keys,t)=>{if(t<=keys[0][0])return keys[0][1];if(t>=keys.at(-1)[0])return keys.at(-1)[1];const i=keys.findIndex(k=>k[0]>t),a=keys[i-1],b=keys[i],u=(t-a[0])/(b[0]-a[0]);return Array.isArray(a[1])?a[1].map((v,j)=>v+(b[1][j]-v)*u):a[1]+(b[1]-a[1])*u;};
 const curvePoints=(node,t)=>{const p=lerp(node.points,t),out=[];let k=0,last,first;for(const cmd of node.commands){if(cmd===0||cmd===1){last=p.slice(k,k+2);k+=2;out.push(last);if(cmd===0)first=last;}else if(cmd===2){const a=last,b=p.slice(k,k+2),c=p.slice(k+2,k+4),d=p.slice(k+4,k+6);k+=6;if(!a.every((v,j)=>v===b[j]&&v===c[j]&&v===d[j]))for(let j=1;j<=48;j++){const u=j/48,v=1-u;out.push([0,1].map(i=>v*v*v*a[i]+3*v*v*u*b[i]+3*v*u*u*c[i]+u*u*u*d[i]));}last=d;}else if(cmd===3){out.push(first);last=first;}}return out;};
 const radius=(n,t)=>n.type==='stroke'?lerp(n.width,t)/2:0;
 const lineDistance=(p,a,b)=>{const dx=b[0]-a[0],dy=b[1]-a[1],u=Math.max(0,Math.min(1,((p[0]-a[0])*dx+(p[1]-a[1])*dy)/(dx*dx+dy*dy||1)));return Math.hypot(p[0]-a[0]-u*dx,p[1]-a[1]-u*dy);};
 const polyDistance=(p,line)=>Math.min(...line.slice(1).map((b,i)=>lineDistance(p,line[i],b)));
 const gapBetween=(a,b,t)=>{const pa=curvePoints(a,t),pb=curvePoints(b,t);return Math.min(...pa.map(p=>polyDistance(p,pb)),...pb.map(p=>polyDistance(p,pa)))-radius(a,t)-radius(b,t);};
 const paintedBox=(n,t)=>{const p=curvePoints(n,t),r=radius(n,t);return [Math.min(...p.map(q=>q[0]))-r,Math.min(...p.map(q=>q[1]))-r,Math.max(...p.map(q=>q[0]))+r,Math.max(...p.map(q=>q[1]))+r];};
 const node=(sc,name)=>{const n=sc.nodes.find(n=>n.name.split('__')[0]===name+'-bold')||sc.nodes.find(n=>n.name.split('__')[0]===name);assert.ok(n,'Missing '+sc.name+'/'+name);return n;};
 const visible=(n,t)=>lerp(n.visibility,t)>=.01;
 const insidePoly=(p,poly)=>{let c=false;for(let i=0,j=poly.length-1;i<poly.length;j=i++){const a=poly[i],b=poly[j];if((a[1]>p[1])!==(b[1]>p[1])&&p[0]<(b[0]-a[0])*(p[1]-a[1])/(b[1]-a[1])+a[0])c=!c;}return c;};
 // kind gap: painted clearance between two separate parts. kind attach: every end of a's
 // centerline lies inside b's paint (a joint). kind inside: a's painted box stays in a region.
 const rules={
  'vinyl-record':[{kind:'gap',a:'groove-0',b:'label',min:1},{kind:'gap',a:'groove-1',b:'label',min:1},{kind:'gap',a:'groove-0',b:'platter',min:1},{kind:'gap',a:'groove-1',b:'platter',min:1},{kind:'gap',a:'arm',b:'groove-0',min:.8},{kind:'gap',a:'arm',b:'groove-1',min:.8},{kind:'gap',a:'arm',b:'label',min:1}],
  'cassette-tape':[{kind:'gap',a:'reel-0',b:'reel-1',min:1},{kind:'gap',a:'reel-0',b:'shell-frame',min:1},{kind:'gap',a:'reel-1',b:'shell-frame',min:1},{kind:'gap',a:'reel-0',b:'window',min:1},{kind:'gap',a:'reel-1',b:'window',min:1}],
  'audio-waveform':[{kind:'gap',a:'bar-1',b:'bar-2',min:.9},{kind:'gap',a:'bar-2',b:'bar-3',min:.9},{kind:'gap',a:'bar-3',b:'bar-4',min:.9}],
  'loudspeaker':[{kind:'gap',a:'woofer',b:'cabinet',min:.9},{kind:'gap',a:'cap',b:'woofer',min:1},{kind:'gap',a:'tweeter',b:'woofer',min:1},{kind:'gap',a:'tweeter',b:'cabinet',min:1}],
  'radio':[{kind:'gap',a:'grille',b:'body',min:1},{kind:'gap',a:'needle',b:'grille',min:1},{kind:'gap',a:'needle',b:'body',min:1},{kind:'gap',a:'scale',b:'body',min:1},{kind:'attach',a:'antenna',b:'body',end:0}],
  'metronome':[{kind:'gap',a:'weight',b:'body-frame',min:1},{kind:'attach',a:'rod',b:'weight',end:1,through:true}],
  'tuner':[{kind:'gap',a:'needle',b:'dial',min:1},{kind:'gap',a:'tick-0',b:'dial',min:1},{kind:'gap',a:'tick-1',b:'dial',min:1},{kind:'gap',a:'tick-2',b:'dial',min:1},{kind:'attach',a:'needle',b:'hub',end:0}],
  'piano-keys':[{kind:'gap',a:'black-0',b:'black-1',min:1},{kind:'gap',a:'black-1',b:'black-2',min:1},{kind:'inside',a:'black-1',box:[3.5,5.75,28.5,22.6]},{kind:'inside',a:'black-2',box:[3.5,5.75,28.5,22.6]}],
  'guitar':[{kind:'gap',a:'sound-hole',b:'body',min:1},{kind:'attach',a:'neck',b:'body',end:0},{kind:'attach',a:'neck',b:'head',end:1}],
  'drum':[{kind:'gap',a:'stick-a',b:'stick-b',min:1}],
  'lyrics':[{kind:'gap',a:'flag',b:'line-0',min:1},{kind:'gap',a:'flag',b:'line-1',min:1},{kind:'gap',a:'head',b:'line-2',min:1},{kind:'gap',a:'line-0',b:'line-1',min:1}],
  'music-album':[{kind:'gap',a:'groove',b:'sleeve',min:1},{kind:'attach',a:'disc',b:'sleeve',end:0},{kind:'attach',a:'disc',b:'sleeve',end:1}],
  'earbuds':[{kind:'gap',a:'bud-a',b:'bud-b',min:1},{kind:'gap',a:'bud-a-stem',b:'bud-b-stem',min:1},{kind:'attach',a:'bud-a-stem',b:'bud-a',end:0}],
  'mute-audio':[{kind:'gap',a:'wave-0',b:'speaker',min:1},{kind:'gap',a:'wave-0',b:'wave-1',min:1},{kind:'gap',a:'cross-0',b:'speaker',min:1}],
  'crossfade':[{kind:'gap',a:'knob',b:'curve-a',min:1},{kind:'gap',a:'knob',b:'curve-b',min:1}],
  'equalizer-preset':[{kind:'gap',a:'knob-0',b:'track-1',min:1},{kind:'gap',a:'knob-1',b:'track-0',min:1},{kind:'gap',a:'knob-1',b:'track-2',min:1},{kind:'gap',a:'knob-2',b:'track-1',min:1},{kind:'inside',a:'knob-0',box:[2,6,30,26]},{kind:'inside',a:'knob-1',box:[2,6,30,26]},{kind:'inside',a:'knob-2',box:[2,6,30,26]}],
  'audio-jack':[{kind:'gap',a:'body-frame',b:'socket',min:1},{kind:'gap',a:'pin',b:'socket',min:.6,direction:'forward',to:0},{kind:'attach',a:'cable',b:'body-frame',end:0}],
  'spatial-audio':[{kind:'gap',a:'arc-right',b:'shoulders',min:1},{kind:'gap',a:'arc-left',b:'shoulders',min:1},{kind:'gap',a:'arc-top',b:'shoulders',min:1},{kind:'gap',a:'arc-right',b:'head',min:1},{kind:'gap',a:'arc-top',b:'head',min:1},{kind:'gap',a:'head',b:'shoulders',min:.9}],
  'tuning-fork':[{kind:'attach',a:'stem',b:'prongs',end:0},{kind:'attach',a:'stem',b:'knob',end:1}],
  'music-note':[{kind:'attach',a:'stem',b:'head',end:0},{kind:'attach',a:'flag',b:'stem',end:0}]
 };
 const clearance={result:'PASS',checks:0,minimumPaintedMargin:32,gaps:{},errors:[]};
 for(const icon of set.icons){assert.ok(rules[icon.name],icon.name+' has no drawing-specific clearance rules');for(const action of icon.animations){
  const sc=readScene(action.name);
  for(let f=0;f<=144;f++){const t=f/60;
   for(const n of sc.nodes.filter(n=>n.name.endsWith('__outline.bold')||n.name.endsWith('__solid'))){if(!visible(n,t))continue;const b=paintedBox(n,t),m=Math.min(b[0],b[1],32-b[2],32-b[3]);clearance.minimumPaintedMargin=Math.min(clearance.minimumPaintedMargin,m);clearance.checks++;if(m<1.99)clearance.errors.push(`${action.name} ${n.name} frame ${f}: painted margin ${m.toFixed(3)}`);}
   for(const rule of rules[icon.name]){
    if(rule.direction&&rule.direction!==action.direction)continue;if(rule.to!=null&&t>rule.to)continue;if(rule.from!=null&&t<rule.from)continue;
    const a=node(sc,icon.name+'-'+rule.a);if(!visible(a,t))continue;const key=icon.name+' '+rule.kind+' '+rule.a+'/'+(rule.b||rule.box);
    const record=(value,min)=>{clearance.gaps[key]=Math.min(clearance.gaps[key]??Infinity,value);clearance.checks++;if(value<min-.015)clearance.errors.push(`${action.name} ${key} frame ${f}: ${value.toFixed(3)} < ${min}`);};
    if(rule.kind==='gap'){const b=node(sc,icon.name+'-'+rule.b);if(!visible(b,t))continue;record(gapBetween(a,b,t),rule.min);}
    else if(rule.kind==='attach'){const b=node(sc,icon.name+'-'+rule.b),pa=curvePoints(a,t),pb=curvePoints(b,t);const touches=p=>{const d=polyDistance(p,pb);return b.type==='stroke'?radius(b,t)-d:(insidePoly(p,pb)?0:-d);};const along=pa.flatMap((p,i)=>i?Array.from({length:12},(_,j)=>[0,1].map(k=>pa[i-1][k]+(p[k]-pa[i-1][k])*j/12)):[p]);record(rule.through?Math.max(...along.map(touches)):touches(pa[rule.end===0?0:pa.length-1]),-.001);}
    else if(rule.kind==='inside'){const b=paintedBox(a,t),[l,tp,r,bt]=rule.box;record(Math.min(b[0]-l,b[1]-tp,r-b[2],bt-b[3]),0);}
   }
  }
 }}
 clearance.result=clearance.errors.length?'FAIL':'PASS';
 fs.writeFileSync(path.join(root,'evidence/clearance-verification.json'),JSON.stringify(clearance,null,2)+'\n');
 if(clearance.errors.length){const first=new Map();for(const e of clearance.errors){const k=e.replace(/ frame \d+:.*/,'');if(!first.has(k))first.set(k,e);}console.log('Clearance errors ('+clearance.errors.length+', first per rule):\n  '+[...first.values()].slice(0,40).join('\n  '));process.exitCode=1;}
 else console.log(`PASS clearance: ${clearance.checks} checks, minimum painted margin ${clearance.minimumPaintedMargin.toFixed(3)}, ${Object.keys(clearance.gaps).length} drawing-specific gaps/joints`);
}
