// motion/icon-sets/30-infrastructure/batch-compute-storage.mjs (draft, revision 02)
import {make,F,rigid,posed,tube,framed,fill,stroke,bothFill,rect,circle,capsule,holes,arc,arcTube} from './helpers.mjs';

const K=.552;
const eLow=(cx,cy,rx,ry)=>`C${cx-rx} ${cy+ry*K} ${cx-rx*K} ${cy+ry} ${cx} ${cy+ry} C${cx+rx*K} ${cy+ry} ${cx+rx} ${cy+ry*K} ${cx+rx} ${cy}`;
const eLowR=(cx,cy,rx,ry)=>`C${cx+rx} ${cy+ry*K} ${cx+rx*K} ${cy+ry} ${cx} ${cy+ry} C${cx-rx*K} ${cy+ry} ${cx-rx} ${cy+ry*K} ${cx-rx} ${cy}`;
const eUp=(cx,cy,rx,ry)=>`C${cx+rx} ${cy-ry*K} ${cx+rx*K} ${cy-ry} ${cx} ${cy-ry} C${cx-rx*K} ${cy-ry} ${cx-rx} ${cy-ry*K} ${cx-rx} ${cy}`;
const ellipse=(cx,cy,rx,ry)=>`M${cx-rx} ${cy} ${eLow(cx,cy,rx,ry)} ${eUp(cx,cy,rx,ry)} Z`;
const solidBox=(x,y,w,h,r)=>rect(x-1.3,y-1.3,w+2.6,h+2.6,r+1.3);
// One-M tube per pin/contact. `tube` already yields the Outline stroke and its 2.6 Solid stroke.
const pin=(name,a,b)=>tube(name,`M${a} L${b}`);

/* ---------- SERVER (unchanged from accepted proposal) ---------- */
const unit=(name,y)=>[
 ...framed(name,rect(4,y,21,8,2.5),holes(solidBox(4,y,21,8,2.5),capsule([8,y+4],[12,y+4],2.6),circle(20.5,y+4,1.3))),
 stroke(name+'-bar',`M8 ${y+4} L12 ${y+4}`),
 fill(name+'-led',circle(20.5,y+4,1.3),'outline')];
const serverDx=F(
 [[0,0],[.08,0],[.42,1.6,'smooth'],[.6,1.6],[1,0,'smooth']],
 [[0,0],[.12,-.6],[.34,3.3],[.46,2.8],[.85,2.8,'hold'],[1.2,0]]);
const server={...make('server','Server',
 'The top rack unit slides out to expose the host, then seats back. Light: quiet 1.6-unit pull. Strong: short push-in, fast 3.9-unit pull with a seated rebound and hold.',
 [...unit('unit-bottom',18),...posed(unit('unit-top',6),rigid((t,s)=>({dx:serverDx(t,s)})))],1,1.2),duotoneFill:true};

/* ---------- DATABASE (unchanged from accepted proposal) ---------- */
const dbDy=F(
 [[0,0],[.08,0],[.45,2,'smooth'],[.6,2],[1,0,'smooth']],
 [[0,0],[.14,-1.6],[.34,2.8,'hit'],[.44,2.3],[.8,2.3,'hold'],[1.2,0]]);
const dbUpper=(t,s)=>{const y=16+dbDy(t,s)-1.3;return `M4.7 8 L4.7 ${y} ${eLow(16,y,11.3,3.4)} L27.3 8 Z`;};
const dbLower=(t,s)=>{const y=16+dbDy(t,s)+1.3;return `M4.7 ${y} ${eLow(16,y,11.3,3.4)} L27.3 24 ${eLowR(16,24,11.3,4.3)} Z`;};
const database={...make('database','Database',
 'A write band drops through the cylinder and returns. Light: 2-unit settle. Strong: lift, 4.4-unit fast drop with a bounded rebound, hold, return.',
 [fill('drum-interior',`M6 8 L6 24 ${eLow(16,24,10,3)} L26 8 ${eUp(16,8,10,3)} Z`,'tint'),
  stroke('drum',`M6 8 L6 24 ${eLow(16,24,10,3)} L26 8`),
  stroke('cap',ellipse(16,8,10,3)),
  fill('cap-solid',holes(ellipse(16,8,11.3,4.3),ellipse(16,8,8.7,1.7)),'solid'),
  fill('tier-upper-solid',dbUpper,'solid'),
  fill('tier-lower-solid',dbLower,'solid'),
  ...posed([stroke('band',`M6 16 ${eLow(16,16,10,3)}`)],rigid((t,s)=>({dy:dbDy(t,s)})))],1,1.2),duotoneFill:true};

/* ---------- PROCESSOR: 12 individual pins buried in the frame paint; larger core, larger well ----------
   Body rect(8,8,16,16,2.5). Pin interior ends 7.5 / 24.5 → cap edge 8.8 / 23.2, inside Bold inner edge 9.3 / 22.7.
   Core side 4.6, corner r .9: max painted radius from (16,16) = 1.4·√2 + .9 = 2.88 at every angle.
   Solid well side 9 (rect 11.5..20.5): minimum inner radius 4.5 → clearance 1.62 throughout rotation (1.25 even for a sharp-cornered core).
   Strong ends at 90°, a symmetric index of the square (same silhouette as 0°); declared cycleDegrees:90. Light returns to numeric 0. */
const cpuAngle=F(
 [[0,0],[.08,0],[.45,20,'smooth'],[.6,20],[1,0,'smooth']],
 [[0,0],[.1,0],[.24,-12],[.4,100],[.52,90],[1,90,'hold']]);
const pinsX=[12,16,20];
const processor={...make('processor','Processor',
 'The core locks with a quarter-turn. Light: quiet 20° twist and release. Strong: 12° wind-up, one 112° snap that overruns to 100°, drops back to the 90° index and holds.',
 [...framed('body',rect(8,8,16,16,2.5),holes(solidBox(8,8,16,16,2.5),rect(11.5,11.5,9,9,2))),
  ...pinsX.flatMap((y,i)=>pin('pin-left-'+i,`5 ${y}`,`7.5 ${y}`)),
  ...pinsX.flatMap((y,i)=>pin('pin-right-'+i,`27 ${y}`,`24.5 ${y}`)),
  ...pinsX.flatMap((x,i)=>pin('pin-top-'+i,`${x} 5`,`${x} 7.5`)),
  ...pinsX.flatMap((x,i)=>pin('pin-bottom-'+i,`${x} 27`,`${x} 24.5`)),
  ...posed(bothFill('core',rect(13.7,13.7,4.6,4.6,.9)),rigid((t,s)=>({angle:cpuAngle(t,s),pivot:[16,16]})))],1,1),duotoneFill:true,cycleDegrees:90};

/* ---------- MEMORY CHIP: five individual contacts buried in the module paint ----------
   Module rect(4,7,24,10); Bold inner edge 15.7. Contact interior start 17.5 → cap edge 16.2, inside the paint. Exterior ends stay 19.
   Rest painted gap to slot centreline 26.5: 7.5 − 2.6 = 4.9; Strong max dy 3.5 leaves 1.4 (unchanged). */
const ramDy=F(
 [[0,0],[.08,0],[.4,1.2,'smooth'],[.55,1.2],[.95,0,'smooth']],
 [[0,0],[.14,-1],[.32,3.5,'hit'],[.42,3,'hold'],[.8,3,'hold'],[1.15,0]]);
const chips=[[6.5,10],[13.5,10],[20.5,10]];
const memoryChip={...make('memory-chip','Memory chip',
 'The RAM module presses toward its slot and lifts clear. Light: 1.2-unit press. Strong: 1-unit lift, 4.5-unit drive to a seated contact, small rebound, release.',
 [...tube('slot','M5 26.5 L27 26.5'),
  ...posed([
   ...framed('module',rect(4,7,24,10,2.5),holes(solidBox(4,7,24,10,2.5),...chips.map(([x,y])=>rect(x,y,5,4,1)))),
   ...chips.map(([x,y],i)=>fill('chip-'+i,rect(x,y,5,4,1),'outline')),
   ...[8,12,16,20,24].flatMap((x,i)=>pin('contact-'+i,`${x} 17.5`,`${x} 19`))
  ],rigid((t,s)=>({dy:ramDy(t,s)})))],.95,1.15),duotoneFill:true};

/* ---------- HARD DRIVE (unchanged from accepted proposal) ---------- */
const hddAngle=F(
 [[0,0],[.08,0],[1.05,360,'smooth']],
 [[0,0],[.14,-25],[.32,210,'pass'],[.5,372],[.62,360],[.95,360]]);
const hardDrive={...make('hard-drive','Hard drive',
 'The platter spins one turn and stops. Light: one smooth revolution. Strong: short reverse wind, a fast full turn that overruns and settles back to the same index.',
 [...framed('body',rect(4,7,24,18,3),holes(solidBox(4,7,24,18,3),circle(11,16,6.5),capsule([20,13.5],[24,13.5],2.6),capsule([20,18.5],[24,18.5],2.6))),
  stroke('status-1','M20 13.5 L24 13.5'),
  stroke('status-2','M20 18.5 L24 18.5'),
  ...bothFill('hub',circle(11,16,1.2)),
  ...posed([stroke('platter',arc(11,16,4,-60,240)),fill('platter-solid',arcTube(11,16,4,-60,240,2.6),'solid')],
   rigid((t,s)=>({angle:hddAngle(t,s),pivot:[11,16]})))],1.05,.95),duotoneFill:true,cycleDegrees:360};

export const icons=[server,database,processor,memoryChip,hardDrive];
