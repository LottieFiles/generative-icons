// Files & folders batch B: page icons whose content is a mark or a mechanism.
// restore-file, file-sync, lock-file, file-search, file-preview, file-tag, file-star, approve-file, rename-file.
import {make,toggle,F,FR,tube,rigid,posed,shown,fill,stroke,circle,polygon,capsule,arc,arcN,hole,page,
 arcArrowSolid,cross,star,lollipop,uTube,polylineTube,movePath,clamp01,lerp,scaleAbout,L} from './helpers.mjs';

// Outline arc arrow: stroke arc that ends at the head base plus a filled triangular head.
// A filled head keeps the inner barb clear of the tight curve at Bold.
const AR=3.6,HEAD=2.2,HALF=1.9;
function arcArrowOutline(name,cx,cy,r,a,b){
 const sign=Math.sign(b-a),rad=d=>d*Math.PI/180,p=(rr,deg)=>[cx+rr*Math.cos(rad(deg)),cy+rr*Math.sin(rad(deg))],baseDeg=b-sign*HEAD/r*180/Math.PI;
 return [stroke(name+'-arc',arc(cx,cy,r,a,baseDeg)),fill(name+'-head',polygon([p(r+HALF,baseDeg),p(r,b),p(r-HALF,baseDeg)],.3),'outline')];
}

// ---------- RESTORE FILE ----------
// A 270° anticlockwise arrow on the sheet turns one full turn. Light: one smooth turn in 0.7 s.
// Strong: a 25° clockwise wind-up, a fast turn past the mark with a hit and a settle back onto it.
const C=[16,18];
const spinBack=F([[0,0],[.7,-360,'smooth']],[[0,0],[.1,25],[.4,-385,'hit'],[.52,-350],[.62,-360]]);
const restoreFile=make('restore-file','Restore file',
 'The restore arrow on the sheet turns one full anticlockwise turn. Light: one smooth turn. Strong: a short clockwise wind-up, a fast turn past the mark with a hit, then a settle back onto it.',[
 ...page('sheet',(t,s)=>[arcArrowSolid(...C,AR,340+spinBack(t,s),70+spinBack(t,s),1.3,HEAD,HALF)]),
 ...posed(arcArrowOutline('arrow',...C,AR,340,70),rigid((t,s)=>({angle:spinBack(t,s),pivot:C})))
],.7,.62);
restoreFile.duotoneFill=true;

// ---------- FILE SYNC ----------
// Two opposed 140° arrows turn one full clockwise turn. Light: one smooth turn. Strong: wind-up, fast turn, hit, settle.
const spin=F([[0,0],[.8,360,'smooth']],[[0,0],[.1,-25],[.4,385,'hit'],[.52,350],[.62,360]]);
const fileSync=make('file-sync','File sync',
 'Two sync arrows on the sheet turn one full clockwise turn. Light: one smooth turn. Strong: an anticlockwise wind-up, a fast turn past the mark with a hit, then a settle.',[
 ...page('sheet',(t,s)=>[arcArrowSolid(...C,AR,200+spin(t,s),340+spin(t,s),1.3,HEAD,HALF),arcArrowSolid(...C,AR,20+spin(t,s),160+spin(t,s),1.3,HEAD,HALF)]),
 ...posed([...arcArrowOutline('arrow-a',...C,AR,200,340),...arcArrowOutline('arrow-b',...C,AR,20,160)],rigid((t,s)=>({angle:spin(t,s),pivot:C})))
],.8,.62);
fileSync.duotoneFill=true;

// ---------- LOCK FILE ----------
// Padlock on the sheet: body x12.2–19.8, y17.4–24.2; shackle legs at x14.1/17.9 whose feet end 1.35 above the
// body so the Solid counters touch without overlapping. Unlocked: the shackle pivots 42° about its left foot and
// rises 0.6. Stateful: unlocked → locked (the shackle slams shut); reverse lifts it open.
const lockBody=polygon([[12.2,17.4],[19.8,17.4],[19.8,24.2],[12.2,24.2]],1.2);
const shackleD='M14.1 16.05 L14.1 14.6 C14.1 13.55 14.95 12.7 16 12.7 C17.05 12.7 17.9 13.55 17.9 14.6 L17.9 16.05';
const PIV=[14.1,16.05];
const lockAngle=FR([[0,-42],[.45,0,'smooth']],[[0,-42],[.08,-48],[.28,0,'hit'],[.5,0]],[[0,0],[.45,-42,'smooth']],[[0,0],[.08,4],[.3,-50,'hit'],[.42,-40],[.55,-42]]);
const lockLift=FR([[0,-.6],[.45,0,'smooth']],[[0,-.6],[.08,-.9],[.28,0,'hit'],[.36,-.25],[.46,0]],[[0,0],[.45,-.6,'smooth']],[[0,0],[.08,.1],[.3,-.8,'hit'],[.42,-.55],[.55,-.6]]);
const lockThump=FR([[0,0],[.45,0]],[[0,0],[.28,0,'hold'],[.33,.45,'hit'],[.44,0]],[[0,0],[.45,0]],[[0,0],[.55,0]]);
const shacklePose=rigid((t,s,r)=>({angle:lockAngle(t,s,r),dy:lockLift(t,s,r),pivot:PIV}));
const bodyPose=rigid((t,s,r)=>({dy:lockThump(t,s,r)}));
const lockFile=toggle('lock-file','Lock file',
 'Unlocked → locked: the raised shackle swings down about its left leg and seats in the body. Strong: a small extra lift, a fast slam with a hit that thumps the body, and a short bounce. Reverse lifts the shackle open.',[
 ...page('sheet',(t,s,r)=>[movePath(lockBody,0,0,0,0,lockThump(t,s,r)),movePath(uTube(14.1,17.9,14.6,16.05,1.3),lockAngle(t,s,r),...PIV,0,lockLift(t,s,r))]),
 ...posed([stroke('body',lockBody),fill('keyway',circle(16,20.8,.9),'outline'),fill('keyway-solid',circle(16,20.8,.9),'solid')],bodyPose),
 ...posed([stroke('shackle',shackleD)],shacklePose)
],.45,.5,'unlocked','locked',.45,.55);
lockFile.duotoneFill=true;

// ---------- FILE SEARCH ----------
// Magnifier on the sheet: lens r3.1 at (14.4,15.6), handle at 45° to (19.56,20.76). Light: one glide 1.4 right
// and 1 down, then back. Strong: a short draw back, a fast dart with a hit, a settle and a slower return.
const LENS=[14.4,15.6],LR=3.1,HANDLE_END=[14.4+7.3*Math.SQRT1_2,15.6+7.3*Math.SQRT1_2];
const seekX=F([[0,0],[.3,1.4],[.62,0,'smooth']],[[0,0],[.08,-.5],[.3,1.9,'hit'],[.42,1.6],[.7,0,'smooth']]);
const seekY=F([[0,0],[.3,1],[.62,0,'smooth']],[[0,0],[.08,-.4],[.3,1.4,'hit'],[.42,1.1],[.7,0,'smooth']]);
const fileSearch=make('file-search','File search',
 'The magnifier glides across the sheet and returns. Light: one 1.4-unit glide. Strong: a short draw back, a fast dart with a hit, a settle, then a slower return.',[
 ...page('sheet',(t,s)=>[movePath(lollipop(...LENS,LR+1.3,45,2.9,1.3),0,0,0,seekX(t,s),seekY(t,s))]),
 ...posed([stroke('lens',circle(...LENS,LR)),stroke('handle',`M${LENS[0]+LR*Math.SQRT1_2} ${LENS[1]+LR*Math.SQRT1_2} L${HANDLE_END}`)],rigid((t,s)=>({dx:seekX(t,s),dy:seekY(t,s)}))),
 fill('lens-core-solid',(t,s)=>circle(LENS[0]+seekX(t,s),LENS[1]+seekY(t,s),LR-1.3),'solid')
],.62,.7);
fileSearch.duotoneFill=true;

// ---------- FILE PREVIEW ----------
// An eye on the sheet opens: the upper lid rises from the lower lid and the pupil fades in. Solid draws the eye
// as filled lids inside a round window. Stateful: hidden → previewed; reverse closes the eye.
const EC=[16,17.6],EW=4.2,EH=5.2,WELL=6.6;
const openEye=FR([[0,0],[.45,1,'smooth']],[[0,0],[.06,0,'hold'],[.26,1.06,'hit'],[.38,.97],[.48,1]],[[0,1],[.4,0,'smooth']],[[0,1],[.06,1.04],[.26,0,'hit'],[.45,0]]);
const dilate=F([[0,1],[.45,1]],[[0,1],[.26,1,'hold'],[.34,1.3,'hit'],[.5,1]]);
const lid=(k)=>`M${EC[0]-EW} ${EC[1]} C${EC[0]-EW*.55} ${EC[1]+EH*k} ${EC[0]+EW*.55} ${EC[1]+EH*k} ${EC[0]+EW} ${EC[1]}`;
const upperLid=(t,s,r)=>lid(1-2*Math.max(0,openEye(t,s,r)));
const pupilShow=(t,s,r)=>clamp01((openEye(t,s,r)-.3)/.35);
const pupilPose=(x,y,t,s,r)=>scaleAbout(x,y,r?1:dilate(t,s),...EC);
const filePreview=toggle('file-preview','File preview',
 'Hidden → previewed: the upper lid lifts off the lower lid and the pupil appears. Strong: the lid snaps open past its mark with a hit while the pupil dilates and settles. Reverse closes the eye.',[
 ...page('sheet',()=>[circle(...EC,WELL)]),
 ...tube('lower-lid',lid(1)),...tube('upper-lid',upperLid),
 ...posed(shown([fill('pupil',circle(...EC,1.4),'outline'),fill('pupil-solid',circle(...EC,1.4),'solid')],pupilShow),pupilPose)
],.45,.5,'hidden','previewed',.4,.45);
filePreview.duotoneFill=true;

// ---------- FILE TAG ----------
// A label tag hangs from its tip on the sheet and swings. Light: one 8° swing and a small return.
// Strong: a 6° pull back, an 18° swing with a hit and a damped return.
const TIP=[11,17.6];
const tagD=polygon([TIP,[14.4,13.9],[20.8,13.9],[20.8,21.3],[14.4,21.3]],[1,.9,.9,.9,.9]);
const swing=F([[0,0],[.18,8],[.42,-4],[.6,0,'smooth']],[[0,0],[.08,-6],[.26,18,'hit'],[.42,-9],[.56,4],[.7,0]]);
const tagPose=rigid((t,s)=>({angle:swing(t,s),pivot:TIP}));
const fileTag=make('file-tag','File tag',
 'A label tag hangs from its tip on the sheet and swings. Light: one 8° swing with a small return. Strong: a pull back, an 18° swing with a hit and a damped return.',[
 ...page('sheet',(t,s)=>[movePath(tagD,swing(t,s),...TIP)]),
 ...posed([stroke('tag',tagD),fill('eyelet',circle(14.6,17.6,.85),'outline'),fill('eyelet-solid',circle(14.6,17.6,.85),'solid')],tagPose)
],.6,.7);
fileTag.duotoneFill=true;

// ---------- FILE STAR ----------
// A star on the sheet fills in. Light: the star swells 8% while its fill fades in. Strong: it presses to 80%,
// pops to 118% with a hit as the fill lands, then settles. Stateful: unstarred → starred; reverse empties it.
const SC=[16,17.8],RO=4.6,RI=2.5;
const starScale=FR([[0,1],[.2,1.08],[.4,1,'smooth']],[[0,1],[.1,.8],[.28,1.18,'hit'],[.4,.95],[.55,1]],[[0,1],[.35,1]],[[0,1],[.1,1.1],[.28,.8,'hit'],[.42,1.03],[.5,1]]);
const starFill=FR([[0,0],[.35,1,'smooth']],[[0,0],[.14,0],[.3,1]],[[0,1],[.35,0,'smooth']],[[0,1],[.12,1],[.28,0]]);
const starPose=(x,y,t,s,r)=>scaleAbout(x,y,starScale(t,s,r),...SC);
const fileStar=toggle('file-star','File star',
 'Unstarred → starred: the star on the sheet fills in with a small swell. Strong: a press, a pop past size with a hit as the fill lands, then a settle. Reverse empties the star.',[
 ...page('sheet',(t,s,r)=>{const k=starScale(t,s,r);return [star(...SC,RO*k,RI*k)];}),
 ...posed([stroke('star',star(...SC,RO,RI)),fill('star-fill',star(...SC,RO,RI),'outline',{visibility:starFill})],starPose),
 fill('star-core-solid',(t,s,r)=>{const k=starScale(t,s,r);return star(...SC,RO*.62*k,RI*.62*k);},'solid',{visibility:starFill})
],.4,.55,'unstarred','starred',.35,.5);
fileStar.duotoneFill=true;

// ---------- APPROVE FILE ----------
// A check draws across the sheet from its left end. Light: one 0.45 s draw. Strong: a fast 0.26 s strike and the
// sheet thumps 0.4 as the stroke lands. Stateful: pending → approved; reverse withdraws the check.
const CK=[[11.8,18.2],[14.8,21.2],[20.4,13.6]],L1=Math.hypot(3,3),L2=Math.hypot(5.6,7.6);
const draw=FR([[0,0],[.45,1,'out']],[[0,0],[.06,0,'hold'],[.3,1]],[[0,1],[.4,0,'smooth']],[[0,1],[.24,0]]);
const stamp=FR([[0,0],[.45,0]],[[0,0],[.3,0,'hold'],[.35,.4,'hit'],[.46,0]],[[0,0],[.4,0]],[[0,0],[.24,0]]);
const pen=(u)=>{const d=clamp01(u)*(L1+L2);const p1=d<=L1?lerp(CK[0],CK[1],d/L1):CK[1],p2=d<=L1?p1:lerp(CK[1],CK[2],(d-L1)/L2);return [CK[0],p1,p2];};
const checkD=(t,s,r)=>{const [a,b,c]=pen(draw(t,s,r));return `M${a} L${b} L${c}`;};
const checkShow=(t,s,r)=>clamp01(draw(t,s,r)/.04);
const approveFile=toggle('approve-file','Approve file',
 'Pending → approved: a check draws across the sheet from its left end. Strong: a fast strike that lands with a hit and thumps the sheet. Reverse withdraws the check.',[
 ...posed(page('sheet',(t,s,r)=>{const u=draw(t,s,r);return [polylineTube(pen(u),Math.max(.002,1.3*clamp01(u/.05)))];}),rigid((t,s,r)=>({dy:stamp(t,s,r)}))),
 ...shown([stroke('check',checkD)],checkShow).map(p=>({...p,pose:(x,y,t,s,r)=>[x,y+stamp(t,s,r)]}))
],.45,.46,'pending','approved',.4,.24);
approveFile.duotoneFill=true;

// ---------- RENAME FILE ----------
// A text line with a caret on the sheet is erased and retyped. Light: the line shortens to a stub and grows back
// smoothly. Strong: it is erased fast, pauses, then retypes in four decisive steps while the caret follows.
const TEXT_Y=16.4,START=11,END=17.4;
const textEnd=F([[0,END],[.08,END],[.3,13.6],[.36,13.6],[.7,END,'smooth']],[[0,END],[.06,END,'hold'],[.2,11.8,'hit'],[.28,11.8,'hold'],[.34,13.2],[.36,13.2,'hold'],[.42,14.6],[.44,14.6,'hold'],[.5,16],[.52,16,'hold'],[.6,END]]);
const renameFile=make('rename-file','Rename file',
 'The title line on the sheet is erased and retyped while the caret follows. Light: a smooth shorten-and-regrow. Strong: a fast erase, a pause, then four decisive typing steps.',[
 ...page('sheet',(t,s)=>[capsule([START,TEXT_Y],[textEnd(t,s),TEXT_Y],2.2),capsule([textEnd(t,s)+3.4,TEXT_Y-2.4],[textEnd(t,s)+3.4,TEXT_Y+2.4],2),capsule([START,21.4],[16.4,21.4],2.2)]),
 stroke('text',(t,s)=>`M${START} ${TEXT_Y} L${textEnd(t,s)} ${TEXT_Y}`),
 stroke('caret',(t,s)=>`M${textEnd(t,s)+3.4} ${TEXT_Y-2.5} L${textEnd(t,s)+3.4} ${TEXT_Y+2.5}`),
 stroke('line-2',`M${START} 21.4 L16.4 21.4`)
],.7,.6);
renameFile.duotoneFill=true;

export const icons=[restoreFile,fileSync,lockFile,fileSearch,filePreview,fileTag,fileStar,approveFile,renameFile];
