// Preferences batch A: language, region, currency, appearance, dark-mode.
import svgpath from 'svgpath';
import {make,toggle,F,P,U,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,arc,arcN,ellipse,hole,blend,clamp01,rotate} from './helpers.mjs';

// ---------- LANGUAGE ----------
// A speech bubble (x5–27, y5–23, tail to 8.5,27.5) holding an "A". Solid keeps the bubble as a
// 2.6 ring so the letter survives. The bubble puffs about its tail tip; the A rides inside.
const BUB=polygon([[5,5],[27,5],[27,23],[14,23],[8.5,27.5],[8.5,23],[5,23]],[3.5,3.5,3.5,.8,.7,.4,3.5]);
const TAIL=[8.5,27.5];
const puff=F([[0,1],[.2,1.045],[.5,1,'smooth']],[[0,1],[.08,.95],[.2,1.07,'hit'],[.32,.985],[.44,1]]);
const aDip=F([[0,0],[.5,0]],[[0,0],[.16,0],[.26,-.9],[.4,.25,'hit'],[.5,0]]);
const bubblePose=(x,y,t,s)=>{const k=puff(t,s);return[TAIL[0]+(x-TAIL[0])*k,TAIL[1]+(y-TAIL[1])*k];};
const aPose=(x,y,t,s)=>{const p=bubblePose(x,y,t,s);return[p[0],p[1]+aDip(t,s)];};
const language=make('language','Language',
 'Light: the speech bubble puffs 4.5% about its tail as if speaking. Strong: it draws in 5%, pops out 7% with a stop and settles while the letter hops inside.',[
 ...posed(framed('bubble',BUB,hole(BUB,rect(7.6,7.6,16.8,12.8,2.2))),bubblePose),
 ...posed([...tube('legs','M12.6 18.2 L16 9.8 L19.4 18.2'),...tube('bar','M13.75 15.4 L18.25 15.4')],aPose)
],.5,.5);
language.duotoneFill=true;

// ---------- REGION ----------
// A globe (13,19.5 r8, meridian ellipse, equator) with a map pin whose tip rests on the rim at -40°.
// The pin lifts and drops back onto the globe; Strong drops from 2.8 and the globe dips on impact.
const GC=[13,19.5],GR=8,TIP=[GC[0]+GR*Math.cos(-40*Math.PI/180),GC[1]+GR*Math.sin(-40*Math.PI/180)];
const HEAD=[TIP[0],9],HR=2.7;
const pinD=(()=>{const d=TIP[1]-HEAD[1],al=Math.asin(HR/d)*180/Math.PI,a0=90+(90-al),a1=90-(90-al)+360;return `M${TIP} `+arc(HEAD[0],HEAD[1],HR,a0,a1).replace(/^M/,'L')+' Z';})();
const lift=F([[0,0],[.18,-1.2],[.5,0,'smooth']],[[0,0],[.14,-2.8],[.3,0,'hit'],[.38,-.7],[.48,0,'hit'],[.56,0]]);
const dip=F([[0,0],[.5,0]],[[0,0],[.3,0,'hold'],[.36,.5,'hit'],[.46,-.15],[.56,0]]);
const globePose=rigid((t,s)=>({dy:dip(t,s)}));
const pinPose=rigid((t,s)=>({dy:lift(t,s)+dip(t,s)}));
const region=make('region','Region',
 'Light: the map pin lifts 1.2 off the globe and settles back on its rim. Strong: it lifts 2.8, drops onto the globe with a stop, bounces once and the globe dips half a unit on impact.',[
 ...posed([...framed('globe',circle(...GC,GR),hole(circle(...GC,GR),circle(...GC,5.4))),...tube('meridian',ellipse(...GC,3.3,GR)),...tube('equator',`M${GC[0]-GR} ${GC[1]} L${GC[0]+GR} ${GC[1]}`)],globePose),
 ...posed([fill('pin-interior',pinD,'tint'),stroke('pin',pinD),fill('pin-solid',hole(pinD,circle(...HEAD,1.1)),'solid')],pinPose)
],.5,.56);
region.duotoneFill=true;

// ---------- CURRENCY ----------
// A coin (r10.5, Solid ring) flips about its vertical axis from "$" to "€". Each glyph shows only
// while its face is at least 35% toward the viewer; the € is drawn pre-mirrored so it reads
// correctly at the end. Strong hops 1.6 during the flip and lands with a stop.
const spin=U([[0,0],[.06,0],[.5,1,'smooth']],[[0,0],[.08,-.07],[.3,1.06,'hit'],[.4,.98],[.5,1]]);
const hop=(t,s,r)=>F([[0,0],[.5,0]],[[0,0],[.08,.4],[.26,-1.6],[.4,0,'hit'],[.5,0]])(t,s);
const face=(t,s,r)=>Math.cos(Math.PI*spin(t,s,r));
const coinPose=(x,y,t,s,r)=>[16+(x-16)*face(t,s,r),y+hop(t,s,r)];
const euroPose=(x,y,t,s,r)=>[16-(x-16)*face(t,s,r),y+hop(t,s,r)];
const S='M19.2 12.2 C18.2 10.6 13 10.4 12.9 13.5 C12.8 16.3 19.2 15.7 19.1 18.6 C19 21.6 13.6 21.5 12.7 19.9';
const currency=toggle('currency','Currency',
 'Light: the coin turns over in half a second, its dollar face giving way to the euro face. Strong: it winds back, flips past flat with a stop, hops 1.6 and lands. Reverse flips it back.',[
 ...posed(framed('coin',circle(16,16,10.5),hole(circle(16,16,10.5),circle(16,16,7.9))),coinPose),
 ...shown(posed([...tube('dollar-s',S),...tube('dollar-bar','M16 9.6 L16 22.4')],coinPose),(t,s,r)=>face(t,s,r)>.35?1:0),
 ...shown(posed([...tube('euro-c',arc(16.9,16,5.3,48,312)),...tube('euro-bar-0','M11.2 14.1 L17.6 14.1'),...tube('euro-bar-1','M11.2 17.9 L17.6 17.9')],euroPose),(t,s,r)=>face(t,s,r)<-.35?1:0)
],.5,.5,'dollar','euro');
currency.duotoneFill=true;

// ---------- APPEARANCE ----------
// A paintbrush (axis 65°, tip at 6.5,21.3) sweeps right 14 units and leaves a painted band
// (y22.2–26.8) behind its tip. The band is retained; reverse wipes it back.
const T0=[6.5,21.3],AX=[Math.cos(65*Math.PI/180),-Math.sin(65*Math.PI/180)];
const along=k=>[T0[0]+AX[0]*k,T0[1]+AX[1]*k];
const HEAD_D=svgpath(rect(-2.1,-6.3,4.2,6.3,1.3)).rotate(25).translate(...T0).toString();
const HANDLE=`M${along(6)} L${along(17.2)}`;
const sweep=U([[0,0],[.06,0],[.6,1,'smooth']],[[0,0],[.1,-.03],[.36,1.04,'hit'],[.46,.99],[.54,1]]);
const tilt=F([[0,0],[.2,-4],[.6,0,'smooth']],[[0,0],[.1,5],[.24,-12],[.36,-12],[.46,-2,'hit'],[.54,0]]);
const brushPose=rigid((t,s,r)=>({dx:14*sweep(t,s,r),angle:(r?-1:1)*tilt(t,s),pivot:T0}));
const bandD=(t,s,r)=>{const w=14*clamp01(sweep(t,s,r));return rect(T0[0]-2.3,22.2,w+4.6,4.6,2.3);};
const bandShow=(t,s,r)=>clamp01((sweep(t,s,r)-.02)/.08);
const appearance=toggle('appearance','Appearance',
 'Light: the brush glides 14 units to the right and paints a band behind its tip. Strong: it dabs, drags its handle back 12° through a fast sweep, stops and straightens. The painted band is retained; reverse wipes it back.',[
 ...framed('band',bandD,bandD,{visibility:bandShow}),
 ...posed([...framed('head',HEAD_D),...tube('handle',HANDLE)],brushPose)
],.6,.54,'blank','painted');
appearance.duotoneFill=true;

// ---------- DARK MODE ----------
// The sun disc (r4.6, four fixed-topology arcs) morphs into a crescent moon built from the same
// four cubics (two outer, two inner), while eight rays retract into it. Strong swings the moon in.
const C1=[16,16],R1=9.5,C2=[20.5,11.5],R2=8.5;
const {a1,a2,b1,b2}=(()=>{const dx=C2[0]-C1[0],dy=C2[1]-C1[1],d=Math.hypot(dx,dy),a=(R1*R1-R2*R2+d*d)/(2*d),h=Math.sqrt(R1*R1-a*a),M=[C1[0]+a*dx/d,C1[1]+a*dy/d],px=-dy/d,py=dx/d;
 const P1=[M[0]+h*px,M[1]+h*py],P2=[M[0]-h*px,M[1]-h*py],deg=(p,c)=>Math.atan2(p[1]-c[1],p[0]-c[0])*180/Math.PI;
 let a1=deg(P1,C1),a2=deg(P2,C1);if(a2<a1)a2+=360;let b2=deg(P2,C2),b1=deg(P1,C2);if(b2<0)b2+=360;return {a1,a2,b1,b2};})();
const SUN=arcN(...C1,4.6,a1,a1+360,4)+' Z';
const MOON=(()=>{const inner=arcN(...C2,R2,b2,b1,2);return arcN(...C1,R1,a1,a2,2)+inner.slice(inner.indexOf(' C'))+' Z';})();
const morph=U([[0,0],[.1,0],[.5,1,'smooth']],[[0,0],[.06,0],[.34,1,'hit'],[.42,1]],[[0,0],[.4,1,'smooth']],[[0,0],[.28,1,'hit'],[.36,1]]);
const retract=U([[0,0],[.24,1,'in']],[[0,0],[.14,1]],[[0,0],[.28,0],[.5,1,'out']],[[0,0],[.22,0],[.38,1]]);
const swing=F([[0,0],[.5,0]],[[0,0],[.06,0],[.34,-50,'hit'],[.44,5],[.52,0]]);
const discD=(t,s,r)=>blend(SUN,MOON,morph(t,s,r));
const discPose=(x,y,t,s,r)=>rotate(x,y,r?0:swing(t,s),...C1);
const rayLen=(t,s,r)=>2.4*(1-retract(t,s,r));
const rayD=i=>(t,s,r)=>{const a=i*45*Math.PI/180,l=Math.max(.01,rayLen(t,s,r));return `M${16+8.2*Math.cos(a)} ${16+8.2*Math.sin(a)} L${16+(8.2+l)*Math.cos(a)} ${16+(8.2+l)*Math.sin(a)}`;};
const rayShow=(t,s,r)=>clamp01((rayLen(t,s,r)-.8)/.5);
const darkMode=toggle('dark-mode','Dark mode',
 'Light: the rays draw into the sun and the disc morphs into a crescent moon. Strong: the rays snap in, the disc grows into the moon with a stop and swings 50° into place. Reverse: the moon morphs back and the rays grow out.',[
 ...posed(framed('disc',discD),discPose),
 ...Array.from({length:8},(_,i)=>tube('ray-'+i,rayD(i),{visibility:rayShow})).flat()
],.5,.52,'day','night',.5,.4);
darkMode.duotoneFill=true;

export const icons=[language,region,currency,appearance,darkMode];
