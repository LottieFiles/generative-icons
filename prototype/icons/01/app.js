import {DotLottie} from '../../vendor/dotlottie-web.js';
import {IconController} from '../03/controller.mjs?v=annotation-preview-stop-1';
import {loadCatalog,assetUrl} from './catalog.mjs?v=categories-1';
DotLottie.setWasmUrl(new URL('../../vendor/dotlottie-player.wasm',import.meta.url).href);
const reduced=matchMedia('(prefers-reduced-motion: reduce)'),cards=[],defaultMotion='strong';
const hint=document.querySelector('#hint'),sequenceButton=document.querySelector('#sequence'),loopButton=document.querySelector('#loop'),settings=document.querySelector('#preview-settings');
let theme='dark',style='outline',weight=2,speed=1,sequence=0,looping=false,sequencing=false,swatch='teal',keyboardInput=true;
// Keep loop intent for the whole library, but run native clocks only for
// visible cards. Sequence review uses one player at a time and can run offscreen.
const inViewport=root=>{const r=root.getBoundingClientRect();return r.bottom>0&&r.top<innerHeight&&r.right>0&&r.left<innerWidth;};
function syncPlaybackVisibility(card){
 const c=card.controller;if(!c)return;
 const suspended=Boolean(document.hidden||(!card.visible&&!sequencing));
 card.root.dataset.inViewport=String(card.visible);card.root.dataset.renderSuspended=String(suspended);c.suspend(suspended);
 if(looping&&card.visible&&!document.hidden&&!c.running)repeatCard(card,sequence);
}
const viewportObserver=new IntersectionObserver(entries=>{
 for(const entry of entries){const card=cards.find(c=>c.root===entry.target);if(card){card.visible=entry.isIntersecting;syncPlaybackVisibility(card);}}
});
const palette={dark:{teal:'#00DDB3',aqua:'#59E4E8',mint:'#B7F5E6'},light:{teal:'#008C76',aqua:'#087D87',mint:'#247866'}};
function paint(p){
 p.setTheme(theme);p.setColorSlot('color.primary',[1,3,5].map(i=>parseInt(palette[theme][swatch].slice(i,i+2),16)/255).concat(1));
 for(const [name,w]of [['light',1.5],['regular',2],['bold',2.5]])p.setScalarSlot('style.outline.'+name,style!=='solid'&&weight===w?100:0);
 p.setScalarSlot('style.solid',style==='solid'?100:0);p.setScalarSlot('style.tint',style==='duotone'?18:0);
}
async function load(canvas,data){
 const p=new DotLottie({canvas,data,autoplay:false,loop:false,useFrameInterpolation:true,renderConfig:{devicePixelRatio:Math.min(devicePixelRatio||1,2),autoResize:true,freezeOnOffscreen:false}});
 await new Promise((resolve,reject)=>{
  const timer=setTimeout(()=>{p.destroy();reject(new Error('Lottie load timeout'));},15000);
  p.addEventListener('load',()=>{clearTimeout(timer);resolve();});
  p.addEventListener('loadError',()=>{clearTimeout(timer);p.destroy();reject(new Error('Lottie load error'));});
 });
 // Expose native playback state for the gallery's browser checks without
 // coupling preview intent to whether a canvas is currently on screen.
 canvas.dataset.renderFrozen=String(p.isFrozen);
 p.addEventListener('freeze',()=>{canvas.dataset.renderFrozen='true';});
 p.addEventListener('unfreeze',()=>{canvas.dataset.renderFrozen='false';});
 if(new URL(location.href||import.meta.url).searchParams.has('verifyPlayback')){
  let frames=0;
  p.addEventListener('frame',e=>{if(++frames%8===0){canvas.dataset.renderFrame=String(e.currentFrame);canvas.dataset.renderEvents=String(frames);canvas.dataset.renderFrozen=String(p.isFrozen);}});
 }
 return p;
}
function updateHint(){
 hint.textContent=reduced.matches?'Reduced motion is on. Icons stay still.':looping?'All icons are looping.':sequencing?'Playing icons in sequence.':'Hover a card to loop. Move away to finish the current cycle.';
 sequenceButton.disabled=reduced.matches||!cards.length||cards.some(c=>!c.controller);loopButton.disabled=sequenceButton.disabled;
 loopButton.setAttribute('aria-pressed',String(looping));loopButton.querySelector('span').textContent=looping?'Stop loop':'Loop all';
}
function stopPlayback(){
 sequence++;looping=false;sequencing=false;
 for(const card of cards){invalidateHover(card);const c=card.controller;if(c&&(c.running||c.icon.previewReset))c.stopPreview();}
 updateHint();
}
const hoverActive=card=>card.hovered||card.keyboardFocused;
function invalidateHover(card){card.hoverEpoch=(card.hoverEpoch||0)+1;card.hoverRunning=false;}
function previewOnce(card){return card.previewTask=card.controller.preview();}
async function repeatHover(card){
 const c=card.controller;
 if(card.hoverRunning||!hoverActive(card)||looping||sequencing||reduced.matches)return;
 const epoch=card.hoverEpoch=(card.hoverEpoch||0)+1;card.hoverRunning=true;
 try{
  while(card.hoverEpoch===epoch&&hoverActive(card)&&!looping&&!sequencing&&!reduced.matches){
   // Adopt an in-progress click preview rather than restart it on entry.
   const done=c.running&&card.previewTask?card.previewTask:(card.previewTask=c.preview({hold:.45,repeat:()=>card.hoverEpoch===epoch&&hoverActive(card)&&!looping&&!sequencing&&!reduced.matches}));
   if(!await done)return;
  }
 }finally{if(card.hoverEpoch===epoch)card.hoverRunning=false;}
}
function update(card,c){
 const root=card.root;root.dataset.playing=String(c.running);root.dataset.state=c.state;root.dataset.motion=c.profile;root.setAttribute('aria-busy','false');
 for(const b of root.querySelectorAll('[data-motion]'))b.setAttribute('aria-pressed',String(b.dataset.motion===c.profile));
 root.querySelector('.stage').disabled=reduced.matches;
 const selected=c.icon.motions.find(m=>m.motionId===c.profile),download=root.querySelector('.download');
 download.href=assetUrl(c.icon,selected);download.setAttribute('aria-label',`Download ${c.icon.label}, ${c.profile} motion`);download.title=`Download ${c.icon.label} · ${selected.motionLabel}`;
}
async function mount(icon,grid){
 const root=document.createElement('article');root.className='card';root.dataset.icon=icon.name;root.id=icon.name;root.setAttribute('aria-labelledby',`${icon.name}-name`);root.setAttribute('aria-busy','true');
 root.innerHTML=`<div class="card-head"><h3 id="${icon.name}-name">${icon.label}</h3><a class="download" download aria-label="Download ${icon.label}"><svg aria-hidden="true"><use href="#ui-download"/></svg></a></div><button disabled class="stage" aria-label="Preview ${icon.label}" aria-describedby="preview-help"><canvas width="128" height="128" aria-hidden="true"></canvas></button><div class="motion" role="group" aria-label="${icon.label} motion"><button disabled data-motion="strong" aria-label="Strong motion" aria-pressed="true">Strong</button><button disabled data-motion="light" aria-label="Light motion" aria-pressed="false">Light</button></div><p class="card-error" hidden>Could not load this icon.</p>`;
 grid.append(root);const card={root,icon,visible:inViewport(root)};cards.push(card);viewportObserver.observe(root);
 try{
  const response=await fetch(assetUrl(icon,icon.motions.find(m=>m.motionId===defaultMotion)),{cache:'no-store'});if(!response.ok)throw new Error('Asset load failed');
  const player=await load(root.querySelector('canvas'),await response.arrayBuffer());
  const controller=new IconController(icon,[player],{paint,reduced:reduced.matches,profile:defaultMotion,change:c=>update(card,c)});card.controller=controller;controller.setSpeed(speed);syncPlaybackVisibility(card);
  for(const b of root.querySelectorAll('[data-motion]'))b.disabled=false;controller.notify();
  const replay=()=>{if(looping||sequencing)stopPlayback();if(hoverActive(card))repeatHover(card);else if(!controller.running)previewOnce(card);};
  // The whole card is the hover target. Nested controls retain their own actions.
  root.addEventListener('pointerenter',e=>{if(e.pointerType==='touch')return;keyboardInput=false;card.keyboardFocused=false;card.hovered=true;repeatHover(card);});
  root.addEventListener('pointerleave',()=>{card.hovered=false;});
  root.addEventListener('pointerdown',e=>{card.keyboardFocused=false;if(e.pointerType==='touch')card.hovered=false;});
  root.addEventListener('focusin',()=>{card.keyboardFocused=keyboardInput;if(card.keyboardFocused)repeatHover(card);});
  root.addEventListener('focusout',e=>{if(!root.contains(e.relatedTarget))card.keyboardFocused=false;});
  root.querySelector('.stage').addEventListener('click',replay);
  root.addEventListener('click',e=>{if(!e.target.closest('button,a'))replay();});
  for(const b of root.querySelectorAll('[data-motion]'))b.addEventListener('click',()=>{
   if(b.dataset.motion===controller.profile)return;
   if(looping){controller.setMotion(b.dataset.motion);repeatCard(card,sequence);return;}
   if(sequencing)stopPlayback();invalidateHover(card);controller.setMotion(b.dataset.motion);if(hoverActive(card))repeatHover(card);else previewOnce(card);
  });
 }catch(e){root.setAttribute('aria-busy','false');root.classList.add('failed');root.querySelector('.card-error').hidden=false;for(const b of root.querySelectorAll('button'))b.disabled=true;console.error(e);}
}
async function runSequence(){
 stopPlayback();if(reduced.matches)return;const token=sequence;sequencing=true;updateHint();
 for(const card of cards){if(sequence!==token||reduced.matches)return;if(card.controller){syncPlaybackVisibility(card);await card.controller.preview({force:true});}}
 if(sequence===token){sequencing=false;updateHint();}
}
async function repeatCard(card,token){
 if(!card.visible||document.hidden)return;
 const c=card.controller,cycle=card.loopEpoch=(card.loopEpoch||0)+1;
 let direction=c.icon.stateful&&c.state===c.icon.endState?'reverse':'forward';
 while(looping&&card.visible&&!document.hidden&&sequence===token&&card.loopEpoch===cycle&&!reduced.matches){
  const completed=await c.play(direction,{force:true,showcase:true,hold:.5,repeat:()=>looping&&sequence===token&&card.loopEpoch===cycle&&!reduced.matches});if(!completed)return;
  if(c.icon.stateful)direction=direction==='forward'?'reverse':'forward';
 }
}
function runLoop(){stopPlayback();if(reduced.matches)return;looping=true;updateHint();for(const card of cards)if(card.controller){card.visible=inViewport(card.root);syncPlaybackVisibility(card);}}
sequenceButton.addEventListener('click',runSequence);loopButton.addEventListener('click',()=>looping?stopPlayback():runLoop());
for(const b of document.querySelectorAll('[data-style]'))b.addEventListener('click',()=>{
 style=b.dataset.style;for(const other of document.querySelectorAll('[data-style]'))other.setAttribute('aria-pressed',String(other===b));
 document.querySelector('#weight').disabled=style==='solid';repaint();
});
function repaint(){document.documentElement.style.setProperty('--accent',palette[theme][swatch]);for(const c of cards)c.controller?.repaint();}
document.querySelector('#color').addEventListener('change',e=>{swatch=e.target.value;repaint();});
document.querySelector('#weight').addEventListener('change',e=>{weight=Number(e.target.value);repaint();});
document.querySelector('#preview-size').addEventListener('change',e=>{
 document.documentElement.style.setProperty('--preview-size',Number(e.target.value)+'px');
 for(const card of cards)for(const player of card.controller?.players||[])player.resize();
});
document.querySelector('#theme').addEventListener('click',e=>{
 theme=theme==='dark'?'light':'dark';document.documentElement.dataset.theme=theme;e.currentTarget.textContent=theme==='dark'?'Light background':'Dark background';e.currentTarget.setAttribute('aria-pressed',String(theme==='light'));repaint();
});
document.querySelector('#speed').addEventListener('change',e=>{speed=Number(e.target.value);for(const c of cards)c.controller?.setSpeed(speed);});
document.addEventListener('click',e=>{if(!settings.contains(e.target))settings.removeAttribute('open');});
document.addEventListener('keydown',()=>{keyboardInput=true;},true);
document.addEventListener('pointerdown',()=>{keyboardInput=false;},true);
settings.addEventListener('keydown',e=>{if(e.key==='Escape'){settings.removeAttribute('open');settings.querySelector('summary').focus();}});
reduced.addEventListener('change',()=>{stopPlayback();for(const c of cards)c.controller?.setReduced(reduced.matches);updateHint();});
document.addEventListener('visibilitychange',()=>{for(const card of cards)syncPlaybackVisibility(card);});
addEventListener('pagehide',e=>{if(e.persisted){for(const c of cards)c.controller?.suspend(true);}else{stopPlayback();viewportObserver.disconnect();for(const c of cards)c.controller?.destroy();}});
addEventListener('pageshow',()=>{for(const card of cards)syncPlaybackVisibility(card);});
try{
 const catalog=await loadCatalog(new URL('../catalog.json',import.meta.url).href);
 document.querySelector('#icon-count').textContent=catalog.icons.length.toLocaleString('en');
 // Create every anchor first, so category navigation works while previews load.
 const groups=catalog.categories.map(category=>{
  const id='category-'+category.id,section=document.createElement('section');
  section.className='category';section.id=id;section.setAttribute('aria-labelledby',id+'-title');section.tabIndex=-1;
  const header=document.createElement('header');header.className='category-heading';
  const title=document.createElement('h2');title.id=id+'-title';title.textContent=category.label;
  const count=document.createElement('span');count.className='category-count';count.textContent=category.icons.length+' icons';
  const grid=document.createElement('div');grid.className='grid';
  header.append(title,count);section.append(header,grid);document.querySelector('#categories').append(section);
  const link=document.createElement('a');link.href='#'+id;link.textContent=category.label;document.querySelector('#category-links').append(link);
  return {category,grid};
 });
 for(const {category,grid}of groups)for(const icon of category.icons)await mount(icon,grid);updateHint();
 if(location.hash)document.getElementById(decodeURIComponent(location.hash.slice(1)))?.scrollIntoView();
}catch(e){hint.classList.add('has-error');hint.textContent='The icons could not load. Refresh to try again.';console.error(e);}
