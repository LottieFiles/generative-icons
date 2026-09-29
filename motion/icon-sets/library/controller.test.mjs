import test from 'node:test';
import {setImmediate as nextTurn} from 'node:timers/promises';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {parseHTML} from 'linkedom';
import {loadCatalog,assetUrl} from '../../../prototype/icons/01/catalog.mjs';
import {IconController} from '../../../prototype/icons/03/controller.mjs';
const preview=new URL('../../../prototype/',import.meta.url),base='http://localhost/icons/01/';
const request=async url=>{const file=new URL(new URL(url).pathname.slice(1),preview);return {ok:fs.existsSync(file),json:async()=>JSON.parse(fs.readFileSync(file)),arrayBuffer:async()=>new ArrayBuffer(8)};};
const catalog=await loadCatalog(new URL('../catalog.json',base).href,request);
const initial=icon=>icon.motions.find(m=>m.motionId==='strong');
async function setup(reduce=false){
 const {document,window}=parseHTML(fs.readFileSync(new URL('icons/01/index.html',preview),'utf8'));
 let onViewport;
 const rects=new Map();window.HTMLElement.prototype.getBoundingClientRect=function(){return rects.get(this)||{top:0,bottom:200,left:0,right:200};};
 class Observer{constructor(callback){onViewport=callback;}observe(){}disconnect(){}}
 const players=[],events={},preference={matches:reduce,addEventListener(n,f){this.change=f;}};
 class Player{
  static setWasmUrl(){}
  constructor(o){this.canvas=o.canvas;this.listeners=new Map();this.slots={};this.currentFrame=0;players.push(this);queueMicrotask(()=>this.emit('load'));}
  addEventListener(n,f){if(!this.listeners.has(n))this.listeners.set(n,new Set());this.listeners.get(n).add(f);}removeEventListener(n,f){this.listeners.get(n)?.delete(f);}emit(n){for(const f of [...(this.listeners.get(n)||[])])f();}
  loadAnimation(id){this.activeAnimationId=id;this.currentFrame=0;}setSpeed(v){this.speed=v;}setLoop(v){this.loop=v;}setSegment(a,b){this.segment=[a,b];}setFrame(v){this.currentFrame=v;}
  resize(){this.resized=true;}setTheme(v){this.theme=v;}setColorSlot(id,v){this.slots[id]=v;}setScalarSlot(id,v){this.slots[id]=v;}
  play(){this.playing=true;}pause(){this.playing=false;}destroy(){this.destroyed=true;this.pause();}
 }
 const source=fs.readFileSync(new URL('icons/01/app.js',preview),'utf8').replace(/^import .*;\n/gm,'').replaceAll('import.meta.url',JSON.stringify(base+'app.js'));
 await vm.runInNewContext(`(async()=>{${source}})()`,{document,IntersectionObserver:Observer,innerHeight:900,innerWidth:1200,DotLottie:Player,IconController,loadCatalog:(url)=>loadCatalog(url,request),assetUrl,matchMedia:()=>preference,devicePixelRatio:2,fetch:request,URL,setTimeout,clearTimeout,console,location:{hash:''},addEventListener:(n,f)=>events[n]=f});
 const fire=(el,type='click',init={})=>el.dispatchEvent(Object.assign(new window.Event(type),init));
 const select=(id,value)=>{const el=document.querySelector('#'+id);Object.defineProperty(el,'value',{configurable:true,value});fire(el,'change');};
 return {document,players,preference,fire,select,events,setVisible:(names)=>{const visible=new Set(names);const entries=[...document.querySelectorAll('.card')].map(target=>{const isIntersecting=visible.has(target.dataset.icon);rects.set(target,{top:isIntersecting?0:2000,bottom:isIntersecting?200:2200,left:0,right:200});return {target,isIntersecting};});onViewport(entries);},card:name=>document.querySelector(`[data-icon="${name}"]`),group:name=>players.filter(p=>p.canvas.closest('.card').dataset.icon===name)};
}
test('the catalog combines all catalog icons in order and resolves every action to its source set',()=>{
 assert.deepEqual(catalog.icons.map(i=>i.name),['heart','bell','search','star','check','sliders','lock','folder','download','upload','copy','trash','edit','send','bookmark','refresh','eye','menu','home','calendar','clock','camera','image','mail','message','phone','microphone','volume','wifi','cloud','sun','moon','location','compass','link','shield','cart','settings','user','users','key','scan','battery','plug','bluetooth','monitor','smartphone','laptop','printer','document','clipboard','book','credit-card','wallet','gift','flag','chart','filter','keyframes','ai-assistant','prompt','system-instructions','context-window','response-stream','autocomplete','rewrite','summarize','translate','brainstorm','reasoning','conversation-branch','prompt-template','tone-control','answer-compare','follow-up','intent-detection','response-length','conversation-memory','structured-output','knowledge-base','semantic-search','citations','embeddings','knowledge-graph','document-chat','source-grounding','retrieve-context','vector-store','dataset','classification','clustering','entity-extraction','fact-check','source-rank','index-data','deduplicate','data-cleaning','context-attach','long-term-memory','image-generate','generative-fill','background-remove','object-select','image-upscale','style-transfer','smart-crop','image-caption','text-to-speech','speech-to-text','voice-chat','audio-generate','audio-clean','music-generate','video-generate','storyboard','motion-generate','code-generate','code-explain','code-review','ai-agent','agent-team','tool-call','agent-workflow','task-plan','agent-handoff','approval-step','guardrails','model-router','model-training','fine-tune','model-evaluation','confidence-score','inference','batch-inference','local-model','model-deploy','model-version','usage-meter','ai-connection','loading-spinner','loading-dots','progress-ring','progress-steps','hourglass','queued','paused','cancelled','error','warning','info','help','no-results','empty-state','offline','reconnect','cloud-sync','saved','notification-dot','session-expired','orbit-comet','counter-arcs','wave-dots','ripple-rings','corner-relay','infinity-run','bead-relay','equalizer-bars','bead-track','planet-orbit','square-snake','sync-arrows','pinwheel','bounce-ball','travel-wave','sweep-hand','helix-strands','atom-rings','morph-ring','stack-cycle','bank','bank-account','bank-transfer','currency-exchange','savings','deposit','withdraw','payment-terminal','contactless-payment','recurring-payment','receipt','invoice','refund','bill-payment','budget','transaction-history','investment','portfolio','calculator','vault','airplane','passport','boarding-pass','suitcase','backpack','train','bus','car-rental','ferry','bicycle','hotel','bed','tent','beach-umbrella','map','route','globe','ticket','luggage-tag','signpost','storefront','shopping-bag','basket','product-tag','product-variants','discount','coupon','loyalty-points','gift-card','subscription-box','checkout','order-confirmed','open-package','packing-tape','return-package','delivery-truck','order-tracking','pickup-locker','warehouse','inventory','voice-listening','voice-speaking','voice-activity','voice-recording','push-to-talk','microphone-muted','wake-word','switch-voice','voice-isolation','voice-speed','play-pause','stop','next-track','previous-track','replay-10','forward-10','shuffle','repeat','playlist','add-to-queue','headphones','podcast','live-broadcast','cast','screen-share','video-call','closed-captions','fullscreen','picture-in-picture','camera-off','timeline','add-keyframe','remove-keyframe','auto-keyframe','keyframe-easing','graph-editor','bezier-handles','motion-path','anchor-point','layer-stack','parent-layers','precompose','trim-layer','split-layer','time-stretch','onion-skin','shape-morph','mask-path','track-matte','render-queue','agent-skills','tool-discovery','mcp-tools','browser-agent','terminal-agent','agent-sandbox','agent-permissions','human-input','agent-checkpoint','resume-run','retry-step','execution-trace','parallel-tools','merge-results','background-run','scheduled-agent','event-trigger','context-compaction','agent-artifact','budget-limit','arrow-right','arrow-left','arrow-up','arrow-down','arrow-up-right','arrow-up-left','arrow-down-right','arrow-down-left','chevron-right','chevron-left','chevron-up','chevron-down','arrows-horizontal','arrows-vertical','arrows-swap','arrow-turn-right','arrow-turn-left','arrow-u-turn','arrows-expand','arrows-collapse','annotation-curve-right','annotation-curve-left','annotation-s-curve','annotation-swoop-up','annotation-swoop-down','annotation-loop','annotation-elbow','annotation-bracket','annotation-callout','annotation-split','annotation-merge','annotation-double-ended','invite-teammate','join-team','team-directory','team-roles','team-presence','assign-task','delegate-task','task-handoff','task-dependency','shared-goal','kanban-board','sprint-cycle','team-roadmap','milestone','team-workload','shared-document','co-edit','live-cursors','follow-collaborator','suggested-edit','add-comment','reply-thread','mention-teammate','resolve-thread','reopen-thread','request-review','approve-change','request-changes','compare-revisions','version-history','share-workspace','invite-link','guest-access','view-access','edit-access','standup','raise-hand','team-poll','decision-log','team-celebration','support-agent','help-center','support-ticket','live-support','callback','file-upload','file-download','attach-file','detach-file','cloud-upload','cloud-download','drag-and-drop','batch-upload','compress-files','extract-files','add-remove','close','undo-redo','more-options','external-link','share','cut','paste','zoom-in-out','select-all','move-tool','grid-list-view','sidebar-toggle','split-view','tab-switcher','command-menu','drag-handle','archive','target','power','sign-in-out','switch-account','id-card','password','passkey','fingerprint','face-authentication','two-factor-code','recovery-codes','private-browsing','cookie-preferences','block-user','checkbox','radio-button','toggle-switch','text-field','number-field','select-field','multiple-choice','signature','form-builder','qr-code','barcode','sort-order','show-columns','resize-column','group-rows','freeze-column','pivot-table','formula','sum','percentage','line-chart','pie-chart','scatter-plot','heatmap','funnel-chart','radar-chart','dashboard','bold','italic','underline','strikethrough','heading','paragraph','text-alignment','bulleted-list','numbered-list','indent-outdent','horizontal-rule','text-color','text-highlight','clear-formatting','spell-check','find-and-replace','page-break','word-wrap','letter-case','line-spacing','accessibility','wheelchair-access','hearing-assistance','high-contrast','reduced-motion','text-size','keyboard','keyboard-shortcuts','focus-indicator','reading-mode','repository','git-branch','git-commit','git-merge','pull-request','repository-fork','merge-conflict','git-stash','debug','breakpoint','debug-step','regular-expression','webhook','server','database','processor','memory-chip','hard-drive','ethernet','router','load-balancer','firewall','usb-drive','inbox','address-book','voicemail','transfer-call','dial-pad','emoji-picker','thumbs-up-down','rss-feed','announcement','date-range','calendar-view','reschedule','alarm-clock','stopwatch','countdown-timer','world-clock','availability','partly-cloudy','rain','snow','hail','thunderstorm','snowflake','wind','fog','tornado','hurricane','rainbow','sunrise','sunset','moon-phase','temperature','humidity','uv-index','umbrella','windsock','weather-radar','heart-rate','step-counter','dumbbell','running','water-intake','sleep-tracking','calories','stethoscope','pill','syringe','first-aid-kit','bandage','weight-scale','blood-pressure','yoga','meditation','nutrition','jump-rope','hospital','dna','smart-bulb','thermostat','smart-lock','door-sensor','garage-door','ceiling-fan','air-conditioner','smart-speaker','video-doorbell','security-camera','window-blinds','robot-vacuum','washing-machine','refrigerator','oven','smoke-alarm','water-leak','power-outlet','energy-usage','motion-sensor','graduation-cap','chalkboard','flashcards','quiz','certificate','notebook','pencil','ruler','lab-flask','microscope','abacus','school-bell','course-progress','lecture','library-shelf','language-learning','alphabet-blocks','homework','exam-grade','geometry-compass','coffee-cup','tea-cup','wine-glass','beer-mug','cocktail','cooking-pot','frying-pan','kettle','toaster','blender','chef-hat','cutlery','pizza','burger','cake','ice-cream','noodle-bowl','bread','recipe-book','restaurant-menu','new-folder','folder-open','move-file','rename-file','duplicate-file','restore-file','approve-file','lock-file','file-search','file-preview','file-tag','file-star','file-sync','import-file','export-file','image-file','code-file','spreadsheet-file','audio-file','video-file','bell-ring','bell-off','bell-snooze','notification-badge','toast-message','banner-alert','push-notification','inbox-zero','mark-all-read','notification-stack','mention-alert','priority-alert','do-not-disturb','reminder-ping','daily-digest','notification-settings','alert-dismiss','alert-expand','sound-alert','vibrate-alert','search-suggest','recent-searches','search-clear','search-voice','search-image','search-filter','sort-ascending','sort-descending','hashtag','tag-add','tag-remove','trending-up','trending-down','telescope','related-items','saved-search','web-search','search-match','search-scope','searching','pin-cluster','current-location','directions','turn-by-turn','route-recalculate','traffic-light','map-layers','satellite','street-view','geofence','distance','elevation','parking','fuel-station','ev-charger','transit-stop','walking-route','roundabout','offline-map','toll-gate','flash-toggle','self-timer','switch-camera','tripod','aperture','portrait-mode','live-photo','burst-mode','panorama','exposure','photo-gallery','photo-album','photo-edit','rotate-photo','flip-photo','brightness','saturation','red-eye-fix','photo-delete','photo-favorite','music-note','vinyl-record','cassette-tape','audio-waveform','loudspeaker','radio','metronome','tuner','piano-keys','guitar','drum','lyrics','music-album','earbuds','mute-audio','crossfade','equalizer-preset','audio-jack','spatial-audio','tuning-fork','pen-tool','brush','eraser','corner-radius','line-tool','rectangle-tool','ellipse-tool','polygon-tool','fill-bucket','eyedropper','group-objects','align-objects','magic-wand','lasso-select','transform-tool','rotate-tool','mirror-tool','snap-to-grid','ruler-guides','scissors','rsvp','event-invite','venue','seat-map','agenda','event-speaker','event-badge','check-in','add-to-calendar','event-reminder','ticket-scan','ticket-transfer','waitlist','sold-out','early-bird','event-live','event-replay','sponsor','networking','event-map','car','electric-car','fuel-pump','speedometer','parking-brake','tire-pressure','engine','seatbelt','headlights','wiper','steering-wheel','car-key','car-lock','car-battery','gear-shift','car-navigation','car-wash','engine-oil','car-window','trunk','gavel','justice-scale','contract','rubber-stamp','official-seal','courthouse','gdpr-shield','audit','policy','terms-of-service','compliance-check','redact','consent','cookie-banner','data-retention','disclaimer','copyright','whistleblower','evidence-bag','legal-hold','language','region','currency','appearance','dark-mode','quiet-hours','privacy-shield','shortcut-key','mouse','auto-rotate','sound-toggle','battery-saver','mobile-data','haptic-feedback','reset-settings','auto-update','startup-apps','pointer-speed','advanced-settings','trackpad']);
 assert.equal(new Set(catalog.icons.map(i=>i.number)).size,catalog.icons.length);
 for(const icon of catalog.icons)for(const motion of icon.motions)for(const meta of [motion,motion.reverse].filter(Boolean)){
  const url=new URL(assetUrl(icon,meta));assert.ok(fs.existsSync(new URL(url.pathname.slice(1),preview)),`Missing ${url}`);
 }
});
test('all catalog cards load still, with one preview and independent motion downloads',async()=>{
 const e=await setup();assert.equal(e.players.length,catalog.icons.length);assert.ok(e.players.every(p=>!p.playing));assert.equal(e.document.querySelector('#sequence').disabled,false);
 assert.equal(e.document.querySelectorAll('input[type="range"]').length,0);
 for(const icon of catalog.icons){
  const card=e.card(icon.name),p=e.group(icon.name)[0];
  assert.equal(card.dataset.motion,'strong');assert.equal(p.activeAnimationId,initial(icon).name);assert.equal(p.currentFrame,initial(icon).rest);
  assert.equal(card.querySelector('[data-motion="strong"]').getAttribute('aria-pressed'),'true');
  assert.equal(card.querySelector('[data-motion="light"]').getAttribute('aria-pressed'),'false');
  assert.equal(card.querySelector('.download').href,assetUrl(icon,initial(icon)));
 }

 const ids=[...e.document.querySelectorAll('[id]')].map(el=>el.id);assert.equal(new Set(ids).size,ids.length,'Duplicate ID can break an icon anchor or SVG control');
 e.select('speed','0.5');e.select('weight','2.5');e.fire(e.document.querySelector('[data-style="duotone"]'));
 for(const name of ['heart','search','check','lock']){
  e.fire(e.card(name).querySelector('[data-motion="strong"]'));
  assert.ok(e.group(name).every(p=>p.activeAnimationId.endsWith('-strong')&&p.speed===.5&&p.slots['style.tint']===18&&p.slots['style.outline.bold']===100));
  assert.match(e.card(name).querySelector('.download').href,new RegExp(`${name}-strong\\.lottie`));
 }
 assert.equal(e.group('bell')[0].activeAnimationId,'bell-strong');assert.match(e.card('search').querySelector('.download').href,/\/02\/assets\//);
 e.events.pagehide({persisted:false});
});
test('hover repeats the card, coalesces nested focus/clicks, and finishes on leave',async()=>{
 const e=await setup(),card=e.card('heart'),p=e.group('heart')[0];e.fire(card,'pointerenter');assert.equal(p.playing,true);p.setFrame(9);
 e.fire(card,'focusin');e.fire(card.querySelector('.stage'));assert.equal(p.currentFrame,9);
 e.fire(e.card('bell'),'pointerenter');assert.equal(e.group('bell')[0].playing,true);assert.equal(p.currentFrame,9);
 p.emit('complete');await nextTurn();assert.equal(p.playing,true);assert.equal(p.currentFrame,0);
 p.setFrame(12);e.fire(card,'pointerleave');assert.equal(p.playing,true);assert.equal(p.currentFrame,12);p.emit('complete');await nextTurn();assert.equal(p.playing,false);
 e.fire(card,'pointerenter');assert.equal(p.playing,true);assert.equal(p.currentFrame,0);e.events.pagehide({persisted:false});
});
test('Stateful icons hover through both directions and return to their original state',async()=>{
 const e=await setup();
 for(const name of catalog.icons.filter(i=>i.stateful).map(i=>i.name)){
  const card=e.card(name),icon=catalog.icons.find(i=>i.name===name),p=e.group(name)[0];e.fire(card,'pointerenter');
  assert.equal(p.activeAnimationId,initial(icon).reverse.name);assert.equal(p.segment[1],Math.ceil((initial(icon).reverse.active+.18)*icon.fps));
  assert.match(card.querySelector('.download').href,new RegExp(`/${name}-strong\\.lottie`));p.emit('complete');await nextTurn();
  assert.equal(p.playing,true);assert.equal(p.activeAnimationId,initial(icon).name);e.fire(card,'pointerleave');p.emit('complete');await nextTurn();
  assert.equal(p.playing,false);assert.equal(card.dataset.state,icon.endState);assert.equal(p.currentFrame,initial(icon).rest);
 }
});
test('Light and Strong changes keep the hovered card looping without stale restarts',async()=>{
 const e=await setup(),card=e.card('heart'),p=e.group('heart')[0];e.fire(card,'pointerenter');
 const stale=[...p.listeners.get('complete')][0];p.emit('complete');
 e.fire(card.querySelector('[data-motion="light"]'));p.setFrame(8);stale();await nextTurn();
 assert.equal(p.activeAnimationId,'heart');assert.equal(p.currentFrame,8);assert.equal(p.playing,true);
 for(const profile of ['strong','light','strong']){
  e.fire(card.querySelector(`[data-motion="${profile}"]`));p.emit('complete');await nextTurn();
  assert.equal(p.playing,true);assert.equal(p.activeAnimationId,profile==='light'?'heart':'heart-strong');assert.equal(p.currentFrame,0);
 }
 assert.equal(e.group('bell')[0].playing,false);e.fire(card,'pointerleave');p.emit('complete');await nextTurn();assert.equal(p.playing,false);
});
test('stateful hover repeats full cycles and completes the return after pointer exit',async()=>{
 for(const name of catalog.icons.filter(i=>i.stateful).map(i=>i.name)){
  const e=await setup(),card=e.card(name),icon=catalog.icons.find(i=>i.name===name),p=e.group(name)[0];e.fire(card,'pointerenter');
  for(let cycle=0;cycle<2;cycle++){
   assert.equal(p.activeAnimationId,initial(icon).reverse.name);p.emit('complete');await nextTurn();
   assert.equal(p.activeAnimationId,initial(icon).name);assert.equal(p.segment[1],Math.ceil((initial(icon).active+.45)*icon.fps));p.emit('complete');await nextTurn();
  }
  e.fire(card,'pointerleave');p.emit('complete');await nextTurn();assert.equal(p.playing,true);assert.equal(p.activeAnimationId,initial(icon).name);
  p.emit('complete');await nextTurn();assert.equal(p.playing,false);assert.equal(card.dataset.state,icon.endState);
 }
});
test('mouse focus does not latch hover playback; keyboard focus loops until it leaves',async()=>{
 const e=await setup(),card=e.card('heart'),p=e.group('heart')[0];
 e.fire(card,'pointerenter');e.fire(e.document,'pointerdown');e.fire(card,'focusin');e.fire(card.querySelector('[data-motion="strong"]'));
 e.fire(card,'pointerleave');p.emit('complete');await nextTurn();assert.equal(p.playing,false);
 e.fire(e.document,'keydown',{key:'Tab'});e.fire(card,'focusin');p.emit('complete');await nextTurn();assert.equal(p.playing,true);
 e.fire(card,'focusout',{relatedTarget:card.querySelector('[data-motion="light"]')});p.emit('complete');await nextTurn();assert.equal(p.playing,true);
 e.fire(card,'focusout',{relatedTarget:e.document.querySelector('#loop')});p.emit('complete');await nextTurn();assert.equal(p.playing,false);
});
test('touch previews stay one-shot and pointer re-entry adopts an in-progress preview',async()=>{
 const e=await setup(),card=e.card('heart'),p=e.group('heart')[0];e.fire(card,'pointerenter',{pointerType:'touch'});assert.equal(p.playing,false);
 e.fire(e.document,'pointerdown',{pointerType:'touch'});e.fire(card,'pointerdown',{pointerType:'touch'});e.fire(card,'focusin');e.fire(card.querySelector('.stage'));
 p.emit('complete');await nextTurn();assert.equal(p.playing,false);
 e.fire(card.querySelector('.stage'));p.setFrame(7);e.fire(card,'pointerenter');assert.equal(p.currentFrame,7);
 e.fire(card,'pointerleave');e.fire(card,'pointerenter');assert.equal(p.currentFrame,7);p.emit('complete');await nextTurn();assert.equal(p.playing,true);assert.equal(p.currentFrame,0);
 e.events.pagehide({persisted:false});
});
test('keyboard focus previews the whole card and reduced motion keeps every icon still',async()=>{
 const active=await setup();active.fire(active.card('search'),'focusin');assert.equal(active.group('search')[0].playing,true);active.events.pagehide({persisted:false});
 const e=await setup(true);
 for(const icon of catalog.icons){e.fire(e.card(icon.name),'pointerenter');e.fire(e.card(icon.name).querySelector('.stage'));e.fire(e.card(icon.name).querySelector('[data-motion="strong"]'));}
 assert.ok(e.players.every(p=>!p.playing));assert.equal(e.document.querySelector('#loop').disabled,true);assert.ok([...e.document.querySelectorAll('.stage')].every(b=>b.disabled));
 for(const name of catalog.icons.filter(i=>i.stateful).map(i=>i.name))assert.equal(e.card(name).dataset.state,catalog.icons.find(i=>i.name===name).endState);
 e.preference.matches=false;e.preference.change();
 assert.equal(e.document.querySelector('#loop').disabled,false);assert.ok([...e.document.querySelectorAll('.stage')].every(b=>!b.disabled),'Turning reduced motion off restores every preview button');
 assert.ok(e.players.every(p=>!p.playing),'Restoring motion does not start playback automatically');
 e.fire(e.document,'keydown');e.fire(e.card('chevron-right'),'focusin');assert.equal(e.group('chevron-right')[0].playing,true);e.events.pagehide({persisted:false});
});
test('Customize preserves colors, native styles, size selection and still poses',async()=>{
 const e=await setup(),frames=e.players.map(p=>p.currentFrame);e.select('color','aqua');e.fire(e.document.querySelector('#theme'));e.fire(e.document.querySelector('[data-style="solid"]'));e.select('preview-size','32');
 assert.ok(e.players.every((p,i)=>p.currentFrame===frames[i]&&!p.playing&&p.theme==='light'&&p.slots['style.solid']===100&&p.resized));
 assert.equal(e.document.documentElement.style.getPropertyValue('--preview-size'),'32px');assert.equal(e.document.querySelector('#weight').disabled,true);
 assert.deepEqual(Array.from(e.players[0].slots['color.primary']),[8/255,125/255,135/255,1]);e.events.pagehide({persisted:false});assert.ok(e.players.every(p=>p.destroyed));
});
test('Loop all starts all catalog icons and repeats each independently with stable downloads',async()=>{
 const e=await setup();e.fire(e.document.querySelector('#loop'));assert.ok(e.players.every(p=>p.playing&&p.loop===Boolean(catalog.icons.find(i=>e.group(i.name).includes(p)).continuous)));assert.equal(e.document.querySelector('#loop span').textContent,'Stop loop');
 const downloads=[...e.document.querySelectorAll('.download')].map(a=>a.href);
 for(const name of catalog.icons.filter(i=>!i.stateful).map(i=>i.name))assert.equal(e.group(name)[0].segment[1],143);
 e.group('bell')[0].setFrame(23);e.group('heart')[0].emit('complete');await nextTurn();assert.equal(e.group('heart')[0].currentFrame,0);assert.equal(e.group('bell')[0].currentFrame,23);
 for(let cycle=0;cycle<4;cycle++){
  for(const name of catalog.icons.filter(i=>i.stateful).map(i=>i.name)){
   const icon=catalog.icons.find(i=>i.name===name),meta=cycle%2===0?initial(icon).reverse:initial(icon),p=e.group(name)[0];
   assert.equal(p.activeAnimationId,meta.name);assert.equal(p.segment[1],Math.ceil((meta.active+.5)*meta.fps));p.emit('complete');
  }await nextTurn();
 }
 assert.deepEqual([...e.document.querySelectorAll('.download')].map(a=>a.href),downloads);
 e.fire(e.document.querySelector('#loop'));await nextTurn();assert.ok(e.players.every(p=>!p.playing));assert.equal(e.document.querySelector('#loop span').textContent,'Loop all');
});
test('motion changes during loops affect only that card and cancel stale work',async()=>{
 const e=await setup();e.fire(e.document.querySelector('#loop'));e.group('bell')[0].setFrame(23);
 const stale=[...e.group('heart')[0].listeners.get('complete')][0];e.group('heart')[0].emit('complete');
 e.fire(e.card('heart').querySelector('[data-motion="light"]'));e.group('heart')[0].setFrame(7);stale();await nextTurn();
 assert.equal(e.group('heart')[0].activeAnimationId,'heart');assert.equal(e.group('heart')[0].currentFrame,7);assert.equal(e.group('bell')[0].currentFrame,23);
 e.fire(e.card('check').querySelector('[data-motion="light"]'));assert.equal(e.group('check')[0].activeAnimationId,'check');assert.ok(e.players.every(p=>p.playing));e.events.pagehide({persisted:false});
});
test('stop, reduced motion, and cleanup cancel pending loops and hover return actions',async()=>{
 for(const mode of ['loop','hover'])for(const action of ['stop','reduced','cleanup']){
  const e=await setup();if(mode==='loop')e.fire(e.document.querySelector('#loop'));else e.fire(e.card('lock'),'pointerenter');
  for(const p of e.players)if(p.playing)p.emit('complete');
  if(action==='stop'){e.fire(e.document.querySelector('#loop'));if(mode==='hover')e.fire(e.document.querySelector('#loop'));}
  if(action==='reduced'){e.preference.matches=true;e.preference.change();}
  if(action==='cleanup')e.events.pagehide({persisted:false});
  await nextTurn();assert.ok(e.players.every(p=>!p.playing));assert.equal(e.document.querySelector('#loop').getAttribute('aria-pressed'),'false');
 }
});
test('Play sequence still reviews all icons one at a time, including both state directions',async()=>{
 const e=await setup();e.fire(e.document.querySelector('#loop'));e.fire(e.document.querySelector('#sequence'));
 for(const icon of catalog.icons){
  assert.equal(e.players.filter(p=>p.playing).length,1);const p=e.group(icon.name)[0];assert.equal(p.playing,true);p.emit('complete');await nextTurn();
  if(icon.stateful){assert.equal(p.playing,true);p.emit('complete');await nextTurn();}
 }
 assert.ok(e.players.every(p=>!p.playing));assert.equal(e.document.querySelector('#loop').getAttribute('aria-pressed'),'false');
});
test('stopping preview resets retains the completed output, including reduced motion',async()=>{
 for(const stop of ['loop','reduced'])for(const phase of ['running','offscreen-reset-complete']){
  const e=await setup(),icons=catalog.icons.filter(i=>i.previewReset);
  e.setVisible(icons.map(i=>i.name));e.fire(e.document.querySelector('#loop'));
  for(const icon of icons){const p=e.group(icon.name)[0];assert.equal(p.activeAnimationId,initial(icon).reverse.name);}
  if(phase==='offscreen-reset-complete'){e.setVisible([]);for(const icon of icons)e.group(icon.name)[0].emit('complete');await nextTurn();}
  if(stop==='loop')e.fire(e.document.querySelector('#loop'));else{e.preference.matches=true;e.preference.change();}
  await nextTurn();
  for(const icon of icons){const p=e.group(icon.name)[0],m=initial(icon);assert.equal(p.activeAnimationId,m.name,icon.name+' retains the output');assert.equal(p.currentFrame,m.rest);assert.equal(p.playing,false);}
  e.events.pagehide({persisted:false});
 }
});
test('every new icon keeps its selected Light or Strong action looping with a matching download',async()=>{
 const e=await setup();
 for(const icon of catalog.icons.slice(8)){
  const card=e.card(icon.name),p=e.group(icon.name)[0];e.fire(card,'pointerenter');
  for(const profile of ['strong','light']){
   e.fire(card.querySelector(`[data-motion="${profile}"]`));const motion=icon.motions.find(m=>m.motionId===profile);
   for(let part=0;part<(icon.stateful?4:2);part++){assert.ok([motion.name,motion.reverse?.name].includes(p.activeAnimationId));assert.equal(p.playing,true);p.emit('complete');await nextTurn();}
   assert.match(card.querySelector('.download').href,new RegExp(`/${motion.name}\\.lottie`));assert.equal(card.dataset.motion,profile);
  }
  e.fire(card,'pointerleave');p.emit('complete');await nextTurn();if(p.playing){p.emit('complete');await nextTurn();}assert.equal(p.playing,false);
 }
 e.events.pagehide({persisted:false});
});

test('category links reach labelled sections and every icon appears once in its category',async()=>{
 const e=await setup();
 assert.deepEqual(catalog.categories.map(c=>c.icons.length),[8,10,20,21,20,20,20,20,20,20,20,20,20,10,20,20,20,20,12,40,5,10,20,12,11,16,20,10,13,10,9,8,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20,20]);
 const links=[...e.document.querySelectorAll('#category-links a')];assert.equal(links.length,catalog.categories.length);
 const seen=[];
 for(const [i,category]of catalog.categories.entries()){
  const section=e.document.getElementById(links[i].getAttribute('href').slice(1));assert.ok(section);
  assert.equal(e.document.getElementById(section.getAttribute('aria-labelledby')).textContent,category.label);
  assert.equal(section.querySelector('.category-count').textContent,category.icons.length+' icons');
  const names=[...section.querySelectorAll('.card')].map(c=>c.dataset.icon);assert.deepEqual(names,category.icons.map(i=>i.name));seen.push(...names);
  assert.ok(section.querySelector('.card h3'),'Card headings are below category headings');
 }
 assert.deepEqual(seen,catalog.icons.map(i=>i.name));e.events.pagehide({persisted:false});
});


test('continuous loaders use native repeats without reloads, finish on exit, and switch safely',async()=>{
 const e=await setup(),loaders=catalog.icons.filter(i=>i.continuous);assert.equal(loaders.length,27);
 for(const icon of loaders){
  const card=e.card(icon.name),p=e.group(icon.name)[0];e.fire(card,'pointerenter');
  assert.equal(p.loop,true);assert.deepEqual(p.segment,[0,143]);
  const stale=[...p.listeners.get('loop')][0];p.setFrame(1);p.emit('loop');await nextTurn();
  assert.equal(p.currentFrame,1,'Native loop must not reload at a boundary');assert.equal(p.playing,true);
  e.fire(card.querySelector('[data-motion="light"]'));p.setFrame(8);stale();await nextTurn();
  assert.equal(p.activeAnimationId,icon.name);assert.equal(p.currentFrame,8);assert.equal(p.loop,true);
  e.fire(card,'pointerleave');assert.equal(p.playing,true);p.emit('loop');await nextTurn();
  assert.equal(p.playing,false);assert.equal(p.loop,false);assert.equal(p.currentFrame,0);
  e.fire(e.document,'pointerdown',{pointerType:'touch'});e.fire(card,'pointerdown',{pointerType:'touch'});e.fire(card.querySelector('.stage'));
  assert.equal(p.playing,true);assert.equal(p.loop,false);p.emit('complete');await nextTurn();assert.equal(p.playing,false);
 }
 e.fire(e.document.querySelector('#loop'));assert.ok(loaders.every(i=>e.group(i.name)[0].loop));
 e.preference.matches=true;e.preference.change();await nextTurn();assert.ok(e.players.every(p=>!p.playing));
 e.events.pagehide({persisted:false});
});


test('Loop all defers offscreen cards, resumes their clocks on entry, and stops pending work',async()=>{
 const e=await setup();e.setVisible(['heart','bell']);e.fire(e.document.querySelector('#loop'));
 assert.deepEqual(e.players.filter(p=>p.playing).map(p=>p.canvas.closest('.card').dataset.icon),['heart','bell']);
 assert.equal(e.card('delivery-truck').dataset.renderSuspended,'true');
 const heart=e.group('heart')[0];heart.setFrame(17);
 e.setVisible(['delivery-truck','return-package']);
 assert.equal(heart.playing,false);assert.equal(heart.currentFrame,17);
 assert.equal(e.group('delivery-truck')[0].playing,true);assert.equal(e.group('return-package')[0].playing,true);
 e.setVisible(['heart']);assert.equal(heart.playing,true);assert.equal(heart.currentFrame,17,'resume the suspended phase');
 e.fire(e.document.querySelector('#loop'));await nextTurn();e.setVisible(['delivery-truck']);
 assert.ok(e.players.every(p=>!p.playing),'offscreen entry must not restart a stopped global loop');
 e.events.pagehide({persisted:false});
});

test('a queued completion cannot restart an offscreen or hidden clock',async()=>{
 const e=await setup();e.setVisible(['heart']);e.fire(e.document.querySelector('#loop'));
 const p=e.group('heart')[0];e.setVisible([]);p.emit('complete');await nextTurn();
 assert.ok(e.players.every(p=>!p.playing));e.setVisible(['heart']);assert.equal(p.playing,true);
 Object.defineProperty(e.document,'hidden',{configurable:true,value:true});e.fire(e.document,'visibilitychange');
 assert.equal(p.playing,false);p.emit('complete');await nextTurn();assert.ok(e.players.every(p=>!p.playing));
 Object.defineProperty(e.document,'hidden',{configurable:true,value:false});e.fire(e.document,'visibilitychange');
 assert.equal(p.playing,true);assert.equal(e.group('bell')[0].playing,false);
 e.events.pagehide({persisted:false});
});
