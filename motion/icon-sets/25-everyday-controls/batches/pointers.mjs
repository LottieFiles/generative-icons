// motion/icon-sets/25-everyday-controls/batch-pointers.mjs — corrected draft, not integrated.
import {make,toggle,F,posed,line,tube,framed,bothFill,stroke,fill,circle,annulus,rigid} from './helpers.mjs';

/* ---------- external-link ----------
 Fixed: open frame (single centerline, permanent gap at top-right; it never moves or opens).
 Moving: rigid arrow = shaft + chevron head, translated along its own 45° diagonal (dx=u, dy=-u).
 Light: quiet 1.1u glide out and back. Strong: 0.9u draw-back (prep), launch 2.3u out, one return to rest.
 Drive is 0 at t=0 and at the end of both profiles, so rest and reverse endpoints are identical. */
const elDrive=F(
 [[0,0],[.26,1.1,'smooth'],[.52,0,'smooth']],
 [[0,0],[.13,-.9,'launch'],[.31,2.3],[.72,0]]);
const elPose=rigid((t,s)=>{const u=elDrive(t,s);return{dx:u,dy:-u};});
const externalLink={...make('external-link','External link',
 'A fixed window frame with an open top-right corner; the arrow slides out through that opening and returns to rest.',[
 ...tube('frame','M13 7 L9 7 C7.9 7 7 7.9 7 9 L7 23 C7 24.1 7.9 25 9 25 L23 25 C24.1 25 25 24.1 25 23 L25 19'),
 ...posed([...line('shaft',[15.5,16.5],[25,7]),...tube('head','M18.5 7 L25 7 L25 13.5')],elPose)
],.52,.72),duotoneFill:false};

/* ---------- share ----------
 Fixed: hub node at (9,16). Moving: two rigid arms (link + outer node) pivoting on the hub centre.
 Light: arms fan 4° open and close. Strong: 4° squeeze inward (prep), fling 12° open, one settle to rest.
 Node Solid r4.1 = Outline centreline r2.8 + Bold half-width 1.3, matching the full painted envelope.
 Duotone tints the three real node discs only. Outline and movement unchanged. */
const shDrive=F(
 [[0,0],[.24,4,'smooth'],[.5,0,'smooth']],
 [[0,0],[.12,-4,'launch'],[.3,12],[.66,0]]);
const shArm=(id,sign)=>posed([
 ...line('link-'+id,[11.46,16-sign*1.33],[19.54,16-sign*5.67]),
 ...framed('node-'+id,circle(22,16-sign*7,2.8),circle(22,16-sign*7,4.1))
],rigid((t,s)=>({angle:-sign*shDrive(t,s),pivot:[9,16]})));
const share={...make('share','Share',
 'Two branch nodes fan out from the source hub and settle back, a hand-off gesture.',[
 ...framed('hub',circle(9,16,2.8),circle(9,16,4.1)),
 ...shArm('top',1),...shArm('bottom',-1)
],.5,.66),duotoneFill:true};

/* ---------- zoom-in-out (toggle: zoom-out → zoom-in, reverse zoom-in → zoom-out) ----------
 Unchanged. Fixed group (lens ring, handle, horizontal bar) shares only the Strong press follower.
 Moving: vertical bar, rigid rotation about the lens centre (14,14). Minus = bar at 90°, plus = 0°.
 Light: 90° smooth turn. Strong: -11° wind-up, snap with 6° overshoot, settle, plus 1u press. */
const zP=F([[0,0],[.45,1,'smooth']],[[0,0],[.1,-.12,'launch'],[.3,1.07],[.54,1]]);
const zV=F([[0,0]],[[0,0],[.1,-.4,'launch'],[.3,1],[.58,0]]);
const zPress=rigid((t,s)=>{const v=zV(t,s);return{dx:v,dy:v};});
const zBar=rigid((t,s,r)=>{const v=zV(t,s),p=zP(t,s);return{angle:r?90*p:90-90*p,pivot:[14,14],dx:v,dy:v};});
const zoomInOut={...toggle('zoom-in-out','Zoom in / out',
 'A magnifier lens: the upright bar swings down onto the horizontal bar (minus) or stands up (plus) while the glass presses in.',[
 ...posed([...framed('lens',circle(14,14,7.5),annulus(14,14,7.5,2.6)),
  ...line('handle',[19.9,19.9],[25.6,25.6]),...line('bar-h',[10.5,14],[17.5,14])],zPress),
 ...posed(line('bar-v',[14,10.5],[14,17.5]),zBar)
],.45,.58,'zoom-out','zoom-in'),duotoneFill:true};

/* ---------- move-tool ----------
 Unchanged. One rigid handle (two centerline bars + four chevron heads) dragged horizontally.
 Light: 1.5u drift right and back. Strong: 1u grab back left (prep), 2.8u decisive drag right, one return. */
const mvDrive=F(
 [[0,0],[.25,1.5,'smooth'],[.5,0,'smooth']],
 [[0,0],[.12,-1,'launch'],[.3,2.8],[.7,0]]);
const mvPose=rigid((t,s)=>({dx:mvDrive(t,s)}));
const heads=[['up','M12.7 9.8 L16 6.5 L19.3 9.8'],['down','M12.7 22.2 L16 25.5 L19.3 22.2'],
 ['left','M9.8 12.7 L6.5 16 L9.8 19.3'],['right','M22.2 12.7 L25.5 16 L22.2 19.3']];
const moveTool={...make('move-tool','Move tool',
 'A four-way handle is picked up and dragged sideways, then set back at its origin.',
 posed([...line('bar-h',[6.5,16],[25.5,16]),...line('bar-v',[16,6.5],[16,25.5]),
  ...heads.flatMap(([n,d])=>tube('head-'+n,d))],mvPose),.5,.7),duotoneFill:false};

/* ---------- target ----------
 Reticle, not a lens: no tint, open centre in Duotone. Fixed: centre dot. Moving: rigid reticle =
 outline ring (stroke) / Solid annulus + four ticks, translated diagonally (dx=dy=u).
 Light: reticle drifts 0.9u off centre and eases back. Strong: drifts 2.0u off (prep), snaps onto
 the dot, 0.5u bounce past centre, settles centred. Timing unchanged from the accepted proposal. */
const tgDrive=F(
 [[0,0],[.25,-.9,'smooth'],[.5,0,'smooth']],
 [[0,0],[.22,-2.0,'launch'],[.36,.5],[.62,0]]);
const tgPose=rigid((t,s)=>{const u=tgDrive(t,s);return{dx:u,dy:u};});
const target={...make('target','Target',
 'A reticle slips off its mark and snaps back to centre on the fixed target dot.',[
 ...bothFill('dot',circle(16,16,1.1)),
 ...posed([stroke('ring',circle(16,16,6.5)),fill('ring-solid',annulus(16,16,6.5,2.6)),
  ...line('tick-up',[16,6],[16,10]),...line('tick-down',[16,22],[16,26]),
  ...line('tick-left',[6,16],[10,16]),...line('tick-right',[22,16],[26,16])],tgPose)
],.5,.62),duotoneFill:false};

export const icons=[externalLink,share,zoomInOut,moveTool,target];
