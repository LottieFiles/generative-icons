// Photos & camera batch C (library): photo-gallery, photo-album, photo-edit, rotate-photo, flip-photo.
import {make,toggle,F,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,bothFill,rotate,holes} from './helpers.mjs';
import {pt,cardParts,hillsParts,sunParts,ringD} from './photo.mjs';
import svgpath from 'svgpath';

// ---------- PHOTO GALLERY ----------
// Four landscape thumbnails in a 2×2 grid. Browsing ripples through them in reading order: each tile
// hops and lands. Rows are 5 apart so a landing tile never meets the one hopping below it at Bold.
const TILES=[[4,6],[18,6],[4,18.5],[18,18.5]];
const hop=i=>F([[0,0],[.05*i,0],[.1+.05*i,-1],[.34+.05*i,0,'smooth']],[[0,0],[.05*i,0],[.06+.05*i,.3],[.16+.05*i,-2.2],[.28+.05*i,0,'hit'],[.34+.05*i,-.3],[.42+.05*i,0]]);
const photoGallery=make('photo-gallery','Photo gallery',
 'Light: the four thumbnails lift one unit each in reading order and settle. Strong: each tile crouches, hops 2.2 units, lands hard and rebounds once, rippling through the grid.',
 TILES.flatMap(([x,y],i)=>posed(cardParts('tile-'+i,x,y,10,7.5,1.6),rigid((t,s)=>({dy:hop(i)(t,s)})))),
 .52,.6);
photoGallery.duotoneFill=true;

// ---------- PHOTO ALBUM ----------
// A bound album whose cover carries the photo. The cover swings on its binding (width × cos) to peek
// inside and closes again; the page behind shows only past the cover's moving edge.
const AX=5.5,AW=21,AY=6,AH=20;
const openA=F([[0,0],[.06,0],[.3,28],[.64,0,'smooth']],[[0,0],[.06,-4],[.24,58,'hit'],[.4,-3],[.52,0]]);
const cosA=(t,s)=>Math.cos(openA(t,s)*Math.PI/180);
const coverPose=(x,y,t,s)=>[AX+(x-AX)*cosA(t,s),y];
const pageD=(t,s)=>{const xr=AX+AW*cosA(t,s),x1=AX+AW,r=2.2,k=.448;return `M${xr} ${AY} L${x1-r} ${AY} C${x1-r*k} ${AY} ${x1} ${AY+r*k} ${x1} ${AY+r} L${x1} ${AY+AH-r} C${x1} ${AY+AH-r*k} ${x1-r*k} ${AY+AH} ${x1-r} ${AY+AH} L${xr} ${AY+AH}`;};
const pageSolidD=(t,s)=>{const xr=AX+AW*cosA(t,s);return rect(xr-.3,AY-1.3,AX+AW+1.3-xr+.3,AH+2.6,3.5);};
const exposed=(t,s)=>AW*(1-cosA(t,s))>.4?1:0;
const photoAlbum=make('photo-album','Photo album',
 'Light: the album cover swings 28° on its binding to peek inside and closes. Strong: it presses, swings 58° with a hard stop, and closes with one small rebound. The page behind shows past the cover\'s edge.',[
 stroke('page',pageD,{visibility:exposed}),fill('page-solid',pageSolidD,'solid',{visibility:exposed}),
 ...posed([...cardParts('cover',AX,AY,AW,AH,2.2),...tube('spine','M9 6 L9 26'),
  ...hillsParts('hills',[[12.6,21.4],[15.8,16.4],[18.2,19],[20,17.2],[23.4,21.4]]),...sunParts('sun',22,11.2,1.3)],coverPose)
],.64,.52);
photoAlbum.duotoneFill=true;

// ---------- PHOTO EDIT ----------
// A photo and a pencil at its lower right. The pencil dips to the photo's edge, strokes along it and lifts.
const pencilLocal=polygon([[0,0],[2.4,-1.5],[6.8,-1.5],[6.8,1.5],[2.4,1.5]],[.3,.4,.8,.8,.4]);
const PT=[22.6,22];
const place=d=>svgpath(d).rotate(-45).translate(...PT).toString();
const pencilD=place(pencilLocal),ferruleD=place('M2.4 -1.5 L2.4 1.5');
const dipF=F([[0,0],[.06,0],[.18,1.1,'out'],[.5,1.1],[.64,0,'smooth']],[[0,0],[.06,-.3],[.14,1.1,'hit'],[.48,1.1],[.6,0,'smooth']]);
const slide=F([[0,0],[.18,0],[.36,-2.2],[.52,0,'smooth']],[[0,0],[.14,0],[.28,-3.6,'pass'],[.46,0,'hit'],[.52,0]]);
const cardJolt=F([[0,0],[.64,0]],[[0,0],[.12,0],[.16,-.4,'hit'],[.26,0],[.44,0],[.48,.3,'hit'],[.58,0]]);
const pencilPose=rigid((t,s)=>{const d=dipF(t,s);return {dx:-d*Math.SQRT1_2,dy:d*Math.SQRT1_2+slide(t,s)};});
const photoEdit=make('photo-edit','Photo edit',
 'Light: the pencil dips to the photo\'s edge, strokes 2.2 units along it and lifts away. Strong: it taps the edge hard, strokes 3.6 units with a fast return, and the photo kicks on each contact.',[
 ...posed([...cardParts('card',3.5,6.5,15.5,15,2.2),...hillsParts('hills',[[5.4,20],[8.8,15.2],[11,17.6],[12.4,16.2],[17.2,20]]),...sunParts('sun',15.6,11,1.3)],rigid((t,s)=>({dx:cardJolt(t,s)}))),
 ...posed([stroke('pencil',pencilD),fill('pencil-solid',pencilD,'solid'),stroke('ferrule',ferruleD)],pencilPose)
],.64,.6);
photoEdit.duotoneFill=true;

// ---------- ROTATE PHOTO ----------
// A landscape photo turns a quarter turn clockwise to portrait under a curved arrow. Reverse turns it back.
const RC=[16,16];
const rotF=F([[0,0],[.06,0],[.6,90,'smooth']],[[0,0],[.08,-10],[.36,98,'hit'],[.5,90]]);
const rotB=F([[0,90],[.54,0,'smooth']],[[0,90],[.08,100],[.34,-6,'hit'],[.46,0]]);
const rotA=(t,s,r)=>r?rotB(t,s):rotF(t,s);
const nodF=F([[0,0],[.5,0]],[[0,0],[.08,-4],[.36,10,'hit'],[.5,0]]);
const nodB=F([[0,0],[.46,0]],[[0,0],[.08,4],[.34,-8,'hit'],[.46,0]]);
const arrowHead=(()=>{const deg=-18,p=pt(deg,11.8,RC),T=[-Math.sin(deg*Math.PI/180),Math.cos(deg*Math.PI/180)],N=[T[1],-T[0]];return polygon([[p[0]+2.9*T[0],p[1]+2.9*T[1]],[p[0]-.3*T[0]+1.7*N[0],p[1]-.3*T[1]+1.7*N[1]],[p[0]-.3*T[0]-1.7*N[0],p[1]-.3*T[1]-1.7*N[1]]],[.35,.35,.35]);})();
const arcD=(()=>{const a=-72,b=-18,A=d=>d*Math.PI/180,p=t=>[16+11.8*Math.cos(t),16+11.8*Math.sin(t)],s=A(a),e=A(b),k=4/3*Math.tan((e-s)/4),u=p(s),v=p(e);return 'M'+u+' C'+[u[0]-k*11.8*Math.sin(s),u[1]+k*11.8*Math.cos(s),v[0]+k*11.8*Math.sin(e),v[1]-k*11.8*Math.cos(e),...v];})();
const rotatePhoto=toggle('rotate-photo','Rotate photo',
 'Landscape to portrait: the photo turns a quarter turn clockwise about its centre. Strong winds back 10°, overshoots to 98° with a hard stop and settles while the arrow nods. Reverse turns it back.',[
 ...posed([...cardParts('card',9.5,10.75,13,10.5,1.6),...hillsParts('hills',[[12,18.6],[14.6,15.1],[16.4,17.1],[17.6,16.1],[20,18.6]],.5)],rigid((t,s,r)=>({angle:rotA(t,s,r),pivot:RC}))),
 ...posed([...tube('arrow',arcD),...bothFill('arrow-head',arrowHead)],rigid((t,s,r)=>({angle:r?nodB(t,s):nodF(t,s),pivot:RC})))
],.6,.5,'landscape','portrait',.54,.46);
rotatePhoto.duotoneFill=true;

// ---------- FLIP PHOTO ----------
// A photo mirrors about a vertical axis (width × cos), landscape and sun ending on the other side.
const fF=F([[0,0],[.06,0],[.56,1,'smooth']],[[0,0],[.06,-.05],[.28,1.07,'hit'],[.4,.98],[.48,1]]);
const fB=F([[0,1],[.5,0,'smooth']],[[0,1],[.06,1.05],[.28,-.07,'hit'],[.4,.02],[.48,0]]);
const flipU=(t,s,r)=>r?fB(t,s):fF(t,s);
const flipPose=(x,y,t,s,r)=>[16+(x-16)*Math.cos(Math.PI*flipU(t,s,r)),y];
const flipPhoto=toggle('flip-photo','Flip photo',
 'Original to mirrored: the photo turns over about the dashed vertical axis, its landscape and sun ending on the other side. Strong flips faster, passes flat and settles with one width rebound. Reverse flips it back.',[
 ...tube('axis-top','M16 4 L16 5.5'),...tube('axis-bottom','M16 26.5 L16 28'),
 ...posed([...cardParts('card',6.5,9,19,14,2),...hillsParts('hills',[[8.6,20],[13.2,15],[16.2,18.2],[18.6,16.2],[23.4,20]]),...sunParts('sun',10.4,12.6,1.2)],flipPose)
],.56,.48,'original','mirrored',.5,.48);
flipPhoto.duotoneFill=true;

export const icons=[photoGallery,photoAlbum,photoEdit,rotatePhoto,flipPhoto];
