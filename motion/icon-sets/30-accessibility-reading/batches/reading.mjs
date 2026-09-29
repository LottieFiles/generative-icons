// Accessibility & reading — batch 1 (text-size, keyboard, keyboard-shortcuts, focus-indicator, reading-mode)
// Revision 2: joined text-size crossbar; focus-indicator brackets in separate lanes; reading-mode prep -.10.
import {make,toggle,stroke,fill,tube,rect,capsule,holes,rigid,posed,drive} from './helpers.mjs';

// Direction-aware driver: forward keys for the toggle action, reverse keys for its true reverse.
const dir=(fL,fS,rL,rS)=>(t,s,r)=>r?drive(t,s,rL,rS):drive(t,s,fL,fS);
const sm='smooth';

/* ---------- 1. text-size (toggle small -> large) ----------
 Fixed: baseline rule. Moving: one rounded "A" whose height h morphs (legs + crossbar are function
 paths, same topology at every time). Anchor: baseline y=24, axis x=16.
 Proportions: leg half-width W=.52h, crossbar at y=24-.30h. The leg centerline at that height sits at
 x=16±.70W, so the bar's endpoints are placed exactly there: bar and legs share one h and one width,
 and the round caps of the bar lie fully inside the leg paint at every frame (intentional connection).
 h 11 -> 19 (apex y 13 -> 5). Strong: prep h 10.2 at .12s, drive to 19.7 at .40s, settle 19 at .60s. */
const aH=dir([[0,11],[.55,19,sm]],[[0,11],[.12,10.2],[.4,19.7],[.6,19]],
             [[0,19],[.55,11,sm]],[[0,19],[.12,19.8],[.4,10.4],[.6,11]]);
const A_W=.52,A_BAR=.30;
const aLegs=h=>{const w=A_W*h;return `M${16-w} 24 L16 ${24-h} L${16+w} 24`;};
const aBar=h=>{const w=A_W*h*(1-A_BAR),y=24-A_BAR*h;return `M${16-w} ${y} L${16+w} ${y}`;};
const textSize={...toggle('text-size','Text size',
 'One rounded letter A grows from reading size to large size on a fixed baseline and holds; reverse shrinks it.',
 [...tube('legs',(t,s,r)=>aLegs(aH(t,s,r))),
  ...tube('bar',(t,s,r)=>aBar(aH(t,s,r))),
  ...tube('rule','M7 27.8 L25 27.8')],
 .55,.6,'small','large',.55,.6),duotoneFill:false};

/* ---------- 2. keyboard (gesture, returns) ---------- UNCHANGED, protected.
 Whole keyboard is one rigid body (body frame + 3 keys + space bar); pivot irrelevant, translation only.
 Light: lifts 1.6 and seats. Strong: prep down 1 (.12s), decisive rise 4.5 (.30s), one undershoot -.4 (.48s), seated (.62s). */
const kbRise=(t,s)=>drive(t,s,[[0,0],[.25,1.6,sm],[.55,0,sm]],[[0,0],[.12,-1],[.3,4.5],[.48,-.4],[.62,0]]);
const kbPose=rigid((t,s,r)=>({dy:-kbRise(t,s)}));
const keyCap=(x,y)=>`M${x-1} ${y} L${x+1} ${y}`;
const keyboard={...make('keyboard','Keyboard',
 'An on-screen keyboard pops up into place: the rigid keyboard rises, seats, and rests.',
 posed([fill('tint',rect(3.3,9.3,25.4,13.4,2.3),'tint'),
  stroke('body',rect(3.3,9.3,25.4,13.4,2.3)),
  stroke('key1',keyCap(8.5,13.3)),stroke('key2',keyCap(16,13.3)),stroke('key3',keyCap(23.5,13.3)),
  stroke('space','M10 18.7 L22 18.7'),
  fill('solid',holes(rect(2,8,28,16,3.6),
   capsule([7.5,13.3],[9.5,13.3],2.6),capsule([15,13.3],[17,13.3],2.6),capsule([22.5,13.3],[24.5,13.3],2.6),
   capsule([10,18.7],[22,18.7],2.6)),'solid')],kbPose),
 .55,.65),duotoneFill:true};

/* ---------- 3. keyboard-shortcuts (gesture, returns) ---------- UNCHANGED, protected.
 Two rigid keycaps. Left = wide modifier (label bar), right = letter key (short vertical mark).
 Press = vertical translation only; no scaling. Modifier presses first, letter key follows, both release.
 Light travel 1; Strong travel 2.6 / 3 with one bounded rebound on the letter key (3 -> 2.4). */
const dL=(t,s)=>drive(t,s,[[0,0],[.18,1,sm],[.45,1],[.7,0,sm]],[[0,0],[.14,2.6],[.5,2.6,'hold'],[.7,0]]);
const dR=(t,s)=>drive(t,s,[[0,0],[.2,0],[.38,1,sm],[.45,1],[.7,0,sm]],[[0,0],[.16,0,'hold'],[.28,3],[.4,2.4],[.55,2.4,'hold'],[.72,0]]);
const shortcuts={...make('keyboard-shortcuts','Keyboard shortcuts',
 'A modifier keycap presses down, the letter keycap presses after it, and both release: a key combination.',
 [...posed([fill('mod-tint',rect(3.3,10.5,11.7,11,2.3),'tint'),
    stroke('mod-frame',rect(3.3,10.5,11.7,11,2.3)),stroke('mod-bar','M7 16 L11.3 16'),
    fill('mod-solid',holes(rect(2,9.2,14.3,13.6,3.6),capsule([7,16],[11.3,16],2.6)),'solid')],
   rigid((t,s)=>({dy:dL(t,s)}))),
  ...posed([fill('key-tint',rect(19.5,10.5,9.2,11,2.3),'tint'),
    stroke('key-frame',rect(19.5,10.5,9.2,11,2.3)),stroke('key-mark','M24.1 14.5 L24.1 17.5'),
    fill('key-solid',holes(rect(18.2,9.2,11.8,13.6,3.6),capsule([24.1,14.5],[24.1,17.5],2.6)),'solid')],
   rigid((t,s)=>({dy:dR(t,s)})))],
 .7,.75),duotoneFill:true};

/* ---------- 4. focus-indicator (toggle first -> second) ----------
 Fixed: two control bars (open centerlines) x11..21 at y=10 and y=22.
 Moving: one rigid pair of open focus brackets, stems at x=4.5 / 27.5, rounded corners r=1.5, arm tips at
 x=7.2 / 24.8, spanning y=7..13 around the first control. The brackets translate 12 units vertically in
 their own lanes (paint x 3.2..8.5 and 23.5..28.8) and never overlap the bars' lane (paint x 9.7..22.3),
 so no y value can produce a crossing. Open geometry: no tint, Duotone equals Outline.
 Strong: prep up .8 (.10s), decisive drop to 12.7 (.36s), settle 12 (.52s). Reverse mirrors upward. */
const bracketY=dir([[0,0],[.5,12,sm]],[[0,0],[.1,-.8],[.36,12.7],[.52,12]],
                   [[0,12],[.5,0,sm]],[[0,12],[.1,12.8],[.36,-.7],[.52,0]]);
const bracketL='M7.2 7 L6 7 C5.172 7 4.5 7.672 4.5 8.5 L4.5 11.5 C4.5 12.328 5.172 13 6 13 L7.2 13';
const bracketR='M24.8 7 L26 7 C26.828 7 27.5 7.672 27.5 8.5 L27.5 11.5 C27.5 12.328 26.828 13 26 13 L24.8 13';
const focus={...toggle('focus-indicator','Focus indicator',
 'A visible keyboard focus indicator, a pair of open brackets, moves from the first control to the next and holds there; reverse moves it back.',
 [...tube('item-a','M11 10 L21 10'),...tube('item-b','M11 22 L21 22'),
  ...posed([...tube('bracket-l',bracketL),...tube('bracket-r',bracketR)],
   rigid((t,s,r)=>({dy:bracketY(t,s,r)})))],
 .5,.55,'first','second',.5,.55),duotoneFill:false};

/* ---------- 5. reading-mode (toggle full -> reader) ----------
 Fixed: page frame, short title line. Moving: three body lines narrow symmetrically about x=16 into a
 reading column (function paths, same topology). u: 0 full (10..22) -> 1 column (12.5..19.5).
 Strong: prep widens to u=-.10 (.12s), drive to 1.15 (.38s), settle 1 (.55s). Reverse overshoot also -.10.
 Prep -.10 keeps 1.25 between Bold line-cap paint (8.5) and the Bold inner page edge (7.25). */
const rmU=dir([[0,0],[.5,1,sm]],[[0,0],[.12,-.10],[.38,1.15],[.55,1]],
              [[0,1],[.5,0,sm]],[[0,1],[.12,1.2],[.38,-.10],[.55,0]]);
const ends=u=>[10+2.5*u,22-2.5*u];
const bodyLine=y=>(t,s,r)=>{const [l,q]=ends(rmU(t,s,r));return `M${l} ${y} L${q} ${y}`;};
const readingMode={...toggle('reading-mode','Reading mode',
 'A page simplifies into a reading view: the body lines settle into a centered reading column and hold; reverse restores full width.',
 [fill('tint',rect(6,3.5,20,25,2.5),'tint'),
  stroke('page',rect(6,3.5,20,25,2.5)),stroke('title','M10 9 L15 9'),
  stroke('l1',bodyLine(14)),stroke('l2',bodyLine(19)),stroke('l3',bodyLine(24)),
  fill('solid',(t,s,r)=>{const [l,q]=ends(rmU(t,s,r));return holes(rect(4.7,2.2,22.6,27.6,3.8),
   capsule([10,9],[15,9],2.6),capsule([l,14],[q,14],2.6),capsule([l,19],[q,19],2.6),capsule([l,24],[q,24],2.6));},'solid')],
 .5,.55,'full','reader',.5,.55),duotoneFill:true};

export const icons=[textSize,keyboard,shortcuts,focus,readingMode];
