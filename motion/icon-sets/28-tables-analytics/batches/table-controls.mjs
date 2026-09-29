import {make,toggle,tube,stroke,fill,framed,rect,holes,posed,rigid,F,drive,mix} from './helpers.mjs';
const cut=(x,y,w,h)=>rect(x,y,w,h,1.2);
const box=(x,y,w,h,cuts)=>holes(rect(x-1.3,y-1.3,w+2.6,h+2.6,4),...cuts.map(c=>cut(...c)));
const fwd=f=>(t,s,r)=>{const p=f(t,s);return r?1-p:p;};                 // symmetric rigid turns only
const dir=(fl,fs,rl,rs)=>(t,s,r)=>r?drive(t,s,rl,rs):drive(t,s,fl,fs); // explicitly directed reverse
const draw=(name,d,q,eps=0.001)=>tube(name,(t,s,r)=>d(Math.max(0,q(t,s,r))),{visibility:(t,s,r)=>q(t,s,r)>eps?1:0});

/* SORT ORDER — fixed: mid bar 4..11. moving: 12-unit arrow rigid about (22.5,16); sweep x15.2..29.8, y8.7..23.3. */
const sortP=fwd(F([[0,0],[.5,1,'smooth']],[[0,0],[.14,0,'hold'],[.42,1]]));
const sortA=fwd(F([[0,0],[.5,1,'smooth']],[[0,0],[.12,-.078],[.42,1.089],[.6,1]]));
const sortOrder=toggle('sort-order','Sort order','Direction arrow winds up and flips 180° beside the bars while the outer bars trade lengths; ascending and descending are retained states.',[
 ...tube('bar-mid','M4 16 L11 16'),
 ...tube('bar-top',(t,s,r)=>`M4 9 L${4+mix(5,8.5,sortP(t,s,r))} 9`),
 ...tube('bar-bottom',(t,s,r)=>`M4 23 L${4+mix(8.5,5,sortP(t,s,r))} 23`),
 ...posed([...tube('arrow-stem','M22.5 10 L22.5 22'),...tube('arrow-head','M20 19.5 L22.5 22 L25 19.5')],rigid((t,s,r)=>({angle:180*sortA(t,s,r),pivot:[22.5,16]}))),
],.5,.6,'ascending','descending');

/* SHOW COLUMNS — fixed: frame 4..28×6..26, divider x12, bar-1 x8. prep: bar-2 shifts 20→16. action: boundary x20 + bar-3 x24 wipe down together. */
const showShift=dir([[0,0],[.2,1,'smooth']],[[0,0],[.12,1]],[[.25,1],[.45,0,'smooth']],[[.16,1],[.38,0]]);
const showWipe=dir([[.18,0],[.45,1,'smooth']],[[.12,0],[.3,1]],[[0,1],[.27,0,'smooth']],[[0,1],[.18,0]]);
const barQ=(t,s,r)=>(showWipe(t,s,r)-.2)/.8;   // bar tip trails the divider tip, meets it at y10
const showColumns={...toggle('show-columns','Show columns','Existing content shifts over to make room, then the third column boundary and its content wipe down in one stroke; hiding retracts the wipe and the content spreads back.',[
 ...framed('frame',rect(4,6,24,20,2.4),box(4,6,24,20,[[5.3,7.3,5.4,17.4],[13.3,7.3,13.4,17.4]])),
 stroke('divider-1','M12 6 L12 26'),
 ...tube('bar-1','M8 10 L8 22'),
 ...posed(tube('bar-2','M20 10 L20 22'),rigid((t,s,r)=>({dx:-4*showShift(t,s,r)}))),
 ...draw('divider-2',q=>`M20 6 L20 ${6+20*q}`,showWipe),
 ...draw('bar-3',q=>`M24 10 L24 ${10+12*q}`,barQ),
],.45,.38,'hidden','shown',.45,.38),duotoneFill:true};

/* RESIZE COLUMN — unchanged from revision 01. */
const resizeDx=F([[0,0],[.35,4,'smooth'],[.45,4],[.8,0,'smooth']],[[0,0],[.1,-1],[.32,6.6],[.44,6,'hold'],[.72,0]]);
const resizeColumn={...make('resize-column','Resize column','The column boundary is dragged right along the table, held, then released back; the frame never resizes.',[
 fill('frame-interior',rect(4,4,24,15,2.4),'tint'),stroke('frame-frame',rect(4,4,24,15,2.4)),
 fill('frame-solid',(t,s)=>{const x=16+resizeDx(t,s);return box(4,4,24,15,[[5.3,5.3,x-6.6,12.4],[x+1.3,5.3,25.4-x,12.4]]);},'solid'),
 ...posed(tube('divider','M16 4 L16 19'),rigid((t,s)=>({dx:resizeDx(t,s)}))),
 ...tube('arrow-shaft','M10 25.5 L22 25.5'),...tube('arrow-left','M12.5 23 L10 25.5 L12.5 28'),...tube('arrow-right','M19.5 23 L22 25.5 L19.5 28'),
],.8,.75),duotoneFill:true};

/* GROUP ROWS — fixed: header y8. forward: rows clear the channel (dx≥3.6) before the spine draws. reverse: spine retracts, then rows return. */
const groupDx=dir([[0,0],[.32,1,'smooth']],[[0,0],[.1,-.15],[.3,1.15],[.5,1]],
                  [[.14,1],[.5,0,'smooth']],[[.1,1],[.2,1.12],[.4,-.12],[.55,0]]);
const groupQ=dir([[.3,0],[.5,1,'smooth']],[[.3,0],[.44,1]],
                 [[0,1],[.18,0,'smooth']],[[0,1],[.14,0]]);
const groupRows={...toggle('group-rows','Group rows','Child rows push right under the header, then a section spine draws down to collect them; ungroup retracts the spine before the rows slide back.',[
 ...tube('header','M6 8 L26 8'),
 ...posed([...tube('row-1','M6 15 L22 15'),...tube('row-2','M6 22 L22 22')],rigid((t,s,r)=>({dx:4*groupDx(t,s,r)}))),
 ...draw('spine',q=>`M6 12 L6 ${12+10*q}`,groupQ),
],.5,.55,'flat','grouped',.5,.55),duotoneFill:false};

/* FREEZE COLUMN — unchanged from revision 01. */
const freezeDx=F([[0,0],[.3,-2.5,'smooth'],[.42,-2.5],[.75,0,'smooth']],[[0,0],[.1,1.5],[.3,-4],[.42,-4,'hold'],[.68,.4],[.8,0]]);
const freezeColumn={...make('freeze-column','Freeze column','The scrolling column content slides left against the frozen boundary and springs back; the leading column never moves.',[
 ...framed('frame',rect(4,6,24,20,2.4),box(4,6,24,20,[[5.3,7.3,5.4,17.4],[13.3,7.3,13.4,17.4]])),
 stroke('divider','M12 6 L12 26'),
 ...tube('frozen-bar','M8 10 L8 22'),
 ...posed(tube('scroll-bar','M20 10 L20 22'),rigid((t,s)=>({dx:freezeDx(t,s)}))),
],.75,.8),duotoneFill:true};

/* PIVOT TABLE — geometry/motion unchanged; duotoneFill declared. */
const pivotA=fwd(F([[0,0],[.5,1,'smooth']],[[0,0],[.12,-.13],[.4,1.11],[.58,1]]));
const pivotTable={...toggle('pivot-table','Pivot table','The body value bar winds back and turns a quarter turn from row orientation to column orientation inside the headed table.',[
 ...framed('frame',rect(4,5,24,22,2.4),box(4,5,24,22,[[5.3,6.3,3.4,2.4],[11.3,6.3,15.4,2.4],[5.3,11.3,3.4,14.4],[11.3,11.3,15.4,14.4]])),
 stroke('header','M4 10 L28 10'),stroke('left','M10 5 L10 27'),
 ...posed(tube('bar','M16 18 L22 18'),rigid((t,s,r)=>({angle:90*pivotA(t,s,r),pivot:[19,18]}))),
],.5,.6,'rows','columns'),duotoneFill:true};

export const icons=[{...sortOrder,duotoneFill:false},showColumns,resizeColumn,groupRows,freezeColumn,pivotTable];
