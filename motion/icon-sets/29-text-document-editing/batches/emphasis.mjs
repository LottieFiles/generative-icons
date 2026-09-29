import {icons as revised} from './emphasis-reverse.mjs';
const [underline,strikethrough]=revised;
// 20-text-editing/emphasis.mjs — Claude first-pass design, Codex supervises.
import {make,toggle,tube,fill,posed,rigid,F,clamp} from './helpers.mjs';

// Dynamic open centerline drawn at the selected weight in Outline and 2.6 in Solid.
const dyn=(name,fn,extra={})=>[{name,d:fn,style:'outline',type:'stroke',width:2,...extra},{name:name+'-solid',d:fn,style:'solid',type:'stroke',width:2.6,...extra}];
const pose=fn=>rigid(fn);

// ---------- BOLD: gesture (make). Whole B is rigid; it stamps on the baseline.
// Fixed: nothing. Moving: glyph + its two counter tints (translate only, pivot irrelevant).
// Centerline visits middle bar → upper bowl → stem → bottom → lower bowl; every segment drawn once.
const B='M11 15.5 L16.5 15.5 C19.4 15.5 21 13.7 21 11.25 C21 8.8 19.4 7 16.5 7 L11 7 L11 25 L17 25 C20.2 25 22 23 22 20.25 C22 17.5 20.2 15.5 17 15.5 L11 15.5';
const bUpper='M11 7 L16.5 7 C19.4 7 21 8.8 21 11.25 C21 13.7 19.4 15.5 16.5 15.5 L11 15.5 Z';
const bLower='M11 15.5 L17 15.5 C20.2 15.5 22 17.5 22 20.25 C22 23 20.2 25 17 25 L11 25 Z';
// Light: quiet lift and set-down. Strong: raise 3.5, hold the wind-up, drive down in .1s, hit .4 below rest, rebound, settle.
const stamp=F([[0,0],[.22,-1.2,'smooth'],[.5,0,'smooth']],
              [[0,0],[.16,-3.5],[.22,-3.5,'hold'],[.32,.4,'hit'],[.48,-.7],[.75,0]]);
const boldPose=pose((t,s)=>({dy:stamp(t,s)}));
const bold=make('bold','Bold','Toggle strong text emphasis: the B lifts and stamps decisively onto its baseline, then rests where it started.',
 posed(tube('glyph',B),boldPose),.5,.75);
bold.duotoneFill=false; // character counters are open space, not a surface

// ---------- ITALIC: toggle upright ⇄ slanted. Serifs slide opposite ways, stem re-angles (true morph → function path).
// k = horizontal shear at the serifs (2.4 units final, ≈15°). Top bar retraces 4 units to keep one M.
const slant=F([[0,0],[.45,2.4,'smooth']],[[0,0],[.12,-.6],[.32,3],[.6,2.4]]);   // Strong: counter-lean, whip past, settle
const shear=(t,s,r)=>r?2.4-slant(t,s):slant(t,s);                                    // reverse mirrors the same drive
const italicGlyph=(t,s,r)=>{const k=shear(t,s,r);return `M${12+k} 7 L${20+k} 7 L${16+k} 7 L${16-k} 25 L${12-k} 25 L${20-k} 25`;};
const italic=toggle('italic','Italic','Toggle slanted text emphasis: the I leans from upright to italic; reverse stands it back up.',
 dyn('glyph',italicGlyph),.45,.6,'upright','italic');
italic.duotoneFill=false;

// ---------- HEADING: gesture (make). Moving: H rises (promotes) and lands. Fixed: level numeral "1".
// Supported level variants: swap the numeral source mark (1/2/3); only H1 exports in this batch.
const H='M7 7 L7 25 L7 16 L16 16 L16 7 L16 25';           // stems retrace their lower halves to keep one M
const level1='M21.5 18 L23.5 16 L23.5 25';
const promote=F([[0,0],[.25,-1.5,'smooth'],[.55,0,'smooth']],
                [[0,0],[.12,.8],[.3,-3.6],[.5,.3,'hit'],[.62,-.4],[.75,0]]);   // Strong: sink, launch 4 up, drop with impact, settle
const heading=make('heading','Heading','Change a block to a heading: the H steps up and lands back on its baseline beside the level mark.',
 [...posed(tube('glyph',H),pose((t,s)=>({dy:promote(t,s)}))),...tube('level',level1)],.55,.75);
heading.duotoneFill=false;

export const icons=[bold,italic,underline,strikethrough,heading];
