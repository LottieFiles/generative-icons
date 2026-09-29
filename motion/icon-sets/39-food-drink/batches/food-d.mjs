// Food & Drink batch D: ice-cream, noodle-bowl, bread, recipe-book, restaurant-menu.
import {make,F,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,bothFill,rotate,holes,capsule} from './helpers.mjs';
import {curveFrom} from './kitchen.mjs';

// ---------- ICE CREAM ----------
// A scoop with a dripping hem sits 1.1 clear above a waffle cone. It jiggles about its base:
// Light a soft stretch-and-squash; Strong a hop, a squashed landing and a wobble.
const scoop='M10 14 L10 10.5 C10 7 12.7 4.8 16 4.8 C19.3 4.8 22 7 22 10.5 L22 14 C21 14.8 19.9 14.8 19 14 C18.1 14.8 16.9 14.8 16 14 C15.1 14.8 13.9 14.8 13 14 C12.1 14.8 11 14.8 10 14 Z';
const cone='M9.5 18.2 L22.5 18.2 L16.7 27.6 C16.35 28.1 15.65 28.1 15.3 27.6 Z';
const BASE=[16,14.4];
// Stretch and hop together keep the scoop's Bold paint ≥2 inside the top edge.
const sy=F([[0,1],[.12,1.06],[.26,.95],[.42,1,'smooth']],[[0,1],[.08,.9],[.2,1.1],[.34,.84,'hit'],[.46,1.05],[.56,.98],[.64,1]]);
const hopS=F([[0,0],[.42,0]],[[0,0],[.1,0],[.22,-.5],[.34,0,'hit'],[.64,0]]);
const scoopPose=(x,y,t,s)=>{const k=sy(t,s),w=1/Math.sqrt(k);return [BASE[0]+(x-BASE[0])*w,BASE[1]+(y-BASE[1])*k+hopS(t,s)];};
const iceCream=make('ice-cream','Ice cream',
 'Light: the scoop stretches up, squashes and settles on the cone. Strong: it crouches, stretches up 10% with a small hop, lands squashed to 84% and wobbles to rest; its volume stays constant.',[
 ...posed(framed('scoop',scoop),scoopPose),
 // Waffle line 3.6 below the cone's top edge: v1 at 2.7 fused with it at Bold.
 ...framed('cone',cone),stroke('waffle','M11.72 21.8 L20.28 21.8')
],.42,.64);
iceCream.duotoneFill=true;

// ---------- NOODLE BOWL ----------
// Chopsticks lift two noodles out of the bowl. Each noodle's foot stays on the rim and its top rides the
// chopstick tip, so it stretches as they rise. Light lifts 1.6 units; Strong 2.6 with a springy return.
// The chopsticks pinch at one tip (their shared joint) and open upward-right 3.5 units apart.
const RIM=17,PINCH=[15.6,11.2],FEET=[13.2,18];
const up=F([[0,0],[.1,0],[.34,-1.6],[.62,0,'smooth']],[[0,0],[.06,.4],[.26,-2.6],[.44,-1.9],[.56,-2.2],[.72,0]]);
const noodleD=(i)=>(t,s)=>{const [tx,ty]=PINCH,x0=FEET[i],top=ty+up(t,s);return curveFrom(u=>[x0+(tx-x0)*u+.25*Math.sin(u*Math.PI*3),RIM+(top-RIM)*u],4);};
const bowl='M5.5 17 L26.5 17 C26.5 22.8 21.8 27 16 27 C10.2 27 5.5 22.8 5.5 17 Z';
const noodleBowl=make('noodle-bowl','Noodle bowl',
 'Light: the chopsticks lift 1.6 units and the two noodles stretch up from the rim to the pinch, then lower. Strong: they dip, pull 2.6 units, bob once and lower.',[
 ...framed('bowl',bowl),
 ...tube('noodle-0',noodleD(0)),...tube('noodle-1',noodleD(1)),
 ...posed([...tube('stick-0','M15.6 11.2 L25.8 6.6'),...tube('stick-1','M15.6 11.2 L27.2 9.8')],rigid((t,s)=>({dy:up(t,s)})))
],.62,.72);
noodleBowl.duotoneFill=true;

// ---------- BREAD ----------
// A loaf proves: from a low dough it rises to a full domed loaf; its scores ride the surface.
// Strong overshoots (oven spring) and settles. Result retained; preview reset rearms.
const loaf='M5.5 26.2 L26.5 26.2 L26.5 20.5 C26.5 14 21.8 10.5 16 10.5 C10.2 10.5 5.5 14 5.5 20.5 Z';
const rise=F([[0,.6],[.06,.6],[.66,1,'smooth']],[[0,.6],[.08,.6,'hold'],[.32,1.1],[.46,.97],[.58,1]]);
const loafPose=(x,y,t,s)=>{const k=rise(t,s);return [16+(x-16)*(1+(1-k)*.12),26.2-(26.2-y)*k];};
// Scores sit ≥1 inside the dome at Bold and open (grow from their midpoints) as the loaf rises,
// so the squashed dough never crowds them against the crust.
const SCORES=[[[11.2,18.4],[13,16.4]],[[15.1,17.2],[16.9,15.2]],[[19,18.4],[20.8,16.4]]];
const open=(t,s)=>Math.max(0,Math.min(1,(rise(t,s)-.72)/.26));
const scoreD=([a,b])=>(t,s)=>{const g=Math.max(.01,open(t,s)),m=[(a[0]+b[0])/2,(a[1]+b[1])/2];return `M${m[0]+(a[0]-m[0])*g} ${m[1]+(a[1]-m[1])*g} L${m[0]+(b[0]-m[0])*g} ${m[1]+(b[1]-m[1])*g}`;};
const bread=make('bread','Bread',
 'Light: the dough rises from a low mound into a domed loaf. Strong: it springs up past full height by 10% and settles. The risen loaf is retained; preview reset rearms.',
 // Solid keeps the scores as real counters at full length; the loaf's own squash keeps them inside it.
 posed([...framed('loaf',loaf,holes(loaf,...SCORES.map(([a,b])=>capsule(a,b,1.9)))),...SCORES.map((d,i)=>({...stroke('score-'+i,scoreD(d)),visibility:(t,s)=>open(t,s)>.02?1:0}))],loafPose),.66,.58,['loaf-interior','loaf-frame','loaf-solid','score-0','score-1','score-2']);
bread.duotoneFill=true;

// ---------- RECIPE BOOK ----------
// A cookbook with a spoon on its cover and a ribbon bookmark hanging from the bottom edge.
// The book is set down: it taps (Light) or hops and lands (Strong) and the ribbon swings from its seam.
const cover=rect(8.5,4.8,15.5,20.4,2);
const bookY=F([[0,0],[.1,-.5],[.24,0,'hit'],[.3,0]],[[0,0],[.12,-1.4],[.24,0,'hit'],[.3,0]]);
const swing=F([[0,0],[.24,0],[.36,14],[.52,-8],[.68,0,'smooth']],[[0,0],[.24,0],[.32,30,'out'],[.46,-20],[.6,10],[.74,-3],[.84,0]]);
const bookPose=rigid((t,s)=>({dy:bookY(t,s)}));
const recipeBook=make('recipe-book','Recipe book',
 'Light: the cookbook taps down and its ribbon swings 14° from the seam. Strong: it hops 1.4 units, lands and the ribbon swings 30° with a damped back-and-forth.',[
 ...posed([...framed('cover',cover,holes(cover,rect(12.8,7.4,9,15.2,1))),stroke('spine','M11.8 4.8 L11.8 25.2'),
  ...bothFill('spoon-bowl','M17.4 8.8 C18.8 8.8 19.6 10.1 19.6 11.6 C19.6 13.1 18.6 14.2 17.4 14.2 C16.2 14.2 15.2 13.1 15.2 11.6 C15.2 10.1 16 8.8 17.4 8.8 Z'),
  ...tube('spoon-handle','M17.4 14.2 L17.4 19.6')],bookPose),
 ...posed(tube('ribbon','M20.4 25.2 L20.4 28.4'),rigid((t,s)=>({dy:bookY(t,s),angle:swing(t,s),pivot:[20.4,25.2]})))
],.68,.84);
recipeBook.duotoneFill=true;

// ---------- RESTAURANT MENU ----------
// A bi-fold menu opens: the front cover swings about the centre hinge (width × cos) from lying over the
// right page to the left. Its front title shows while facing up, its inside item lines once it has turned;
// the right page's items appear as they are uncovered. Result retained; preview reset rearms.
const HX=16,PAGE=rect(16,5.5,10.5,21,1.6),PAGE_S=holes(rect(16,5.5,10.5,21,1.6),rect(17.3,6.8,7.9,18.4,.4));
const ITEMS=[['M18.6 11 L23.9 11',11],['M18.6 15 L23.9 15',15],['M18.6 19 L22.4 19',19]];
const flip=F([[0,0],[.06,0],[.56,1,'smooth']],[[0,0],[.06,-.04],[.34,1.08],[.46,.97],[.56,1]]);
const scaleX=(t,s)=>Math.cos(Math.PI*Math.min(1.08,flip(t,s)))*(flip(t,s)>1?1+(flip(t,s)-1)*1.2:1);
const coverPose=(x,y,t,s)=>[HX+(x-HX)*scaleX(t,s),y];
// Right-page items (and its tint) show only once the cover is past the fold, so nothing shows through it.
const facing=(t,s)=>scaleX(t,s)>.12?1:0, turned=(t,s)=>scaleX(t,s)<-.12?1:0, uncovered=turned;
const restaurantMenu=make('restaurant-menu','Restaurant menu',
 'Light: the menu\'s cover swings open about its centre fold, revealing items on both pages. Strong: it lifts, swings past flat by 8% and settles open. The open menu is retained; preview reset rearms.',[
 ...(([tint,...rest])=>[{...tint,visibility:uncovered},...rest])(framed('page',PAGE,PAGE_S)),
 ...ITEMS.flatMap(([d],i)=>shown(tube('item-'+i,d),uncovered)),
 ...posed([...framed('cover',PAGE,PAGE_S),
  ...shown([...tube('title-0','M18.6 10.2 L23.9 10.2'),...tube('title-1','M18.6 13.4 L22 13.4')],facing),
  ...ITEMS.flatMap(([d],i)=>shown(tube('inside-'+i,d),turned))],coverPose)
],.56,.56,['page-interior','cover-interior','cover-frame','cover-solid','title-0','title-0-solid','title-1','title-1-solid','inside-0','inside-0-solid','inside-1','inside-1-solid','inside-2','inside-2-solid','item-0','item-0-solid','item-1','item-1-solid','item-2','item-2-solid']);
restaurantMenu.duotoneFill=true;

export const icons=[iceCream,noodleBowl,bread,recipeBook,restaurantMenu];
