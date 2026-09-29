import {finish,well,F,progress,framed,tube,annulus,circle,rect,rigid,posed,bothFill,shown,fill,stroke} from './helpers.mjs';
const allowance=F([[0,0],[.24,2],[.7,0,'smooth']],[[0,0],[.08,-.7],[.27,5],[.45,4],[.55,5],[.85,0]]);
const apiQuota=finish('api-quota','API quota','Check request allowances: one permit pulls out from a stack of three tickets and returns. Light travels 2 units; Strong pulls 5 units after preparation and catches once. Remaining permits stay fixed; this is a discrete quota, not a usage gauge.',[
 ...framed('permit-top',rect(6,5,16,5,2)),...framed('permit-bottom',rect(6,23,16,5,2)),
 ...posed(framed('permit-active',rect(6,14,16,5,2)),rigid((t,s)=>({dx:allowance(t,s),pivot:[14,16.5]})))
],.7,.85);

const health=progress(.72,.42);
const modelStatus=finish('model-status','Model status','Enable a ready model: the flat diagnostic line becomes a heartbeat and the status lamp fills. Light grows the signal steadily; Strong drives the complete waveform in 0.34 seconds. Reverse returns to the flat offline signal; housing and lamp rim remain visible.',[
 ...well('processor',7,5,18,14,2),...tube('status-lead','M16 19 L16 21.4'),...framed('lamp',circle(16,25,3.6),annulus(16,25,3.6,2.6)),
 ...tube('signal',(t,s,r)=>{const p=health(t,s,r);return `M11 12 L14 12 L16 ${12-3*p} L18 ${12+3*p} L21 12`;}),
 ...shown(bothFill('ready-light',circle(16,25,1.5)),(t,s,r)=>health(t,s,r))
],.72,.42,['offline','ready']);

const backup=progress(.78,.46);
const fallbackModel=finish('fallback-model','Fallback model','Use the reserve model after a primary failure: the upper status stroke turns to a slash while an elbow path reaches the lower backup card. Light extends the route steadily; Strong makes the complete downward handoff in 0.38 seconds. Reverse clears the handoff and restores the primary.',[
 ...well('primary-card',5,5,13,10,1.8),...well('reserve-card',16,20,11,8,1.8),
 ...posed(tube('primary-status','M9.7 10 L13.3 10'),rigid((t,s,r)=>({angle:45*backup(t,s,r),pivot:[11.5,10]}))),
 ...tube('handoff',(t,s,r)=>{const p=backup(t,s,r),v=Math.min(1,p/.65),h=Math.max(0,(p-.65)/.35);return `M9 15 L9 ${15+9*v} L${9+7*h} ${15+9*v}`;}),
 ...shown(tube('backup-ready','M20 24 L23 24'),(t,s,r)=>backup(t,s,r))
],.78,.46,['primary active','backup active']);

// Cached work remains in its own complete housing; only the confirmation ink
// changes state. No transparent document crosses the tray outline.
const cacheHit=progress(.65,.34);
const cacheNod=F([[0,0],[.65,0]],[[0,0],[.08,0],[.2,-.65],[.37,.2],[.5,0]]);
const promptCacheHit=finish('prompt-cache-hit','Prompt cache hit','Reuse cached prompt work: a check draws inside the stored-result card above a fixed cache tray. Light draws in 0.61 seconds; Strong draws in 0.26 seconds while the card lifts 0.65 units and catches once. Reverse retracts the check; the complete card and tray remain visible.',[
 ...well('cache-tray',5,21,22,7,1.8),...tube('cache-grip','M13 24.5 L19 24.5'),
 ...posed([...well('cached-result',10,4,13,12,1.8),
  ...shown(tube('hit',(t,s,r)=>{const p=cacheHit(t,s,r),a=Math.min(1,p/.35),b=Math.max(0,(p-.35)/.65);return `M13.5 10 L${13.5+2*a} ${10+2*a} L${13.5+2*a+4*b} ${10+2*a-4*b}`;}),(t,s,r)=>cacheHit(t,s,r)>.01?1:0)
 ],rigid((t,s)=>({dy:cacheNod(t,s),pivot:[16.5,10]})))
],.65,.5,['cached','cache hit']);

const imageDown=F([[0,0],[.26,1.5],[.65,0,'smooth']],[[0,0],[.07,-.6],[.25,3],[.43,2.5],[.55,3],[.79,0]]);
const audioDown=F([[0,0],[.12,0],[.4,1.5],[.8,0,'smooth']],[[0,0],[.2,0],[.38,3],[.52,2.5],[.64,3],[.94,0]]);
const multimodalInput=finish('multimodal-input','Multimodal input','Accept image and audio together: an image tile and waveform descend toward an input tray in sequence. Light descends 1.5 units; Strong prepares the image and travels 3 units with a short catch. Both input types return; the receiving tray stays fixed.',[
 ...tube('input-tray','M6 22 L6 27 L26 27 L26 22'),
 ...posed([...well('image',4.5,4.5,12,11,1.8),...tube('landscape','M8 11.5 L10.5 8.5 L13 11.5')],rigid((t,s)=>({dy:imageDown(t,s),pivot:[10.5,10]}))),
 ...posed([...tube('audio-left','M21 8 L21 12'),...tube('audio-middle','M24.5 5.5 L24.5 14.5'),...tube('audio-right','M28 8 L28 12')],rigid((t,s)=>({dy:audioDown(t,s),pivot:[24.5,10]})))
],.8,.94);
export const icons=[apiQuota,modelStatus,fallbackModel,promptCacheHit,multimodalInput];
