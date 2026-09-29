// Tables and analytics · batch A (charts), revision 02. 32-unit grid, tube strokes 2 / Solid 2.6, Solid exteriors expanded 1.3.
import {make,F,tube,stroke,fill,framed,bothFill,posed,shown,rect,circle,capsule,holes,polygon,rigid,geometricRearm} from './helpers.mjs';
const deg=a=>a*Math.PI/180;
const flag=(icon,duotoneFill)=>Object.assign(icon,{duotoneFill});

// 1 SCATTER PLOT · result state, geometricRearm. Fixed: axes. Moving: 4 rigid dots, each on its own vertical column.
// Every route is a straight drop in its own column; no traveller ever passes within 1 unit of another dot, route or axis
// (worst cases: dot1 top (20,12) vs dot0 (24,9) = 5.0 centre distance, gap 1.8; dot3 top (10.5,16) vs dot2 (15,14) = 4.92, gap 1.72).
const dots=[[24,9],[20,17],[15,14],[10.5,21]],DROP=5;
export const scatterPlot=flag(geometricRearm(make('scatter-plot','Scatter plot',
 'Points plot onto fixed x/y axes, each dropping into its own column. Light lowers each point 1.5 units while it fades in. Strong drops each point 5 units with a 0.15 unit pull-up and a decisive landing exactly on its value, 0.1 s stagger, no overshoot. Reset retracts the same columns.',
 [...tube('axes','M6 6 L6 26 L26 26'),
  ...dots.flatMap(([x,y],i)=>{const d=i*.1;
   const q=F([[d,0],[d+.34,1,'smooth']],[[d,0],[d+.05,-.03,'launch'],[d+.25,1]]);
   const v=F([[d,0],[d+.16,1,'smooth']],[[d,0],[d+.08,1,'out']]);
   return shown(posed(bothFill('dot-'+i,circle(x,y,1.6)),rigid((t,s)=>({dy:-(s?DROP:1.5)*(1-q(t,s))}))),v);})],
 .7,.6)),false);

// 2 HEATMAP · gesture. Centres 7.5/16/24.5 (8.5 spacing). Cold: outlined 4.5 squares (tinted interior), Solid ring 7.1 outer / 3.0 hole.
// Hot: filled squares with distinct rest sizes; peak swell 6.6 keeps >=1.65 to cold Solid neighbours, >=2.3 hot-to-hot.
const C=[7.5,16,24.5],HOT={'00':4.5,'01':5.2,'11':6,'22':5.2},ORDER=['00','01','11','22'],heatParts=[];
C.forEach((cy,r)=>C.forEach((cx,c)=>{const key=r+''+c,w0=HOT[key];
 if(w0==null){heatParts.push(...framed('cell-'+key,rect(cx-2.25,cy-2.25,4.5,4.5,1),holes(rect(cx-3.55,cy-3.55,7.1,7.1,2.3),rect(cx-1.5,cy-1.5,3,3,.6))));return;}
 const h=ORDER.indexOf(key),d=h*.09;
 const g=F(h===2?[[0,w0],[.24,w0+.5,'smooth'],[.5,w0,'smooth']]:[[0,w0]],[[d,w0],[d+.07,w0-.6,'launch'],[d+.2,w0+.6],[d+.42,w0]]);
 heatParts.push(...bothFill('hot-'+key,(t,s)=>{const w=g(t,s);return rect(cx-w/2,cy-w/2,w,w,1);}));
}));
export const heatmap=flag(make('heatmap','Heatmap',
 'A 3x3 value matrix: hot cells filled at three sizes, cool cells outlined. Light breathes only the hottest centre cell by 0.5 units. Strong runs a diagonal heat wave: each hot cell dips 0.6 then swells 0.6 and settles, 0.09 s apart.',
 heatParts,.5,.7),true);

// 3 FUNNEL CHART · gesture. Unchanged drawing. Strong: 0.375 lift, 0.10 s drive to 1.5, one damped 0.15 rebound, settle. Stagger 0.03 keeps the top-to-middle delta <=0.75 at the top peak.
const stages=[[5,5,22],[8.5,14,15],[12,23,8]],AMP_L=[.7,.5,.3],AMP_S=[1.5,1.1,.7];
export const funnelChart=flag(make('funnel-chart','Funnel chart',
 'Three centred stage bars narrow downward. Light: a soft downward pulse travels top to bottom. Strong: the top stage lifts, drives down 1.5 units in 0.1 s, rebounds 0.15 and settles; lower stages follow 30 ms later and smaller.',
 stages.flatMap(([x,y,w],i)=>{const d=i*.03;
  const q=F([[0,0],[.22,1,'smooth'],[.5,0,'smooth']],[[d,0],[d+.07,-.25,'launch'],[d+.17,1],[d+.32,-.1],[d+.48,0]]);
  return posed(framed('stage-'+i,rect(x,y,w,4.5,2.25),rect(x-1.3,y-1.3,w+2.6,7.1,3.55)),rigid((t,s)=>({dy:(s?AMP_S[i]:AMP_L[i])*q(t,s)})));}),
 .5,.6),true);

// 4 RADAR CHART · result state, geometricRearm. Three radial axes with an open centre (spokes r 4.5..12, hub left empty).
// Moving: one closed value triangle, vertices on the axes; each vertex plots 30 ms after the previous. Only its interior is tinted.
const AX=[-90,30,150],V=[9,6.5,8],pt=(i,r)=>[16+r*Math.cos(deg(AX[i])),16+r*Math.sin(deg(AX[i]))];
const k=F([[0,.65],[.5,1,'smooth']],[[0,.65],[.1,.5,'launch'],[.34,1]]);
const web=(t,s)=>polygon(V.map((v,i)=>pt(i,v*k(t-i*.03,s))),.8);
export const radarChart=flag(geometricRearm(make('radar-chart','Radar chart',
 'Three radial axes with an open centre; a closed value triangle sits on them. Light grows the triangle from 65% to full values. Strong pulls it in to 50% then plots each vertex out to its full value with no overshoot.',
 [...AX.flatMap((_,i)=>tube('axis-'+i,`M${pt(i,4.5).join(' ')} L${pt(i,12).join(' ')}`)),
  fill('web-interior',web,'tint'),...tube('web',web)],
 .6,.5)),true);

// 5 DASHBOARD · result state, geometricRearm. Fixed: header tile + KPI bar. Moving: 2 rigid tiles; dy is positive downward, so q 1→0 is upward travel from below.
export const dashboard=flag(geometricRearm(make('dashboard','Dashboard',
 'A wide header widget stays; two data widgets arrive below it. Light: each rises 1.2 units while fading in. Strong: each starts 3.2 units below, dips 0.2 further, rises past by 0.4, and settles; the second widget follows 80 ms later.',
 [...framed('header',rect(5,5,22,7,2),holes(rect(3.7,3.7,24.6,9.6,3.3),capsule([9,8.5],[13,8.5],2.6))),
  stroke('bar','M9 8.5 L13 8.5'),
  ...[[5,'left'],[18,'right']].flatMap(([x,n],i)=>{const d=i*.08;
   const q=F([[d,1],[d+.4,0,'smooth']],[[d,1],[d+.05,1.06,'launch'],[d+.18,-.12],[d+.32,0]]);
   const v=F([[d,0],[d+.2,1,'smooth']],[[d,0],[d+.1,1,'out']]);
   return shown(posed(framed('widget-'+n,rect(x,16,9,9,2),rect(x-1.3,14.7,11.6,11.6,3.3)),rigid((t,s)=>({dy:(s?3.2:1.2)*q(t,s)}))),v);})],
 .5,.45)),true);

export const icons=[scatterPlot,heatmap,funnelChart,radarChart,dashboard];
