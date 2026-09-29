// Attach / transfer group, part 1 of the 10-icon set. 32-unit grid.
// All hardware is rigid+posed with static string geometry; paths are one M each.
import {make,geometricRearm,F,rigid,posed,tube,line,framed,rect} from './helpers.mjs';

const open=i=>({...i,duotoneFill:false}); // no enclosed surface: Duotone == Outline

// ---- Paperclip: legs x10/14/18/22 with 4-unit channels; shifted down .4 to protect Strong top margin.
const CLIP='M10 23.4 L10 11.9 C10 7.9 22 7.9 22 11.9 L22 20.4 C22 24.4 14 24.4 14 20.4 L14 15.4 C14 12.7 18 12.7 18 15.4 L18 19.2';
// Page is an open sheet: its top edge has a real aperture x7..25 where the clip legs pass,
// so clip and page paint never cross. Not a closed surface: no tint.
const PAGE='M6.3 12 C5.7 12 5 12.7 5 13.5 L5 27 C5 27.8 5.7 28.5 6.5 28.5 L25.5 28.5 C26.3 28.5 27 27.8 27 27 L27 13.5 C27 12.7 26.3 12 25.7 12';

const clipDown=F([[0,-5],[.58,0,'out']],
                 [[0,-5],[.1,-5.5],[.3,.5,'hit'],[.48,0]]);   // Strong: lift-prep, drive, slide-stop, settle
const clipUp  =F([[0,0],[.58,-5,'out']],
                 [[0,0],[.1,.5],[.3,-5.5,'launch'],[.48,-5]]); // Strong: press-prep, snap free, settle

// fixed: page. driver: clip rig (vertical translate only).
export const attachFile=open(geometricRearm(make('attach-file','Attach file',
 'The clip drops from above and slides onto the sheet through its top aperture; Strong lifts first, drives down and stops with a short slide.',
 [...tube('page',PAGE),...posed(tube('clip',CLIP),rigid((t,s)=>({dy:clipDown(t,s)})))],.6,.5)));

export const detachFile=open(geometricRearm(make('detach-file','Detach file',
 'The clip pulls up off the sheet and rests above it; Strong presses down first, then snaps free and settles.',
 [...tube('page',PAGE),...posed(tube('clip',CLIP),rigid((t,s)=>({dy:clipUp(t,s)})))],.6,.5)));

// ---- Drag and drop: item card 8x5 at top-left, open tray destination bottom-right.
// Final card inside tray: 1.5u painted clearance to walls, 1u to floor at Bold.
const TRAY='M12 17 L12 26.5 C12 27.6 12.9 28.5 14 28.5 L26 28.5 C27.1 28.5 28 27.6 28 26.5 L28 17';
const dragX=F([[0,0],[.1,0],[.42,10,'out']],
              [[0,0],[.08,-.6],[.3,10.4,'hit'],[.46,10]]);
const dragY=F([[0,0],[.1,-1],[.3,-1],[.7,14.5,'out']],
              [[0,0],[.08,-1],[.24,-1],[.4,15,'hit'],[.54,14.5]]);
// fixed: tray. driver: card rig (x then y; Strong picks up, flies, lands with one settle).
export const dragAndDrop=geometricRearm(make('drag-and-drop','Drag and drop',
 'The card lifts, glides right and drops into the tray; Strong picks up sharply, flies straight in and lands with one damped settle.',
 [...tube('tray',TRAY),...posed(framed('card',rect(6,5,8,4.5,1.2),rect(4.7,3.7,10.6,7.1,2.5)),rigid((t,s)=>({dx:dragX(t,s),dy:dragY(t,s)})))],.7,.55));

// ---- Compress files: open-top archive box, a press plate with handle, three sheets.
// Stack closes progressively; each sheet is its own rigid rig driven by the plate contact.
const BOX='M5.5 10 L5.5 26.5 C5.5 27.6 6.4 28.5 7.5 28.5 L24.5 28.5 C25.6 28.5 26.5 27.6 26.5 26.5 L26.5 10';
const lidY =F([[0,0],[.6,5,'out']],       [[0,0],[.1,-.8],[.3,5.3,'hit'],[.5,5]]);
const s1Y  =F([[0,0],[.1,0],[.6,2.8,'out']],[[0,0],[.14,0],[.3,3.1,'hit'],[.5,2.8]]);
const s2Y  =F([[0,0],[.18,0],[.6,1.6,'out']],[[0,0],[.16,0],[.3,1.9,'hit'],[.5,1.6]]);
const s3Y  =F([[0,0],[.26,0],[.6,.4,'out']],[[0,0],[.18,0],[.3,.7,'hit'],[.5,.4]]);
const bar=(n,y)=>line(n,[10,y],[22,y]);
// fixed: box. driver: press rig (plate+handle share one rig). followers: sheets 1..3.
export const compressFiles=open(geometricRearm(make('compress-files','Compress files',
 'The press plate descends into the archive and the three sheets close their gaps in sequence; Strong winds up, strikes, and settles once.',
 [...tube('box',BOX),
  ...posed([...line('plate',[9.5,8],[22.5,8]),...line('handle',[16,5],[16,8])],rigid((t,s)=>({dy:lidY(t,s)}))),
  ...posed(bar('sheet-1',14),rigid((t,s)=>({dy:s1Y(t,s)}))),
  ...posed(bar('sheet-2',19),rigid((t,s)=>({dy:s2Y(t,s)}))),
  ...posed(bar('sheet-3',24),rigid((t,s)=>({dy:s3Y(t,s)})))],.65,.5)));

// ---- Extract files: archive box with two hinged lid flaps; the card rises out once the flaps clear.
const XBOX='M7 14 L7 27 C7 28 7.9 28.5 9 28.5 L23 28.5 C24.1 28.5 25 28 25 27 L25 14';
const flapL=F([[0,0],[.38,-100,'out']],[[0,0],[.2,-112],[.32,-100]]);
const flapR=F([[0,0],[.38,100,'out']], [[0,0],[.2,112],[.32,100]]);
const cardY=F([[0,0],[.3,0],[.85,-13,'out']],[[0,0],[.16,0],[.42,-14,'hit'],[.58,-13]]);
// fixed: box walls. drivers: flap hinges at (7,14) and (25,14); follower: card rig after flaps clear.
export const extractFiles=geometricRearm(make('extract-files','Extract files',
 'Both lid flaps swing open on the box corners, then the card rises clear of the rim and holds; Strong flings the flaps past open and lifts the card with one overshoot.',
 [...tube('box',XBOX),
  ...posed(line('flap-left',[7,14],[14,14]),rigid((t,s)=>({angle:flapL(t,s),pivot:[7,14]}))),
  ...posed(line('flap-right',[25,14],[18,14]),rigid((t,s)=>({angle:flapR(t,s),pivot:[25,14]}))),
  ...posed(framed('card',rect(11,17.7,10,7,1.5),rect(9.7,16.4,12.6,9.6,2.8)),rigid((t,s)=>({dy:cardY(t,s)})))],.85,.6));

export const icons=[attachFile,detachFile,dragAndDrop,compressFiles,extractFiles];
