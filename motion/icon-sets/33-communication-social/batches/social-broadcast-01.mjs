// social-broadcast batch — Claude first design pass. 32-unit grid.
import {make,toggle,F,fill,stroke,tube,bothFill,rigid,posed,circle,polygon,arc,holes,segment} from './helpers.mjs';

/* ---------- 1. emoji-picker (gesture) ----------
 fixed: nothing (whole face is one rigid group, pivot 16,16)
 moving: face+eyes tilt rigidly; mouth is the only true morph (function path)
 Solid: filled disk r12.3 with real counters (2 eye holes, mouth band) */
const face=circle(16,16,11),eyeL=circle(12,12.5,1.3),eyeR=circle(20,12.5,1.3);
// o = smile amount: 0 rest, 1.5 Light peak, 3.5 Strong peak (controls y 25, gap to inner rim >=1.35)
const mouthPts=o=>({a:10.5-o*.4,b:21.5+o*.4,y:17.5-o*.2,c:21.5+o,x1:12-o*.3,x2:20+o*.3});
const mouth=o=>{const m=mouthPts(o);return `M${m.a} ${m.y} C${m.x1} ${m.c} ${m.x2} ${m.c} ${m.b} ${m.y}`;};
// Solid mouth counter: 2.6-wide band envelope around the same cubic (matches family width at mid-span)
const mouthCut=o=>{const m=mouthPts(o);return `M${m.a-1.3} ${m.y} C${m.x1-1} ${m.c+1.75} ${m.x2+1} ${m.c+1.75} ${m.b+1.3} ${m.y} L${m.b-1.3} ${m.y} C${m.x2-1} ${m.c-1.75} ${m.x1+1} ${m.c-1.75} ${m.a+1.3} ${m.y} Z`;};
const smile=F([[0,0],[.28,1.5,'smooth'],[.62,1.5],[.95,0,'smooth']],
              [[0,0],[.1,-.6],[.32,3.5],[.5,2.9],[.85,2.9,'hold'],[1.15,0]]);
const tilt=F([[0,0]],[[0,0],[.1,-4],[.32,10],[.5,7],[.85,7,'hold'],[1.15,0]]);
const facePose=rigid((t,s)=>({angle:tilt(t,s),pivot:[16,16]}));
const emojiPicker=make('emoji-picker','Emoji picker',
 'Light: the smile quietly widens and relaxes. Strong: a small head cock back, then the face tips 10 degrees and breaks into a broad grin, holds the expression, and returns.',
 posed([
  fill('face-interior',face,'tint'),
  stroke('face-frame',face),
  fill('eye-l',eyeL,'outline'),fill('eye-r',eyeR,'outline'),
  stroke('mouth',(t,s)=>mouth(smile(t,s))),
  fill('face-solid',(t,s)=>holes(circle(16,16,12.3),eyeL,eyeR,mouthCut(smile(t,s))),'solid'),
 ],facePose),1.0,1.2);
emojiPicker.duotoneFill=true;

/* ---------- 2. thumbs-up-down (toggle, paired states) ----------
 states: 'Thumbs up' (angle 0) <-> 'Thumbs down' (angle 180), reverse plays 180 -> 0
 fixed: none; whole hand+cuff is one rigid group, pivot 16,16. Sweep radius <=13.9 incl. paint.
 cuff = centerline tube at x=6 (gap to hand paint 1.5 at Solid) */
const hand=polygon([[10.1,14],[13,14],[15,8],[18,6.5],[20.2,8.5],[19.6,14],[24,14],[25.4,15.8],[24,24.5],[10.1,24.5]],[.8,1.2,1.6,1.8,1.6,1,1.2,1.8,1.8,.8]);
const handSolid=polygon([[8.8,12.7],[12.2,12.7],[14.2,6.9],[18,4.9],[21.4,7.6],[21,12.7],[24.5,12.7],[26.7,15.4],[25.3,25.8],[8.8,25.8]],[1.2,1.6,2,2.4,2,1.4,1.6,2.4,2.4,1.2]);
const spin=F([[0,0],[.55,180,'smooth']],
             [[0,0],[.12,-14],[.42,192],[.62,180]]);
const thumbPose=rigid((t,s,r)=>({angle:r?180-spin(t,s):spin(t,s),pivot:[16,16]}));
const thumbs=toggle('thumbs-up-down','Thumbs up / down',
 'Thumbs up turns 180 degrees into thumbs down and holds. Light: one smooth turn. Strong: a 14-degree wind-up, a fast turn that overshoots 12 degrees, then settles on the exact down pose. Reverse turns back to up.',
 posed([
  fill('hand-interior',hand,'tint'),
  stroke('hand-frame',hand),
  fill('hand-solid',handSolid,'solid'),
  ...tube('cuff',segment([6,15],[6,23.5])),
 ],thumbPose),.55,.62,'Thumbs up','Thumbs down');
thumbs.duotoneFill=true;

/* ---------- 3. rss-feed (gesture) ----------
 fixed: signal dot (11,23) r2 (true filled disk)
 moving: two arcs, radius morph only (fixed 1-segment topology), centered on the dot
 rest radii 8 / 14; Strong peaks 10.5 / 17 (outer paint x<=29.3, y>=4.7) */
const ring=r=>arc(11,23,r,-90,0);
const inner=F([[0,0],[.25,.9,'smooth'],[.65,0,'smooth']],
              [[0,0],[.12,-1],[.36,2.5],[.55,2],[.85,0]]);
const outer=F([[0,0],[.1,0],[.35,1.1,'smooth'],[.75,0,'smooth']],
              [[0,0],[.08,0],[.2,-1],[.44,3],[.63,2.4],[.93,0]]);
const rss=make('rss-feed','RSS feed',
 'Light: the two arcs breathe outward from the signal point, inner first, and settle. Strong: both arcs draw in toward the dot, then push out in a staggered, decisive emit with one settle back to rest. The dot never moves.',
 [
  ...bothFill('dot',circle(11,23,2)),
  ...tube('arc-inner',(t,s)=>ring(8+inner(t,s))),
  ...tube('arc-outer',(t,s)=>ring(14+outer(t,s))),
 ],.8,.95);
rss.duotoneFill=false;

/* ---------- 4. announcement (gesture) ----------
 fixed: two sound arcs centered on the horn mouth (18.5,15); radius morph only
 moving: horn body + handle as one rigid group, pivot (12.3,17.5) at handle root
 handle legs terminate on the body edge centerline (caps buried in body paint) */
const horn=polygon([[4,12.5],[8.6,12.5],[18.5,7.5],[18.5,22.5],[8.6,17.5],[4,17.5]],[1,.8,1.4,1.4,.8,1]);
const hornSolid=polygon([[2.7,11.2],[8.4,11.2],[19.8,6],[19.8,24],[8.4,18.8],[2.7,18.8]],[1.6,1.2,1.8,1.8,1.2,1.6]);
const wave=r=>arc(18.5,15,r,-35,35);
const kick=F([[0,0],[.3,-2.5,'smooth'],[.7,0,'smooth']],
             [[0,0],[.12,3],[.34,-7],[.55,-4.5],[.9,0]]);
const flare=F([[0,0],[.3,.7,'smooth'],[.75,0,'smooth']],
              [[0,0],[.12,0],[.34,1.5],[.55,1.2],[.9,0]]);
const hornPose=rigid((t,s)=>({angle:kick(t,s),pivot:[12.3,17.5]}));
const announcement=make('announcement','Announcement',
 'Light: the megaphone eases back a touch while its sound arcs swell and settle. Strong: the horn dips 3 degrees to set, then kicks back 7 degrees as both arcs blast outward 1.5 units, one damped return to rest.',
 [
  ...posed([
   fill('horn-interior',horn,'tint'),
   stroke('horn-frame',horn),
   fill('horn-solid',hornSolid,'solid'),
   ...tube('handle','M10 18 L10 24 L14.6 24 L14.6 20.5'),
  ],hornPose),
  ...tube('wave-inner',(t,s)=>wave(4.8+flare(t,s))),
  ...tube('wave-outer',(t,s)=>wave(8.6+flare(t,s))),
 ],.75,.9);
announcement.duotoneFill=true;

export const icons=[emojiPicker,thumbs,rss,announcement];
