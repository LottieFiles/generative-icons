// Everyday controls · grid-list-view, sidebar-toggle, split-view, tab-switcher, undo-redo (revision 3)
import {make,toggle,geometricRearm,F,P,posed,rigid,rect,polygon,capsule,arc,holes,mix,tube,framed,bothFill,fill,stroke,circle} from './helpers.mjs';

const rev=fn=>(t,s,r)=>r?1-fn(t,s):fn(t,s);           // toggle reverse: same drive from u=1 back to 0
const cl=x=>Math.max(0,Math.min(1,x));
const WIN=rect(4,6,24,20,3);
const WIN_SOLID=holes(rect(2.7,4.7,26.6,22.6,4.3),rect(5.3,7.3,21.4,17.4,1.7));
const shell=name=>[stroke(name,WIN),fill(name+'-solid',WIN_SOLID,'solid')];
const pane=x=>polygon([[4,6],[x,6],[x,26],[4,26]],[3,0,0,3]);

/* 1. GRID / LIST VIEW — toggle grid ⇄ list. Four 7x7 tiles (cols 6..13 / 19..26, rows cy 9.5 / 22.5).
   Phase A (u 0..0.5): each tile flattens in its own column and slides to its row line (7.5 / 13 / 18.5 / 24).
   Phase B (u 0.5..1): flat bars extend to x 6..26. No overlap between phases. Row centreline gap 5.5.
   Vertical edge gap between columns' tiles = 6 + 5*hf (never below 6 for hf>=0; 5.27 at the Strong puff).
   Outline row = zero-height rounded rect (bar); Solid row = 2.6 capsule; tint = same rect as outline, zero area when flat. */
const gl=rev(F([[0,0],[.5,1,'smooth']],[[0,0],[.1,-.08],[.36,1.05],[.52,1]]));
const tileD=(left,cy0,cy1,solid)=>(t,s,r)=>{
 const u=gl(t,s,r),hf=Math.min(1,u/.5),wf=Math.max(0,(u-.5)/.5);
 const H=mix(7,0,hf),W=mix(7,20,wf),x=left?6:26-W,y=mix(cy0,cy1,cl(hf))-H/2;
 return solid?rect(x-1.3,y-1.3,W+2.6,H+2.6,3.3):rect(x,y,W,H,2);};
const tile=(n,left,cy0,cy1)=>framed(n,tileD(left,cy0,cy1,false),tileD(left,cy0,cy1,true));
const gridList={...toggle('grid-list-view','Grid / list view',
 'Four grid tiles puff, flatten into bars in place, then stretch across the window into four list rows. Reverse rebuilds the 2x2 grid.',
 [...tile('tl',true,9.5,7.5),...tile('tr',false,9.5,13),...tile('bl',true,22.5,18.5),...tile('br',false,22.5,24)],
 .5,.55,'grid','list'),duotoneFill:true};

/* 2. SIDEBAR TOGGLE — unchanged. */
const sb=rev(F([[0,0],[.4,1,'smooth']],[[0,0],[.1,-.12],[.3,1.08],[.46,1]]));
const sx=(t,s,r)=>mix(13,8,sb(t,s,r));
const sidebar={...toggle('sidebar-toggle','Sidebar toggle',
 'The panel divider leans open for a beat, then drives left to a narrow rail as the chevron flips to point back out. Reverse re-opens the panel.',
 [fill('panel',(t,s,r)=>pane(sx(t,s,r)),'tint'),...shell('frame'),
  ...posed(tube('divider','M13,6 L13,26'),rigid((t,s,r)=>({dx:sx(t,s,r)-13}))),
  ...posed(tube('chevron','M21.2,12 L17.8,16 L21.2,20'),rigid((t,s,r)=>({angle:180*sb(t,s,r),pivot:[19.5,16]})))],
 .42,.5,'open','closed'),duotoneFill:true};

/* 3. SPLIT VIEW — unchanged. */
const sv=F([[0,16],[.25,19,'smooth'],[.5,16,'smooth']],[[0,16],[.1,15.2],[.34,21.8],[.62,15.6],[.78,16]]);
const svPose=rigid((t,s)=>({dx:sv(t,s)-16}));
const split={...make('split-view','Split view',
 'The centre divider and its grip are dragged toward the right pane to compare, then slide back to the even split.',
 [fill('pane',(t,s)=>pane(sv(t,s)),'tint'),...shell('frame'),
  ...posed(tube('divider','M16,6 L16,26'),svPose),
  ...posed(bothFill('grip',capsule([16,13],[16,19],4)),svPose)],
 .5,.8),duotoneFill:true};

/* 4. TAB SWITCHER — unchanged. */
const tabFrame=rect(3.3,5,25.4,22,3.5);
const tabRing=holes(rect(2,3.7,28,24.6,4.8),rect(4.6,6.3,22.8,19.4,2.2));
const tabDx=F([[0,0],[.4,11,'smooth']],[[0,0],[.1,-.9],[.34,11.8],[.5,11]]);
const dotR=P([[0,1],[.06,1],[.14,0,'out']],[[0,1],[.1,1],[.17,0,'out']]);
const dotL=P([[0,0],[.24,0],[.32,1,'out']],[[0,0],[.26,0],[.34,1,'out']]);
const tabs=geometricRearm({...make('tab-switcher','Tab switcher',
 'Inside the tab strip the active-tab bar pulls back, then drives across to the second slot; the slot markers swap so the neighbour becomes active.',
 [fill('content',polygon([[3.3,14],[28.7,14],[28.7,27],[3.3,27]],[0,0,3.5,3.5]),'tint'),
  stroke('frame',tabFrame),fill('frame-solid',tabRing,'solid'),
  ...tube('strip','M3.3,14 L28.7,14'),
  ...posed(tube('active','M8,9.5 L13,9.5'),rigid((t,s)=>({dx:tabDx(t,s)}))),
  ...bothFill('slot-right',circle(21,9.5,1.2),{visibility:dotR}),
  ...bothFill('slot-left',circle(10.5,9.5,1.2),{visibility:dotL})],
 .4,.55),duotoneFill:true});

/* 5. UNDO / REDO — toggle undo ⇄ redo. One rigid object: head (apex [8,13], barbs 4.5 at 45°) on a horizontal approach
   y=13 to [19,13], a 180° bend r5 about [19,18] (top→right→bottom), tail back to [14,23]. Whole object rotates 180° about [16,16].
   Undo = head left, arch right, head on top. Redo = head right, arch left, head below. Fixed topology, no visibility, no morph.
   Sweep radius from [16,16]: apex 8.54 + Solid half 1.3 = 9.84 → paint stays inside [6.2,25.8]. Open path: no tint. */
const STEM='M8,13 L19,13 '+arc(19,18,5,270,450).replace(/^M/,'L')+' L14,23';
const HEAD='M12.5,8.5 L8,13 L12.5,17.5';
const ud=rev(F([[0,0],[.42,1,'smooth']],[[0,0],[.1,-.07],[.36,1.06],[.52,1]]));
const udPose=rigid((t,s,r)=>({angle:180*ud(t,s,r),pivot:[16,16]}));
const undoRedo={...toggle('undo-redo','Undo / redo',
 'The whole undo arrow winds back a few degrees, then swings a half turn about the canvas centre so its head points forward: undo becomes redo. Reverse swings it back.',
 [...posed(tube('stem',STEM),udPose),...posed(tube('head',HEAD),udPose)],
 .42,.52,'undo','redo'),duotoneFill:false};

export const icons=[gridList,sidebar,split,tabs,undoRedo];
