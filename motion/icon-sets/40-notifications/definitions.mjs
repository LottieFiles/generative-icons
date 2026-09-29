// Notifications (40). Bells, badges, cards and modes: each icon has a named drawing on the
// 32-unit grid, a Light and a Strong action, and (where a two-state meaning is natural) an
// authored reverse. Design notes sit beside each icon; evidence/ holds the native proofs.
import svgpath from 'svgpath';
import {make,F,P,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,capsule,arc,holes,rotate,bothFill,timePair,track,mix} from '../library/ai-authoring.mjs';

// ---------- shared assembly ----------
const toggle=(name,label,intent,parts,light,strong,initialState,finalState,reverseLight=light,reverseStrong=strong)=>
 ({...make(name,label,intent,parts,light,strong),stateful:true,initialState,finalState,reverseSuffix:'-reverse',forwardLabel:finalState,reverseLabel:initialState,reverseTimings:timePair(reverseLight,reverseStrong)});
const duo=icon=>({...icon,duotoneFill:true});
const open=icon=>({...icon,duotoneFill:false});
const clamp=x=>Math.max(0,Math.min(1,x));
const L=d=>d.replace(/^M/,'L');
const move=(d,dx,dy,k=1)=>svgpath(d).scale(k).translate(dx,dy).toString();
// A growing line from a to the pen tip; hidden while it is a round-cap dot.
const drawn=(name,a,b,u,extra={})=>shown(tube(name,(t,s,r)=>{const k=Math.max(.001,u(t,s,r));return `M${a} L${mix(a[0],b[0],k)} ${mix(a[1],b[1],k)}`;},extra),(t,s,r)=>u(t,s,r)>.03?1:0);
// Z glyph centred on (cx,cy) with half size h.
// Draw-on with fixed topology: every segment becomes a cubic; segments past the pen collapse at
// the tip, the current one is split (de Casteljau), finished ones are complete.
const cubics=d=>{const seg=svgpath(d).abs().unarc().unshort().segments,out=[];let last,first;for(const [c,...v] of seg){if(c==='M'){last=v;first=v;}else if(c==='L'){out.push([last,[mix(last[0],v[0],1/3),mix(last[1],v[1],1/3)],[mix(last[0],v[0],2/3),mix(last[1],v[1],2/3)],v]);last=v;}else if(c==='C'){out.push([last,[v[0],v[1]],[v[2],v[3]],[v[4],v[5]]]);last=[v[4],v[5]];}else if(c==='Z'){if(last[0]!==first[0]||last[1]!==first[1])out.push([last,last,first,first]);last=first;}}return out;};
const splitCubic=(c,u)=>{const m=(a,b)=>[mix(a[0],b[0],u),mix(a[1],b[1],u)];const p01=m(c[0],c[1]),p12=m(c[1],c[2]),p23=m(c[2],c[3]),p012=m(p01,p12),p123=m(p12,p23),q=m(p012,p123);return [c[0],p01,p012,q];};
const drawPath=(cs,p)=>{let d='M'+cs[0][0],tip=cs[0][0];for(let i=0;i<cs.length;i++){const u=clamp(p-i);const c=u>=1?cs[i]:u<=0?[tip,tip,tip,tip]:splitCubic(cs[i],u);tip=c[3];d+=' C'+[...c[1],...c[2],...c[3]];}return d;};
const zed=(cx,cy,h)=>`M${cx-h} ${cy-h} L${cx+h} ${cy-h} L${cx-h} ${cy+h} L${cx+h} ${cy+h}`;

// ---------- the family bell ----------
// Crown at (16,5), skirt to y22.4, base width 16.8; clapper arc 1.3 clear of the skirt at Bold.
// All bells reuse this drawing so the category reads as one voice; accessories carry meaning.
const BELL='M16 5 C11.8 5 9.2 8.6 9.2 13 L9.2 18.6 C9.2 20.2 7.6 21 7.6 22.4 L24.4 22.4 C24.4 21 22.8 20.2 22.8 18.6 L22.8 13 C22.8 8.6 20.2 5 16 5 Z';
const CLAPPER='M13.8 26.2 C14.2 27.4 17.8 27.4 18.2 26.2';
const bell=(dx=0,dy=0,k=1)=>({body:move(BELL,dx,dy,k),clapper:move(CLAPPER,dx,dy,k),crown:[16*k+dx,5*k+dy]});

// ---------- BELL RING ----------
// The bell rocks about its crown; the clapper lags behind it. Two short strike marks flash beside
// the crown on the hard swing. Light rocks 6°; Strong winds up, strikes 14° and rings down.
{
 var bellRing;
 const b=bell();
 const rock=F([[0,0],[.1,6],[.3,-5],[.48,0,'smooth']],[[0,0],[.06,-7],[.18,14,'hit'],[.32,-11],[.46,7],[.58,-3],[.68,0]]);
 const lag=F([[0,0],[.14,8],[.34,-7],[.52,0,'smooth']],[[0,0],[.1,-9],[.23,19,'hit'],[.37,-15],[.5,10],[.62,-4],[.72,0]]);
 const flash=(t,s)=>s?track([[.14,0],[.18,1,'out'],[.3,1],[.4,0,'in']],t):0;
 bellRing=make('bell-ring','Bell ring',
  'Light: the bell rocks 6° about its crown and the clapper follows. Strong: it winds back, strikes 14° with the clapper swinging wide, two strike marks flash, and it rings down to rest.',[
  ...posed(framed('body',b.body),rigid((t,s)=>({angle:rock(t,s),pivot:b.crown}))),
  ...posed(tube('clapper',b.clapper),rigid((t,s)=>({angle:lag(t,s),pivot:b.crown}))),
  ...shown([...tube('mark-l','M8.6 5.6 L10.8 7.4'),...tube('mark-r','M23.4 5.6 L21.2 7.4')],flash)
 ],.48,.72);
}

// ---------- BELL OFF ----------
// A slash draws from upper-left to lower-right across the bell. Solid carries a real gap: a
// capsule counter along the slash inside the body (2.3 clear of the wall) whose width opens
// with the slash, so the "on" state has no hole. Reverse retracts the slash to its start.
{
 var bellOff;
 const b=bell(),A=[6.5,6.5],B=[25.5,25.5],HA=[12.2,12.2],HB=[19.8,19.8];
 const len=Math.hypot(B[0]-A[0],B[1]-A[1]),hLen=Math.hypot(HB[0]-HA[0],HB[1]-HA[1]),aToHa=Math.hypot(HA[0]-A[0],HA[1]-A[1]);
 const fwd=F([[0,0],[.06,0],[.5,1,'smooth']],[[0,0],[.08,0,'hold'],[.24,1.05,'hit'],[.36,1]]);
 const back=F([[0,1],[.42,0,'smooth']],[[0,1],[.06,1.04],[.26,0,'hit']]);
 const u=(t,s,r)=>r?back(t,s):fwd(t,s);
 const shake=F([[0,0],[.5,0]],[[0,0],[.06,3],[.16,-4],[.26,3],[.36,-1.5],[.46,0]]);
 const tilt=(t,s,r)=>r?0:shake(t,s);
 const gapD=(t,s,r)=>{const tip=u(t,s,r)*len,end=clamp((tip-aToHa-2.3)/hLen)*hLen,w=4.6*clamp(end/2.3);const e=[HA[0]+(HB[0]-HA[0])*end/hLen,HA[1]+(HB[1]-HA[1])*end/hLen];return capsule(HA,e,Math.max(.001,w));};
 const holed=(t,s,r)=>holes(b.body,gapD(t,s,r));
 const pose=rigid((t,s,r)=>({angle:tilt(t,s,r),pivot:b.crown}));
 bellOff=toggle('bell-off','Bell off',
  'Light: a slash draws across the bell from upper-left to lower-right and stays. Strong: the bell shakes "no" while the slash strikes through with a small overshoot. Reverse retracts the slash.',[
  ...posed([fill('body-interior',holed,'tint'),stroke('body-frame',b.body),fill('body-solid',holed,'solid'),...tube('clapper',b.clapper)],pose),
  ...drawn('slash',A,B,u)
 ],.5,.48,'on','off',.42,.3);
}

// ---------- BELL SNOOZE ----------
// The bell nods off: it leans 12° on its crown while two Z's rise in beside it. Strong overshoots
// the lean and the Z's pop up with more travel. Reverse wakes it: the Z's sink away and it rights.
{
 var bellSnooze;
 const b=bell(.2,.6,.94);
 const leanF=F([[0,0],[.06,0],[.56,12,'smooth']],[[0,0],[.06,-3],[.28,14],[.42,11],[.54,12]]);
 const leanB=F([[0,12],[.44,0,'smooth']],[[0,12],[.08,14],[.3,-2],[.42,0]]);
 const lean=(t,s,r)=>r?leanB(t,s):leanF(t,s);
 const zIn=d=>F([[.26+d,0],[.5+d,1,'smooth']],[[.2+d,0],[.34+d,1,'out']]);
 const zOut=d=>F([[.04+d,1],[.3+d,0,'smooth']],[[.02+d,1],[.2+d,0,'in']]);
 const zees=[[23.6,8.6,1.5,0],[27,4.6,1.2,.12]];
 bellSnooze=toggle('bell-snooze','Bell snooze',
  'Light: the bell leans 12° asleep and two Z\'s fade up beside it. Strong: it nods over with an overshoot while the Z\'s pop up 3 units. Reverse: the Z\'s sink away and the bell rights itself.',[
  ...posed([...framed('body',b.body),...tube('clapper',b.clapper)],rigid((t,s,r)=>({angle:lean(t,s,r),pivot:b.crown}))),
  ...zees.flatMap(([x,y,h,d],i)=>{const inn=zIn(d),out=zOut(d);const k=(t,s,r)=>r?out(t,s):inn(t,s);
   return posed(shown(tube('z-'+i,zed(x,y,h)),(t,s,r)=>k(t,s,r)>.02?Math.min(1,k(t,s,r)*1.4):0),rigid((t,s,r)=>({dy:(1-k(t,s,r))*(s?3:2)})));})
 ],.66,.56,'awake','snoozed',.46,.44);
}

// ---------- SOUND ALERT ----------
// The bell sits left; two sound arcs on its right radiate outward: each grows 1.2 in radius
// while fading, then reforms at rest. Light pulses the outer arc once; Strong shudders the bell
// and pulses inner then outer arc twice, fast.
{
 var soundAlert;
 const b=bell(-3.2,0),C=[19.6,14];
 const wave=(delay,s)=>s?[[delay,0],[delay+.14,1,'out'],[delay+.15,0]]:[[delay,0],[delay+.32,1,'out'],[delay+.33,0]];
 const inner=(t,s)=>s?Math.max(track(wave(.06,1),t),track(wave(.34,1),t)):0;
 const outer=(t,s)=>s?Math.max(track(wave(.14,1),t),track(wave(.42,1),t)):track(wave(.08,0),t);
 const shiver=F([[0,0],[.4,0]],[[0,0],[.06,-.5],[.12,.5],[.18,-.4],[.24,.3],[.3,-.15],[.36,0]]);
 const arcD=(r,span,k)=>(t,s)=>arc(C[0],C[1],r+1.2*k(t,s),-span,span);
 soundAlert=make('sound-alert','Sound alert',
  'Light: the outer sound arc swells outward from the bell and fades. Strong: the bell shivers while inner and outer arcs radiate out twice in quick succession.',[
  ...posed([...framed('body',b.body),...tube('clapper',b.clapper)],rigid((t,s)=>({dx:shiver(t,s)}))),
  ...shown(tube('arc-inner',arcD(4.2,30,inner)),(t,s)=>1-inner(t,s)),
  ...shown(tube('arc-outer',arcD(7.8,34,outer)),(t,s)=>1-outer(t,s))
 ],.42,.6);
}

// ---------- PRIORITY ALERT ----------
// The bell sits left; a filled badge with an exclamation counter sits at its upper right. The
// badge pulses; the bell rocks away from it. Strong strikes the bell 10° and double-pulses the badge.
{
 var priorityAlert;
 const b=bell(-.7,.4,.92),BC=[25,7.5];
 const badge=holes(circle(BC[0],BC[1],3.3),capsule([25,5.3],[25,7.6],1.1),circle(25,9.5,.6));
 const pop=F([[0,0],[.08,1],[.3,0,'smooth']],[[0,0],[.06,1],[.16,0,'in'],[.22,1],[.36,0,'smooth']]);
 const rock=F([[0,0],[.12,5],[.36,0,'smooth']],[[0,0],[.06,-3],[.16,10,'hit'],[.28,-4],[.4,5],[.52,-2],[.62,0]]);
 const badgePose=(x,y,t,s)=>{const k=1+pop(t,s)*(s?.22:.14);return [BC[0]+(x-BC[0])*k,BC[1]+(y-BC[1])*k];};
 priorityAlert=make('priority-alert','Priority alert',
  'Light: the exclamation badge swells 14% and the bell nods 5°. Strong: the bell strikes 10° away from the badge and rings down while the badge pulses twice.',[
  ...posed([...framed('body',b.body),...tube('clapper',b.clapper)],rigid((t,s)=>({angle:rock(t,s),pivot:b.crown}))),
  ...posed(bothFill('badge',badge),badgePose)
 ],.36,.62);
}

// ---------- NOTIFICATION BADGE ----------
// An app tile with a filled badge dot off its upper-right corner (2.7 painted clearance at Bold).
// Light: the badge swells 18% and settles. Strong: the tile jolts and the badge hops with a pop.
{
 var notificationBadge;
 const BC=[25,7.4];
 const swell=F([[0,0],[.12,1],[.36,0,'smooth']],[[0,0],[.08,1.3,'hit'],[.2,-.15],[.3,.4],[.42,0]]);
 const hop=F([[0,0],[.36,0]],[[0,0],[.08,-1.2],[.18,0,'hit'],[.26,-.4],[.34,0]]);
 const jolt=F([[0,0],[.36,0]],[[0,0],[.06,-2.5],[.16,2],[.26,-1],[.36,0]]);
 const badgePose=(x,y,t,s)=>{const k=1+swell(t,s)*(s?.24:.18);return [BC[0]+(x-BC[0])*k,BC[1]+(y-BC[1])*k+hop(t,s)];};
 notificationBadge=make('notification-badge','Notification badge',
  'Light: the badge dot on the app tile swells 18% and settles. Strong: the tile jolts and the badge hops 1.2 units, pops to 130% and lands with a small rebound.',[
  ...posed(framed('tile',rect(4.5,10.5,16,16,3),rect(3.2,9.2,18.6,18.6,4.3)),rigid((t,s)=>({angle:jolt(t,s),pivot:[12.5,18.5]}))),
  ...posed(bothFill('badge',circle(BC[0],BC[1],3.4)),badgePose)
 ],.36,.42);
}

// ---------- TOAST MESSAGE ----------
// A screen frame with two content lines; a toast pill rises 2.5 units into place at the bottom
// while fading in and stays. Strong lifts it with an overshoot and settle. Reverse sinks it away.
{
 var toastMessage;
 const frame=rect(4.5,4.5,23,23,3),frameSolid=holes(rect(3.2,3.2,25.6,25.6,4.3),rect(5.8,5.8,20.4,20.4,1.7));
 const riseF=F([[0,2.5],[.06,2.5],[.46,0,'smooth']],[[0,3],[.06,3,'hold'],[.2,-1,'hit'],[.3,.4],[.4,0]]);
 const riseB=F([[0,0],[.4,2.5,'smooth']],[[0,0],[.06,-.6],[.26,3,'in']]);
 const showF=F([[.06,0],[.34,1,'out']],[[.06,0],[.16,1,'out']]);
 const showB=F([[.04,1],[.34,0,'in']],[[.04,1],[.22,0,'in']]);
 const rise=(t,s,r)=>r?riseB(t,s):riseF(t,s),show=(t,s,r)=>r?showB(t,s):showF(t,s);
 toastMessage=toggle('toast-message','Toast message',
  'Light: a toast pill fades in and rises 2.5 units into place at the bottom of the screen. Strong: it springs up past its mark and settles. Reverse: it sinks and fades away.',[
  ...framed('frame',frame,frameSolid),
  ...tube('line-0','M9 10 L21 10'),...tube('line-1','M9 14 L17 14'),
  ...posed(shown(bothFill('toast',capsule([9.5,20.5],[22.5,20.5],4.4)),show),rigid((t,s,r)=>({dy:rise(t,s,r)})))
 ],.46,.4,'hidden','shown',.4,.3);
}

// ---------- BANNER ALERT ----------
// A pennant ribbon unfurls from a rolled strip about its centre; an exclamation mark shows once
// it is wide enough. Solid keeps the mark as real counters. Reverse furls it back up.
{
 var bannerAlert;
 const ribbon='M4 9.5 L28 9.5 L24.5 16 L28 22.5 L4 22.5 L7.5 16 Z';
 const bar=capsule([16,13],[16,16.2],2.6),dot=circle(16,19.2,.9);
 const kF=F([[0,.12],[.06,.12],[.56,1,'smooth']],[[0,.12],[.06,.12,'hold'],[.28,1.06,'hit'],[.4,.98],[.5,1]]);
 const kB=F([[0,1],[.46,.12,'smooth']],[[0,1],[.06,1.04],[.3,.12,'in']]);
 const k=(t,s,r)=>r?kB(t,s):kF(t,s);
 const wave=F([[0,0],[.56,0]],[[0,0],[.1,0],[.22,1.2],[.34,-.8],[.44,.3],[.54,0]]);
 const pose=(x,y,t,s,r)=>[16+(x-16)*k(t,s,r),y+(r?0:wave(t,s))*(x-16)/12];
 const mark=(t,s,r)=>k(t,s,r)>.62?1:0;
 bannerAlert=toggle('banner-alert','Banner alert',
  'Light: the rolled banner unfurls about its centre to full width, revealing an exclamation mark. Strong: it snaps open past full width with a ripple and settles. Reverse rolls it back up.',[
  ...posed([fill('ribbon-interior',ribbon,'tint'),stroke('ribbon-frame',ribbon),fill('ribbon-solid',holes(ribbon,circle(16,19.2,2.1),capsule([16,13],[16,16.2],4.4)),'solid'),
   ...shown([...tube('bar','M16 13 L16 16.2'),...bothFill('dot',dot)],mark)],pose)
 ],.56,.56,'furled','unfurled',.46,.3);
}

// ---------- PUSH NOTIFICATION ----------
// A phone whose screen receives a card: it drops 3.5 units from the top of the screen while
// fading in and stays. Strong buzzes the phone and lands the card with a bounce. Preview reset rearms.
{
 var pushNotification;
 const phone=rect(9.5,4,13,24,3),phoneSolid=holes(rect(8.2,2.7,15.6,26.6,4.3),rect(10.8,5.3,10.4,21.4,1.7));
 const drop=F([[0,-3.5],[.06,-3.5],[.42,0,'smooth']],[[0,-3.5],[.08,-3.5,'hold'],[.22,.6,'hit'],[.3,-.3],[.38,0]]);
 const show=F([[.06,0],[.3,1,'out']],[[.08,0],[.18,1,'out']]);
 const buzz=F([[0,0],[.4,0]],[[0,0],[.22,0],[.25,.6],[.29,-.6],[.33,.4],[.37,-.2],[.41,0]]);
 const pose=rigid((t,s)=>({dx:buzz(t,s)}));
 pushNotification=make('push-notification','Push notification',
  'Light: a notification card fades in and drops 3.5 units into the top of the phone screen, then stays. Strong: it lands with a bounce and the phone buzzes on impact. Preview reset rearms.',[
  ...posed([...framed('phone',phone,phoneSolid),...tube('home','M14 25.4 L18 25.4')],pose),
  ...posed(shown(bothFill('card',rect(12.2,8.4,7.6,4.6,1.6)),show),rigid((t,s)=>({dx:buzz(t,s),dy:drop(t,s)})))
 ],.42,.42,['card','card-solid']);
}

// ---------- VIBRATE ALERT ----------
// A phone rocks about its centre between two motion arcs. Light rocks 4° twice; Strong shakes 8°
// five times while the arcs kick outward and settle back.
{
 var vibrateAlert;
 const phone=rect(10.5,4,11,24,2.8),phoneSolid=holes(rect(9.2,2.7,13.6,26.6,4.1),rect(11.8,5.3,8.4,21.4,1.5));
 const shake=F([[0,0],[.08,4],[.22,-4],[.36,0,'smooth']],[[0,0],[.05,8],[.11,-8],[.17,7],[.23,-6],[.29,4],[.35,-2],[.42,0]]);
 const kick=F([[0,0],[.4,0]],[[0,0],[.06,1],[.2,0,'smooth'],[.24,.8],[.42,0,'smooth']]);
 const arcD=(a,b)=>(t,s)=>arc(16,16,10.5+kick(t,s),a,b);
 vibrateAlert=make('vibrate-alert','Vibrate alert',
  'Light: the phone rocks 4° to each side and settles. Strong: it shakes 8° five times while the motion arcs kick outward and settle back.',[
  ...posed([...framed('phone',phone,phoneSolid),...tube('home','M14 25.4 L18 25.4')],rigid((t,s)=>({angle:shake(t,s),pivot:[16,16]}))),
  ...tube('arc-l',arcD(160,200)),...tube('arc-r',arcD(-20,20))
 ],.36,.42);
}

// ---------- INBOX ZERO ----------
// An open tray with its shelf; a check draws in above it, stroke by stroke, and stays.
// Strong lands the second stroke with a hit that dips the tray. Preview reset rearms.
{
 var inboxZero;
 const tray='M4.5 15.5 L4.5 24 C4.5 25.7 5.8 27 7.5 27 L24.5 27 C26.2 27 27.5 25.7 27.5 24 L27.5 15.5';
 const shelf='M4.5 19.5 L10.5 19.5 L12.5 22.5 L19.5 22.5 L21.5 19.5 L27.5 19.5';
 const interior='M4.5 19.5 L10.5 19.5 L12.5 22.5 L19.5 22.5 L21.5 19.5 L27.5 19.5 L27.5 24 C27.5 25.7 26.2 27 24.5 27 L7.5 27 C5.8 27 4.5 25.7 4.5 24 Z';
 const body='M3.2 18.2 L11.1 18.2 L13.1 21.2 L18.9 21.2 L20.9 18.2 L28.8 18.2 L28.8 24 C28.8 26.4 26.9 28.3 24.5 28.3 L7.5 28.3 C5.1 28.3 3.2 26.4 3.2 24 Z';
 const CK=cubics('M10 10.2 L14.6 14.6 L22.6 6.2');
 const draw=F([[0,0],[.06,0],[.46,2,'smooth']],[[0,0],[.08,0,'hold'],[.18,1,'pass'],[.3,2,'hit']]);
 const dip=F([[0,0],[.5,0]],[[0,0],[.3,0],[.36,1,'hit'],[.46,-.3],[.54,0]]);
 const trayPose=rigid((t,s)=>({dy:dip(t,s)}));
 inboxZero=make('inbox-zero','Inbox zero',
  'Light: a check draws in above the empty tray, stroke by stroke. Strong: the check snaps in and its landing stroke dips the tray, which springs back. The check is retained; preview reset rearms.',[
  ...posed([fill('tray-interior',interior,'tint'),stroke('tray-frame',tray),stroke('shelf',shelf),fill('tray-solid',body,'solid'),
   {name:'arm-l-solid',d:'M4.5 15.5 L4.5 19.5',style:'solid',type:'stroke',width:2.6},{name:'arm-r-solid',d:'M27.5 15.5 L27.5 19.5',style:'solid',type:'stroke',width:2.6}],trayPose),
  ...shown(tube('check',(t,s)=>drawPath(CK,draw(t,s))),(t,s)=>draw(t,s)>.06?1:0)
 ],.46,.54,['check','check-solid']);
}

// ---------- MARK ALL READ ----------
// Three list rows, each with an unread dot and a text line. Top to bottom, each dot fades as a
// small check draws in its place. Strong nudges each row as its check lands. Preview reset rearms.
{
 var markAllRead;
 const ROWS=[8,16,24];
 const at=i=>F([[.04+i*.14,0],[.3+i*.14,2,'smooth']],[[.04+i*.09,0],[.16+i*.09,2,'hit']]);
 const nudge=i=>F([[0,0],[.6,0]],[[0,0],[.14+i*.09,0],[.18+i*.09,1,'hit'],[.28+i*.09,0]]);
 markAllRead=open(make('mark-all-read','Mark all read',
  'Light: from the top row down, each unread dot fades while a check draws in its place. Strong: the checks snap in faster and each row nudges right as its check lands. Result retained; preview reset rearms.',
  ROWS.flatMap((y,i)=>{const k=at(i),n=nudge(i),ck=cubics(`M5.4 ${y+.2} L7.4 ${y+2.2} L11 ${y-1.8}`);
   return [
    ...posed(shown(bothFill('dot-'+i,circle(8.2,y,1.5)),(t,s)=>1-clamp(k(t,s)*2)),rigid((t,s)=>({dx:n(t,s)}))),
    ...posed(shown(tube('check-'+i,(t,s)=>drawPath(ck,k(t,s))),(t,s)=>k(t,s)>.06?1:0),rigid((t,s)=>({dx:n(t,s)}))),
    ...posed(tube('line-'+i,`M14.5 ${y} L26.5 ${y}`),rigid((t,s)=>({dx:n(t,s)})))];}),
  .6,.48,['dot-0','dot-1','dot-2','check-0','check-1','check-2']));
}

// ---------- NOTIFICATION STACK ----------
// A front notification card (dot and line inside) over two stacked card edges. The front card
// lifts and drops back; the edges compress under it. Strong hops it 3 units and lands hard.
{
 var notificationStack;
 const card=rect(5,7,22,9.5,2.2),cardSolid=holes(rect(3.7,5.7,24.6,12.1,3.5),rect(6.3,8.3,19.4,6.9,1.2));
 const lift=F([[0,0],[.12,-1.2],[.34,0,'smooth']],[[0,0],[.06,.5],[.2,-3],[.36,.4,'hit'],[.46,-.3],[.54,0]]);
 const press=F([[0,0],[.34,0]],[[0,0],[.36,0],[.4,1,'hit'],[.5,-.3],[.58,0]]);
 const cardPose=rigid((t,s)=>({dy:lift(t,s)}));
 notificationStack=make('notification-stack','Notification stack',
  'Light: the front card lifts 1.2 units and settles back on the stack. Strong: it crouches, hops 3 units and lands hard; the stacked edges below compress and rebound.',[
  ...posed([...framed('card',card,cardSolid),...bothFill('dot',circle(9.6,11.75,1.4)),...tube('line','M13.5 11.75 L23 11.75')],cardPose),
  ...posed(tube('edge-0','M7 20.5 L25 20.5'),rigid((t,s)=>({dy:press(t,s)*.7}))),
  ...posed(tube('edge-1','M9 25 L23 25'),rigid((t,s)=>({dy:press(t,s)*.35})))
 ],.34,.58);
}

// ---------- MENTION ALERT ----------
// An @ glyph draws from a visible inner ring through its tail and outer sweep; a badge dot pops in at its
// upper right and stays. Strong draws faster and lands the dot with a bounce. Preview reset rearms.
{
 var mentionAlert;
 const INNER=cubics(arc(15.5,16.5,3.6,0,360)).length;
 const GLYPH=cubics(arc(15.5,16.5,3.6,0,360)+' C19.1 19.5 20.8 21.8 22.87 21.66 '+L(arc(15.5,16.5,9,35,325)));
 const N=GLYPH.length,DC=[26,8];
 const draw=F([[0,INNER],[.06,INNER],[.66,N,'smooth']],[[0,INNER],[.06,INNER,'hold'],[.4,N,'out']]);
 const pop=F([[.66,0],[.86,1,'out']],[[.4,0],[.5,1.3,'hit'],[.58,.9],[.66,1]]);
 const dotPose=(x,y,t,s)=>{const k=Math.max(.001,pop(t,s));return [DC[0]+(x-DC[0])*k,DC[1]+(y-DC[1])*k];};
 mentionAlert=open(make('mention-alert','Mention alert',
  'Light: the @ draws from a visible inner ring through the tail and outer sweep, then a badge dot fades in at its upper right. Strong: the sweep draws in 0.34 s and the dot pops in with a bounce. Result retained; preview reset restores the visible inner ring.',[
  ...shown(tube('glyph',(t,s)=>drawPath(GLYPH,draw(t,s))),(t,s)=>draw(t,s)>.08?1:0),
  ...posed(shown(bothFill('dot',circle(DC[0],DC[1],2)),(t,s)=>pop(t,s)>.05?1:0),dotPose)
 ],.86,.66,['glyph','glyph-solid','dot','dot-solid']));
}

// ---------- DO NOT DISTURB ----------
// A ring; the bar draws outward from its centre in both directions and stays. Solid opens a
// real counter around the bar as it grows. Strong squashes the ring on the hit. Reverse retracts.
{
 var doNotDisturb;
 const R=10.5,HALF=5.5;
 const uF=F([[0,0],[.06,0],[.4,1,'smooth']],[[0,0],[.06,0,'hold'],[.2,1.06,'hit'],[.3,1]]);
 const uB=F([[0,1],[.36,0,'smooth']],[[0,1],[.06,1.05],[.24,0,'in']]);
 const u=(t,s,r)=>r?uB(t,s):uF(t,s);
 const squash=F([[0,1],[.4,1]],[[0,1],[.06,1],[.2,.94,'hit'],[.3,1.03],[.4,1]]);
 const ringPose=(x,y,t,s,r)=>{const k=r?1:squash(t,s);return [16+(x-16)/Math.sqrt(k),16+(y-16)*k];};
 const barD=(t,s,r)=>{const h=Math.max(.001,HALF*u(t,s,r));return `M${16-h} 16 L${16+h} 16`;};
 const discD=(t,s,r)=>{const h=HALF*u(t,s,r),w=4.6*clamp(h/2);return holes(circle(16,16,R+1.3),capsule([16-h,16],[16+h,16],Math.max(.001,w)));};
 doNotDisturb=toggle('do-not-disturb','Do not disturb',
  'Light: the bar draws outward from the centre of the ring in both directions and stays. Strong: it slams out with an overshoot and the ring squashes on the hit. Reverse retracts the bar.',[
  ...posed([fill('ring-interior',circle(16,16,R),'tint'),stroke('ring-frame',circle(16,16,R)),fill('ring-solid',discD,'solid'),
   ...shown(tube('bar',barD),(t,s,r)=>u(t,s,r)>.04?1:0)],ringPose)
 ],.4,.4,'off','on',.36,.3);
}

// ---------- REMINDER PING ----------
// A small clock with a ping at its upper right: two arcs radiate from the rim, fade and reform.
// Light: the arcs flash out in sequence. Strong: the clock hops with a hit and the arcs radiate twice.
{
 var reminderPing;
 const C=[14.5,17.5],RIM=[C[0]+9*Math.cos(-Math.PI/4),C[1]+9*Math.sin(-Math.PI/4)];
 const wave=(delay,dur)=>[[delay,0],[delay+dur,1,'out'],[delay+dur+.01,0]];
 const inner=(t,s)=>s?Math.max(track(wave(.04,.14),t),track(wave(.32,.14),t)):track(wave(.06,.3),t);
 const outer=(t,s)=>s?Math.max(track(wave(.1,.14),t),track(wave(.38,.14),t)):track(wave(.18,.3),t);
 const hop=F([[0,0],[.12,-.5],[.3,0,'smooth']],[[0,0],[.06,.4],[.16,-1.5],[.3,0,'hit'],[.38,-.3],[.46,0]]);
 const clockPose=rigid((t,s)=>({dy:hop(t,s)}));
 const arcD=(r,k)=>(t,s)=>arc(RIM[0],RIM[1],r+1.2*k(t,s),-75,15);
 reminderPing=make('reminder-ping','Reminder ping',
  'Light: the clock nudges and two ping arcs radiate from its upper right in sequence. Strong: it hops 1.5 units, lands with a hit and the arcs radiate out twice, fast.',[
  ...posed([...framed('face',circle(C[0],C[1],9),holes(circle(C[0],C[1],10.3),circle(C[0],C[1],7.7))),...tube('minute',`M${C} L${C[0]} 12.4`),...tube('hour',`M${C} L18 ${C[1]}`)],clockPose),
  ...shown(tube('ping-inner',arcD(4,inner)),(t,s)=>1-inner(t,s)),
  ...shown(tube('ping-outer',arcD(6.8,outer)),(t,s)=>1-outer(t,s))
 ],.5,.54);
}

// ---------- DAILY DIGEST ----------
// A page with a dated header (dot and title line); three summary lines draw in from the left,
// one after another, and stay. Strong taps the page down and snaps the lines in. Preview reset rearms.
{
 var dailyDigest;
 const page=rect(7,4,18,24,2.5),pageSolid=holes(rect(5.7,2.7,20.6,26.6,3.8),rect(8.3,5.3,15.4,21.4,1.2));
 const LINES=[[14,21],[18,21],[22,17.5]];
 const grow=i=>F([[.06+i*.12,0],[.34+i*.12,1,'smooth']],[[.1+i*.07,0],[.22+i*.07,1,'hit']]);
 const tap=F([[0,0],[.6,0]],[[0,0],[.06,-.4],[.12,.6,'hit'],[.22,-.2],[.3,0]]);
 const pagePose=rigid((t,s)=>({dy:tap(t,s)}));
 dailyDigest=make('daily-digest','Daily digest',
  'Light: three summary lines draw in under the dated header, one after another. Strong: the page taps down and the lines snap in with a quick stagger. Result retained; preview reset rearms.',[
  ...posed([...framed('page',page,pageSolid),...bothFill('date',circle(11.6,9,1.6)),...tube('title','M15 9 L21 9')],pagePose),
  ...LINES.flatMap(([y,x1],i)=>{const g=grow(i);return posed(drawn('line-'+i,[11,y],[x1,y],g),pagePose);})
 ],.6,.44,['line-0','line-1','line-2']);
}

// ---------- NOTIFICATION SETTINGS ----------
// A small bell over a level slider. The knob slides from low to high and the bell rings harder
// as the level rises, settling upright at the end. Reverse slides the knob back with a quiet nod.
{
 var notificationSettings;
 const b=bell(3.2,0,.8),LO=10.5,HI=21.5;
 const kF=F([[0,LO],[.06,LO],[.5,HI,'smooth']],[[0,LO],[.06,LO,'hold'],[.26,HI+.8,'hit'],[.36,HI-.2],[.44,HI]]);
 const kB=F([[0,HI],[.44,LO,'smooth']],[[0,HI],[.06,HI+.6],[.28,LO-.4,'hit'],[.38,LO]]);
 const knobX=(t,s,r)=>r?kB(t,s):kF(t,s);
 const rockF=F([[0,0],[.14,0],[.26,-3],[.4,4],[.54,-2],[.66,0,'smooth']],[[0,0],[.1,0],[.2,-5],[.3,9,'hit'],[.42,-7],[.52,4],[.6,-1.5],[.68,0]]);
 const rockB=F([[0,0],[.1,2],[.3,-1],[.44,0,'smooth']],[[0,0],[.08,3],[.24,-2],[.38,0]]);
 const rock=(t,s,r)=>r?rockB(t,s):rockF(t,s);
 notificationSettings=toggle('notification-settings','Notification settings',
  'Light: the level knob slides right along its track and the bell rings gently as the level rises, settling upright. Strong: the knob snaps to high with an overshoot and the bell rings 9° before settling. Reverse slides the knob back to low with a quiet nod.',[
  ...posed([...framed('body',b.body),...tube('clapper',b.clapper)],rigid((t,s,r)=>({angle:rock(t,s,r),pivot:b.crown}))),
  ...tube('track','M7 27 L25 27'),
  ...posed(bothFill('knob',circle(LO,27,2.3)),rigid((t,s,r)=>({dx:knobX(t,s,r)-LO})))
 ],.66,.68,'low','high',.44,.38);
}

// ---------- ALERT DISMISS ----------
// Two stacked alert cards; the top one carries a close mark. It flicks off to the right and fades,
// and the lower card rises into its place. Strong tilts the leaving card and springs the other up.
// Preview reset rearms.
{
 var alertDismiss;
 const cardAt=y=>[rect(5,y,20,10,2.4),holes(rect(3.7,y-1.3,22.6,12.6,3.7),rect(6.3,y+1.3,17.4,7.4,1.4))];
 const [A,AS]=cardAt(5),[B,BS]=cardAt(18);
 const slide=F([[0,0],[.06,0],[.4,4,'in']],[[0,0],[.06,-.8],[.28,4.5,'out']]);
 const fade=F([[.1,1],[.34,0,'in']],[[.1,1],[.24,0,'in']]);
 const tilt=F([[0,0],[.4,0]],[[0,0],[.06,1],[.28,-7,'out']]);
 const riseB=F([[0,0],[.3,0],[.62,-13,'smooth']],[[0,0],[.24,0,'hold'],[.42,-13.6,'hit'],[.5,-12.8],[.58,-13]]);
 const aPose=rigid((t,s)=>({dx:slide(t,s),angle:tilt(t,s),pivot:[5,10]}));
 const bPose=rigid((t,s)=>({dy:riseB(t,s)}));
 alertDismiss=make('alert-dismiss','Alert dismiss',
  'Light: the top alert card slides off to the right and fades while the card below rises into its place. Strong: it flicks away with a 7° tilt and the lower card springs up with a small overshoot. Result retained; preview reset rearms.',[
  ...posed(shown([...framed('card-a',A,AS),...bothFill('dot-a',circle(9.6,10,1.4)),...tube('line-a','M13.5 10 L17.5 10'),...tube('x-0','M20.4 8.4 L23.6 11.6'),...tube('x-1','M23.6 8.4 L20.4 11.6')],(t,s)=>fade(t,s)>.02?fade(t,s):0),aPose),
  ...posed([...framed('card-b',B,BS),...bothFill('dot-b',circle(9.6,23,1.4)),...tube('line-b','M13.5 23 L21 23')],bPose)
 ],.62,.58,['card-a-interior','card-a-frame','card-a-solid','dot-a','dot-a-solid','line-a','line-a-solid','x-0','x-0-solid','x-1','x-1-solid','card-b-interior','card-b-frame','card-b-solid','dot-b','dot-b-solid','line-b','line-b-solid']);
}

// ---------- ALERT EXPAND ----------
// A collapsed alert card (dot and title) grows downward to reveal two detail lines, which appear
// as the card passes them. Strong overshoots the height and settles. Reverse collapses it.
{
 var alertExpand;
 const TOP=6,H0=9,H1=20;
 const hF=F([[0,H0],[.06,H0],[.5,H1,'smooth']],[[0,H0],[.06,H0,'hold'],[.26,H1+1.4,'hit'],[.36,H1-.3],[.44,H1]]);
 const hB=F([[0,H1],[.44,H0,'smooth']],[[0,H1],[.06,H1+.6],[.28,H0,'hit']]);
 const h=(t,s,r)=>r?hB(t,s):hF(t,s);
 const cardD=(t,s,r)=>rect(5,TOP,22,h(t,s,r),2.4);
 const cardS=(t,s,r)=>{const hh=h(t,s,r);return holes(rect(3.7,TOP-1.3,24.6,hh+2.6,3.7),rect(6.3,TOP+1.3,19.4,hh-2.6,1.4));};
 const lineAt=y=>(t,s,r)=>{const bottom=TOP+h(t,s,r)-1.25-1.3;return bottom>=y+.6?1:0;};
 alertExpand=toggle('alert-expand','Alert expand',
  'Light: the collapsed alert card grows downward to more than twice its height, revealing two detail lines as it passes them. Strong: it springs open past full height and settles. Reverse collapses it.',[
  fill('card-interior',cardD,'tint'),stroke('card-frame',cardD),fill('card-solid',cardS,'solid'),
  ...bothFill('dot',circle(9.6,10.5,1.4)),...tube('title','M13.5 10.5 L23 10.5'),
  ...shown(tube('detail-0','M9 17 L23 17'),lineAt(17)),...shown(tube('detail-1','M9 21.5 L19 21.5'),lineAt(21.5))
 ],.5,.44,'collapsed','expanded',.44,.3);
}

const all=[bellRing,bellOff,bellSnooze,notificationBadge,toastMessage,bannerAlert,pushNotification,inboxZero,markAllRead,notificationStack,mentionAlert,priorityAlert,doNotDisturb,reminderPing,dailyDigest,notificationSettings,alertDismiss,alertExpand,soundAlert,vibrateAlert].map(i=>i.duotoneFill===undefined?duo(i):i);
export const icons=all.map((icon,index)=>({...icon,number:626+index}));
