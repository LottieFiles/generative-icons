// text-document-editing batch B, clearance correction: find-and-replace and word-wrap replacements
import {make,F,tube,line,rigid,posed,shown,geometricRearm,clamp} from './helpers.mjs';

// ---------- find-and-replace (result icon, preview reset) ----------
// FIXED: row y=11 (6..26), stub y=21 (6..8). MOVING: match (12.5..18.5) kicks down 6u and fades;
// replacement (12.5..22.5) drops from the y=15 channel and lands at y=21.
// Stub end 8 to fragment start 12.5: 4.5u centerline, 1.9u painted at Solid 2.6 (constant: fragments move vertically only).
const frOld=F([[0,0],[.35,1,'smooth']],[[0,0],[.1,-.1],[.3,1]]);
const frNew=F([[0,0],[.2,0],[.6,1,'smooth']],[[0,0],[.18,0,'hold'],[.4,1.08],[.55,1]]);
export const findAndReplace=geometricRearm(make('find-and-replace','Find and replace',
 'The matched fragment kicks down out of the row and fades; the replacement drops into the same channel and lands as a longer bar. Reset retracts geometrically.',
 [...tube('row',`M6 11 L26 11`),...tube('stub',`M6 21 L8 21`),
  ...shown(posed(line('match',[12.5,21],[18.5,21]),rigid((t,s)=>({dy:6*frOld(t,s)}))),(t,s)=>clamp(1-frOld(t,s))),
  ...shown(posed(line('replacement',[12.5,21],[22.5,21]),rigid((t,s)=>({dy:-6*(1-frNew(t,s))}))),(t,s)=>clamp(3*frNew(t,s)))],
 .6,.55));
Object.assign(findAndReplace,{duotoneFill:false});

// ---------- word-wrap (gesture, true return) ----------
// FIXED rows y=9 (6..24), y=16 (6..14), y=23 (6..11).
// MOVING rigid return arrow: stem x=27 y13..17.5, r=3 quarter bend to (24,20.5), 5.5u straight approach
// to tip (18.5,20.5), 45° barbs to (21.5,17.5)/(21.5,23.5).
// Barb/bend: bend endpoint to barb segment 3.89u -> 1.29u painted. Strong peak tip (15,20.5): row-2 end
// (14,16) to upper barb segment 3.89u -> 1.29u; tip to row-3 end (11,23) 4.30u -> 1.70u.
// Prep +0.9 paints stem to 29.2; bottom paint 24.8; top 11.7.
const wwDx=F([[0,0],[.3,-1.8,'smooth'],[.7,0,'smooth']],
             [[0,0],[.1,.9],[.26,-3.5],[.42,.35],[.6,0]]);
export const wordWrap=make('word-wrap','Word wrap',
 'The return arrow presses left toward the wrapped line end and comes back; Strong winds right, throws harder, and settles.',
 [...tube('row-1',`M6 9 L24 9`),...tube('row-2',`M6 16 L14 16`),...tube('row-3',`M6 23 L11 23`),
  ...posed(tube('arrow',`M27 13 L27 17.5 C27 19.157 25.657 20.5 24 20.5 L18.5 20.5 L21.5 17.5 L18.5 20.5 L21.5 23.5`),
    rigid((t,s)=>({dx:wwDx(t,s)})))],
 .7,.65);
Object.assign(wordWrap,{duotoneFill:false});

export const icons=[findAndReplace,wordWrap];
