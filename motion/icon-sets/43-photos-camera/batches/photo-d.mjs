// Photos & camera batch D (adjustments): brightness, saturation, red-eye-fix, photo-delete, photo-favorite.
import {make,toggle,F,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,bothFill,holes,capsule} from './helpers.mjs';
import {arcN,L,clamp01,cardParts,hillsParts,sunParts} from './photo.mjs';

// ---------- BRIGHTNESS ----------
// A half-filled disc with eight rays (the brightness dial, not the weather sun). The rays flare outward
// and settle back; the disc breathes with them.
const RAY_A=[0,45,90,135,180,225,270,315];
const rayEnd=F([[0,10.6],[.06,10.6],[.24,11.8],[.58,10.6,'smooth']],[[0,10.6],[.06,10],[.2,12.6,'hit'],[.36,10.3],[.48,10.6]]);
const core=F([[0,1],[.06,1],[.24,1.06],[.58,1,'smooth']],[[0,1],[.06,.96],[.2,1.1,'hit'],[.36,.98],[.48,1]]);
const corePose=(x,y,t,s)=>{const k=core(t,s);return [16+(x-16)*k,16+(y-16)*k];};
const halfD=arcN(16,16,4.2,90,270,2)+' Z';
const brightness=make('brightness','Brightness',
 'Light: the eight rays lengthen 1.2 units and ease back while the disc breathes. Strong: the rays draw in, shoot out 2 units with a hard stop and settle; the disc pops with them.',[
 ...posed([fill('disc-interior',circle(16,16,4.2),'tint'),stroke('disc',circle(16,16,4.2)),fill('half',halfD,'outline'),fill('disc-solid',circle(16,16,5.5),'solid')],corePose),
 ...RAY_A.flatMap(a=>tube('ray-'+a,(t,s)=>{const c=Math.cos(a*Math.PI/180),n=Math.sin(a*Math.PI/180),e=rayEnd(t,s);return `M${16+7.8*c} ${16+7.8*n} L${16+e*c} ${16+e*n}`;}))
],.58,.48);
brightness.duotoneFill=true;

// ---------- SATURATION ----------
// A colour drop whose fill level rises from low to high and holds. Reverse drains it back.
const DC=[16,18.5],DRr=7;
const dropD=(()=>{const tipL=[15.34,5.92],tipR=[16.66,5.92];return `M${tipL} C15.78 5.17 16.22 5.17 ${tipR} L22.02 14.93 ${L(arcN(...DC,DRr,-30.7,210.7,4))} Z`;})();
const levF=F([[0,23.4],[.06,23.4],[.6,15.6,'smooth']],[[0,23.4],[.06,23.4,'hold'],[.34,15.6,'hit'],[.42,15.6]]);
const levB=F([[0,15.6],[.54,23.4,'smooth']],[[0,15.6],[.06,15.6,'hold'],[.32,23.4,'hit']]);
const level=(t,s,r)=>r?levB(t,s):levF(t,s);
const liquidD=(t,s,r)=>{const y=level(t,s,r),w=Math.sqrt(DRr*DRr-(y-DC[1])**2),phi=Math.asin((y-DC[1])/DRr)*180/Math.PI;return `M${DC[0]+w} ${y} ${L(arcN(...DC,DRr,phi,180-phi,4))} Z`;};
const wobble=F([[0,1],[.6,1]],[[0,1],[.34,1],[.38,1.07,'hit'],[.46,.97],[.54,1]]);
const dropPose=(x,y,t,s,r)=>[16+(x-16)*(r?1:wobble(t,s)),y];
const saturation=toggle('saturation','Saturation',
 'Low to high: the colour level inside the drop rises from a fifth to four fifths and holds. Strong fills fast with a hard stop and the drop wobbles once. Reverse drains it back down.',
 posed([fill('drop-interior',dropD,'tint'),...tube('drop',dropD),fill('level',liquidD,'outline'),fill('level-solid',liquidD,'solid')],dropPose),
 .6,.54,'low','high',.54,.5);
saturation.duotoneFill=true;

// ---------- RED-EYE FIX ----------
// An eye whose pupil is a hollow ring. The fix fills the pupil while a sparkle flashes at the corner.
// The corrected eye is retained; preview reset rearms.
const eyeD='M4.5 16 C8.5 7.2 23.5 7.2 27.5 16 C23.5 24.8 8.5 24.8 4.5 16 Z';
const fillK=F([[0,0],[.1,0],[.48,1,'smooth']],[[0,0],[.08,0,'hold'],[.24,1.12,'hit'],[.36,1]]);
const sparkK=F([[0,0],[.3,0],[.44,1.1],[.56,1,'smooth'],[.7,1]],[[0,0],[.18,0],[.3,1.2,'hit'],[.4,1]]);
const sparkV=F([[0,0],[.3,0],[.4,1],[.58,1],[.76,0,'smooth']],[[0,0],[.18,0],[.24,1],[.4,1],[.54,0,'smooth']]);
const SP=[25,7.6];
const starD=polygon([[25,5.2],[25.7,6.9],[27.4,7.6],[25.7,8.3],[25,10],[24.3,8.3],[22.6,7.6],[24.3,6.9]],.2);
const scaleAbout=(c,k)=>(x,y)=>[c[0]+(x-c[0])*k,c[1]+(y-c[1])*k];
const redEyeFix=make('red-eye-fix','Red-eye fix',
 'The hollow pupil fills in as the fix is applied and a sparkle flashes at the eye\'s corner. Strong snaps the fill with a small overshoot and a sharper sparkle. The corrected eye is retained; preview reset rearms.',[
 fill('eye-interior',eyeD,'tint'),...tube('eye',eyeD),fill('eye-solid-core',holes(eyeD,circle(16,16,4.4)),'solid'),
 ...tube('pupil',circle(16,16,3.2)),
 ...posed(shown(bothFill('pupil-fill',circle(16,16,3.2)),(t,s)=>fillK(t,s)>.08?1:0),(x,y,t,s)=>scaleAbout([16,16],Math.max(.04,fillK(t,s)))(x,y)),
 ...posed(shown(bothFill('sparkle',starD),sparkV),(x,y,t,s)=>scaleAbout(SP,Math.max(.04,sparkK(t,s)))(x,y))
],.76,.54,['pupil-fill']);
redEyeFix.duotoneFill=true;

// ---------- PHOTO DELETE ----------
// A photo drops into a bin and fades as it passes the lid. Strong drops it fast; the bin jolts and the lid
// hops on impact. The emptied bin is retained; preview reset brings the photo back above it.
const binU='M8.6 16.7 L9.7 26.2 C9.8 27.1 10.4 27.6 11.2 27.6 L20.8 27.6 C21.6 27.6 22.2 27.1 22.3 26.2 L23.4 16.7';
const binSolid=holes(binU+' Z',capsule([13.6,19.8],[13.6,24.2],1.9),capsule([18.4,19.8],[18.4,24.2],1.9));
const dropY=F([[0,0],[.06,0],[.54,11,'in']],[[0,0],[.06,0,'hold'],[.3,11,'in']]);
const cardV=(t,s)=>1-clamp01((dropY(t,s)-1.2)/3.2);
const binJolt=F([[0,0],[.54,0]],[[0,0],[.3,0],[.33,.5,'hit'],[.46,0]]);
const lidHop=F([[0,0],[.54,0]],[[0,0],[.3,0],[.34,-.7,'hit'],[.48,0]]);
const cardPose=rigid((t,s)=>({dy:dropY(t,s)}));
const photoDelete=make('photo-delete','Photo delete',
 'The photo drops into the bin and fades as it passes the lid. Strong drops it fast, the bin jolts and the lid hops on impact. The emptied bin is retained; preview reset brings the photo back.',[
 ...posed([...tube('bin',binU),fill('bin-solid-core',binSolid,'solid'),...tube('slot-l','M13.6 19.8 L13.6 24.2'),...tube('slot-r','M18.4 19.8 L18.4 24.2')],rigid((t,s)=>({dy:binJolt(t,s)}))),
 ...posed(tube('lid','M7 16.5 L25 16.5'),rigid((t,s)=>({dy:lidHop(t,s)}))),
 ...posed(shown([...cardParts('card',10,3.5,12,9.1,1.6),stroke('hills','M10 11.4 L14.3 7.2 L16.7 9.6 L18.5 8 L22 11.4'),fill('hills-solid',polygon([[10,11.4],[14.3,7.2],[16.7,9.6],[18.5,8],[22,11.4]],.4),'solid')],cardV),cardPose)
],.54,.48,['card','hills']);
photoDelete.duotoneFill=true;

// ---------- PHOTO FAVORITE ----------
// A photo with a heart beside it. Favouriting pops the heart and fills it; reverse empties it again.
const HC=[23.6,16.6];
const heartD='M23.6 20.2 C23.6 20.2 19.9 17.8 19.9 15.3 C19.9 13.9 20.9 13 22 13 C22.8 13 23.4 13.5 23.6 14.1 C23.8 13.5 24.4 13 25.2 13 C26.3 13 27.3 13.9 27.3 15.3 C27.3 17.8 23.6 20.2 23.6 20.2 Z';
const popF=F([[0,1],[.06,1],[.2,1.18],[.5,1,'smooth']],[[0,1],[.06,.86],[.2,1.26,'hit'],[.34,.94],[.46,1]]);
const popB=F([[0,1],[.1,.9],[.34,1,'smooth']],[[0,1],[.06,1.08],[.2,.9,'hit'],[.36,1]]);
const hkF=F([[0,0],[.1,0],[.4,1,'smooth']],[[0,0],[.06,0,'hold'],[.2,1,'hit']]);
const hkB=F([[0,1],[.4,0,'smooth']],[[0,1],[.06,1,'hold'],[.22,0,'in']]);
const heartK=(t,s,r)=>r?hkB(t,s):hkF(t,s);
const pop=(t,s,r)=>r?popB(t,s):popF(t,s);
const heartPose=(x,y,t,s,r)=>{const k=pop(t,s,r);return [HC[0]+(x-HC[0])*k,HC[1]+(y-HC[1])*k];};
const fillPose=(x,y,t,s,r)=>{const k=pop(t,s,r)*Math.max(.04,heartK(t,s,r));return [HC[0]+(x-HC[0])*k,HC[1]+(y-HC[1])*k];};
const photoFavorite=toggle('photo-favorite','Photo favorite',
 'Unmarked to favourite: the heart pops and fills in from its centre, then holds. Strong squeezes the heart first and pops it bigger with one settle. Reverse empties the heart with a small dip.',[
 ...cardParts('card',3.5,7.5,12,17,2.2),...hillsParts('hills',[[5.8,21],[8.4,16.6],[10.2,18.8],[11.4,17.4],[13.2,21]]),...sunParts('sun',12.2,11.6,1.2),
 ...posed([fill('heart-interior',heartD,'tint'),...tube('heart',heartD)],heartPose),
 ...posed(shown(bothFill('heart-fill',heartD),(t,s,r)=>heartK(t,s,r)>.08?1:0),fillPose)
],.5,.46,'unmarked','favorite',.4,.36);
photoFavorite.duotoneFill=true;

export const icons=[brightness,saturation,redEyeFix,photoDelete,photoFavorite];
