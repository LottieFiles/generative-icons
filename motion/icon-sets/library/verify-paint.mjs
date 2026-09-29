import {verifyFilledEllipsis} from './verify-filled-ellipsis.mjs';
import {STROKE_WIDTH_TOKEN,REGULAR_WEIGHT} from './paint-tokens.mjs';
// Audit the actual exported paint contract across the current catalog.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
const here=path.dirname(fileURLToPath(import.meta.url));
const root=path.resolve(here,'../../..'), base=path.join(root,'prototype/icons');
const weights={light:1.5,regular:2,bold:2.5};
const openIcons=new Set(['download','upload','refresh','menu','wifi','scan','chart','filter','speech-to-text','motion-generate','code-generate','code-review','model-training','fine-tune','usage-meter','response-stream','summarize','translate','prompt-template','tone-control','response-length','structured-output','embeddings','classification','audio-clean','loading-spinner','progress-ring','offline','orbit-comet','counter-arcs','wave-dots','ripple-rings','corner-relay','infinity-run','bead-relay','equalizer-bars','bead-track','square-snake','sync-arrows','pinwheel','travel-wave','sweep-hand','helix-strands','atom-rings','morph-ring','stack-cycle','transaction-history','bicycle','route','basket','open-package','voice-listening','voice-activity','voice-isolation','replay-10','forward-10','shuffle','repeat','add-to-queue','podcast','live-broadcast','fullscreen','timeline','graph-editor','bezier-handles','motion-path','render-queue','terminal-agent','resume-run','retry-step','execution-trace','parallel-tools','event-trigger','context-compaction','budget-limit','arrow-right','arrow-left','arrow-up','arrow-down','arrow-up-right','arrow-up-left','arrow-down-right','arrow-down-left','chevron-right','chevron-left','chevron-up','chevron-down','arrows-horizontal','arrows-vertical','arrows-swap','arrow-turn-right','arrow-turn-left','arrow-u-turn','arrows-expand','arrows-collapse','annotation-curve-right','annotation-curve-left','annotation-s-curve','annotation-swoop-up','annotation-swoop-down','annotation-loop','annotation-elbow','annotation-bracket','annotation-split','annotation-merge','annotation-double-ended','kanban-board','sprint-cycle','team-roadmap','team-workload','co-edit','mention-teammate','resolve-thread','reopen-thread','version-history','invite-link','team-poll','attach-file','detach-file','cloud-upload','cloud-download','batch-upload','compress-files','add-remove','close','undo-redo','more-options','external-link','cut','move-tool','command-menu','drag-handle','target','power','sign-in-out','fingerprint','face-authentication','signature','barcode','sort-order','group-rows','formula','sum','percentage','line-chart','scatter-plot','bold','italic','underline','strikethrough','heading','paragraph','text-alignment','bulleted-list','numbered-list','indent-outdent','horizontal-rule','text-highlight','clear-formatting','spell-check','find-and-replace','word-wrap','letter-case','line-spacing','hearing-assistance','text-size','focus-indicator','git-branch','git-commit','git-merge','pull-request','repository-fork','merge-conflict','breakpoint','debug-step','regular-expression','webhook','transfer-call','dial-pad','rss-feed']);
const sha=x=>crypto.createHash('sha256').update(x).digest('hex');
const values=p=>p.a ? p.k.flatMap(k=>[k.s,k.e].filter(Boolean)) : [p.k];
let strokePaints=0;const records=[];
for(const ref of JSON.parse(fs.readFileSync(path.join(base,'catalog.json'))).sets){
 const folder=path.dirname(path.join(base,ref));
 for(const icon of JSON.parse(fs.readFileSync(path.join(base,ref))).icons){
  for(const action of icon.animations||icon.motions){
   const bytes=fs.readFileSync(path.join(folder,action.name+'.json')),data=JSON.parse(bytes);
   assert.equal(data.w,32);assert.equal(data.h,32);
   assert.equal(data.slots?.[STROKE_WIDTH_TOKEN]?.p?.k,REGULAR_WEIGHT,action.name+' stroke-width token slot');
   for(const theme of ['dark','light']){const rules=JSON.parse(fs.readFileSync(path.join(folder,`${action.name}.theme-${theme}.json`))).rules;assert.ok(rules.some(r=>r.id===STROKE_WIDTH_TOKEN&&r.type==='Scalar'&&r.value===REGULAR_WEIGHT),action.name+' '+theme+' theme stroke-width rule');}
   let tint=0;const found=new Set();
   function shapes(items){for(const s of items||[]){
    if(s.ty==='gr')shapes(s.it);
    if(s.ty==='tr'&&s.s)for(const scale of values(s.s))assert.ok(scale.every(v=>v===100),'Scaled group changes stroke weight: '+action.name);
    if(['st','fl'].includes(s.ty)&&s.o?.sid==='style.tint')tint++;
    if(s.ty!=='st'||!s.o?.sid?.startsWith('style.outline.'))continue;
    const preset=s.o.sid.split('.').at(-1);assert.ok(preset in weights);
    for(const v of values(s.w))assert.equal(Array.isArray(v)?v[0]:v,weights[preset],action.name+' '+preset+' stroke');
    // Regular strokes carry the stroke-width token so the runtime can render any weight; the authored Light/Bold layers stay fixed.
    assert.equal(s.w.sid,preset==='regular'?STROKE_WIDTH_TOKEN:undefined,action.name+' '+preset+' stroke-width token');
    assert.equal(s.lc,2,action.name+' round cap');assert.equal(s.lj,2,action.name+' round join');
    strokePaints++;found.add(preset);
   }}
   for(const layer of data.layers){
    if(layer.ks?.s)for(const scale of values(layer.ks.s))assert.ok(scale.every(v=>v===100),'Scaled layer changes stroke weight: '+action.name);
    shapes(layer.shapes);
   }
   if(icon.name==='more-options'){assert.equal(found.size,0);verifyFilledEllipsis(data,action.name);}else assert.deepEqual([...found].sort(),Object.keys(weights).sort());
   if(openIcons.has(icon.name))assert.equal(tint,0,icon.name+' has no enclosed surface to tint');
   if(action.duotoneFill!==undefined)assert.equal(action.duotoneFill,tint>0,action.name+' stale Duotone metadata');
   records.push({icon:icon.name,action:action.name,sha256:sha(bytes),duotoneFill:tint>0});
  }
 }
}
const report={result:'PASS',icons:new Set(records.map(r=>r.icon)).size,actions:records.length,strokePaints,weights,strokeWidthToken:STROKE_WIDTH_TOKEN,units:'32-unit icon grid',openIcons:[...openIcons],records};
fs.mkdirSync(path.join(here,'evidence'),{recursive:true});
fs.writeFileSync(path.join(here,'evidence/paint-verification.json'),JSON.stringify(report,null,2)+'\n');
console.log(`PASS ${report.icons} icons / ${report.actions} actions: ${strokePaints} constant outline stroke paints; no scaled layers/groups; open icons have no tint.`);
