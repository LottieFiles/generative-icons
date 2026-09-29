// Artwork remains in Lottie. This controller owns preview selection and state.
export class IconController {
  constructor(icon,players,{paint=()=>{},change=()=>{},reduced=false,profile='light'}={}){
    this.icon=icon;this.players=players;this.paint=paint;this.change=change;this.reduced=reduced;
    this.meta=icon.motions.find(m=>m.motionId===profile)||icon.motions[0];this.profile=this.meta.motionId;this.direction='forward';this.state=this.meta.endState;
    this.running=false;this.paused=false;this.suspended=false;this.speed=1;this.epoch=0;this.disposed=false;
    this.load(this.meta);this.rest();
  }
  notify(){this.change(this);}
  cancel(){this.epoch++;for(const p of this.players)p.pause();if(this.complete)this.players[0].removeEventListener('complete',this.complete);this.complete=null;if(this.loopComplete)this.players[0].removeEventListener('loop',this.loopComplete);this.loopComplete=null;this.running=false;const resolve=this.resolve;this.resolve=null;resolve?.(false);}
  load(meta){this.meta=meta;for(const p of this.players){p.loadAnimation(meta.name);if(p.activeAnimationId!==meta.name)throw new Error('Animation selection failed: '+meta.name);p.setSpeed(this.speed);p.setLoop(false);p.setSegment(0,meta.frames-1);this.paint(p);}}
  rest(){for(const p of this.players){p.pause();p.setLoop(false);p.setSegment(0,this.meta.frames-1);p.setFrame(this.meta.rest);}this.running=false;this.paused=false;this.notify();}
  stopPreview(){
    this.cancel();
    // A preview reset is a demo operation, not the icon's retained result.
    if(this.icon.previewReset){const motion=this.icon.motions.find(m=>m.motionId===this.profile);this.direction='forward';this.state=motion.endState;this.load(motion);}
    this.rest();
  }
  finish(){if(this.complete)this.players[0].removeEventListener('complete',this.complete);this.complete=null;if(this.loopComplete)this.players[0].removeEventListener('loop',this.loopComplete);this.loopComplete=null;this.running=false;this.rest();const resolve=this.resolve;this.resolve=null;resolve?.(true);}
  play(direction='forward',{force=false,showcase=false,hold=0,repeat=null}={}){
    if(this.disposed||this.running&&!force)return Promise.resolve(false);
    this.cancel();this.direction=direction;
    const motion=this.icon.motions.find(m=>m.motionId===this.profile),meta=direction==='reverse'?motion.reverse:motion;
    if(!meta)return Promise.resolve(false);
    this.load(meta);this.state=meta.endState;
    if(this.reduced){this.rest();return Promise.resolve(true);}
    this.running=true;this.paused=false;const epoch=this.epoch;
    const finished=new Promise(resolve=>{this.resolve=resolve;});
    this.complete=()=>{if(this.epoch===epoch&&this.running)this.finish();};this.players[0].addEventListener('complete',this.complete);
    // Continuous loaders use the native clock across loop boundaries.
    // The predicate is checked at the wrap so pointer exit finishes the cycle.
    if(meta.continuous && repeat){this.loopComplete=()=>{if(this.epoch===epoch&&this.running&&!repeat())this.finish();};this.players[0].addEventListener('loop',this.loopComplete);}
    const end=meta.continuous?meta.frames-1:showcase&&!this.icon.stateful?meta.frames-1:Math.min(meta.frames-1,Math.ceil((meta.active+hold)*meta.fps));
    for(const p of this.players){p.setSegment(0,end);p.setLoop(Boolean(meta.continuous&&repeat));p.setFrame(0);if(!this.suspended)p.play();}
    this.notify();return finished;
  }
  toggle(){if(!this.icon.stateful)return this.play();return this.play(this.state===this.icon.endState?'reverse':'forward');}
  async preview({force=false,hold=0,repeat=null}={}){
    if(this.reduced||this.disposed||this.running&&!force)return false;
    const direction=this.icon.stateful&&this.state===this.icon.endState?'reverse':'forward';
    const action=this.play(direction,{force,hold:this.icon.stateful?.18:hold,repeat}),epoch=this.epoch;
    if(!await action||this.epoch!==epoch)return false;
    if(!this.icon.stateful)return true;
    return this.play(direction==='forward'?'reverse':'forward',{hold});
  }
  setMotion(profile){if(!this.icon.motions.some(m=>m.motionId===profile))return;this.cancel();this.profile=profile;const motion=this.icon.motions.find(m=>m.motionId===profile);this.load(this.direction==='reverse'?motion.reverse:motion);this.rest();}
  setSpeed(value){this.speed=value;for(const p of this.players)p.setSpeed(value);}
  setReduced(value){this.reduced=value;if(value)this.stopPreview();else this.notify();}
  scrub(frame){this.cancel();this.paused=true;for(const p of this.players){p.setSegment(0,this.meta.frames-1);p.setFrame(frame);}this.notify();}
  repaint(){for(const p of this.players){const frame=p.currentFrame;this.paint(p);p.setFrame(frame);}this.notify();}
  suspend(hidden){this.suspended=hidden;for(const p of this.players)if(hidden)p.pause();else if(this.running&&!this.reduced)p.play();}
  destroy(){this.cancel();this.disposed=true;for(const p of this.players)p.destroy();}
}
