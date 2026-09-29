// social-broadcast-01 — emoji-picker revision 02 (Claude). 32-unit grid.
import {make,F,fill,stroke,rigid,posed,circle,arc,holes,arcTube} from './helpers.mjs';

const face=circle(16,16,11),eyeL=circle(12,12.5,1.3),eyeR=circle(20,12.5,1.3);

// Smile amount o: 0 rest, 1.5 Light peak, 3.5 Strong peak, -0.6 Strong prep.
// Circular arc through the old cubic's endpoints (16±h, y) with sag s.
// Clamp keeps span in (95°,170°) so arc() always emits 2 cubics per arc.
const mouthGeo=o=>{
 o=Math.max(-.8,Math.min(3.8,o));
 const h=5.5+.4*o,s=3+.9*o,y=17.5-.2*o,r=(h*h+s*s)/(2*s),ang=Math.asin(h/r)*180/Math.PI;
 return {cy:y+s-r,r,a:90+ang,b:90-ang}; // a>b: left → bottom → right
};
const mouth=o=>{const m=mouthGeo(o);return arc(16,m.cy,m.r,m.a,m.b);};
// Solid counter: 2.6-wide closed envelope with round caps, constant 8 cubics.
// Negative winding (opposite to circle()) so hole() appends it as-is.
const mouthCut=o=>{const m=mouthGeo(o);return arcTube(16,m.cy,m.r,m.a,m.b,2.6);};

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

export const icons=[emojiPicker];
