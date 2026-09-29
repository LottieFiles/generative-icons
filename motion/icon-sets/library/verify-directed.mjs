import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import crypto from "node:crypto";
import assert from "node:assert/strict";
const root = path.resolve(process.argv[2]);
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
  if(icon.sharedBuilderSha256) assert.equal(hash(fs.readFileSync(path.join(root,"../library/build-set.mjs"))),icon.sharedBuilderSha256,"Stale shared builder");
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
  assert.deepEqual([...strokeWidths].sort(), [1.5, 2, 2.5]);
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
    if(node.visibility.at(-1)[1]>0)for(const k of restPoints)assert.deepEqual(k[1],points.at(-1)[1],icon.name+": moving after rest");
    for(const k of node.visibility.filter(k=>k[0]>=icon.rest/60))assert.equal(k[1],node.visibility.at(-1)[1],icon.name+": visibility moving after rest");
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
  if(!icon.cycleDegrees)for(let i=0;i<light.nodes.length;i++){const a=ends(light)[i],b=ends(strong)[i];assert.equal(a.alphaFirst,b.alphaFirst,icon.name+' initial visibility');assert.equal(a.alphaLast,b.alphaLast,icon.name+' final visibility');if(a.alphaFirst>0)assert.deepEqual(a.first,b.first,icon.name+' initial drawing');if(a.alphaLast>0)assert.deepEqual(a.last,b.last,icon.name+' final drawing');}
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
 const fixed={star:[],sliders:[],folder:['folder-back-frame'],download:[],upload:meta.motionId==='strong'?[]:['upload-tray'],copy:['copy-source'],trash:[],edit:[],send:[],bookmark:[],refresh:[],eye:[],menu:[],home:['home-shell','home-doorway'],calendar:['calendar-frame-frame','calendar-header'],clock:['clock-frame'],camera:['camera-body','camera-lens'],image:['image-frame'],mail:[],message:['message-body'],phone:[],microphone:['microphone-stand','microphone-foot'],volume:['volume-speaker-frame'],wifi:['wifi-source'],cloud:[],sun:['sun-disc-frame'],moon:[],location:[],compass:['compass-frame'],link:[],shield:[],cart:[],settings:[],user:[],users:['users-front-shoulders-frame'],key:[],scan:[],battery:['battery-shell-frame','battery-terminal'],plug:[],bluetooth:['bluetooth-upper','bluetooth-lower'],monitor:['monitor-screen-frame','monitor-stand','monitor-foot'],smartphone:['smartphone-frame-frame','smartphone-speaker'],laptop:['laptop-base','laptop-hinge'],printer:['printer-housing'],document:['document-page','document-fold'],clipboard:['clipboard-board'],book:['book-left-page','book-right-page','book-spine'], 'credit-card':['credit-card-reader'],wallet:['wallet-body'],gift:['gift-box','gift-ribbon','gift-solid-box'],flag:['flag-pole'],chart:[],filter:['filter-tier-0']}[n]||[];
 for(const part of fixed)still(sc,part);
 if(n==='upload'&&meta.motionId==='strong'){
  const tray=pick(sc,'upload-tray'),base=tray.points[0][1];
  for(const[,p]of tray.points){const dy=p[1]-base[1];assert.ok(dy>=-.12003&&dy<=.45003,'Upload tray recoil range');for(let i=0;i<p.length;i+=2){close(p[i],base[i],'Tray fixed x');close(p[i+1]-base[i+1],dy,'Tray rigid recoil');}}
  assert.deepEqual(tray.points.at(-1)[1],base,'Tray returns after launch');
 }
 if(n==='scan')increasing(pick(sc,'scan-bar'),p=>p[1],'scan');
 if(n==='battery')increasing(pick(sc,'battery-charge'),p=>extent(p,0),'charge');
 if(n==='clock'){
  const a=pick(sc,'clock-minute'),b=pick(sc,'clock-hour');assert.ok(a.transform&&b.transform);
  for(let f=0;f<=144;f++)close(sample(a.transform.rotation,f/60),sample(b.transform.rotation,f/60)*12,'Clock 12:1 ratio');
  increasing({points:a.transform.rotation.map(([t,v])=>[t,[v]])},p=>p[0],'clock direction',-1);
 }
 if(n==='plug')for(let f=0;f<=144;f++){const a=sample(pick(sc,'plug-body-frame').points,f/60),b=sample(pick(sc,'plug-cable').points,f/60);close(b[0],16,'Cable attachment x');close(b[1],Math.max(...xy(a,1)),'Cable attachment y');close(b.at(-2),16,'Cable end x');close(b.at(-1),27,'Cable end y');}
 if(n==='laptop')for(const[,p]of pick(sc,'laptop-lid-frame').points)close(Math.max(...xy(p,1)),23,'Laptop hinge');
 if(n==='printer')for(let f=0;f<=144;f++){const a=sample(pick(sc,'printer-sheet').points,f/60),b=sample(pick(sc,'printer-line').points,f/60);close(Math.min(...xy(a,1)),18,'Printer slot');close(Math.max(...xy(a,1))-b[1],2.7,'Ink attached to page');}
 if(n==='gift')for(let f=0;f<=144;f++){const a=sample(pick(sc,'gift-lid-frame').points,f/60);for(const side of ['left','right']){const b=sample(pick(sc,'gift-bow-'+side).points,f/60);close(b[0],16,'Bow knot x');close(b[1],Math.min(...xy(a,1)),'Bow knot y');}}
 if(n==='book'){const a=pick(sc,'book-turning-leaf');assert.ok(a.points.some(k=>k[1][6]<16),'Leaf never crosses spine');for(const[,p]of a.points){close(p[0],16,'Page spine x');close(p[1],8,'Page spine y');}}
 if(n==='chart')for(let i=0;i<3;i++){const a=pick(sc,'chart-bar-'+i+'-frame');increasing(a,p=>extent(p,1),'Chart target');for(const[,p]of a.points){close(Math.max(...xy(p,1)),26,'Chart baseline');close(extent(p,0),0,'Single bar centerline');close(p[0],8+i*8,'Fixed bar column');}}
 if(n==='filter')for(let i=1;i<3;i++)increasing(pick(sc,'filter-tier-'+i),p=>extent(p,0),'Filter order',-1);
 if(n==='keyframes'){
  for(const part of ['keyframes-key-old-frame','keyframes-key-new-frame','keyframes-rail-left'])still(sc,part);
  // Independent measurements of exported filled stroke envelopes, including
  // per-weight junction positions; these checks do not call authored drivers.
  const bands=sc.nodes.filter(p=>/^keyframes-rail-stem-(light|regular|bold|solid)__/.test(p.name));
  for(const band of bands){
   const w=band.name.includes('-light__')?1.5:band.name.includes('-bold__')?2.5:band.name.includes('-solid__')?2.6:2;
   for(let f=0;f<=144;f++){
    const p=sample(band.points,f/60),mid=p.slice(0,36),right=p.slice(36,72),stem=p.slice(74,110);
    close(extent(mid,1),w,'Keyframes middle rail weight');
    close(extent(right,1),w,'Keyframes right rail weight');
    close(extent(stem,0),w,'Keyframes stem weight');
    close((Math.min(...xy(mid,1))+Math.max(...xy(mid,1)))/2,20,'Keyframes rail baseline');
    const tab=sample(pick(sc,'keyframes-tab').points,f/60);
    close(Math.min(...xy(stem,1))+w/2,Math.max(...xy(tab,1)),'Playhead attachment');
    const visible=sample(pick(sc,'keyframes-key-new-frame').visibility,f/60)>0;
    if(visible && w<2.6){
     assert.ok(Math.max(...xy(mid,0))<=18.319,'Rail enters new diamond');
     assert.ok(Math.min(...xy(right,0))>=24.681,'Right rail enters new diamond');
     assert.ok(Math.max(...xy(stem,1))<=16.819,'Stem enters new diamond');
    }
   }
  }
 }

 if(n==='image-upscale')for(let f=0;f<=144;f++){const frame=sample(pick(sc,'image-upscale-frame').points,f/60),arrow=sample(pick(sc,'image-upscale-arrow').points,f/60);assert.ok(Math.min(...xy(arrow,0))-Math.max(...xy(frame,0))>=1.749,'Upscale arrow crosses growing corner in x');assert.ok(Math.min(...xy(frame,1))-Math.max(...xy(arrow,1))>=1.749,'Upscale arrow crosses growing corner in y');}
 if(n==='flag')for(const[,p]of pick(sc,'flag-cloth-frame').points){close(p[0],8,'Flag hoist x');close(p[1],6,'Flag hoist y');}
 if(n==='smartphone')for(const node of sc.nodes.filter(n=>n.name.includes('tile')))for(let f=0;f<=144;f++){if(sample(node.visibility,f/60)>0){const p=sample(node.points,f/60);assert.ok(Math.min(...xy(p,1))>=10.6-3e-5&&Math.max(...xy(p,1))<=24.31,'Phone clipping');}}
 // Typed text advances in discrete character states. Check the complete source
 // cadence instead of requiring extra interpolated motion just to raise a raster count.
 if(['image-caption','code-generate'].includes(n)&&meta.direction==='forward'){
  for(const [part,count]of n==='image-caption'?[['l1',7],['l2',6]]:[['l1',6],['l2',5]]){
   const node=pick(sc,n+'-'+part);const widths=new Set(node.points.map(([,p])=>Math.round(extent(p,0)*1e4)));
   assert.equal(widths.size,count,n+' typed character states '+part);
  }
 }
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
 report.checks.push({name:meta.name,fixedParts:fixed,topology:true,stateBoundaries:true,hold:true,designConstraints:meta.constraints||[]});
}
fs.writeFileSync(path.join(root,'evidence/directed-source-verification.json'),JSON.stringify(report,null,2)+'\n');
console.log(`PASS ${set.id}: ${report.icons} icons, ${report.actions} native actions; provenance, styles, weights, topology, state boundaries, holds and physical constraints.`);
