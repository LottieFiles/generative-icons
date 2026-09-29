// Weather batch E: weather-radar, umbrella, windsock.
import {make,F,tube,framed,rigid,posed,shown,fill,stroke,circle,bothFill,annulus} from './helpers.mjs';
import {pt,curveFrom} from './sky.mjs';

// ---------- WEATHER RADAR ----------
// Scope r11 with a sweep arm (tip paint ≥1 inside the rim at Bold). One full sweep finds a storm cell,
// which appears as the arm passes it and is retained; Strong adds a ripple from the cell.
const BLIP=pt(-35,5.6);
const sweepA=F([[0,0],[.04,0],[.92,360,'smooth']],[[0,0],[.06,-12],[.5,366],[.62,360]]);
const blipK=F([[0,0],[.2,0],[.32,1,'out']],[[0,0],[.14,0],[.2,1.45,'out'],[.32,.9],[.4,1]]);
const ripple=F([[0,0]],[[0,0],[.16,0],[.52,1]]);
const weatherRadar=make('weather-radar','Weather radar',
 'Light: the sweep makes one clockwise turn and a storm cell appears as the arm passes it. Strong: the arm kicks back, sweeps faster with a small overrun, and the cell pops in with a ripple ring. The cell is retained; preview reset rearms.',[
 // Solid is a ring: a filled scope hid the arm and the storm cell.
 ...framed('scope',circle(16,16,11),annulus(16,16,11,2.6)),
 // v1 read as a clock face (hand at 12 plus a dot). A fixed range ring at the cell's radius makes it a scope;
 // the cell sits on the ring. Ring paint clears the hub by 2.75 and the rim by 2.9.
 ...tube('range',circle(16,16,5.6)),
 ...posed(tube('arm','M16 16 L16 8.6'),rigid((t,s)=>({angle:sweepA(t,s),pivot:[16,16]}))),
 ...bothFill('hub',circle(16,16,1.6)),
 ...shown(posed(bothFill('cell',circle(...BLIP,1.7)),(x,y,t,s)=>{const k=Math.max(.001,blipK(t,s));return[BLIP[0]+(x-BLIP[0])*k,BLIP[1]+(y-BLIP[1])*k];}),(t,s)=>blipK(t,s)>.08?1:0),
 ...shown(tube('ripple',(t,s)=>circle(...BLIP,1.8+2.4*ripple(t,s))),(t,s)=>{const r=ripple(t,s);return s&&r>0&&r<1?(1-r):0;})
],.92,.62,['cell','cell-solid','ripple']);
weatherRadar.duotoneFill=true;

// ---------- UMBRELLA ----------
// Canopy: dome r11.5 over a four-scallop hem; the shaft hangs from the hem's centre point and follows it.
const HEM=16.2,APEX=4.7;
const canopyD=`M4.5 ${HEM} C4.5 9.85 9.65 ${APEX} 16 ${APEX} C22.35 ${APEX} 27.5 9.85 27.5 ${HEM} C26.54 15 25.58 14.4 24.63 14.4 C23.67 14.4 22.71 15 21.75 ${HEM} C20.79 15 19.83 14.4 18.88 14.4 C17.92 14.4 16.96 15 16 ${HEM} C15.04 15 14.08 14.4 13.13 14.4 C12.17 14.4 11.21 15 10.25 ${HEM} C9.29 15 8.33 14.4 7.38 14.4 C6.42 14.4 5.46 15 4.5 ${HEM} Z`;
const open=F([[0,1],[.12,.9],[.28,1.02],[.44,1,'smooth']],[[0,1],[.14,.62],[.24,1.06,'hit'],[.36,.97],[.48,1]]);
const flex=(c)=>(x,y)=>[16+(x-16)*c,y+(1-c)*Math.max(0,y-APEX)*.35];
const hemY=(t,s)=>flex(open(t,s))(16,HEM)[1];
const shaftD=(t,s)=>`M16 ${hemY(t,s)} L16 25.2 C16 26.8 14.9 27.8 13.6 27.8 C12.4 27.8 11.5 26.9 11.5 25.8`;
const umbrella=make('umbrella','Umbrella',
 'Light: the canopy flexes in as rain lands and springs back. Strong: it half closes, its edges drooping, then snaps open 6% wider and settles; the shaft stays attached to the hem.',[
 ...posed(framed('canopy',canopyD),(x,y,t,s)=>flex(open(t,s))(x,y)),
 ...tube('shaft',shaftD)
],.44,.48);
umbrella.duotoneFill=true;

// ---------- WINDSOCK ----------
// Pole at x7; the sock's mouth is fastened beside it. The sock is a flexible tapered tube: its axis
// droops by `droop` degrees and carries a travelling ripple; cross-sections stay vertical so the
// mouth edge and the stripes stay straight.
const M=[8.8,8.4],LEN=18;
const droop=F([[0,28],[.12,16],[.5,28,'smooth']],[[0,28],[.08,31],[.3,-2],[.5,8],[.66,28]]);
const rip=(t,s)=>{const e=s?F([[0,0]],[[0,0],[.1,0],[.28,1],[.66,0]])(t,s):F([[0,0],[.12,1],[.5,0,'smooth']],[[0,0]])(t,s);return (s?1.3:.6)*e;};
const axis=(u,t,s)=>{const a=droop(t,s)*Math.PI/180,r=rip(t,s)*Math.sin(2*Math.PI*(1.1*u-t*2.2))*u;return[M[0]+LEN*u*Math.cos(a)-r*Math.sin(a),M[1]+LEN*u*Math.sin(a)+r*Math.cos(a)];};
const half=u=>3.2*(1-u)+1.3*u;
const edge=(sign,t,s)=>u=>{const p=axis(u,t,s);return[p[0],p[1]+sign*half(u)];};
const sockD=(t,s)=>{const top=curveFrom(edge(-1,t,s),3),bot=curveFrom(u=>edge(1,t,s)(1-u),3);return top+' '+bot.replace(/^M/,'L')+' Z';};
const stripeD=u=>(t,s)=>`M${edge(-1,t,s)(u)} L${edge(1,t,s)(u)}`;
const windsock=make('windsock','Windsock',
 'Light: a gust lifts the drooping sock 12° with a small ripple and lets it sag back. Strong: the sock sags, is blown out to horizontal with a travelling ripple, and falls back through a short lift to its droop.',[
 ...tube('pole','M7 4.8 L7 27.6'),
 ...tube('foot','M4.6 27.6 L9.4 27.6'),
 fill('sock-interior',sockD,'tint'),stroke('sock-frame',sockD),fill('sock-solid',sockD,'solid'),
 // Stripes are Outline/Duotone detail only: Solid stripe caps poked outside the sock silhouette.
 stroke('stripe-0',stripeD(.36)),stroke('stripe-1',stripeD(.68))
],.5,.66);
windsock.duotoneFill=true;

export const icons=[weatherRadar,umbrella,windsock];
