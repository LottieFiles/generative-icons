// transfer-01.mjs — File/Cloud/Batch transfer. Claude Code revision 2 (proposal).
import {make,geometricRearm,F,rigid,posed,tube,framed,polygon,segment,stroke} from './helpers.mjs';

// ---------- shared drawings (32 grid) ----------
// Page: 7-unit diagonal cut. Outline adds a joined fold; Solid keeps the cut silhouette only.
const pageD=polygon([[4,5],[10,5],[17,12],[17,27],[4,27]],[1.6,.5,.5,1.6,1.6]);
const foldD='M10 5 L10 12 L17 12';
// Port rim under the arrow column (open, no tint).
const portD=segment([21,28],[28,28]);
// Arrow column x=24.5.
const upShaft=segment([24.5,23.5],[24.5,16.5]),upHead='M21.7 19.3 L24.5 16.5 L27.3 19.3';
const dnShaft=segment([24.5,7.5],[24.5,14.5]),dnHead='M21.7 11.7 L24.5 14.5 L27.3 11.7';
// Cloud: open-bottom outline, lobes end at x9 and x23.5; arrow space between them.
const cloudD='M9 24 C6 24 4 22 4 19.5 C4 16.5 6.5 14.5 9 14.5 C9.5 10.5 13 7.5 17 7.5 C21 7.5 24 10.3 24.5 13.5 C27 14 28.5 16.5 28.5 19 C28.5 21.5 26.5 24 23.5 24';
const cuShaft=segment([16,27.5],[16,21.5]),cuHead='M13.2 24.3 L16 21.5 L18.8 24.3';
// Batch tray: open rim y24..27.5 (no tint).
const trayD='M4 24 L4 25 C4 26.4 5.1 27.5 6.5 27.5 L25.5 27.5 C26.9 27.5 28 26.4 28 25 L28 24';
const shift=(dt,keys)=>keys.map(([t,v,e])=>[t+dt,v,e]);
const page=()=>[...framed('page',pageD),stroke('fold',foldD)];

// ---------- 1. File upload ----------
// Fixed: page, fold. Driver: arrow dy (rigid). Follower: port recoil. Both end at -4.5.
const fuArrow=F([[0,0],[.58,-4.5,'smooth']],
                [[0,0],[.1,.8,'launch'],[.24,-5.6],[.36,-4.5,'hold'],[.44,-4.5]]);
const fuPort =F([[0,0],[.1,.3,'smooth'],[.46,0,'smooth']],
                [[0,0],[.1,0],[.16,.5,'hit'],[.36,0]]);
const fileUpload={...geometricRearm(make('file-upload','File upload',
 'Cause: page is ready. Action: arrow preloads down, launches up its column and stays advanced; port recoils and settles. Strong preloads deeper, overshoots once and settles to the same advanced position.',
 [...page(),
  ...posed(tube('port',portD),rigid((t,s)=>({dy:fuPort(t,s)}))),
  ...posed([...tube('shaft',upShaft),...tube('head',upHead)],rigid((t,s)=>({dy:fuArrow(t,s)})))],
 .62,.5)),duotoneFill:true};

// ---------- 2. File download ----------
// Driver: arrow dy down. Follower: port dips on impact. Both end at +8.5.
const fdArrow=F([[0,0],[.58,8.5,'smooth']],
                [[0,0],[.12,-1.2,'launch'],[.26,9.5,'hit'],[.34,8.3],[.42,8.5,'hold'],[.5,8.5]]);
const fdPort =F([[0,0],[.52,0],[.62,.3,'smooth'],[.74,0,'smooth']],
                [[0,0],[.26,0],[.31,.6,'hit'],[.48,0]]);
const fileDownload={...geometricRearm(make('file-download','File download',
 'Cause: remote arrow above the page. Action: short lift, drive down the column toward the port; port dips on impact and springs back; arrow rests seated at the same depth in both profiles.',
 [...page(),
  ...posed(tube('port',portD),rigid((t,s)=>({dy:fdPort(t,s)}))),
  ...posed([...tube('shaft',dnShaft),...tube('head',dnHead)],rigid((t,s)=>({dy:fdArrow(t,s)})))],
 .76,.52)),duotoneFill:true};

// ---------- 3. Cloud upload ----------
// Fixed: none. Driver: arrow rigid dy between the lobes. Follower: cloud lifts on catch. Both end at -6.
const cuArrow=F([[0,0],[.6,-6,'smooth']],
                [[0,0],[.1,.8,'launch'],[.26,-7.2],[.38,-6,'hold'],[.46,-6]]);
const cuCloud=F([[0,0],[.5,0],[.62,-.3,'smooth'],[.76,0,'smooth']],
                [[0,0],[.24,0],[.32,-.7,'hit'],[.48,0]]);
const cloudUpload={...geometricRearm(make('cloud-upload','Cloud upload',
 'Cause: arrow waits below the open cloud. Action: preload, then it drives up between the lobes and stays; the cloud lifts on the catch and settles.',
 [...posed(tube('cloud',cloudD),rigid((t,s)=>({dy:cuCloud(t,s)}))),
  ...posed([...tube('shaft',cuShaft),...tube('head',cuHead)],rigid((t,s)=>({dy:cuArrow(t,s)})))],
 .78,.5)),duotoneFill:false};

// ---------- 4. Cloud download ----------
// Fixed: shaft top y13 inside the cloud. Driver: shaft length; head rides the tip. Both end at 13.5.
const cdExt=F([[0,5],[.6,13.5,'smooth']],
              [[0,5],[.12,4,'launch'],[.28,14.7,'hit'],[.36,13.1],[.44,13.5,'hold'],[.52,13.5]]);
const cdCloud=F([[0,0],[.16,.25,'smooth'],[.48,0,'smooth']],
                [[0,0],[.12,0],[.2,.6,'hit'],[.42,0]]);
const cloudDownload={...geometricRearm(make('cloud-download','Cloud download',
 'Cause: arrow stub sits inside the cloud. Action: brief draw-back, then the shaft extends down out of the opening with its head on the tip; the cloud sags on release and recovers. Extended arrow stays.',
 [...posed(tube('cloud',cloudD),rigid((t,s)=>({dy:cdCloud(t,s)}))),
  ...tube('shaft',(t,s)=>`M16 13 L16 ${13+cdExt(t,s)}`),
  ...posed(tube('head','M13.2 15.2 L16 18 L18.8 15.2'),rigid((t,s)=>({dy:cdExt(t,s)-5})))],
 .64,.55)),duotoneFill:false};

// ---------- 5. Batch upload ----------
// Drivers: three arrows (x 8/16/24) staggered. Follower: tray recoil. All end at -5.
const bL=[[0,0],[.58,-5,'smooth']];
const bS=[[0,0],[.08,.7,'launch'],[.22,-6.2],[.32,-5,'hold'],[.4,-5]];
const bTray=F([[0,0],[.12,.35,'smooth'],[.5,0,'smooth']],
              [[0,0],[.08,0],[.14,.5,'hit'],[.32,0]]);
const bArrow=(x,i)=>{const d=F(shift(i*.05,bL),shift(i*.06,bS));
 return posed([...tube('shaft'+i,segment([x,19],[x,14.5])),...tube('head'+i,`M${x-2} 16.5 L${x} 14.5 L${x+2} 16.5`)],rigid((t,s)=>({dy:d(t,s)})));};
const batchUpload={...geometricRearm(make('batch-upload','Batch upload',
 'Cause: three queued items over one tray. Action: they preload and launch upward in quick stagger, each settling at the same advanced position; the tray recoils once.',
 [...posed(tube('tray',trayD),rigid((t,s)=>({dy:bTray(t,s)}))),...bArrow(8,0),...bArrow(16,1),...bArrow(24,2)],
 .72,.55)),duotoneFill:false};

export const icons=[fileUpload,fileDownload,cloudUpload,cloudDownload,batchUpload];
