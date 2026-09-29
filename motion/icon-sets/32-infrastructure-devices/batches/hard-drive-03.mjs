// motion/icon-sets/30-infrastructure/batch-hard-drive.mjs (draft, revision 03)
import {make,F,rigid,posed,framed,fill,stroke,bothFill,rect,circle,capsule,holes,arc,arcTube} from './helpers.mjs';

const solidBox=(x,y,w,h,r)=>rect(x-1.3,y-1.3,w+2.6,h+2.6,r+1.3);

/* HARD DRIVE
   Body rect(4,7,24,18,3). Bold inner counter: x 5.25..26.75, y 8.25..23.75.
   Spindle centre (CX,16) shared by pivot, hub, platter and Solid well. */
const CX=11.8;
const hddAngle=F(
 [[0,0],[.08,0],[1.05,360,'smooth']],
 [[0,0],[.14,-25],[.32,210,'pass'],[.5,372],[.62,360],[.95,360]]);
const hardDrive={...make('hard-drive','Hard drive',
 'The platter spins one turn and stops. Light: one smooth revolution. Strong: short reverse wind, a fast full turn that overruns and settles back to the same index.',
 [...framed('body',rect(4,7,24,18,3),holes(solidBox(4,7,24,18,3),circle(CX,16,6.5),capsule([21,13.5],[24,13.5],2.6),capsule([21,18.5],[24,18.5],2.6))),
  stroke('status-1','M21 13.5 L24 13.5'),
  stroke('status-2','M21 18.5 L24 18.5'),
  ...bothFill('hub',circle(CX,16,1.2)),
  ...posed([stroke('platter',arc(CX,16,4,-60,240)),fill('platter-solid',arcTube(CX,16,4,-60,240,2.6),'solid')],
   rigid((t,s)=>({angle:hddAngle(t,s),pivot:[CX,16]})))],1.05,.95),duotoneFill:true,cycleDegrees:360};

export const icons=[hardDrive];
