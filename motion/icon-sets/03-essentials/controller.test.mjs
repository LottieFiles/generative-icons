import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {IconController} from '../../../prototype/icons/03/controller.mjs';
const set=JSON.parse(fs.readFileSync(new URL('../../../prototype/icons/03/assets/set.json',import.meta.url)));
class Player{
 constructor(){this.currentFrame=0;this.listeners=new Map();this.playCount=0;}
 loadAnimation(id){this.activeAnimationId=id;this.currentFrame=0;}
 setSpeed(v){this.speed=v;}setLoop(v){this.loop=v;}setSegment(a,b){this.segment=[a,b];}setFrame(v){this.currentFrame=v;}
 play(){this.playing=true;this.playCount++;}pause(){this.playing=false;}destroy(){this.destroyed=true;}
 addEventListener(n,f){if(!this.listeners.has(n))this.listeners.set(n,new Set());this.listeners.get(n).add(f);}
 removeEventListener(n,f){this.listeners.get(n)?.delete(f);}
 emit(n){for(const f of [...(this.listeners.get(n)||[])])f();}
}
function setup(name,options={}){const icon=set.icons.find(i=>i.name===name),players=[new Player(),new Player(),new Player(),new Player()],c=new IconController(icon,players,options);return {c,players,icon};}
test('state actions hold their result and only explicit reverse changes state',async()=>{
 for(const name of ['check','lock']){const {c,players,icon}=setup(name);assert.equal(c.state,icon.endState);const reverse=c.toggle();assert.equal(c.meta.direction,'reverse');assert.equal(c.state,icon.startState);players[0].emit('complete');assert.equal(await reverse,true);assert.ok(players.every(p=>p.currentFrame===c.meta.rest&&!p.playing&&!p.loop));const forward=c.toggle();assert.equal(c.meta.direction,'forward');assert.equal(c.state,icon.endState);players[0].emit('complete');assert.equal(await forward,true);}
});
test('selecting intensity preserves the open or pending state, speed, and paint',async()=>{
 const {c,players,icon}=setup('lock',{paint:p=>p.painted=true});const action=c.toggle();players[0].emit('complete');await action;c.setSpeed(.5);c.setMotion('strong');assert.equal(c.state,'open');assert.equal(c.meta.name,'lock-unlock-strong');assert.equal(c.meta,icon.motions[1].reverse);assert.ok(players.every(p=>p.currentFrame===c.meta.rest&&p.speed===.5&&p.painted));assert.equal(c.running,false);
});
test('reduced motion changes meaningful states without starting playback',async()=>{
 for(const name of ['check','lock']){const {c,players,icon}=setup(name,{reduced:true});await c.toggle();assert.equal(c.state,icon.startState);assert.ok(players.every(p=>p.playCount===0&&p.currentFrame===c.meta.rest));await c.play('forward');assert.equal(c.state,icon.endState);assert.ok(players.every(p=>p.playCount===0));}
});
test('reduced motion during a transition settles to the requested state',async()=>{
 const {c,players}=setup('check');const action=c.toggle();c.setReduced(true);assert.equal(await action,false);assert.equal(c.state,'pending');assert.equal(c.running,false);assert.ok(players.every(p=>p.currentFrame===c.meta.rest&&!p.playing));
});
test('rapid retriggers coalesce and stale completion cannot finish a newer action',async()=>{
 const {c,players}=setup('star');const first=c.play();const oldComplete=c.complete;assert.equal(await c.play(),false);const second=c.play('forward',{force:true});assert.equal(await first,false);oldComplete();assert.equal(c.running,true);players[0].emit('complete');assert.equal(await second,true);assert.equal(c.running,false);
});
test('repaint preserves paused frames and each card selects motion independently',()=>{
 const a=setup('star'),b=setup('folder');a.c.scrub(13);a.c.repaint();assert.ok(a.players.every(p=>p.currentFrame===13));b.c.setMotion('strong');assert.equal(b.c.meta.name,'folder-strong');assert.equal(a.c.profile,'light');assert.ok(a.players.every(p=>p.currentFrame===13));
});
test('showcase holds apply only to same-state gestures',async()=>{
 for(const name of ['star','check','sliders','lock','folder']){const {c,players,icon}=setup(name);const done=c.play('forward',{showcase:true});assert.equal(players[0].segment[1],icon.stateful?Math.ceil(c.meta.active*c.meta.fps):c.meta.frames-1);assert.equal(players[0].loop,false);players[0].emit('complete');await done;}
});
test('suspension resumes the current gesture and cleanup releases pending actions',async()=>{
 const {c,players}=setup('sliders');const done=c.play();c.suspend(true);assert.ok(players.every(p=>!p.playing));c.suspend(false);assert.ok(players.every(p=>p.playing));c.destroy();assert.equal(await done,false);assert.ok(players.every(p=>p.destroyed));assert.equal(await c.play(),false);
});
