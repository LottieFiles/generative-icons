// 20-text-editing/emphasis-reverse.mjs — Claude reverse-Strong correction, Codex supervises.
import {toggle,tube,posed,rigid,F,clamp,flow,track} from './helpers.mjs';

const dyn=(name,fn,extra={})=>[{name,d:fn,style:'outline',type:'stroke',width:2,...extra},{name:name+'-solid',d:fn,style:'solid',type:'stroke',width:2.6,...extra}];
const pose=fn=>rigid(fn);

// ---------- UNDERLINE (geometry, Light, forward Strong unchanged)
const U='M10 7 L10 17 C10 20.9 12.7 23 16 23 C19.3 23 22 20.9 22 17 L22 7';
const ruleLen=F([[0,0],[.5,1,'smooth']],[[0,0],[.2,0,'hold'],[.34,1.08],[.55,1]]);
// Reverse Strong only: pen backs 0.84 units past the rule end (0–.10s), then one
// continuous withdrawal to zero length (.10–.44s), rest to the .70s active end.
const uRetract=t=>flow([[0,1],[.1,1.06,'launch'],[.44,0]],t);
const uLen=(t,s,r)=>r?(s?uRetract(t):1-ruleLen(t,false)):ruleLen(t,s);
const uDip=F([[0,0]],[[0,0],[.12,-.8],[.2,-.8,'hold'],[.34,.4,'hit'],[.5,-.2],[.7,0]]);
// Follower with a cause: the U lifts as the rule is pulled off it, settles by .5s.
const uLift=t=>track([[0,0],[.1,0],[.24,-.6,'out'],[.5,0,'smooth']],t);
const underline=toggle('underline','Underline','Add or remove an underline: a rule is drawn beneath the U; reverse retracts it.',
 [...posed(tube('glyph',U),pose((t,s,r)=>({dy:r?(s?uLift(t):0):uDip(t,s)}))),
  ...dyn('rule',(t,s,r)=>`M9 27.5 L${9+14*uLen(t,s,r)} 27.5`,{visibility:(t,s,r)=>clamp(uLen(t,s,r)*40)})],
 .5,.7,'plain','underlined');
underline.duotoneFill=false;

// ---------- STRIKETHROUGH (geometry, Light, forward Strong unchanged)
const S='M20.5 9 C19.5 7 17.8 6.2 16 6.2 C13 6.2 11 8 11 10.5 C11 16 21 15.5 21 21.5 C21 24 19 25.8 16 25.8 C13.5 25.8 11.6 24.8 10.8 22.8';
const slash=F([[0,0],[.45,1,'smooth']],[[0,0],[.1,.15],[.16,.15,'hold'],[.3,1.07],[.5,1]]);
// Reverse Strong only: 0.9-unit overrun (0–.08s), one withdrawal (.08–.42s), rest to .60s.
const sRetract=t=>flow([[0,1],[.08,1.05,'launch'],[.42,0]],t);
const sLen=(t,s,r)=>r?(s?sRetract(t):1-slash(t,false)):slash(t,s);
const strikethrough=toggle('strikethrough','Strikethrough','Mark text as struck out: a line slashes through the S; reverse withdraws it.',
 [...tube('glyph',S),...dyn('rule',(t,s,r)=>`M7 16 L${7+18*sLen(t,s,r)} 16`,{visibility:(t,s,r)=>clamp(sLen(t,s,r)*40)})],
 .45,.6,'plain','struck');
strikethrough.duotoneFill=false;

export const icons=[underline,strikethrough];
