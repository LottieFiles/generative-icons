// Legal & compliance batch C: compliance-check, redact, consent, cookie-banner, data-retention.
import {make,toggle,dir,F,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,rotate,holes,bothFill,capsule,clamp01,arcN,ellArc,L,reveal,hole} from './helpers.mjs';

// ---------- COMPLIANCE CHECK ----------
// A clipboard with three requirement rows. Ticks draw in one after another from their start; the
// checked board is retained and the reverse erases them last-first. Strong ticks fast and the board
// dips as the last one lands.
const BOARD='M11.3 5.5 L7.7 5.5 C6.5 5.5 5.5 6.5 5.5 7.7 L5.5 25.3 C5.5 26.5 6.5 27.5 7.7 27.5 L24.3 27.5 C25.5 27.5 26.5 26.5 26.5 25.3 L26.5 7.7 C26.5 6.5 25.5 5.5 24.3 5.5 L20.7 5.5';
const BOARD_C=rect(5.5,5.5,21,22,2.2),ROWS=[12,17,22];
const tickP=i=>dir(F([[.08+.18*i,0],[.3+.18*i,1,'smooth']],[[.04+.1*i,0],[.16+.1*i,1]]),F([[.05+.15*(2-i),1],[.25+.15*(2-i),0,'smooth']],[[.03+.08*(2-i),1],[.12+.08*(2-i),0]]));
const ticks=ROWS.map((y,i)=>({y,p:tickP(i),pts:[[9.2,y+.2],[11.2,y+2.1],[14.4,y-1.6]]}));
const boardDip=dir(F([[0,0],[.7,0]],[[0,0],[.34,0],[.38,.5,'hit'],[.48,0]]),F([[0,0],[.5,0]],[[0,0],[.3,0]]));
const complianceCheck=toggle('compliance-check','Compliance check',
 'Light: three ticks draw in down the clipboard, one after another. Strong: they snap in quickly and the board dips as the last one lands. The checked board is retained; the reverse erases the ticks.',
 posed([fill('board-interior',BOARD_C,'tint'),stroke('board-frame',BOARD),fill('board-solid',holes(BOARD_C,rect(6.8,6.8,18.4,19.4,1)),'solid'),
  ...framed('clip',rect(12.5,3.5,7,4,1.5)),
  ...ticks.flatMap(({y,p,pts},i)=>[...shown(tube('tick-'+i,(t,s,r)=>reveal(pts,p(t,s,r))),(t,s,r)=>p(t,s,r)>.03?1:0),...tube('row-'+i,`M17 ${y} L23 ${y}`)])],
  rigid((t,s,r)=>({dy:boardDip(t,s,r)}))),
 .7,.5,'pending','compliant',.55,.3,{forward:'Check',reverse:'Clear'});
complianceCheck.duotoneFill=true;

// ---------- REDACT ----------
// A sheet of four lines. Two marker bars (3.6 tall, filled in every style) extend from the left over
// the second and fourth lines. Stateful: visible → redacted; the reverse withdraws the bars.
const SHEET=rect(7,4.5,18,23,2);
const barW=(w,l,h)=>dir(F([[0,.01],[l,.01],[l+.34,w,'smooth']],[[0,.01],[h,.01],[h+.16,w]]),F([[0,w],[l,w],[l+.3,.01,'smooth']],[[0,w],[h,w],[h+.14,.01]]));
const bars=[{y:13.5,w:barW(13.8,.06,.04)},{y:21.5,w:barW(8.5,.26,.18)}];
const kick=dir(F([[0,0],[.6,0]],[[0,0],[.3,0],[.34,.4,'hit'],[.44,0]]),F([[0,0],[.56,0]],[[0,0],[.32,0]]));
const redact=toggle('redact','Redact',
 'Light: two marker bars sweep across the second and fourth lines of the sheet. Strong: the bars are struck through in 0.16 s each and the sheet kicks as the marker lifts. The redacted sheet is retained; the reverse withdraws the bars.',
 posed([...framed('sheet',SHEET,holes(SHEET,rect(8.3,5.8,15.4,20.4,.9))),
  ...[[9.5,11],[13.5,11],[17.5,7],[21.5,11]].flatMap(([y,w],i)=>tube('line-'+i,`M10.5 ${y} L${10.5+w} ${y}`)),
  ...bars.flatMap(({y,w},i)=>shown(bothFill('bar-'+i,(t,s,r)=>rect(8.7,y-1.8,w(t,s,r),3.6,1.8)),(t,s,r)=>w(t,s,r)>.3?1:0))],
  rigid((t,s,r)=>({dy:kick(t,s,r)}))),
 .6,.44,'visible','redacted',.56,.32,{forward:'Redact',reverse:'Reveal'});
redact.duotoneFill=true;

// ---------- CONSENT ----------
// A large consent box beside its statement lines. Agreeing draws the tick inside the box; Strong
// squashes the box under the tap and snaps the tick in. The reverse clears the tick.
const BOX=rect(5,10,12,12,2.4),BC=[11,16],TICK=[[8.4,16.2],[10.5,18.3],[13.9,14.1]];
const agree=dir(F([[0,0],[.06,0],[.42,1,'smooth']],[[0,0],[.06,0],[.2,1]]),F([[0,1],[.3,0,'smooth']],[[0,1],[.2,0]]));
const squashB=dir(F([[0,1],[.42,1]],[[0,1],[.06,1],[.2,.92,'hit'],[.3,1.04],[.4,1]]),F([[0,1],[.3,1]],[[0,1],[.2,1]]));
const boxPose=(x,y,t,s,r)=>{const k=squashB(t,s,r);return [BC[0]+(x-BC[0])*k,BC[1]+(y-BC[1])*k];};
const consent=toggle('consent','Consent',
 'Light: a tick draws inside the consent box. Strong: the box squashes under the tap and the tick snaps in. The agreed state is retained; the reverse clears the tick.',[
 ...posed([...framed('box',BOX,holes(BOX,rect(6.3,11.3,9.4,9.4,1.1))),...shown(tube('tick',(t,s,r)=>reveal(TICK,agree(t,s,r))),(t,s,r)=>agree(t,s,r)>.03?1:0)],boxPose),
 ...tube('statement-0','M20.5 13.5 L27.5 13.5'),...tube('statement-1','M20.5 18.5 L25.5 18.5')
],.42,.4,'pending','agreed',.3,.2,{forward:'Agree',reverse:'Withdraw'});
consent.duotoneFill=true;

// ---------- COOKIE BANNER ----------
// A screen; a banner rises from its bottom edge carrying a bitten cookie and a message line.
// Stateful: hidden → shown. Strong overshoots to 10.2 units tall before settling at 8.5.
const SCREEN=rect(4.5,4.5,23,23,2.5);
const bannerH=dir(F([[0,.01],[.06,.01],[.5,8.5,'smooth']],[[0,.01],[.04,.01],[.22,10.2,'hit'],[.34,8],[.44,8.5]]),F([[0,8.5],[.45,.01,'smooth']],[[0,8.5],[.06,9.3],[.28,.01]]));
const bannerD=h=>rect(7.5,24.2-h,17,h,Math.min(2,h/2));
const bannerS=h=>holes(bannerD(h),rect(8.8,24.2-h+1.3,14.4,Math.max(.01,h-2.6),Math.min(.8,Math.max(.005,(h-2.6)/2))));
const CK=[12.4,19.95];
const cookieD=arcN(...CK,3.1,-15.2,285.2,4)+' '+L(arcN(14.59,17.76,1.6,210.1,59.9,2))+' Z';
const cookieMark=holes(cookieD,circle(11.2,19.2,.7),circle(12.9,21.4,.7));
const contentVis=(t,s,r)=>clamp01((bannerH(t,s,r)-6)/1.5);
const cookieBanner=toggle('cookie-banner','Cookie banner',
 'Light: a banner rises from the bottom of the screen and its cookie and message fade in. Strong: it shoots up past its height, settles and the content lands. The shown banner is retained; the reverse drops it away.',[
 ...framed('screen',SCREEN,holes(SCREEN,rect(5.8,5.8,20.4,20.4,1.3))),
 ...shown([fill('banner-interior',(t,s,r)=>bannerD(bannerH(t,s,r)),'tint'),stroke('banner-frame',(t,s,r)=>bannerD(bannerH(t,s,r))),fill('banner-solid',(t,s,r)=>bannerS(bannerH(t,s,r)),'solid')],(t,s,r)=>bannerH(t,s,r)>.3?1:0),
 ...shown([...bothFill('cookie',cookieMark),...tube('message','M17.6 19.95 L21.2 19.95')],contentVis)
],.5,.44,'hidden','shown',.45,.28,{forward:'Show',reverse:'Dismiss'});
cookieBanner.duotoneFill=true;

// ---------- DATA RETENTION ----------
// A database cylinder with a retention clock badged over its lower-right corner; the cylinder's
// outline stops 7.4 units from the clock centre so the badge never crosses it. The hand sweeps a
// full turn. Strong winds back 25°, sweeps past and settles while the clock jolts.
const CC=[24,22.3];
const sweep=F([[0,0],[.06,0],[.8,360,'smooth']],[[0,0],[.1,-25],[.4,372],[.54,357],[.64,360]]);
const jolt=F([[0,0],[.8,0]],[[0,0],[.4,0],[.43,.5,'hit'],[.54,0]]);
const cylS='M5 8 '+L(ellArc(13,8,8,2.6,180,360,4))+' L21 16.87 '+L(arcN(...CC,6.2,241,164,2))+' '+L(ellArc(13,22,8,2.6,51,180,3))+' Z';
const ellTube=(cx,cy,rx,ry,a,b,w)=>{const h=w/2,p=deg=>[cx+rx*Math.cos(deg*Math.PI/180),cy+ry*Math.sin(deg*Math.PI/180)],start=p(a),end=p(b);return ellArc(cx,cy,rx+h,ry+h,a,b,3)+L(arcN(...end,h,b,b+180,2))+L(ellArc(cx,cy,rx-h,ry-h,b,a,3))+L(arcN(...start,h,a+180,a+360,2))+' Z';};
const handD=`M${CC} L${CC[0]+2.3} ${CC[1]}`;
const dataRetention=make('data-retention','Data retention',
 'Light: the retention clock badged on the database sweeps one full turn. Strong: the hand winds back, whips round past twelve and settles as the clock jolts.',[
 fill('cylinder-interior',cylS,'tint'),
 stroke('top',ellArc(13,8,8,2.6,0,360,4)+' Z'),stroke('side-l','M5 8 L5 22'),stroke('side-r','M21 8 L21 15.5'),
 stroke('band',ellArc(13,15,8,2.6,0,180,3)),stroke('bottom',ellArc(13,22,8,2.6,180,65,3)),
 fill('cylinder-solid',holes(cylS,ellTube(13,8,8,2.6,6,174,1.6),ellTube(13,15,8,2.6,8,172,1.6)),'solid'),
 ...posed([fill('clock-interior',circle(...CC,3.9),'tint'),stroke('clock-frame',circle(...CC,3.9)),
  fill('clock-solid',(t,s)=>holes(circle(...CC,3.9),capsule(CC,rotate(CC[0]+2.3,CC[1],45+sweep(t,s),...CC),1.6)),'solid')],rigid((t,s)=>({dy:jolt(t,s)}))),
 ...posed([stroke('hand',handD)],rigid((t,s)=>({angle:45+sweep(t,s),pivot:CC,dy:jolt(t,s)})))
],.8,.64);
dataRetention.duotoneFill=true;

export const icons=[complianceCheck,redact,consent,cookieBanner,dataRetention];
