import {finish,well,F,setting,progress,framed,tube,annulus,circle,rect,polygon,rigid,posed,bothFill,shown} from './helpers.mjs';
const cardTip=F([[0,0],[.18,-5],[.5,0,'smooth']],[[0,0],[.08,3],[.25,-11],[.42,2],[.58,0]]);
const modelCard=finish('model-card','Model card','Inspect a model specification: the complete card tips forward and returns. Light turns 5 degrees; Strong prepares, turns 11 and settles once. The model emblem and specification lines ride the card rigidly.',posed([
 ...well('card',7,5,18,22,2),...framed('model',polygon([[16,8],[19,11],[16,14],[13,11]],.7)),...tube('spec-a','M11 18 L21 18'),...tube('spec-b','M11 22 L18 22')
],rigid((t,s)=>({angle:cardTip(t,s),pivot:[16,16]}))),.5,.58);

const unlock=setting([[0,0],[.08,0],[.66,38,'smooth']],[[0,0],[.08,-5],[.32,43],[.5,38]],[[0,38],[.06,38],[.66,0,'smooth']],[[0,38],[.08,43],[.35,0,'hit'],[.5,0]]);
const openWeights=finish('open-weights','Open weights','Make parameter weights accessible: a shackle opens above three unequal weight columns. Light opens 38 degrees; Strong first presses closed, opens to 43 and settles at 38. Reverse closes at the shared anchor.',[
 ...well('weights',6,16,20,11,2),...tube('weight-small','M10 22 L10 23'),...tube('weight-large','M16 20 L16 23'),...tube('weight-medium','M22 21 L22 23'),
 ...posed(tube('shackle','M11 16 L11 10 C11 7.2 13.2 5 16 5 C18.8 5 21 7.2 21 10 L21 16'),rigid((t,s,r)=>({angle:unlock(t,s,r),pivot:[21,16]})))
],.66,.5,['restricted','accessible']);

const choose=setting([[0,0],[.08,0],[.7,14,'smooth']],[[0,0],[.08,-1],[.36,15],[.52,14]],[[0,14],[.08,14],[.7,0,'smooth']],[[0,14],[.08,15],[.36,-1],[.52,0]]);
const modelPicker=finish('model-picker','Model picker','Choose a model from two options. The selection chevron moves 14 units between fixed option wells. Strong prepares by one unit and catches one unit beyond the destination; reverse restores the first option.',[
 ...well('option-top',11,5,15,8,2),...well('option-bottom',11,19,15,8,2),...tube('top-label','M16 9 L21 9'),...tube('bottom-label','M16 23 L21 23'),
 ...posed(tube('cursor','M5 7 L7 9 L5 11'),rigid((t,s,r)=>({dy:choose(t,s,r),pivot:[7,9]})))
],.7,.52,['first model','second model']);

const switchTo=setting([[0,0],[.06,0],[.74,16,'smooth']],[[0,0],[.09,-.6],[.35,16.5],[.53,16]],[[0,16],[.06,16],[.74,0,'smooth']],[[0,16],[.09,16.6],[.35,-.5],[.53,0]]);
const providerSwitch=finish('provider-switch','Provider switch','Move the active service selector between two provider cabinets. Light slides the contact 16 units along its rail; Strong backs up then crosses quickly with a bounded catch. Both cabinets remain fixed and unbranded.',[
 ...well('provider-a',5,6,8,9,2),...well('provider-b',19,6,8,9,2),...tube('port-a','M8.5 10.5 L9.5 10.5'),...tube('port-b','M22.5 10.5 L23.5 10.5'),
 ...tube('rail','M8 24 L24 24'),...tube('lead-a','M9 15 L9 18'),...tube('lead-b','M23 15 L23 18'),
 ...posed(framed('contact',circle(8,24,2.8),annulus(8,24,2.8,2.6)),rigid((t,s,r)=>({dx:switchTo(t,s,r),pivot:[8,24]})))
],.74,.53,['provider A','provider B']);

const ping=F([[0,0],[.28,3],[.66,0,'smooth']],[[0,0],[.09,0,'hold'],[.26,4.5],[.36,4.5],[.55,0]]);
const pong=F([[0,0],[.18,0],[.46,-3],[.82,0,'smooth']],[[0,0],[.26,0,'hold'],[.44,-4.5],[.54,-4.5],[.73,0]]);
const latencyMeter=finish('latency-meter','Latency meter','Measure a round-trip request between two endpoints. Opposed arrows travel in sequence: Light 3 units, Strong 4.5 units with a faster outbound stroke and a separate return. Endpoint circles stay fixed.',[
 ...framed('source',circle(5.5,9,2.6)),...framed('destination',circle(26.5,23,2.6)),
 ...posed([...tube('outbound','M12 9 L22 9'),...tube('outbound-head','M18 5 L22 9 L18 13')],rigid((t,s)=>({dx:ping(t,s),pivot:[20,9]}))),
 ...posed([...tube('inbound','M20 23 L10 23'),...tube('inbound-head','M14 19 L10 23 L14 27')],rigid((t,s)=>({dx:pong(t,s),pivot:[12,23]}))),
 ...tube('duration','M14 16 L18 16')
],.82,.73);
export const icons=[modelCard,openWeights,modelPicker,providerSwitch,latencyMeter];
