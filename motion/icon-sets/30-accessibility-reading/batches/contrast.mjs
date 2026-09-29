// Accessibility & reading, batch A. Contrast correction: High contrast retimed only.
// Geometry and paint are byte-identical to the accepted study.
import {make,F,rigid,posed,framed,bothFill,circle,annulus,arc} from './helpers.mjs';

// Dial turn about the disk centre. Light: quiet nudge. Strong: wind-up, one decisive
// twist to 110 deg with an impact at the peak, then one damped return to exactly 0.
// Every key is bounded; no key reaches 360, no segment crosses an angle wrap.
const turn=F([[0,0],[.3,28,'smooth'],[.65,0,'smooth']],
             [[0,0],[.12,-14],[.4,110,'hit'],[.56,88],[.95,0]]);
const half=rigid((t,s)=>({angle:turn(t,s),pivot:[16,16]}));

const highContrast=make('high-contrast','High contrast',
 'The half-filled disk turns like a contrast dial: a small nudge in Light, a decisive wider twist in Strong, and a single controlled return to its original split.',[
 ...framed('ring',circle(16,16,11),annulus(16,16,11,2.6)),
 ...posed(bothFill('half',arc(16,16,11,90,270)+' Z'),half),
],.65,.95);
highContrast.duotoneFill=true;   // enclosed ring disk, unchanged

export const icons=[highContrast];
