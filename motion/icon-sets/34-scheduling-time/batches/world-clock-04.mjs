// world-clock-revision-03.mjs — World clock only, globe grid simplified. 32-unit grid.
import {make,F,tube,framed,circle,annulus,rigid,posed,geometricRearm} from './helpers.mjs';

const K=.5523,ellipse=(x,y,rx,ry)=>`M${x} ${y-ry} C${x+rx*K} ${y-ry} ${x+rx} ${y-ry*K} ${x+rx} ${y} C${x+rx} ${y+ry*K} ${x+rx*K} ${y+ry} ${x} ${y+ry} C${x-rx*K} ${y+ry} ${x-rx} ${y+ry*K} ${x-rx} ${y} C${x-rx} ${y-ry*K} ${x-rx*K} ${y-ry} ${x} ${y-ry} Z`;
// Duotone tints only the two real enclosed disks (globe, dial) emitted by framed(); tubes add no tint.
const duo=icon=>({...icon,duotoneFill:true});

// Globe (9.9,20.6) r6.5 unchanged. Equator removed. Meridian rx 2.7→2.2 so the two crescent
// lobes and the lens share the disk: centerline widths 4.3 / 4.4 / 4.3.
// Painted clearance at equator height: Regular 2.3/2.4, Bold 1.8/1.9, Solid 1.7/1.8.
// ≥1 painted clearance holds for |dy|≤3.5 (lobes) and ≤3.7 (lens) at Solid; the taper
// beyond that is the intentional pole attachment. Useful openings: 3 globe + 1 dial = 4.
// Dial (22.4,9.6) r6.2 and hands unchanged: minute 2.4 (paint 3.7) vs inner rim 4.9 → 1.2 gap.
// Globe–dial centre distance 16.65, painted radii 7.8+7.5 → 1.35 gap. Sweep x 2.1..29.9, y 2.1..28.4.
const wcMin=F([[0,0],[.08,0],[.62,180,'smooth']],[[0,0],[.1,-8],[.36,186],[.52,180]]);
const wcHour=F([[0,0],[.08,0],[.62,15,'smooth']],[[0,0],[.1,0,'hold'],[.36,15.6],[.52,15]]);
export const worldClock=duo(geometricRearm(make('world-clock','World clock',
 'A globe with one curved meridian sits beside a small zone dial; the minute hand jumps half a turn to the other zone while the hour hand follows. Strong winds back slightly then snaps with one settle. Time retained; preview reset rearms.',
 [...framed('globe',circle(9.9,20.6,6.5),annulus(9.9,20.6,6.5,2.6)),
  ...tube('meridian',ellipse(9.9,20.6,2.2,6.5)),
  ...framed('dial',circle(22.4,9.6,6.2),annulus(22.4,9.6,6.2,2.6)),
  ...posed(tube('minute','M22.4 9.6 L22.4 7.2'),rigid((t,s)=>({angle:wcMin(t,s),pivot:[22.4,9.6]}))),
  ...posed(tube('hour','M22.4 9.6 L24.2 9.6'),rigid((t,s)=>({angle:wcHour(t,s),pivot:[22.4,9.6]})))],
 .66,.56)));

export const icons=[worldClock];
