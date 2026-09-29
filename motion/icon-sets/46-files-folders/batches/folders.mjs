// Files & folders batch A: new-folder, folder-open, move-file, duplicate-file, import-file, export-file.
import {make,toggle,F,FR,tube,framed,rigid,posed,shown,fill,stroke,polygon,rect,capsule,hole,cross,openRounded,line,clamp01,scaleAbout,L} from './helpers.mjs';

// ---------- NEW FOLDER ----------
// Folder x5–27, y7–27 with a front edge at y13.4 and a plus on the front. Light: the plus pops 15%.
// Strong: it presses to 80%, pops to 125% with a hit and the whole folder thumps 0.5 down.
const folderBody='M8 7 L12.5 7 C13.1 7 13.6 7.24 14 7.68 L15.85 9.7 L25 9.7 C26.1 9.7 27 10.6 27 11.7 L27 24.6 C27 25.925 25.925 27 24.6 27 L7.4 27 C6.075 27 5 25.925 5 24.6 L5 9 C5 7.9 5.9 7 7 7 Z';
const PLUS=[16,20.2],ARM=2.6,HALF=1.1;
const pop=F([[0,1],[.2,1.15],[.45,1,'smooth']],[[0,1],[.1,.8],[.26,1.25,'hit'],[.38,.96],[.5,1]]);
const thump=F([[0,0],[.45,0]],[[0,0],[.26,0,'hold'],[.3,.5,'hit'],[.42,-.1],[.5,0]]);
const folderPose=rigid((t,s)=>({dy:thump(t,s)}));
const plusPose=(x,y,t,s)=>{const p=scaleAbout(x,y,pop(t,s),...PLUS);return[p[0],p[1]+thump(t,s)];};
const newFolder=make('new-folder','New folder',
 'Light: the plus on the folder front pops 15% and settles. Strong: it presses down, pops to 125% with a hit and the folder thumps half a unit before settling.',[
 ...posed([fill('body-interior',folderBody,'tint'),stroke('body',folderBody),stroke('front-edge','M6.6 13.4 L25.4 13.4'),
  fill('body-solid',(t,s)=>hole(hole(folderBody,rect(6.8,12.6,18.4,1.6,.8)),cross(...PLUS,ARM*pop(t,s),HALF)),'solid')],folderPose),
 ...posed([stroke('plus-h',`M${PLUS[0]-ARM} ${PLUS[1]} L${PLUS[0]+ARM} ${PLUS[1]}`),stroke('plus-v',`M${PLUS[0]} ${PLUS[1]-ARM} L${PLUS[0]} ${PLUS[1]+ARM}`)],plusPose)
],.45,.5);
newFolder.duotoneFill=true;

// ---------- FOLDER OPEN ----------
// Shell x5–24 with a tab; the front panel leans open: its top edge slides right 3.5 and rises 0.5 while
// its bottom stays put. The shell path carries the exposed front corner so topology never changes.
// Solid keeps a slit along the front's top edge. Stateful: closed → open; reverse closes.
const openU=FR([[0,0],[.5,1,'smooth']],[[0,0],[.1,-.08],[.32,1.18,'hit'],[.46,.94],[.6,1]],[[0,1],[.5,0,'smooth']],[[0,1],[.1,1.1],[.3,-.06,'hit'],[.42,.04],[.55,0]]);
const front=(u)=>({sh:3.5*u,y:14-.5*u});
const shellD=(t,s,r)=>{const {sh,y}=front(openU(t,s,r));return polygon([[5,26],[5,7],[12.5,7],[15.4,9.8],[24,9.8],[24,y],[24+sh,y],[24,26]],[1.7,1.2,.8,.8,1.2,.6,.8,1.7]);};
const frontTop=(t,s,r)=>{const {sh,y}=front(openU(t,s,r));return `M${5+sh} ${y} L${24} ${y}`;};
const frontLeft=(t,s,r)=>{const {sh,y}=front(openU(t,s,r));return `M5 26 L${5+sh} ${y}`;};
const frontLeftShow=(t,s,r)=>clamp01(openU(t,s,r)*4);
const slit=(t,s,r)=>{const {sh,y}=front(openU(t,s,r));return capsule([5+Math.max(0,sh)+1.7,y],[22.4,y],1.5);};
const folderOpen=toggle('folder-open','Folder open',
 'Closed → open: the front panel leans open, its top edge sliding right and up while the bottom stays hinged. Strong: a small inward pull, then the panel swings past open with a hit and settles. Reverse closes it.',[
 fill('shell-interior',shellD,'tint'),stroke('shell',shellD),
 stroke('front-top',frontTop),...shown([stroke('front-left',frontLeft)],frontLeftShow),
 fill('shell-solid',(t,s,r)=>hole(shellD(t,s,r),slit(t,s,r)),'solid')
],.5,.6,'closed','open',.5,.55);
folderOpen.duotoneFill=true;

// ---------- MOVE FILE ----------
// A 9 x 13 sheet hops from the left shelf mark to the right one (13 units). Light: one smooth arc, lift 1.2.
// Strong: a short backward pull, a fast 0.24 s flight with lift 2.4, a hit and a settle. Stateful, reverse hops back.
const moveU=FR([[0,0],[.6,1,'smooth']],[[0,0],[.08,-.06],[.32,1.06,'hit'],[.42,.98],[.5,1]],[[0,1],[.6,0,'smooth']],[[0,1],[.08,1.06],[.32,-.06,'hit'],[.42,.02],[.5,0]]);
const lift=(u,s)=>-(s?2.4:1.2)*Math.sin(Math.PI*clamp01(u));
const sheet=polygon([[5,8],[11,8],[14,11],[14,21],[5,21]],[1.4,.7,.7,1.4,1.4]);
const sheetFold='M11 8.8 L11 10.4 C11 10.8 11.2 11 11.6 11 L13.2 11';
const sheetTri=polygon([[11.7,9.4],[13,10.7],[11.7,10.7]],.2);
const sheetPose=rigid((t,s,r)=>{const u=moveU(t,s,r);return {dx:13*u,dy:lift(u,s)};});
const moveFile=toggle('move-file','Move file',
 'Origin → destination: the sheet lifts off its shelf mark and lands on the empty one. Strong: a short pull back, a fast flight with a higher arc, a hit and a settle. Reverse moves it back.',[
 ...posed([fill('sheet-interior',sheet,'tint'),stroke('sheet',sheet),stroke('sheet-fold',sheetFold),fill('sheet-solid',hole(sheet,sheetTri),'solid')],sheetPose),
 ...line('mark-a',[5.4,25],[13.6,25]),...line('mark-b',[18.4,25],[26.6,25])
],.6,.5,'origin','destination',.6,.5);
moveFile.duotoneFill=true;

// ---------- DUPLICATE FILE ----------
// Original sheet x7–21, y4–20 stays; the copy slides out 4 units down-right. The original is drawn only
// where it is exposed (top and left edges), attached to the copy's outline. Stateful, reverse tucks it back.
const dupD=FR([[0,0],[.55,4,'smooth']],[[0,0],[.1,-.5],[.34,4.5,'hit'],[.46,3.8],[.6,4]],[[0,4],[.5,0,'smooth']],[[0,4],[.1,4.5],[.32,-.4,'hit'],[.44,.15],[.55,0]]);
const orig=polygon([[7,4],[16,4],[21,9],[21,20],[7,20]],[1.7,.9,.9,1.7,1.7]);
const origFold='M16 5 L16 8 C16 8.6 16.4 9 17 9 L20 9';
const origTri=polygon([[17,6],[19,8],[17,8]],.25);
const copyPose=rigid((t,s,r)=>{const d=dupD(t,s,r);return {dx:d,dy:d};});
const exposed=(t,s,r)=>{const d=Math.max(0,dupD(t,s,r));return openRounded([[16+d,4+d],[16,4],[7,4],[7,20],[7+d,20]],[.9,1.7,1.7]);};
const exposedSolid=(t,s,r)=>{const d=Math.max(0,dupD(t,s,r)-1.3);return polygon([[7,4],[16,4],[16+d,4+d],[7+d,4+d],[7+d,20],[7,20]],[1.7,.9,.2,.2,.2,1.7]);};
const duplicateFile=toggle('duplicate-file','Duplicate file',
 'Single → duplicated: a copy slides out from behind the sheet to the lower right; the original stays visible along its top and left edges. Strong: a short pull, a fast slide past the mark with a hit and a settle. Reverse tucks the copy back.',[
 fill('orig-interior',exposedSolid,'tint'),stroke('orig',exposed),fill('orig-solid',exposedSolid,'solid'),
 ...posed([fill('copy-interior',orig,'tint'),stroke('copy',orig),stroke('copy-fold',origFold),fill('copy-solid',hole(orig,origTri),'solid')],copyPose)
],.55,.6,'single','duplicated',.5,.55);
duplicateFile.duotoneFill=true;

// ---------- IMPORT / EXPORT ----------
// A sheet with a 9.8-unit opening in one side edge; a rightward arrow travels through it. Import: the sheet sits
// right and the arrow enters from the left. Export: the sheet sits left and the arrow leaves to the right.
// Light: a 2.2-unit push and return. Strong: a 1-unit draw back, a fast 3.5-unit drive with a hit, then return.
const push=F([[0,0],[.3,2.2],[.6,0,'smooth']],[[0,0],[.1,-1],[.3,3.5,'hit'],[.42,3.1],[.66,0,'smooth']]);
const GAP=[12.6,22.4];
function gappedSheet(x0,x1,side){ // side 'left' opens the left edge, 'right' the right edge
 const w=x1-x0,fold=6,top=4,bottom=28;
 const outline=side==='left'
  ?openRounded([[x0,GAP[1]],[x0,bottom],[x1,bottom],[x1,top+fold],[x1-fold,top],[x0,top],[x0,GAP[0]]],[1.8,1.8,.9,.9,1.8])
  :openRounded([[x1,GAP[0]],[x1,top+fold],[x1-fold,top],[x0,top],[x0,bottom],[x1,bottom],[x1,GAP[1]]],[.9,.9,1.8,1.8,1.8]);
 const depth=4.6,nx=side==='left'?x0+depth:x1-depth,ex=side==='left'?x0:x1;
 const solid=side==='left'
  ?polygon([[x0,top],[x1-fold,top],[x1,top+fold],[x1,bottom],[x0,bottom],[x0,GAP[1]],[nx,GAP[1]],[nx,GAP[0]],[x0,GAP[0]]],[1.8,.9,.9,1.8,1.8,.4,.8,.8,.4])
  :polygon([[x0,top],[x1-fold,top],[x1,top+fold],[x1,GAP[0]],[nx,GAP[0]],[nx,GAP[1]],[x1,GAP[1]],[x1,bottom],[x0,bottom]],[1.8,.9,.9,.4,.8,.8,.4,1.8,1.8]);
 const foldD=`M${x1-fold} ${top+1} L${x1-fold} ${top+5} C${x1-fold} ${top+5.6} ${x1-fold+.4} ${top+6} ${x1-fold+1} ${top+6} L${x1-1} ${top+6}`;
 const tri=polygon([[x1-fold+1,top+2],[x1-fold+3.5,top+4.5],[x1-fold+1,top+4.5]],.3);
 return [fill('sheet-interior',solid,'tint'),stroke('sheet',outline),stroke('sheet-fold',foldD),fill('sheet-solid',hole(solid,tri),'solid')];
}
const arrowParts=(tipX,tail)=>[...tube('shaft',`M${tail} 17.5 L${tipX} 17.5`),...tube('head',`M${tipX-1.14} 16.24 L${tipX} 17.5 L${tipX-1.14} 18.76`)];
const arrowPose=rigid((t,s)=>({dx:push(t,s)}));
const importFile=make('import-file','Import file',
 'An arrow drives into the opening on the sheet\'s left edge and returns. Light: a 2.2-unit push. Strong: a short draw back, a fast 3.5-unit drive through the opening with a hit, then a return.',[
 ...gappedSheet(11.5,27,'left'),...posed(arrowParts(10,4.6),arrowPose)
],.6,.66);
importFile.duotoneFill=true;
const exportFile=make('export-file','Export file',
 'An arrow leaves through the opening on the sheet\'s right edge and returns. Light: a 2.2-unit push. Strong: a short draw back, a fast 3.5-unit drive out of the sheet with a hit, then a return.',[
 ...gappedSheet(5,20.5,'right'),...posed(arrowParts(22,16),arrowPose)
],.6,.66);
exportFile.duotoneFill=true;

export const icons=[newFolder,folderOpen,moveFile,duplicateFile,importFile,exportFile];
