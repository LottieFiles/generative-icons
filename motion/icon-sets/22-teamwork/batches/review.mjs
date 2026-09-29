import {make,stroke,fill,tube,bothFill,rigid,curve,track,circle,rect,polygon,capsule,holes,segment,timePair,mix,geometricRearm} from './helpers.mjs';

const lerp=(a,b,u)=>[a[0]+(b[0]-a[0])*u,a[1]+(b[1]-a[1])*u];
const cut=(a,b,c,d,u)=>{const e=lerp(a,b,u),f=lerp(b,c,u),g=lerp(c,d,u),h=lerp(e,f,u),i=lerp(f,g,u);return[e,h,lerp(h,i,u)];};
// Partial trace with fixed topology: unreached segments collapse onto the pen.
function trace(start,route,p){let left=p*route.reduce((s,r)=>s+r[0],0),d='M'+start,prev=start,cur=start;
 for(const r of route){const u=Math.max(0,Math.min(1,left/r[0]));
  if(r[1]==='L'){if(u)cur=lerp(prev,r[2],u);d+=' L'+cur;}
  else{const c=u?cut(prev,r[2],r[3],r[4],u):[cur,cur,cur];cur=c[2];d+=' C'+c[0]+' '+c[1]+' '+c[2];}
  prev=r.at(-1);left-=r[0];}
 return d;}
const tubeFn=(name,fn,extra={})=>[stroke(name,fn,extra),{name:name+'-solid',d:fn,style:'solid',type:'stroke',width:2.6,...extra}];
const back=(t,a,b)=>track([[a,1],[b,0,'out']],t);

// 1 request-review ---------------------------------------------------------
function requestReview(){
 const sheet=rect(4.5,8,11,17,2);
 const lift=rigid((t,s)=>({dy:curve(t,s,[[0,0],[.22,-1.2,'out'],[.6,0]],[[0,0],[.18,-3.5],[.42,.5],[.62,0]])}));
 const open=(t,s)=>curve(t,s,[[0,0],[.2,0],[.6,1,'out']],[[0,0],[.14,0],[.45,1]]);
 const lower=(t,s)=>{const cy=6.5+10*open(t,s);return `M19.5 11.5 C21.5 ${cy} 25.5 ${cy} 27.5 11.5`;};
 const parts=[
  fill('sheet-interior',sheet,'tint',{pose:lift}),
  stroke('sheet-frame',sheet,{pose:lift}),
  fill('sheet-solid',holes(sheet,capsule([8,13],[12,13],2),capsule([8,17.5],[12,17.5],2)),'solid',{pose:lift}),
  stroke('sheet-line-1',segment([8,13],[12,13]),{pose:lift}),
  stroke('sheet-line-2',segment([8,17.5],[12,17.5]),{pose:lift}),
  ...tube('lid-upper','M19.5 11.5 C21.5 6.5 25.5 6.5 27.5 11.5'),
  ...tubeFn('lid-lower',lower), // opaque join: starts coincident with lid-upper
  ...bothFill('pupil',circle(23.5,11.5,1.2),{visibility:(t,s)=>curve(t,s,[[0,0],[.6,0],[.75,1,'out']],[[0,0],[.45,0],[.6,1]])}),
 ];
 return {...make('request-review','Request review','A sheet is offered up; the reviewer\'s closed eye opens on it and stays open.',parts,.8,.7,['lid-lower','pupil']),duotoneFill:true};
}

// 2 approve-change ----------------------------------------------------------
function approveChange(){
 const card=rect(5,16,22,12.5,2.5);
 const press=rigid((t,s)=>({dy:curve(t,s,[[0,0],[.28,3.4,'in'],[.34,3.4],[.7,0]],[[0,0],[.12,-1.2],[.3,3.4,'hit'],[.48,-.5],[.7,0]])}));
 const p=(t,s)=>curve(t,s,[[0,0],[.3,0],[.7,1,'out']],[[0,0],[.3,0],[.55,1]]);
 const route=[[4.9,'L',[14.5,24.8]],[8.5,'L',[21.5,19.7]]];
 const parts=[
  fill('card-interior',card,'tint'),
  ...tube('card',card), // Solid keeps a 2.6 rim so the drawn check stays visible
  ...tubeFn('check',(t,s)=>trace([10.5,22],route,p(t,s)),{visibility:(t,s)=>p(t,s)>.02?1:0}),
  ...bothFill('stamp-knob',circle(16,5,1.3),{pose:press}),
  ...tube('stamp-stem',segment([16,5],[16,9.5]),{pose:press}), // buried in knob and base
  ...bothFill('stamp-base',capsule([10,10.5],[22,10.5],3.2),{pose:press}), // contact: base-on-card overlaps rim
 ];
 return {...make('approve-change','Approve change','The stamp presses the change card; a check is traced and holds.',parts,.85,.85,['check']),duotoneFill:true};
}

// 3 request-changes ---------------------------------------------------------
function requestChanges(){
 const body=[[18.89,25.23],[27.56,16.56],[24.44,13.44],[15.77,22.11]],bodyD=polygon(body,1);
 const tap=rigid((t,s)=>({angle:curve(t,s,[[0,0],[.18,-6,'out'],[.45,0]],[[0,0],[.14,-16],[.34,3],[.5,0]]),pivot:[14.5,26.5]}));
 const p=(t,s)=>curve(t,s,[[0,0],[.12,0],[.7,1,'out']],[[0,0],[.1,0],[.55,1]]);
 const g=(t,s)=>curve(t,s,[[0,0],[.68,0],[.85,1,'out']],[[0,0],[.52,0],[.65,1]]);
 const route=[[11,'L',[11,6.5]],[8.6,'C',[8.24,6.5],[6,8.74],[6,11.5]],[5.5,'L',[6,17]]];
 const parts=[
  fill('pencil-interior',bodyD,'tint',{pose:tap}),
  stroke('pencil-body',bodyD,{pose:tap}),
  fill('pencil-body-solid',bodyD,'solid',{pose:tap}),
  ...bothFill('pencil-tip',polygon([[14.5,26.5],[15.77,22.11],[18.89,25.23]],.6),{pose:tap}), // shares body edge
  ...tubeFn('arrow',(t,s)=>trace([22,6.5],route,p(t,s)),{visibility:(t,s)=>p(t,s)>.02?1:0}),
  ...tubeFn('head',(t,s)=>{const k=2.6*g(t,s);return `M${6-k} ${17-k} L6 17 L${6+k} ${17-k}`;},{visibility:(t,s)=>g(t,s)>.02?1:0}),
 ];
 return {...make('request-changes','Request changes','The pencil taps; a return arrow draws back and its head lands. Arrow holds.',parts,.9,.7,['arrow','head']),duotoneFill:true};
}

// 4 compare-revisions -------------------------------------------------------
function compareRevisions(){
 const frame=rect(3.5,6.5,25,19,2.5);
 const slide=rigid((t,s)=>({dx:curve(t,s,[[0,0],[.3,2,'out'],[.75,0]],[[0,0],[.28,5.5],[.45,4.9],[.95,0]])}));
 const parts=[
  fill('frame-interior',frame,'tint'),
  ...tube('frame',frame),
  ...tube('bar-left',segment([7.2,10.5],[9.3,10.5])),
  ...tube('bar-right-1',segment([22.2,10.5],[24.8,10.5])),
  ...tube('bar-right-2',segment([22.2,21.5],[24.8,21.5])),
  ...tube('divider',segment([13,6.5],[13,25.5]),{pose:slide}), // ends buried in frame rim
  ...bothFill('knob',circle(13,16,2.4),{pose:slide}),
 ];
 return {...make('compare-revisions','Compare revisions','The before/after slider scrubs across the revisions and returns.',parts,.8,1),duotoneFill:true};
}

// 5 version-history (geometric reverse, no fades) --------------------------
function versionHistory(){
 const sp=(t,s,r)=>r?back(t,.12,.3):curve(t,s,[[0,0],[.05,0],[.4,1,'out']],[[0,0],[.03,0],[.22,1]]);
 const nd=(t,s,r)=>r?back(t,.06,.2):curve(t,s,[[0,0],[.35,0],[.55,1,'out']],[[0,0],[.18,0],[.3,1.18],[.42,1]]);
 const br=(t,s,r)=>r?back(t,0,.12):curve(t,s,[[0,0],[.5,0],[.85,1,'out']],[[0,0],[.28,0],[.5,1]]);
 const parts=[
  ...tubeFn('spine',(t,s,r)=>`M8 24 L8 ${13.8-5.8*sp(t,s,r)}`), // top end buries into node-new
  ...bothFill('node-mid',circle(8,16,2.2)),
  ...bothFill('node-old',circle(8,24,2.2)),
  ...tube('bar-mid',segment([13.5,16],[22,16])),
  ...tube('bar-old',segment([13.5,24],[24,24])),
  ...bothFill('node-new',(t,s,r)=>circle(8,8,2.2*nd(t,s,r)),{visibility:(t,s,r)=>nd(t,s,r)>.03?1:0}),
  ...tubeFn('bar-new',(t,s,r)=>`M13.5 8 L${13.5+12.5*br(t,s,r)} 8`,{visibility:(t,s,r)=>br(t,s,r)>.02?1:0}),
 ];
 return {...make('version-history','Version history','The spine extends up, a new version dot commits and its entry bar draws. Entry holds.',parts,.9,.55),
  stateful:true,initialState:'prepared',finalState:'result',previewReset:true,reverseSuffix:'-reset',forwardLabel:'Play action',reverseLabel:'Reset preview',reverseTimings:timePair(.3,.3),duotoneFill:false};
}

export const icons=[geometricRearm(requestReview()),approveChange(),geometricRearm(requestChanges()),compareRevisions(),versionHistory()];
