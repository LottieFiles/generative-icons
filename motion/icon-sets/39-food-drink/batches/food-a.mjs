// Food & Drink batch A: coffee-cup, tea-cup, wine-glass, beer-mug, cocktail.
import {make,F,P,tube,framed,rigid,posed,shown,fill,stroke,circle,polygon,rotate} from './helpers.mjs';
import {arcN,wispD,recycle,L} from './kitchen.mjs';

// ---------- COFFEE CUP ----------
// Mug x7–21 (handle to 25.8). Two steam wisps rise, fade and recycle while invisible. Wisp paint clears
// the rim by 1.3 at Bold. Strong: faster, taller rise and the wisps lean as they go.
const WISPS=[[11.4,0],[16.6,.1]];
// v1 wisps (3.8 long, 1.3 wiggle) read as S-blobs at Bold/Solid; the mug moved down 1 so they can be
// longer (4.6) and calmer (0.9 wiggle).
const mugBody='M7 14.4 L21 14.4 L21 22.8 C21 25.4 18.9 27.5 16.3 27.5 L11.7 27.5 C9.1 27.5 7 25.4 7 22.8 Z';
const coffeeCup=make('coffee-cup','Coffee cup',
 'Light: two steam wisps drift up 1.6 units, fade and reform above the mug. Strong: the wisps shoot up 2.4 units in 0.3 s and lean as they rise before reforming.',[
 ...framed('body',mugBody),
 ...tube('handle','M21 16.6 C24.6 16.6 25.8 17.6 25.8 19.9 C25.8 22.1 24.5 23.4 21 23.4'),
 ...WISPS.flatMap(([x,d],i)=>{const {rise,show}=recycle(P,d,[1.6,2.4],[.5,.2],[.3,.18,.04]);
  const lean=F([[0,0]],[[0,0],[.04+d,0],[.34+d,1],[.37+d,1],[.38+d,0,'linear']]);
  return shown(tube('wisp-'+i,wispD(x,10.6,4.6,.9)),show).map(p=>({...p,pose:(px,py,t,s)=>[px+lean(t,s)*.35*(10.6-py),py-rise(t,s)]}));})
],.8,.62);
coffeeCup.duotoneFill=true;

// ---------- TEA CUP ----------
// A spoon stands in the cup, its foot on the rim. Stirring: the foot travels along the rim while the
// handle's top traces a small loop. Light one loop; Strong two faster loops and the cup jiggles.
const teaCup='M6.5 13.5 L23.5 13.5 C23.5 19 20 23 15 23 C10 23 6.5 19 6.5 13.5 Z';
const stir=F([[0,0],[.06,0],[.76,360,'smooth']],[[0,0],[.04,-20],[.66,720],[.78,720]]);
const jiggle=F([[0,0],[.8,0]],[[0,0],[.2,.5],[.34,-.45],[.48,.3],[.62,-.15],[.78,0]]);
const spoonD=(t,s)=>{const a=stir(t,s)*Math.PI/180,xb=15+2.6*Math.cos(a),xt=xb+3+1.2*Math.sin(a);return `M${xb} 13.5 L${xt} 5.6`;};
const teaPose=rigid((t,s)=>({dx:jiggle(t,s)}));
const teaCupIcon=make('tea-cup','Tea cup',
 'Light: the spoon stirs one loop, its foot sliding along the rim. Strong: it stirs two quick loops after a short backstroke and the cup jiggles on its saucer.',[
 ...posed([...framed('cup',teaCup),...tube('handle','M23.3 16 C25.9 15.8 27 17 26.5 18.5 C26.1 19.7 24.6 20.4 21.2 20.2'),...tube('spoon',spoonD)],teaPose),
 ...tube('saucer','M5.5 26.8 L24.5 26.8')
],.76,.78);
teaCupIcon.duotoneFill=true;

// ---------- WINE GLASS ----------
// Bowl: circle (16,9.2) r7.4 below the rim chord y5.6. Wine is poured in: the level rises from a low
// segment to y10.4; its surface ends ride the bowl's centreline. The pour stream shows only while pouring.
const BC=[16,9.2],BR=7.4,RIM=5.6;
const bowlD=(()=>{const a=Math.asin((RIM-BC[1])/BR)*180/Math.PI,w=Math.sqrt(BR*BR-(RIM-BC[1])**2);return `M${16-w} ${RIM} L${16+w} ${RIM} ${L(arcN(16,9.2,7.4,a,180-a,4))} Z`;})();
const wine=F([[0,15.2],[.06,15.2],[.62,10.4,'smooth']],[[0,15.2],[.06,15.2,'hold'],[.34,10.4]]);
const wslosh=(t,s)=>{const u=Math.max(0,t-.06),T=s?.66:.6;return u>=T?0:(s?1.2:.5)*Math.sin(u*(s?18:13))*Math.exp(-u*3.5)*(1-u/T)**1.5;};
const wsurf=(t,s)=>{const y=wine(t,s),w=Math.sqrt(BR*BR-(y-BC[1])**2),a=wslosh(t,s);return {y,w,a};};
const wineSurfD=(t,s)=>{const {y,w,a}=wsurf(t,s);return `M${16-w} ${y} C${16-w/3} ${y-a} ${16+w/3} ${y+a} ${16+w} ${y}`;};
const wineD=(t,s)=>{const {y}=wsurf(t,s),th=Math.asin((y-BC[1])/BR)*180/Math.PI;return wineSurfD(t,s)+' '+L(arcN(16,9.2,7.4,th,180-th,3))+' Z';};
const pouring=(t,s)=>t>.02&&t<(s?.3:.56)?1:0;
const dip=F([[0,0],[.6,0]],[[0,0],[.08,0],[.34,.5,'hit'],[.48,-.1],[.6,0]]);
const glassPose=rigid((t,s)=>({dy:dip(t,s)}));
const wineGlass=make('wine-glass','Wine glass',
 'Light: a stream pours in and the wine level rises to the bowl\'s widest part with a small slosh. Strong: it fills in 0.28 s, the glass dips half a unit on the pour and the surface sloshes harder before settling. The filled glass is retained; preview reset rearms.',[
 ...posed([stroke('bowl',bowlD),{name:'bowl-solid',d:bowlD,style:'solid',type:'stroke',width:2.6},
  fill('wine-interior',wineD,'tint'),fill('wine-solid',wineD,'solid'),...tube('surface',wineSurfD),
  ...tube('stem','M16 16.6 L16 26.4'),...tube('foot','M11.2 26.6 L20.8 26.6')],glassPose),
 ...shown(tube('stream',(t,s)=>`M16.6 3.6 L16.6 ${Math.max(4.2,wine(t,s)-3.4)}`),pouring)
],.66,.74,['wine-interior','wine-solid','surface','stream']);
wineGlass.duotoneFill=true;

// ---------- BEER MUG ----------
// Glass x7–20.5 with a scalloped foam head sharing its top edge. The head swells upward from that edge.
// Strong: the mug is raised in a toast (8° about its base corner) and the head overshoots.
const beerGlass='M7 11.8 L20.5 11.8 L20.5 24.5 C20.5 25.6 19.6 26.5 18.5 26.5 L9 26.5 C7.9 26.5 7 25.6 7 24.5 Z';
const foam='M6.2 11.8 C4.9 11.8 4.6 9.2 6.4 8.8 C6.6 6.6 8.9 5.6 10.6 6.8 C11.8 5.2 14.6 5.1 15.8 6.7 C17.3 5.5 20 6 20.4 8.2 C22.2 8.2 22.8 11.8 21.2 11.8 Z';
const swell=F([[0,0],[.14,.14],[.42,0,'smooth']],[[0,0],[.1,-.04],[.28,.16],[.42,.04],[.56,.07],[.68,0]]);
// Raised foam and toast together keep the head's paint ≥2 inside the top edge.
const toast=F([[0,0],[.5,0]],[[0,0],[.06,-1.5],[.26,6],[.44,3.8],[.6,.6],[.72,0]]);
const mugPose=(x,y,t,s)=>rotate(x,y,toast(t,s),20.5,26.5);
const foamPose=(x,y,t,s)=>{const k=swell(t,s),w=Math.max(0,11.8-y)/6.7;return mugPose(13.7+(x-13.7)*(1+k*.18*w),11.8-(11.8-y)*(1+k),t,s);};
const beerMug=make('beer-mug','Beer mug',
 'Light: the foam head swells up 14% and settles back. Strong: the mug is raised 6° in a toast, the head surges 16% and settles as the mug comes back down.',[
 ...posed(framed('glass',beerGlass),mugPose),
 ...posed([stroke('foam',foam),fill('foam-solid',foam,'solid')],foamPose),
 ...posed(tube('handle','M20.5 14.5 L23.2 14.5 C24.8 14.5 25.8 15.5 25.8 17.1 L25.8 20.4 C25.8 22 24.8 23 23.2 23 L20.5 23'),mugPose),
 ...posed([stroke('ridge-0','M11.2 15.6 L11.2 22.8'),stroke('ridge-1','M16.2 15.6 L16.2 22.8')],mugPose)
],.42,.72);
beerMug.duotoneFill=true;

// ---------- COCKTAIL ----------
// Martini glass tilts about its bowl centre in a toast; the drink's surface stays level in the world and
// lags with a small slosh. Liquid = the triangle below the surface, computed in the glass's own frame.
const PIV=[16,15],RL=[6,7],RR=[26,7],TIP=[16,17.4],P0=[16,10.6];
const tilt=F([[0,0],[.14,6],[.46,0,'smooth']],[[0,0],[.08,-4],[.26,14,'hit'],[.42,-5],[.56,2],[.68,0]]);
const lag=F([[0,0],[.2,-2],[.5,0,'smooth']],[[0,0],[.14,2],[.32,-6],[.5,3],[.66,0]]);
const hit=(a,b)=>{ // surface line through P0 at angle phi (local) meets wall a→b
 return phi=>{const dx=Math.cos(phi),dy=Math.sin(phi),ex=b[0]-a[0],ey=b[1]-a[1],den=dx*ey-dy*ex,u=((a[0]-P0[0])*ey-(a[1]-P0[1])*ex)/den;return [P0[0]+u*dx,P0[1]+u*dy];};};
const hitL=hit(RL,TIP),hitR=hit(RR,TIP);
const surfPts=(t,s)=>{const phi=(-tilt(t,s)+lag(t,s))*Math.PI/180;return [hitL(phi),hitR(phi)];};
const pose=(x,y,t,s)=>rotate(x,y,tilt(t,s),...PIV);
const glassD=polygon([RL,RR,TIP],[.8,.8,1.1]);
const cocktail=make('cocktail','Cocktail',
 'Light: the glass tips 6° in a toast while the drink stays level. Strong: it draws back, tips 14° with a hard stop and the drink sloshes against the wall before both settle.',[
 ...posed([stroke('glass',glassD),{name:'glass-solid',d:glassD,style:'solid',type:'stroke',width:2.6},
  fill('drink-interior',(t,s)=>{const [a,b]=surfPts(t,s);return `M${a} L${b} L${TIP} Z`;},'tint'),
  fill('drink-solid',(t,s)=>{const [a,b]=surfPts(t,s);return `M${a} L${b} L${TIP} Z`;},'solid'),
  ...tube('surface',(t,s)=>{const [a,b]=surfPts(t,s);return `M${a} L${b}`;}),
  ...tube('stem','M16 17.4 L16 26.2'),...tube('foot','M11.5 26.4 L20.5 26.4')],pose)
],.46,.68);
cocktail.duotoneFill=true;

export const icons=[coffeeCup,teaCupIcon,wineGlass,beerMug,cocktail];
