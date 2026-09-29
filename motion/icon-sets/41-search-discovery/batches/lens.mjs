// Search & discovery batch A (magnifier family): recent-searches, search-voice, search-filter, saved-search,
// search-match, searching, search-scope, search-image.
import {make,toggle,continuous,K,FWD,ring,tube,fill,posed,shown,rigid,circle,rect,polygon,arc,mix,clamp,drive,F,spin,deg} from './helpers.mjs';

// Family magnifier: lens r8.5 at (13.5,13.5), handle from the 45° rim point to (26,26). Paint stays ≥2.75 inside the canvas.
const C=[13.5,13.5],R=8.5;
const magnifier=()=>[...ring('lens',circle(...C,R)),...tube('handle','M19.5 19.5 L26 26')];

// ---------- RECENT SEARCHES ----------
// A clock face inside the lens. The hands rewind: Light wags the minute hand back 60° and returns;
// Strong winds up 14° then turns the minute hand one full turn backwards while the hour hand recoils.
const minute=F([[0,0],[.06,0],[.3,-60,'out'],[.6,0,'smooth']],[[0,0],[.06,14],[.44,-346],[.56,-360]]);
const hour=F([[0,0],[.6,0]],[[0,0],[.06,3],[.36,-16],[.56,0]]);
const recentSearches=make('recent-searches','Recent searches',
 'Light: the minute hand inside the lens swings back 60° and returns. Strong: it winds up, turns one full revolution backwards and the hour hand recoils before both settle.',[
 ...magnifier(),
 ...posed(tube('hour',`M13.5 13.5 L${13.5+3.2*Math.cos(deg(-150))} ${13.5+3.2*Math.sin(deg(-150))}`),rigid((t,s)=>({angle:hour(t,s),pivot:C}))),
 ...posed(tube('minute','M13.5 13.5 L13.5 8.3'),rigid((t,s)=>({angle:minute(t,s),pivot:C})))
],.6,.58);
recentSearches.duotoneFill=true;

// ---------- SEARCH VOICE ----------
// Three voice bars inside the lens (pitch 3.7 keeps 1.2 clear at Bold). A wave passes left to right:
// Light bulges each bar 1.2; Strong 2.2 with a crouch and a bounce. Tips stay ≥1 inside the rim.
const BARS=[[9.8,2],[13.5,3.5],[17.2,2]];
const wave=i=>F([[0,0],[.05+.08*i,0],[.2+.08*i,1],[.5+.08*i,0,'smooth']],[[0,0],[.04+.06*i,-.15],[.16+.06*i,1],[.3+.06*i,-.25],[.42+.06*i,.1],[.54+.06*i,0]]);
const searchVoice=make('search-voice','Search voice',
 'Light: a wave runs across the three voice bars in the lens, each bulging 1.2 units. Strong: the bars crouch, spike 2.2 units in turn and bounce before settling.',[
 ...magnifier(),
 ...BARS.flatMap(([x,h],i)=>{const w=wave(i);return tube('bar-'+i,(t,s)=>{const H=h+(s?2.2:1.2)*w(t,s);return `M${x} ${13.5-H} L${x} ${13.5+H}`;});})
],.68,.68);
searchVoice.duotoneFill=true;

// ---------- SEARCH FILTER ----------
// Three equal lines in the lens narrow into a funnel (8 / 5 / 2). Strong presses the whole magnifier
// down 0.8 with a hit as the lines snap past and settle. Reverse widens them again.
const filterK=K([[0,0],[.06,0],[.5,1,'smooth']],[[0,0],[.06,-.06],[.26,1.1],[.4,.96],[.5,1]],[[0,1],[.45,0,'smooth']],[[0,1],[.3,-.05],[.44,0]]);
const press=FWD([[0,0]],[[0,0],[.06,0],[.2,.8,'hit'],[.34,0]]);
const FLINES=[[9.5,8,8],[13.5,8,5],[17.5,8,2]];
const searchFilter=toggle('search-filter','Search filter',
 'Light: the three lines inside the lens narrow into a funnel. Strong: the magnifier presses down with a hit while the lines snap past their funnel widths and settle. Reverse widens the lines back.',[
 ...posed([...magnifier(),...FLINES.flatMap(([y,a,b],i)=>tube('line-'+i,(t,s,r)=>{const L=mix(a,b,filterK(t,s,r));return `M${13.5-L/2} ${y} L${13.5+L/2} ${y}`;}))],rigid((t,s,r)=>({dy:press(t,s,r)})))
],.52,.52,'Unfiltered','Filtered',.46,.46);
searchFilter.duotoneFill=true;

// ---------- SAVED SEARCH ----------
// A bookmark ribbon inside the lens fills when the search is saved. The ribbon dips as it is pressed:
// Light 0.6, Strong 1.2 with a hit. Its corners stay ≥0.7 inside the rim at the deepest dip.
const RIB=polygon([[11.25,10],[15.75,10],[15.75,17],[13.5,15.3],[11.25,17]],[.6,.6,.5,.5,.5]);
const fillK=K([[0,0],[.12,0],[.4,1,'smooth']],[[0,0],[.14,0,'hold'],[.26,1]],[[0,1],[.3,0,'smooth']],[[0,1],[.06,1,'hold'],[.24,0]]);
const dip=(t,s,r)=>r?drive(t,s,[[0,0],[.15,-.4],[.4,0,'smooth']],[[0,0],[.12,-.8],[.3,.15],[.45,0]]):drive(t,s,[[0,0],[.08,0],[.24,.6],[.5,0,'smooth']],[[0,0],[.06,-.3],[.2,1.2,'hit'],[.34,-.2],[.46,.1],[.6,0]]);
const savedSearch=toggle('saved-search','Saved search',
 'Light: the bookmark in the lens dips 0.6 and fills as the search is saved. Strong: it is pressed 1.2 with a hit, rebounds and the fill snaps in. Reverse lifts the ribbon and empties it.',[
 ...magnifier(),
 ...posed([...tube('ribbon',RIB),fill('mark',RIB,'outline',{visibility:fillK}),fill('mark-solid',RIB,'solid',{visibility:fillK})],rigid((t,s,r)=>({dy:dip(t,s,r)})))
],.52,.62,'Unsaved','Saved',.42,.47);
savedSearch.duotoneFill=true;

// ---------- SEARCH MATCH ----------
// A check draws inside the lens from its left end to its tip (fixed topology; the unreached segment
// collapses at the pen). Strong draws it in 0.28 s and stamps the magnifier down as it completes.
const CA=[9.6,13.9],CB=[12.6,16.9],CC=[18,11.3];
const checkK=K([[0,0],[.08,0],[.55,1,'smooth']],[[0,0],[.1,0,'hold'],[.38,1]],[[0,1],[.4,0,'smooth']],[[0,1],[.04,1,'hold'],[.3,0]]);
const checkD=(t,s,r)=>{const p=checkK(t,s,r),u=Math.min(1,p/.42),v=Math.max(0,(p-.42)/.58);const b=[CA[0]+(CB[0]-CA[0])*u,CA[1]+(CB[1]-CA[1])*u],c=[b[0]+(CC[0]-CB[0])*v,b[1]+(CC[1]-CB[1])*v];return `M${CA} L${b} L${c}`;};
const stamp=FWD([[0,0]],[[0,0],[.3,0],[.4,.9,'hit'],[.56,0]]);
const searchMatch=toggle('search-match','Search match',
 'Light: a check draws inside the lens from left to tip. Strong: it draws in 0.28 s and the magnifier stamps down 0.9 with a hit as the tip lands. Reverse erases the check from its tip.',[
 ...posed([...magnifier(),...shown(tube('check',checkD),(t,s,r)=>checkK(t,s,r)>.02?1:0)],rigid((t,s,r)=>({dy:stamp(t,s,r)})))
],.57,.58,'No match','Matched',.42,.32);
searchMatch.duotoneFill=true;

// ---------- SEARCHING ----------
// Continuous indicator: a 270° arc orbits inside the lens at constant speed (one turn per cycle in
// Light, two in Strong), so the wrap is position- and velocity-continuous.
const orbit=(t,s)=>(s?720:360)*t/2.4;
const searching=continuous('searching','Searching',
 'Continuous: a three-quarter arc orbits inside the lens, one turn per cycle in Light and two turns in Strong. Reduced motion shows the still arc.',[
 ...magnifier(),
 ...posed(tube('sweep',arc(13.5,13.5,4.6,-90,180)),rigid((t,s)=>({angle:orbit(t,s),pivot:C})))
],true);

// ---------- SEARCH SCOPE ----------
// Four viewfinder brackets around a smaller magnifier. Narrowing the scope draws the brackets 2.5 in
// (corner 3.5 → 6); Strong overshoots to 6.3 and settles. The handle end stays ≥1.2 from the BR arm.
const scopeK=K([[0,0],[.06,0],[.5,1,'smooth']],[[0,0],[.06,-.06],[.24,1.12],[.38,.96],[.5,1]],[[0,1],[.45,0,'smooth']],[[0,1],[.05,1,'hold'],[.3,-.06],[.44,0]]);
const bracket=a=>(t,s,r)=>{const c=3.5+2.5*scopeK(t,s,r),d=`M${c+5} ${c} L${c+1.2} ${c} C${c+.54} ${c} ${c} ${c+.54} ${c} ${c+1.2} L${c} ${c+5}`;return a?spin(d,a):d;};
const searchScope=toggle('search-scope','Search scope',
 'Light: the four brackets glide 2.5 units in around the magnifier, narrowing the scope. Strong: they flare out slightly, snap in past the narrow position and settle. Reverse widens the scope.',[
 ...[0,90,180,270].flatMap((a,i)=>tube('bracket-'+i,bracket(a))),
 ...ring('lens',circle(14.8,14.8,5.2)),...tube('handle','M18.5 18.5 L22 22')
],.52,.52,'Wide','Narrow',.47,.46);
searchScope.duotoneFill=true;

// ---------- SEARCH IMAGE ----------
// A picture frame with a mountain line; a small magnifier sweeps across the sky and returns.
// Light: 9.4 units and back. Strong: a crouch, a fast 9.9 sweep to a hit, a short hold, then the return.
const FRAME=rect(3.5,4,25,23,2.5);
const sweep=F([[0,0],[.06,0],[.36,9.4,'smooth'],[.7,0,'smooth']],[[0,0],[.05,-.6],[.22,9.9,'hit'],[.32,9.2],[.44,9.5,'hold'],[.7,0]]);
const searchImage=make('search-image','Search image',
 'Light: the small magnifier sweeps 9.4 units across the picture and glides back. Strong: it crouches, shoots across to a hard stop, holds on the find and returns.',[
 ...ring('frame',FRAME),
 ...tube('mountain','M8 23.5 L12.5 18.5 L16 22 L19.5 19 L24 23.5'),
 ...posed([...tube('lens',circle(10.8,11,3.4)),...tube('handle','M13.2 13.4 L15.2 15.4')],rigid((t,s)=>({dx:sweep(t,s)})))
],.7,.72);
searchImage.duotoneFill=true;

export const icons=[recentSearches,searchVoice,searchFilter,savedSearch,searchMatch,searching,searchScope,searchImage];
