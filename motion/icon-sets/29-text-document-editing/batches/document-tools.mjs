import {letterCase} from './document-letter-case.mjs';
import {findAndReplace,wordWrap} from './document-clearance.mjs';
// text-document-editing batch B: find-and-replace, page-break, word-wrap, letter-case, line-spacing
import {make,F,tube,line,rigid,posed,shown,rect,hole,framed,toggle,geometricRearm,clamp} from './helpers.mjs';

// ---------- 2. page-break (inserted output, preview reset) ----------
// FIXED: page frame (rect 8,4 16x24), text y=8 and short y=12. MOVING: following line y=16 -> y=24 (dy 8).
// Break dashes at y=18 draw outward from x=14 and x=18 after the line has passed (retract before return via rearm clock).
const pbMove=F([[0,0],[.45,1,'smooth']],[[0,0],[.1,-.03],[.32,1.04],[.45,1]]);
const pbDash=F([[0,0],[.3,0],[.55,1,'smooth']],[[0,0],[.3,0,'hold'],[.45,1]]);
const dashVis=(t,s)=>clamp(20*pbDash(t,s));                                   // zero-length dash is hidden
export const pageBreak=make('page-break','Page break',
 'The following line is pushed down to the next page position; the dashed break then draws outward across the gap. Result holds.',
 [...framed('page',rect(8,4,16,24,2),hole(rect(6.7,2.7,18.6,26.6,3.3),rect(9.3,5.3,13.4,21.4,.7))),
  ...tube('text-1',`M12 8 L20 8`),...tube('text-2',`M12 12 L17 12`),
  ...posed(tube('next-line',`M12 16 L20 16`),rigid((t,s)=>({dy:8*pbMove(t,s)}))),
  ...tube('dash-l',(t,s)=>`M14 18 L${14-2*pbDash(t,s)} 18`,{visibility:dashVis}),
  ...tube('dash-r',(t,s)=>`M18 18 L${18+2*pbDash(t,s)} 18`,{visibility:dashVis})],
 .55,.5);
export const pageBreakIcon=geometricRearm({...pageBreak,duotoneFill:true});

// ---------- 5. line-spacing (gesture, true return) ----------
// FIXED: double arrow x=6 (one path, barbs by retrace), middle row y=16. MOVING: top row (y=11) up, bottom row (y=21) down.
const lsV=F([[0,0],[.35,1,'smooth'],[.75,0,'smooth']],[[0,0],[.1,-.3],[.3,1.15],[.45,.85],[.7,0]]);
export const lineSpacing=make('line-spacing','Line spacing',
 'Outer rows spread apart from the middle row along the double arrow and settle back; Strong squeezes in first, then spreads wider.',
 [...tube('arrow',`M3.5 11.5 L6 9 L8.5 11.5 L6 9 L6 23 L3.5 20.5 L6 23 L8.5 20.5`),
  ...tube('row-mid',`M13 16 L27 16`),
  ...posed(tube('row-top',`M13 11 L27 11`),rigid((t,s)=>({dy:-3*lsV(t,s)}))),
  ...posed(tube('row-bottom',`M13 21 L27 21`),rigid((t,s)=>({dy:3*lsV(t,s)})))],
 .75,.7);
Object.assign(lineSpacing,{duotoneFill:false});

export const icons=[findAndReplace,pageBreakIcon,wordWrap,letterCase,lineSpacing];
