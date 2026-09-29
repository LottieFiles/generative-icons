// Weather batch C: wind, tornado, rainbow, sunrise, sunset.
import {make,F,P,tube,rigid,posed,shown,fill,stroke} from './helpers.mjs';
import {arcN,pt} from './sky.mjs';

// ---------- WIND ----------
// Three gust lines; the top one curls up, the bottom one curls down. Curls have ≥3-unit counters.
const GUSTS=[
 ['M4.8 11.6 L19.2 11.6 C21.8 11.6 23.4 10 23.4 8.2 C23.4 6.4 22 5.2 20.4 5.2 C18.9 5.2 17.8 6.3 17.8 7.6',0],
 ['M4.8 17.2 L24.6 17.2',.05],
 ['M4.8 22.8 L15.4 22.8 C18 22.8 19.6 24.3 19.6 26 C19.6 27.4 18.5 28.3 17.2 28.3 C16 28.3 15.2 27.5 15.2 26.5',.1]];
const blow=d=>F([[0,0],[d,0],[d+.24,1.1],[d+.52,0,'smooth']],[[0,0],[d,0],[d+.07,-.6,'launch'],[d+.27,3],[d+.42,2.2],[d+.66,0]]);
const wind=make('wind','Wind',
 'Light: the three gust lines push 1.1 units right in a short stagger and return. Strong: each line draws back 0.6, is driven 3 units right with one damped return.',
 GUSTS.flatMap(([d,delay],i)=>posed(tube('gust-'+i,d),rigid((t,s)=>({dx:blow(delay)(t,s)})))),.62,.76);
wind.duotoneFill=false;

// ---------- TORNADO ----------
// Five stacked lines narrowing downward. A sway wave travels down the funnel: the top line is
// tethered (small amplitude), the tip swings most.
const TLINES=[[6.2,25.8,6.6],[8.4,23.8,11.2],[10.6,21.4,15.8],[12.4,19.8,20.4],[14,17.8,25]];
const GAIN=[.3,.55,.8,1,1.2];
const sweep=(t,s,k)=>{const u=t-(s?.06:0)-k*(s?.045:.04);if(u<=0)return 0;const T=s?.62:.5;if(u>=T)return 0;const env=Math.sin(Math.PI*u/T)**(s?1:1.4);return (s?2.1:.8)*GAIN[k]*env*Math.sin(2*Math.PI*u/T*(s?1.5:1));};
const tornado=make('tornado','Tornado',
 'Light: one sway travels down the funnel, the tip moving 1 unit. Strong: a wider wave with a second half-swing travels down; the tip swings 2.5 units before settling.',
 TLINES.flatMap(([a,b,y],k)=>posed(tube('band-'+k,`M${a} ${y} L${b} ${y}`),rigid((t,s)=>({dx:sweep(t,s,k)})))),.7,.9);
tornado.duotoneFill=false;

// ---------- RAINBOW ----------
// Result icon: three concentric arcs draw from the left base over to the right, outer first.
// Fixed four-segment arcs keep the topology constant; a zero-length arc is hidden.
// The arcs rise from a fixed horizon, which keeps the prepared (empty) state visible and grounds the drawing.
// Arc feet clear the horizon's Bold paint by 1.2.
const C=[16,20.8],RADII=[11.6,8,4.4],GROUND=24.5;
// Strong draws each band in 0.16 s (Light 0.30 s) with a tight 0.05 s stagger, then the bands pop.
const drawK=k=>F([[0,0],[.04+k*.1,0],[.34+k*.1,1,'smooth']],[[0,0],[.06+k*.05,0,'hold'],[.22+k*.05,1]]);
const pop=F([[0,0],[.6,0]],[[0,0],[.3,0],[.36,.6,'hit'],[.48,-.15],[.58,0]]);
const rainbow=make('rainbow','Rainbow',
 'Light: three arcs draw over the horizon from the left base, outer first, in a calm stagger. Strong: they sweep over in a tight burst and the finished bands pop outward half a unit and settle. The rainbow is retained; preview reset rearms.',
 [...tube('horizon',`M4.4 ${GROUND} L27.6 ${GROUND}`),...RADII.flatMap((r,k)=>{const p=drawK(k),arc=(t,s)=>arcN(C[0],C[1],r+pop(t,s),180,180+180*Math.max(.0005,p(t,s)));
  return shown([stroke('band-'+k,arc),{name:'band-'+k+'-solid',d:arc,style:'solid',type:'stroke',width:2.6}],(t,s)=>p(t,s)>.004?1:0);})],
 .54,.6,['band-0','band-1','band-2']);
rainbow.duotoneFill=false;

// ---------- SUNRISE / SUNSET ----------
// A real horizon: only the part of the sun above it is drawn, as a fixed-topology arc whose ends ride
// the horizon, closed by the chord for Solid/Duotone. A sun fully below the horizon is hidden.
// Rays are separate paths from 9.5 (Bold gap ≥1 to the rim) outward by RAY·o; o = 0 hides them.
const H=19.4,R=6,RAY=1.7;
const cap=yc=>{const d=Math.max(-R+.001,Math.min(R-.001,yc-H)),a=Math.asin(d/R)*180/Math.PI;return [180+a,360-a];};
const sunArc=yc=>{const [a,b]=cap(yc);return arcN(16,yc,R,a,b,4);};
const sunDisc=yc=>sunArc(yc)+' Z';
const RAYS=[210,240,270,300,330];
function horizonSun(name,label,intent,chevron,chevronDy,light,strong,sunY,rayOut){
 const up=(t,s)=>sunY(t,s)<H+R-.35?1:0;
 const parts=[
  ...tube('horizon',`M4.4 ${H} L27.6 ${H}`),
  ...posed(tube('chevron',chevron),rigid((t,s)=>({dy:chevronDy(t,s)}))),
  ...shown([fill('sun-interior',(t,s)=>sunDisc(sunY(t,s)),'tint'),stroke('sun-frame',(t,s)=>sunArc(sunY(t,s))),fill('sun-solid',(t,s)=>sunDisc(sunY(t,s)),'solid')],up),
  ...RAYS.flatMap((a,i)=>{const d=(t,s)=>{const c=[16,sunY(t,s)],o=Math.max(.001,rayOut(t,s,i));return `M${pt(a,9.5,c)} L${pt(a,9.5+RAY*o,c)}`;};
    return shown([stroke('ray-'+i,d),{name:'ray-'+i+'-solid',d,style:'solid',type:'stroke',width:2.6}],(t,s)=>rayOut(t,s,i)>.03?1:0);})
 ];
 const icon=make(name,label,intent,parts,light,strong,['sun-interior','sun-frame','sun-solid','ray-0','ray-1','ray-2','ray-3','ray-4']);
 icon.duotoneFill=true;
 return icon;
}
// Staggered ray growth: Light eases to full; Strong overshoots to `over` and settles.
function grow(t,s,i,l,st,over){const d=i*(s?.025:.03),a=(s?st:l)[0]+d,b=(s?st:l)[1]+d;if(t<=a)return 0;if(!s){const u=Math.min(1,(t-a)/(b-a));return u*u*(3-2*u);}if(t<=b){const u=(t-a)/(b-a);return over*(1-(1-u)**3);}const v=Math.min(1,(t-b)/.12);return over+(1-over)*v*v*(3-2*v);}
// Sunrise: from fully below the horizon (hidden) to a half disc; rays grow in once it is up.
const riseY=F([[0,H+R],[.06,H+R],[.5,H,'smooth']],[[0,H+R],[.08,H+R,'hold'],[.3,H-1.1],[.44,H+.2],[.54,H]]);
export const sunrise=horizonSun('sunrise','Sunrise',
 'Light: the sun rises from below the horizon to a half disc and its rays grow in a stagger; the up chevron nudges up. Strong: the sun overshoots 1.1 units above the half position and settles, and the rays flare past full length and settle. Result retained; preview reset rearms.',
 'M13 26.8 L16 24.1 L19 26.8',F([[0,0],[.1,0],[.3,-.6],[.5,0,'smooth']],[[0,0],[.08,.3],[.22,-.9],[.36,.15],[.5,0]]),
 .72,.7,riseY,(t,s,i)=>grow(t,s,i,[.36,.58],[.26,.46],1.3));
// Sunset: from high in the sky (long rays) down to the half disc; rays shorten as it sets.
// Longest ray (Strong prep) keeps ≥2 units of painted margin at the top edge.
const setY=F([[0,H-4],[.06,H-4],[.56,H,'smooth']],[[0,H-4],[.06,H-4.2],[.3,H+.9],[.44,H-.15],[.56,H]]);
const setLen=F([[0,1.35],[.06,1.35],[.5,1,'smooth']],[[0,1.35],[.08,1.4],[.3,.85],[.44,1.05],[.56,1]]);
export const sunset=horizonSun('sunset','Sunset',
 'Light: the sun sinks from high in the sky to a half disc on the horizon while its long rays shorten; the down chevron nudges down. Strong: the sun lifts slightly first, drops 0.9 units past the half disc and settles. Result retained; preview reset rearms.',
 'M13 24.1 L16 26.8 L19 24.1',F([[0,0],[.1,0],[.3,.6],[.5,0,'smooth']],[[0,0],[.08,-.35],[.26,1.2],[.4,-.2],[.54,0]]),
 .56,.62,setY,(t,s)=>setLen(t,s));

export const icons=[wind,tornado,rainbow,sunrise,sunset];
