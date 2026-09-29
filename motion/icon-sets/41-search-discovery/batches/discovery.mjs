// Search & discovery batch C (discovery objects): telescope, related-items, web-search.
import {make,toggle,K,ring,tube,fill,stroke,posed,shown,rigid,circle,polygon,bothFill,F,ellipse,sparkle,place,deg} from './helpers.mjs';

// ---------- TELESCOPE ----------
// A tapered tube on a post and base, aimed 30° up at a sparkle. The tube tilts about the post top
// (its underside passes through the pivot, so the post cap stays buried). Light lifts 5° and lowers;
// Strong lifts 13° with a wind-up, holds on the find and lowers. The sparkle twinkles in reply.
const TUBE=place(polygon([[-8.5,-4.2],[8.5,-5.4],[8.5,.2],[-8.5,-1]],[1,1.2,1.2,1]),-30,16,18);
const tilt=F([[0,0],[.06,0],[.26,-5],[.6,0,'smooth']],[[0,0],[.06,3],[.28,-13],[.4,-10,'hold'],[.56,-10,'hold'],[.8,0]]);
const twinkle=F([[0,1],[.2,1],[.34,1.3],[.55,1,'smooth']],[[0,1],[.24,1],[.34,1.6,'hit'],[.46,.9],[.58,1]]);
const STAR=[25.5,6.5];
const telescope=make('telescope','Telescope',
 'Light: the telescope tilts 5° up toward the sparkle, which twinkles, then lowers. Strong: it dips, swings 13° up, holds on the find while the sparkle flares to 160%, then lowers to rest.',[
 ...posed(ring('tube',TUBE),rigid((t,s)=>({angle:tilt(t,s),pivot:[16,18]}))),
 ...tube('post','M15.8 17.65 L15.8 27.5'),
 ...tube('base','M11 27.5 L21 27.5'),
 ...posed(bothFill('star',sparkle(...STAR,2.6)),(x,y,t,s)=>{const k=twinkle(t,s);return [STAR[0]+(x-STAR[0])*k,STAR[1]+(y-STAR[1])*k];})
],.6,.8);
telescope.duotoneFill=true;

// ---------- RELATED ITEMS ----------
// A hub ring with three linked items. Discovering related items sends them out along their spokes
// from radius 5 (hidden inside the hub) to 11; they fade in over the first 40% of the travel.
// Strong pops them to 11.6 and settles. Reverse draws them back in and fades them out.
const ANG=[-90,30,150];
const relK=K([[0,0],[.06,0],[.6,1,'smooth']],[[0,0],[.06,-.05],[.3,1.1],[.44,.94],[.56,1.02],[.7,1]],[[0,1],[.5,0,'smooth']],[[0,1],[.05,1,'hold'],[.3,-.04],[.42,.02],[.55,0]]);
const radius=k=>5+6*k,alpha=k=>Math.min(1,Math.max(0,k/.4));
const relatedItems=toggle('related-items','Related items',
 'Light: three items slide out from the hub along their spokes and fade in. Strong: they pop out past their positions with a bounce and settle. Reverse draws them back into the hub.',[
 fill('hub-interior',circle(16,16,3.3),'tint'),stroke('hub',circle(16,16,3.3)),fill('hub-solid',circle(16,16,4.55),'solid'),
 ...ANG.flatMap((a,i)=>{const c=Math.cos(deg(a)),sn=Math.sin(deg(a)),vis=(t,s,r)=>alpha(relK(t,s,r));
  return [...shown(tube('link-'+i,(t,s,r)=>{const R=radius(relK(t,s,r));return `M${16+3.3*c} ${16+3.3*sn} L${16+R*c} ${16+R*sn}`;}),vis),
   ...posed(shown(bothFill('item-'+i,circle(16+11*c,16+11*sn,2.2)),vis),rigid((t,s,r)=>{const R=radius(relK(t,s,r));return {dx:(R-11)*c,dy:(R-11)*sn};}))];})
],.62,.72,'Single','Related',.52,.57);
relatedItems.duotoneFill=true;

// ---------- WEB SEARCH ----------
// A globe (rim, equator, two meridians 90° apart) with a small magnifier at its lower right. The globe
// turns: each meridian's apparent width follows |cos| of its longitude, so a 90° (Light) or 180° (Strong)
// turn ends on the same drawing. Strong also taps the magnifier onto the globe with a hit.
const theta=F([[0,45],[.06,45],[.6,135,'smooth']],[[0,45],[.06,36],[.5,225]]);
const meridian=off=>(t,s)=>{const rx=Math.round(Math.abs(Math.cos(deg(theta(t,s)+off)))*7.5*1e6)/1e6;return ellipse(12,12,rx,7.5);};
const tap=F([[0,0],[.6,0]],[[0,0],[.2,0],[.3,1,'hit'],[.46,0]]);
const webSearch=make('web-search','Web search',
 'Light: the globe turns a quarter turn, its meridians sweeping across the disc. Strong: it winds back and spins a half turn while the magnifier taps onto it with a hit.',[
 ...ring('globe',circle(12,12,7.5)),
 ...tube('equator','M4.5 12 L19.5 12'),
 ...tube('meridian-0',meridian(0)),...tube('meridian-1',meridian(90)),
 ...posed([...ring('lens',circle(23,23,3.8)),...tube('handle','M25.7 25.7 L27.6 27.6')],rigid((t,s)=>({dy:tap(t,s)})))
],.62,.54);
webSearch.duotoneFill=true;

export const icons=[telescope,relatedItems,webSearch];
