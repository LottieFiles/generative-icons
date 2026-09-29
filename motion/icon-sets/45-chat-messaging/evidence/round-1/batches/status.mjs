import {icon,bubble,surface,tube,fill,bothFill,circle,rigid,posed,appear,state,F,track} from './helpers.mjs';
// Status receipts occupy a separate lower lane, never crossing the bubble tail.
const receipt=()=>[...surface('bubble',bubble(5,5,22,12)),...tube('text','M10 10.8 L20 10.8')];
const sent=icon('message-sent','Message sent',
 'Send acknowledgement → a single check enters the receipt lane → it seats → sent is retained. Light: a 2-unit rise over .64 s. Strong: a 3-unit rise with a decisive .30 s arrival. Reverse withdraws the check while the message stays.',
 [...receipt(),...appear(tube('check','M17 24 L20 27 L26 21'),0,2,[21,24])],.8,.72,['Pending','Sent']);
const delivered=icon('message-delivered','Message delivered',
 'Recipient receives → the second receipt check joins the first → both align → delivered is retained. Light: a 3-unit leftward seat. Strong: a 4.5-unit snap with delayed release. Reverse removes only the second receipt.',
 [...receipt(),...tube('first-check','M9 24 L12 27 L18 21'),...appear(tube('second-check','M18 27 L24 21'),3,0,[21,24])],.8,.72,['Sent','Delivered']);
const eye='M14 24 C17.5 18.6 23.5 18.6 27 24 C23.5 29.4 17.5 29.4 14 24 Z';
const read=icon('message-read','Message read',
 'Reader opens → the eye opens in the receipt lane → its pupil appears → read is retained. Light: the lid opens smoothly. Strong: the lid opens decisively after a short wait. Reverse closes the lid and removes the pupil.',
 [...receipt(),...surface('eye',eye,{pose:(x,y,t,s,r)=>[x,24+(y-24)*(.12+.88*state(t,s,r))]}),...bothFill('pupil',circle(20.5,24,1.45),{visibility:(t,s,r)=>state(t,s,r)})],.8,.72,['Unread','Read']);
const heart='M21 27 C19.8 26 15.8 23.2 15.8 21 C15.8 18.1 19.3 17.2 21 19.5 C22.7 17.2 26.2 18.1 26.2 21 C26.2 23.2 22.2 26 21 27 Z';
const reaction=icon('message-reaction','Message reaction',
 'React → a heart rises below the message → it settles beside the tail → reaction stays. Light: a 1.6-unit rise. Strong: a 2.4-unit rise and a 14-degree turn. Reverse lowers and removes the heart.',
 [...surface('bubble',bubble(5,5,22,9)),...tube('text','M10 9.5 L21 9.5'),...surface('heart',heart,{pose:rigid((t,s,r)=>{const u=state(t,s,r);return {dy:1.6*(1-u),angle:s?-14*Math.sin(u*Math.PI):0,pivot:[21,23]};}),visibility:(t,s,r)=>state(t,s,r)})],.8,.72,['No reaction','Reacted']);
// Pin is a physical object; its needle ends in the chat's fixed attachment point.
const pinParts=[...surface('pin-head','M18 7 L25 7 L23.5 10 L25.5 12 L17.5 12 L19.5 10 Z'),...tube('needle','M21.5 12 L21.5 17')];
const pin=icon('pin-message','Pin message',
 'Pin command → the pin descends into the upper edge of the chat → its needle seats → pinned is retained. Light: 2 units of insertion. Strong: 2 units with an 18-degree preparation. Reverse lifts the pin out.',
 [...surface('bubble',bubble(5,17,22,9)),...tube('text','M10 21.5 L20 21.5'),...pinParts.map(p=>({...p,pose:rigid((t,s,r)=>{const u=state(t,s,r);return {dy:-2*(1-u),angle:s?18*Math.sin(u*Math.PI):0,pivot:[21.5,17]};}),visibility:(t,s,r)=>state(t,s,r)}))],.8,.72,['Unpinned','Pinned']);
// The slash has a clear central lane and does not intersect an unbroken outline.
const mute=icon('mute-chat','Mute chat',
 'Mute command → a diagonal strike spans the conversation → notifications stop → muted stays. Light: the stroke wipes in over .64 s. Strong: a fast .30 s wipe after .10 s preparation. Reverse retracts the strike.',
 [...surface('bubble',bubble(5,6,22,17)),...tube('slash',(t,s,r)=>{const u=state(t,s,r);return `M10 10 L${10+12*u} ${10+11*u}`;},{visibility:(t,s,r)=>state(t,s,r)>.01?1:0})],.8,.72,['Audible','Muted']);
export const icons=[sent,delivered,read,reaction,pin,mute];
