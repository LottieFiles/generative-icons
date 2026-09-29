// Everyday controls · batch 1 · Claude revision 2 (draft, not integrated)
import {make,toggle,F,posed,rigid,line,tube,fill,circle,arc} from './helpers.mjs';

const arcL=(...a)=>arc(...a).replace(/^M/,'L'); // continue an arc inside one M path

// ---------------------------------------------------------------- add-remove (unchanged)
// Toggle 'add' -> 'remove'. FIXED: bar-h centerline y=16. MOVING: bar-v, rigid about [16,16].
const foldFwd=F([[0,0],[.5,90,'smooth']],[[0,0],[.12,-14],[.3,100],[.52,90]]);
const foldRev=F([[0,90],[.5,0,'smooth']],[[0,90],[.12,104],[.3,-10],[.52,0]]);
const fold=rigid((t,s,r)=>({angle:(r?foldRev:foldFwd)(t,s),pivot:[16,16]}));
const addRemove={...toggle('add-remove','Add / remove',
 'The vertical bar folds down onto the horizontal bar: plus becomes minus and holds; reverse stands it back up.',
 [...line('bar-h',[9,16],[23,16]),
  ...posed(line('bar-v',[16,9],[16,23]),fold)],
 .5,.52,'add','remove'),duotoneFill:false};

// ---------------------------------------------------------------- close (+ cycleDegrees)
// Gesture. Both strokes MOVING as one rigid body about [16,16]. The endpoint is the same
// aggregate paint after a 90° turn, not per-part identity: declared via cycleDegrees:90.
const twistDrive=F([[0,0],[.5,90,'smooth']],[[0,0],[.1,-22],[.28,108],[.52,90]]);
const twist=rigid((t,s)=>({angle:twistDrive(t,s),pivot:[16,16]}));
const close={...make('close','Close',
 'The X twists a quarter turn to dismiss; Strong winds back and whips through with one overshoot.',
 posed([...line('stroke-a',[9.64,9.64],[22.36,22.36]),
        ...line('stroke-b',[22.36,9.64],[9.64,22.36])],twist),
 .5,.52),duotoneFill:false,cycleDegrees:90};

// ---------------------------------------------------------------- power (unchanged)
// Toggle 'off' -> 'on'. FIXED: ring arc r=9 about (16,17), gap ±48° at top. MOVING: bar dy.
const riseFwd=F([[0,5],[.45,0,'smooth']],[[0,5],[.1,6.4],[.27,-1.2],[.5,0]]);
const riseRev=F([[0,0],[.45,5,'smooth']],[[0,0],[.1,-1],[.27,6.2],[.5,5]]);
const plunger=rigid((t,s,r)=>({dy:(r?riseRev:riseFwd)(t,s)}));
const power={...toggle('power','Power',
 'The bar rises out of the ring and locks into the top gap to switch on; reverse sinks it back to off.',
 [...tube('ring',arc(16,17,9,-48,228)),
  ...posed(line('bar',[16,6.5],[16,15.5]),plunger)],
 .45,.5,'off','on'),duotoneFill:false};

// ---------------------------------------------------------------- more-options (revised)
// Gesture, horizontal ellipsis (vertical orientation is a listed variant, not built).
// FIXED: dot-2 at (16,16). MOVING: dot-1, dot-3 rigid dx mirrored outward, no pivot.
// Outline dot: filled disk whose radius equals the selected stroke width w
// (Light r=1.5, Regular r=2, Bold r=2.5) via the d(t,s,r,w) function argument.
// Solid dot: its own drawing, r=2.6, matching the family's Solid 2.6 envelope.
// Same topology at every time and weight; no strokes, no invisible parts.
const spread=F([[0,0],[.25,1.6,'smooth'],[.55,0,'smooth']],[[0,0],[.1,-1.2],[.26,3.4],[.54,0]]);
const dot=(n,x)=>[fill('dot-'+n,(t,s,r,w=2)=>circle(x,16,w),'outline'),
                  fill('dot-'+n+'-solid',circle(x,16,2.6),'solid')];
const moreOptions={...make('more-options','More options',
 'The outer dots spread away from the centre dot to open the overflow, then settle back.',
 [...dot(2,16),
  ...posed(dot(1,8),rigid((t,s)=>({dx:-spread(t,s)}))),
  ...posed(dot(3,24),rigid((t,s)=>({dx:spread(t,s)})))],
 .55,.54),duotoneFill:false};

// ---------------------------------------------------------------- command-menu (revised)
// Gesture. One closed ⌘ path: inner square 12..20, four r=2.75 loops, straight crossings
// at the four inner corners. Open glyph: its five counters are holes, not a key surface,
// so there is no tint part and Duotone equals Outline. Whole glyph MOVING rigidly, dy only.
const cmd=`M12 12 ${arcL(9.25,9.25,2.75,0,-270)} L12 12 L20 12 ${arcL(22.75,9.25,2.75,90,-180)} L20 12 L20 20 ${arcL(22.75,22.75,2.75,180,-90)} L20 20 L12 20 ${arcL(9.25,22.75,2.75,-90,-360)} L12 20 Z`;
const press=F([[0,0],[.22,1.5,'smooth'],[.5,0,'smooth']],[[0,0],[.08,-1.2],[.2,3,'hit'],[.36,-.8],[.56,0]]);
const commandMenu={...make('command-menu','Command menu',
 'The command key is pressed: it sinks, bottoms out and springs back to rest.',
 posed(tube('key',cmd),rigid((t,s)=>({dy:press(t,s)}))),
 .5,.56),duotoneFill:false};

export const icons=[addRemove,close,power,moreOptions,commandMenu];
