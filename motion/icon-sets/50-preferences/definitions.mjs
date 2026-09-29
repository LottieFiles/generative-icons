// Preferences (50). Designs, reviews and corrections: evidence/design-review.md.
import {icons as a} from './batches/prefs-a.mjs';
import {icons as b} from './batches/prefs-b.mjs';
import {icons as c} from './batches/prefs-c.mjs';
import {icons as d} from './batches/prefs-d.mjs';
import svgpath from 'svgpath';
import {F,U,rigid} from './batches/helpers.mjs';
const all=new Map([...a,...b,...c,...d].map(i=>[i.name,i]));
// Keep these clearance corrections local to this set's assembled definitions.
const reviseParts=(name,select,revise)=>{
 const icon=all.get(name);
 icon.parts=icon.parts.map(part=>select(part.name.slice(name.length+1))?revise(part):part);
};
const adjustRigid=(part,adjust)=>({...part,pose:rigid((t,s,r)=>adjust(part.pose.transform(t,s,r),t,s,r))});

// Sample the reset ring's rigid turn as path geometry so its full-turn endpoint
// and the reverse's zero-turn start export identical coordinates and pixels.
reviseParts('reset-settings',n=>n==='ring'||n.startsWith('ring-'),part=>({
 ...part,pose:(x,y,t,s,r)=>part.pose(x,y,t,s,r)
}));

// Seat the complete pin 0.04 outward: within the rim contact tolerance while
// leaving room for the sampled globe centre's small offset in both profiles.
reviseParts('region',n=>n==='pin'||n.startsWith('pin-'),part=>({
 ...part,d:svgpath(part.d).translate(.04*Math.cos(-40*Math.PI/180),.04*Math.sin(-40*Math.PI/180)).toString()
}));
// Raise the bolt with all of its style variants; retain the keycap's full press.
reviseParts('shortcut-key',n=>n==='bolt'||n.startsWith('bolt-'),part=>({...part,d:svgpath(part.d).translate(0,-1.5).toString()}));

const mouseBody=all.get('mouse').parts.find(p=>p.name==='mouse-body-frame').pose.transform;
reviseParts('mouse',()=>true,part=>adjustRigid(part,(pose,t,s,r)=>s?{
 ...pose,dy:pose.dy-mouseBody(t,s,r).dy/3
}:pose));
// Keep the quarter-turn endpoints and rigid phone; trim only Strong preparation,
// overshoot and hop, in both directions.
const phoneTurn=U([],[[0,0],[.08,-.02],[.3,1.02,'hit'],[.4,.98],[.48,1]]);
reviseParts('auto-rotate',n=>n.startsWith('phone-')||n==='speaker',part=>adjustRigid(part,(pose,t,s,r)=>{
 if(!s)return pose;
 return {...pose,angle:90*phoneTurn(t,s,r),dy:pose.dy*(pose.dy>0?.4:.8)};
}));
const leafPop=F([],[[0,.7],[.1,.7],[.22,1.01,'hit'],[.34,1]]);
reviseParts('battery-saver',n=>n==='leaf'||n.startsWith('leaf-')||n==='rib',part=>({
 ...part,pose:(x,y,t,s,r)=>s&&!r?[14.5+(x-14.5)*leafPop(t,s),16+(y-16)*leafPop(t,s)]:part.pose(x,y,t,s,r)
}));
reviseParts('haptic-feedback',n=>n.startsWith('phone-')||n==='speaker',part=>adjustRigid(part,(pose,t,s)=>s?{
 ...pose,dx:pose.dx*1.15/1.4
}:pose));
for(const [name,before,after] of [
 ['mouse','lifts 0.6','lifts 0.4'],
 ['auto-rotate','winds back 8°','winds back 1.8°'],
 ['auto-rotate','hopping 1.2','hopping 0.96'],
 ['battery-saver','10% past full','1% past full'],
 ['haptic-feedback','rattles 1.4','rattles 1.15']
])for(const field of ['description','intent','brief'])all.get(name)[field]=all.get(name)[field].replace(before,after);
const ORDER=['language','region','currency','appearance','dark-mode','quiet-hours','privacy-shield','shortcut-key','mouse','auto-rotate','sound-toggle','battery-saver','mobile-data','haptic-feedback','reset-settings','auto-update','startup-apps','pointer-speed','advanced-settings','trackpad'];
if(ORDER.length!==all.size||ORDER.some(n=>!all.has(n)))throw Error('Preferences order and batches disagree');
const fixed={"appearance":[],"quiet-hours":[],"shortcut-key":["base"],"auto-rotate":["arrow-a","head-a","arrow-b","head-b"],"battery-saver":["body-frame","stub"],"mobile-data":[],"haptic-feedback":[],"reset-settings":["track-0","track-1"],"pointer-speed":[],"trackpad":["pad-frame","divider"]};
const rigidParts={"region":["globe-frame","meridian","equator","pin"],"appearance":["head-frame","handle"],"quiet-hours":["bell-frame","clapper"],"privacy-shield":["shield-frame"],"shortcut-key":["cap-frame","bolt"],"mouse":["body-frame","divider","wheel"],"auto-rotate":["phone-frame","speaker"],"sound-toggle":["speaker-frame","wave-0","wave-1"],"haptic-feedback":["phone-frame","arc-l","arc-r"],"reset-settings":["ring","knob-0","knob-1"],"auto-update":["ring","shaft","head"],"startup-apps":["body","fin-l","fin-r"],"pointer-speed":["cursor-frame"],"advanced-settings":["wrench-frame"],"trackpad":["finger-0","finger-1"]};
export const icons=ORDER.map(n=>all.get(n)).map((icon,index)=>({...icon,number:826+index,constraints:[...(fixed[icon.name]||[]).map(p=>({type:'fixed',part:icon.name+'-'+p})),...(rigidParts[icon.name]||[]).map(p=>({type:'rigid',part:icon.name+'-'+p}))]}));
