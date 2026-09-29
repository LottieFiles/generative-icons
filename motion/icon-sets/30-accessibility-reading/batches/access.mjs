import {icons as contrastIcons} from './contrast.mjs';
const highContrast=contrastIcons[0];
// Accessibility & reading, batch A (5 of 10). Access correction: Hearing assistance redrawn.
import {make,toggle,F,rigid,posed,tube,framed,bothFill,circle,annulus,arc,mix} from './helpers.mjs';

// ---------- 1. accessibility (unchanged shape/timing) ----------
const armL=F([[0,0],[.3,12,'smooth'],[.6,0,'smooth']],
             [[0,0],[.14,-6],[.38,30,'hit'],[.52,26],[.85,0]]);
const leftArm=rigid((t,s)=>({angle:armL(t,s),pivot:[16,14.4]}));
const rightArm=rigid((t,s)=>({angle:-armL(t,s),pivot:[16,14.4]}));
const accessibility=make('accessibility','Accessibility',
 'The universal-access figure lifts both arms open from the shoulder and lowers them; ring and head stay fixed.',[
 ...framed('ring',circle(16,16,12.5),annulus(16,16,12.5,2.6)),
 ...bothFill('head',circle(16,9.2,1.6)),
 ...tube('torso','M16 13.2 L16 18.6 L12.8 23.8'),
 ...tube('leg','M16 18.6 L19.2 23.8'),
 ...posed(tube('arm-left','M16 14.4 L11 13.6'),leftArm),
 ...posed(tube('arm-right','M16 14.4 L21 13.6'),rightArm),
],.6,.85);
accessibility.duotoneFill=true;   // enclosed ring disk

// ---------- 2. wheelchair-access (unchanged shape/timing) ----------
const roll=F([[0,0],[.3,2,'smooth'],[.6,0,'smooth']],
             [[0,0],[.14,-.6],[.4,4.4,'launch'],[.56,4.9],[1,0]]);
const chair=rigid((t,s)=>({dx:roll(t,s)}));
const wheelchairAccess=make('wheelchair-access','Wheelchair access',
 'The whole chair rolls forward a short step-free distance and rolls back to rest; nothing bends or squashes.',
 posed([
  ...bothFill('head',circle(13.5,6.8,1.7)),
  ...tube('body','M14.5 11 L14.5 16.5 L20 16.5 L23 22.5'),
  ...framed('wheel',circle(12,22,5.5),annulus(12,22,5.5,2.6)),
 ],chair),.6,1);
wheelchairAccess.duotoneFill=true;   // enclosed wheel disk

// ---------- 3. hearing-assistance: REVISED ear + incoming arcs ----------
// Fixed ear, three open paths, no tint.
//  helix: top hood = true circular arc r6.5 about (21.5,14), then descent and lobe cubics.
//  fold:  inner hook inside the hood, all points 1.97..2.65 units from (21.5,14).
// Moving: two arcs as one rigid group, dx only. Source centre (1,15), sweep -30..30 deg.
//  small arc r3.8 (nearer source), large arc r7.8 (nearer ear); centreline spacing 4.0.
const arrive=F([[0,0],[.3,1.2,'smooth'],[.6,0,'smooth']],
               [[0,0],[.12,-.8],[.36,2.4,'hit'],[.5,1.9],[.9,0]]);
const sound=rigid((t,s)=>({dx:arrive(t,s)}));
const hearingAssistance=make('hearing-assistance','Hearing assistance',
 'Two sound arcs from the left push in toward the fixed ear and ease back; helix, inner fold and lobe never move.',[
 ...tube('helix',arc(21.5,14,6.5,180,360)
   +' C28 18 24.5 19 23.8 22.3 C23.1 25.6 19.1 26.3 17.3 24.6'),
 ...tube('fold','M18.9 18 C18.9 14.5 18.9 11.8 21.5 11.8 C23.3 11.8 23.9 13.6 23.3 14.8'),
 ...posed(tube('arc-small',arc(1,15,3.8,-30,30)),sound),
 ...posed(tube('arc-large',arc(1,15,7.8,-30,30)),sound),
],.6,.9);
hearingAssistance.duotoneFill=false;   // fully open symbol: Duotone must equal Outline

// ---------- 5. reduced-motion (unchanged shape/timing) ----------
const reduce=F([[0,0],[.5,1,'smooth']],
               [[0,0],[.14,-.12],[.42,1.1,'hit'],[.62,.96],[.8,1]]);
const trail=y=>(t,s,r)=>{const p=reduce(t,s);const x=mix(4,8.8,r?1-p:p);return `M${x} ${y} L11 ${y}`;};
const reducedMotion=toggle('reduced-motion','Reduced motion',
 'Motion trails behind the ball shorten to stubs (reduced); reverse extends them again (moving).',[
 ...framed('ball',circle(20.5,16,5.5),circle(20.5,16,6.8)),
 ...tube('trail-top',trail(12)),
 ...tube('trail-mid',trail(16)),
 ...tube('trail-bottom',trail(20)),
],.5,.8,'moving','reduced');
reducedMotion.duotoneFill=true;   // enclosed ball disk

export const icons=[accessibility,wheelchairAccess,hearingAssistance,highContrast,reducedMotion];
