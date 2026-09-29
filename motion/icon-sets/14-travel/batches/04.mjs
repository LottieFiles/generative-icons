import {make,P,F,posed,rigid,rect,circle,polygon,fill,stroke,ink,plainRing,surface,policy,capsule,bothFill,mix} from './helpers.mjs';
const progress=F([[0,0],[.05,0],[.72,1,'out']],[[0,0],[.10,0],[.27,.5,'pass'],[.45,1]]);
const routeC=[[[8,20.5],[8,14],[13,17],[16,16]],[[16,16],[19,15],[24,18],[24,11.5]]];
const lerp=(a,b,u)=>a.map((v,i)=>mix(v,b[i],u));
const split=(c,u)=>{let a=lerp(c[0],c[1],u),b=lerp(c[1],c[2],u),d=lerp(c[2],c[3],u),e=lerp(a,b,u),f=lerp(b,d,u);return [c[0],a,e,lerp(e,f,u)];};
function routeParts(t,s){let p=progress(t,s),a=split(routeC[0],Math.min(1,p*2)),b=p>.5?split(routeC[1],(p-.5)*2):[a[3],a[3],a[3],a[3]];return {d:`M${a[0]} C${a.slice(1).join(' ')} C${b.slice(1).join(' ')}`,head:b[3]};}
const route=policy(make('route','Route','A traveller follows the exact head of a growing journey path, carries speed through the bend, and holds at the destination.',[
 ...plainRing('origin',8,24,2.5),...plainRing('destination',24,8,2.5),...ink('path',(t,s)=>routeParts(t,s).d,{visibility:(t,s)=>progress(t,s)>.00001?1:0}),
 ...posed(bothFill('traveller',circle(0,0,1.3)),rigid((t,s)=>({dx:routeParts(t,s).head[0],dy:routeParts(t,s).head[1],pivot:[0,0]})))
],.8,1.23,['path','traveller']),false);
route.constraints=[{type:'fixed',part:'route-origin'},{type:'fixed',part:'route-destination'}];
const spin=F([[0,0],[.35,-8,'out'],[.75,0,'smooth']],[[0,0],[.15,5],[.5,-26],[.9,4],[1.2,0]]);
const meridian='M16 5.2 C13.68 5.2 11.8 9.36 11.8 14.5 C11.8 19.64 13.68 23.8 16 23.8 C18.32 23.8 20.2 19.64 20.2 14.5 C20.2 9.36 18.32 5.2 16 5.2 Z';
const globe=policy(make('globe','Globe','A flick turns the graticule within a fixed spherical rim. The stand remains planted while the globe eases back.',[
 fill('sphere-tint',circle(16,14.5,9.3),'tint'),...plainRing('sphere',16,14.5,9.3),...ink('stand','M16 23.8 L16 27.5'),...ink('foot','M11 27.5 L21 27.5'),
 ...posed([...ink('meridian',meridian),...ink('equator','M6.7 14.5 L25.3 14.5')],rigid((t,s)=>({angle:spin(t,s),pivot:[16,14.5]})))
],.83,1.28));globe.constraints=[{type:'fixed',part:'globe-sphere'},{type:'fixed',part:'globe-stand'},{type:'rigid',part:'globe-meridian'}];
const ticketD='M8.5 10 L16 10 C17.105 10 18 10.895 18 12 L18 20 C18 21.105 17.105 22 16 22 L8.5 22 C7.395 22 6.5 21.105 6.5 20 L6.5 17.5 C7.328 17.5 8 16.828 8 16 C8 15.172 7.328 14.5 6.5 14.5 L6.5 12 C6.5 10.895 7.395 10 8.5 10 Z';
const stub=F([[0,0],[.35,-3.5,'out'],[.75,0,'smooth']],[[0,0],[.14,1.5],[.45,-5],[.88,1.5],[1.2,0]]);
const ticket=policy(make('ticket','Ticket','The detachable ticket stub presents with a small hinged gesture, overshoots in Strong, and settles alongside the ticket.',[
 ...surface('body',ticketD,[capsule([10.5,14],[14,14],2.1),capsule([10.5,18],[13.5,18],2.1)]),stroke('detail-one','M10.5 14 L14 14'),stroke('detail-two','M10.5 18 L13.5 18'),
 ...posed(surface('stub',rect(22,10,3.5,12,1.5)),rigid((t,s)=>({angle:stub(t,s),pivot:[22,22]})))
],.83,1.28));ticket.constraints=[{type:'fixed',part:'ticket-body'},{type:'rigid',part:'ticket-stub'}];
const swing=F([[0,0],[.35,6,'out'],[.75,0,'smooth']],[[0,0],[.3,12],[.68,-7],[1,3],[1.28,0]]);
const luggage=policy(make('luggage-tag','Luggage tag','The luggage tag swings beneath a fixed ring; its strap and printed face remain one rigid assembly.',[
 ...plainRing('ring',16,7.5,2.5),...posed([...ink('strap','M16 10 L16 13'),...surface('tag',rect(10,13,12,14,2),[capsule([13.5,18],[18.5,18],2.2),capsule([13.5,22],[17.5,22],2.2)]),stroke('label-one','M13.5 18 L18.5 18'),stroke('label-two','M13.5 22 L17.5 22')],rigid((t,s)=>({angle:swing(t,s),pivot:[16,7.5]})))
],.83,1.28));luggage.constraints=[{type:'fixed',part:'luggage-tag-ring'},{type:'rigid',part:'luggage-tag-tag'},{type:'rigid',part:'luggage-tag-strap'}];
const upper=F([[0,0],[.3,-2.5,'out'],[.75,0,'smooth']],[[0,0],[.14,2],[.42,-8],[.82,2.5],[1.2,0]]);
const lower=F([[0,0],[.4,2.5,'out'],[.8,0,'smooth']],[[0,0],[.22,-2],[.5,8],[.9,-2.5],[1.3,0]]);
const edge=(center,sign,a)=>center+sign*3/Math.cos(a*Math.PI/180);
const signpost=policy(make('signpost','Signpost','A gust turns the two directional boards with a short stagger. Exposed post segments remain attached to the current board edges.',[
 ...ink('post-between',(t,s)=>`M16 ${edge(9.5,1,upper(t,s))} L16 ${edge(20,-1,lower(t,s))}`),...ink('post-lower',(t,s)=>`M16 ${edge(20,1,lower(t,s))} L16 27.5`),...ink('foot','M11 27.5 L21 27.5'),
 ...posed(surface('upper-board',polygon([[6,6.5],[21,6.5],[25,9.5],[21,12.5],[6,12.5]],1.2)),rigid((t,s)=>({angle:upper(t,s),pivot:[16,9.5]}))),
 ...posed(surface('lower-board',polygon([[26,17],[11,17],[7,20],[11,23],[26,23]],1.2)),rigid((t,s)=>({angle:lower(t,s),pivot:[16,20]})))
],.88,1.33));signpost.constraints=[{type:'fixed',part:'signpost-foot'},{type:'rigid',part:'signpost-upper-board'},{type:'rigid',part:'signpost-lower-board'}];
export const icons=[route,globe,ticket,luggage,signpost];
