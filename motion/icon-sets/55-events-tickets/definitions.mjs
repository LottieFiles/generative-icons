// Events & tickets (55). Twenty icons: invitations, venues, seating, badges, tickets and live shows.
// Designs, reviews and corrections: evidence/design-review.md. Shared assembly comes from the library;
// every silhouette and motion track below is set-specific.
import {make,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,bothFill,holes,capsule,arc,polygon,timePair,drive,track,clamp} from '../library/ai-authoring.mjs';
import {rigid as nativeRigid} from '../library/drawing.mjs';

// ---------- shared assembly (no artwork) ----------
const toggle=(name,label,intent,parts,light,strong,initialState,finalState,reverseLight=light,reverseStrong=strong)=>({...make(name,label,intent,parts,light,strong),stateful:true,initialState,finalState,reverseSuffix:'-reverse',forwardLabel:finalState,reverseLabel:initialState,reverseTimings:timePair(reverseLight,reverseStrong)});
// Two-state driver: forward keys per profile plus an authored reverse per profile.
const T=(fl,fs,rl,rs)=>(t,s,r)=>drive(t,s,r?rl:fl,r?rs:fs);
// Gesture driver (no reverse action exists for these icons).
const F=(l,s)=>(t,p)=>drive(t,p,l,s);
const L=d=>d.replace(/^M/,'L');
const K=.552285;
const still=v=>[[0,v]];
// Preserve the designed forward action. During the gallery-only reset the changing parts retrace their
// own route as opaque geometry (copied from 39-food-drink/batches/helpers.mjs); still parts stay still.
function geometricRearm(icon){
 const differs=part=>[false,true].some(s=>{
  const snap=t=>JSON.stringify({d:typeof part.d==='function'?part.d(t,s,false,2):part.d,pose:part.pose?.transform?part.pose.transform(t,s,false):part.pose?[part.pose(0,0,t,s,false),part.pose(32,32,t,s,false)]:null,v:part.visibility?.(t,s,false)??1});
  return snap(0)!==snap(2.4);
 });
 for(const part of icon.parts){
  const changing=differs(part);
  const clock=(t,s,r)=>r?(changing?track([[0,icon.timings[s?'strong':'light'].active],[.3,0,'smooth']],t):0):t;
  if(typeof part.d==='function'){const d=part.d;part.d=(t,s,r,w)=>d(clock(t,s,r),s,false,w);}
  if(part.pose){const pose=part.pose;part.pose=pose.transform?nativeRigid((t,s,r)=>pose.transform(clock(t,s,r),s,false)):(x,y,t,s,r)=>pose(x,y,clock(t,s,r),s,false);}
  if(part.visibility){const v=part.visibility;part.visibility=(t,s,r)=>v(clock(t,s,r),s,false);}
 }
 return {...icon,stateful:true,initialState:icon.initialState||'prepared',finalState:icon.finalState||'result',previewReset:true,reverseSuffix:'-reset',forwardLabel:'Play action',reverseLabel:'Reset preview',reverseTimings:timePair(.3,.3)};
}
// A polyline drawn by a pen: segments not yet reached collapse at the current tip (constant topology).
function polyDraw(pts){
 const len=pts.slice(1).map((p,i)=>Math.hypot(p[0]-pts[i][0],p[1]-pts[i][1])),total=len.reduce((a,b)=>a+b,0);
 return u=>{let left=clamp(u)*total,d='M'+pts[0];for(let i=1;i<pts.length;i++){const l=len[i-1];let cur;if(left>=l){cur=pts[i];left-=l;}else{cur=[pts[i-1][0]+(pts[i][0]-pts[i-1][0])*left/l,pts[i-1][1]+(pts[i][1]-pts[i-1][1])*left/l];left=0;}d+=' L'+cur;}return d;};
}
const corner=(ax,ay,bx,by,cx,cy)=>` C${ax+(cx-ax)*K} ${ay+(cy-ay)*K} ${bx+(cx-bx)*K} ${by+(cy-by)*K} ${bx} ${by}`; // quarter turn a→b about the sharp corner c
const star=(cx,cy,ro,ri)=>polygon(Array.from({length:10},(_,i)=>{const a=-90+i*36,r=i%2?ri:ro;return[cx+r*Math.cos(a*Math.PI/180),cy+r*Math.sin(a*Math.PI/180)];}),.35);

// Landscape ticket with stub notches on the divider. Solid: a 2.6 ring around a real counter over the
// main body; the divider band and stub stay solid so the ticket reads as a filled object.
function ticketD(x,y,w,h,r,n,dx){
 return `M${x+r} ${y} L${dx-n} ${y} ${L(arc(dx,y,n,180,0))} L${x+w-r} ${y}${corner(x+w-r,y,x+w,y+r,x+w,y)} L${x+w} ${y+h-r}${corner(x+w,y+h-r,x+w-r,y+h,x+w,y+h)} L${dx+n} ${y+h} ${L(arc(dx,y+h,n,0,-180))} L${x+r} ${y+h}${corner(x+r,y+h,x,y+h-r,x,y+h)} L${x} ${y+r}${corner(x,y+r,x+r,y,x,y)} Z`;
}
const TK={x:4,y:9,w:24,h:14,r:2,n:2.4,dx:21};
// The Solid counter ends in a semicircle that clears both notch rings by 0.15, so it reaches close to
// the divider without cutting into the notches (a counter outside the outer contour would paint).
function ticketCounter(x,y,h,n,dx){
 const R=(h-2.6)/2,cy=y+h/2,ex=dx+R-Math.sqrt((R+n+1.3)**2-(h/2)**2)-.15,x0=x+1.3,y0=y+1.3,y1=y+h-1.3;
 return `M${x0+1} ${y0} L${ex-R} ${y0} ${L(arc(ex-R,cy,R,-90,90))} L${x0+1} ${y1}${corner(x0+1,y1,x0,y1-1,x0,y1)} L${x0} ${y0+1}${corner(x0,y0+1,x0+1,y0,x0,y0)} Z`;
}
const ticketParts=(name,{x,y,w,h,r,n,dx}=TK)=>[
 ...framed(name,ticketD(x,y,w,h,r,n,dx),holes(ticketD(x-1.3,y-1.3,w+2.6,h+2.6,r+1.3,n+1.3,dx),ticketCounter(x,y,h,n,dx))),
 ...tube(name+'-divider',`M${dx} ${y+n+.8} L${dx} ${y+h-n-.8}`)];
// Calendar leaf shared by add-to-calendar and event-reminder (same construction as the scheduling set).
const leaf=n=>[
 ...framed(n,rect(5,6,22,21,2.7),holes(rect(3.7,4.7,24.6,23.6,4),rect(6.3,7.3,19.4,18.4,1.4))),
 ...tube('peg-l','M11 4 L11 8'),...tube('peg-r','M21 4 L21 8'),...tube('header','M5.5 11.5 L26.5 11.5')];

// ---------- 1. RSVP (pending → accepted) ----------
// A reply card with a header band; a check is drawn by a pen. Strong: the pen hits the vertex, the card
// dips 0.6 and the long stroke finishes fast. Reverse: the pen retraces the check.
const CARD=rect(5,6,22,20,2.4),CARD_S=holes(rect(3.7,4.7,24.6,22.6,3.7),rect(6.3,12.3,19.4,12.4,1.1));
const rsvpU=T([[0,0],[.06,0],[.5,1,'smooth']],[[0,0],[.05,0,'hold'],[.16,.335,'hit'],[.36,1]],[[0,1],[.4,0,'smooth']],[[0,1],[.28,0,'out']]);
const rsvpDip=T(still(0),[[0,0],[.16,0,'hold'],[.23,.6,'hit'],[.34,-.15],[.46,0]],still(0),still(0));
const rsvpCheck=polyDraw([[10.5,18.3],[14.3,21.6],[21.5,14.7]]);
const rsvp=toggle('rsvp','RSVP',
 'Light: a check is drawn onto the reply card in one quiet stroke. Strong: the pen strikes the vertex, the card dips and the long stroke snaps to its end. Reverse retraces the check.',
 posed([...framed('card',CARD,CARD_S),...tube('header','M5.5 11 L26.5 11'),
  ...shown(tube('check',(t,s,r)=>rsvpCheck(rsvpU(t,s,r))),(t,s,r)=>rsvpU(t,s,r)>.02?1:0)],rigid((t,s,r)=>({dy:rsvpDip(t,s,r)}))),
 .5,.46,'pending','accepted',.4,.28);

// ---------- 2. EVENT INVITE (sealed → opened) ----------
// The invitation card rises 7 out of the envelope; its cut ends sit on the envelope's top edge so the
// hidden part needs no occlusion. Strong: the envelope dips first, the card shoots past and settles.
const ENV=rect(5,11,22,15,2),ENV_S=holes(rect(3.7,9.7,24.6,17.6,3.3),rect(6.3,12.3,19.4,12.4,1));
const rise=T([[0,0],[.05,0],[.55,7,'smooth']],[[0,0],[.1,0,'hold'],[.3,7.7,'hit'],[.42,6.7],[.52,7]],[[0,7],[.45,0,'smooth']],[[0,7],[.06,7.3],[.3,0,'hit'],[.34,0]]);
const envDip=T(still(0),[[0,0],[.1,.5],[.2,0,'hit'],[.28,0]],still(0),[[0,0],[.3,0,'hold'],[.34,.4,'hit'],[.42,0]]);
const cardTop=(t,s,r)=>11-Math.max(0,rise(t,s,r));
const cardD=(t,s,r)=>{const top=cardTop(t,s,r),h=11-top,rr=Math.min(1.6,h/2);return `M9 11 L9 ${top+rr}${corner(9,top+rr,9+rr,top,9,top)} L${23-rr} ${top}${corner(23-rr,top,23,top+rr,23,top)} L23 11`;};
const cardLine=(t,s,r)=>{const y=cardTop(t,s,r)+2.8;return `M12.5 ${y} L19.5 ${y}`;};
const envPose=rigid((t,s,r)=>({dy:envDip(t,s,r)}));
const eventInvite=toggle('event-invite','Event invite',
 'Light: the invitation card slides 7 units up out of the envelope. Strong: the envelope dips, the card shoots up past its mark and settles. Reverse slides the card back in.',[
 ...posed([...framed('envelope',ENV,ENV_S),...tube('flap','M5.6 11.6 L16 19 L26.4 11.6')],envPose),
 ...posed([stroke('card',cardD),fill('card-solid',(t,s,r)=>cardD(t,s,r)+' Z','solid'),fill('card-interior',(t,s,r)=>cardD(t,s,r)+' Z','tint'),
  stroke('card-line',cardLine,{visibility:(t,s,r)=>clamp((11-cardTop(t,s,r)-3.8)/1.2)})],(x,y,t,s,r)=>[x,y+envDip(t,s,r)])
],.55,.52,'sealed','opened',.45,.42);

// ---------- 3. VENUE (closed → open) ----------
// A hall with a pediment roof and an arched doorway. Two door leaves slide into the jambs (their
// width collapses toward the hinge side) and vanish; the open arch stays. Strong: a fast slide and thump.
const BODY=rect(5,12,22,14,1.2),BODY_S=holes(rect(3.7,10.7,24.6,16.6,2.5),'M13.3 24.7 L13.3 19 C13.3 17.8 14.5 16.8 16 16.8 C17.5 16.8 18.7 17.8 18.7 19 L18.7 24.7 Z');
const doorK=T([[0,1],[.06,1],[.5,0,'smooth']],[[0,1],[.06,1,'hold'],[.24,0,'hit'],[.3,0]],[[0,0],[.45,1,'smooth']],[[0,0],[.06,0,'hold'],[.26,1,'hit'],[.32,1]]);
const hallThump=T(still(0),[[0,0],[.24,0,'hold'],[.3,.45,'hit'],[.4,-.1],[.5,0]],still(0),[[0,0],[.26,0,'hold'],[.32,.35,'hit'],[.42,0]]);
const leafPose=x0=>(x,y,t,s,r)=>[x0+(x-x0)*doorK(t,s,r),y+hallThump(t,s,r)];
const leafShown=(t,s,r)=>doorK(t,s,r)>.08?1:0;
const hallPose=rigid((t,s,r)=>({dy:hallThump(t,s,r)}));
const venue=toggle('venue','Venue',
 'Light: the two door leaves slide into the walls and the arched doorway stands open. Strong: they snap open in 0.18 s and the hall thumps once. Reverse slides the doors shut.',[
 ...posed([...framed('hall',BODY,BODY_S),stroke('roof','M4 12.4 L16 5.2 L28 12.4'),fill('roof-solid',polygon([[3.2,12.9],[16,4.4],[28.8,12.9]],[.6,.9,.6]),'solid'),
  ...tube('doorway','M12 26 L12 19 C12 16.8 13.8 15.5 16 15.5 C18.2 15.5 20 16.8 20 19 L20 26')],hallPose),
 ...shown(posed(bothFill('leaf-l','M13.6 25 L13.6 19.2 C13.6 17.9 14.6 17.2 15.7 17.2 L15.7 25 Z'),leafPose(12.6)),leafShown),
 ...shown(posed(bothFill('leaf-r','M18.4 25 L18.4 19.2 C18.4 17.9 17.4 17.2 16.3 17.2 L16.3 25 Z'),leafPose(19.4)),leafShown)
],.5,.5,'closed','open',.45,.42);

// ---------- 4. SEAT MAP (available → selected) ----------
// A stage bar above three rows of four seats. The chosen seat grows to 1.5× with a hop; the others hold.
const SEATS=[[8.2,14],[13.4,14],[18.6,14],[23.8,14],[8.2,19.4],[13.4,19.4],[18.6,19.4],[23.8,19.4],[8.2,24.8],[13.4,24.8],[18.6,24.8],[23.8,24.8]];
const PICK=6;
const pick=T([[0,1],[.05,1],[.45,1.5,'smooth']],[[0,1],[.08,.82],[.24,1.72,'hit'],[.36,1.42],[.46,1.5]],[[0,1.5],[.4,1,'smooth']],[[0,1.5],[.06,1.6],[.26,1,'hit'],[.32,1]]);
const seatHop=T(still(0),[[0,0],[.08,.3],[.2,-1.6],[.3,0,'hit'],[.38,-.3],[.46,0]],still(0),still(0));
const seatMap=toggle('seat-map','Seat map',
 'Light: the chosen seat grows to 1.5× and settles. Strong: it crouches, hops 1.6 units, lands larger and settles to the same selected size. Reverse shrinks it back.',[
 ...framed('stage',rect(5,4,22,6,3)),
 ...SEATS.flatMap(([cx,cy],i)=>{const d=rect(cx-1.4,cy-1.4,2.8,2.8,.9);return i===PICK?posed(bothFill('seat-'+i,d),(x,y,t,s,r)=>{const k=pick(t,s,r);return[cx+(x-cx)*k,cy+(y-cy)*k+seatHop(t,s,r)];}):bothFill('seat-'+i,d);})
],.45,.46,'available','selected',.4,.32);

// ---------- 5. AGENDA (rows drawn, preview reset) ----------
// A clipboard whose three session rows extend from their bullets in sequence. Strong: quick decisive
// draws while the board bumps once. The filled agenda is retained; the preview reset retracts the rows.
const rowU=i=>F([[.12*i,0],[.12*i+.34,1,'smooth']],[[.08*i,0],[.08*i+.15,1,'out']]);
const boardBump=F(still(0),[[0,0],[.14,0,'hold'],[.2,.5,'hit'],[.3,0]]);
const ROWS=[13.5,18,22.5];
const agenda=geometricRearm(make('agenda','Agenda',
 'Light: three session rows extend from their bullets one after another. Strong: they snap in quickly while the clipboard bumps. Retained; the preview reset retracts the rows.',
 posed([...framed('board',rect(7,7,18,20,2),holes(rect(5.7,5.7,20.6,22.6,3.3),rect(8.3,8.3,15.4,17.4,.9))),...bothFill('clip',rect(13,5,6,4,1.8)),
  ...ROWS.flatMap((y,i)=>{const u=rowU(i);return[...bothFill('dot-'+i,circle(10.8,y,1.3)),...shown(tube('row-'+i,(t,s)=>`M14 ${y} L${14+7.6*clamp(u(t,s))} ${y}`),(t,s)=>u(t,s)>.03?1:0)];})
 ],rigid((t,s)=>({dy:boardBump(t,s)}))),.58,.34));

// ---------- 6. EVENT SPEAKER (gesture) ----------
// A lectern: a slanted reading desk on a flared column, with a gooseneck microphone. Two sound arcs
// leave the mic head, drift right and fade. Strong: the mic rocks on its base and the arcs pulse twice.
// (v1 drew a flat slab on a tapered body, which read as a bin.)
const rock=F(still(0),[[0,0],[.05,-6],[.16,7,'hit'],[.3,-3],[.42,0]]);
const waveA=[F([[0,0],[.1,1,'out'],[.3,1],[.5,0,'smooth']],[[0,0],[.06,1],[.2,0],[.26,0,'hold'],[.32,1],[.5,0]]),
 F([[.1,0],[.22,1,'out'],[.34,1],[.55,0,'smooth']],[[.04,0],[.12,1],[.24,0],[.3,0,'hold'],[.38,1],[.56,0]])];
const waveX=F([[0,0],[.5,.6,'linear'],[.55,0,'linear']],[[0,0],[.2,1.4,'out'],[.24,1.4],[.26,0,'linear'],[.32,0,'hold'],[.5,1.4,'out'],[.56,0,'linear']]);
const micPose=rigid((t,s)=>({angle:rock(t,s),pivot:[15.4,14.6]}));
const eventSpeaker=make('event-speaker','Event speaker',
 'Light: two sound arcs leave the microphone, drift right 0.6 and fade. Strong: the mic rocks on its gooseneck and the arcs pulse twice, punching out 1.4 units each time.',[
 ...framed('column',polygon([[13,16.4],[19,15.4],[20.6,27],[11.4,27]],[.5,.5,1.2,1.2])),
 ...bothFill('desk',polygon([[7.6,14.6],[24.4,11.8],[24.4,14.4],[7.6,17.2]],.8)),
 ...posed([...tube('stem','M15.4 14.6 C15.4 11.4 15.8 9.4 18 7.8'),...bothFill('head',circle(19,7.1,1.6))],micPose),
 ...[3.6,5.8].flatMap((r,i)=>shown(posed(tube('wave-'+i,arc(19,7.1,r,-36+i*4,36-i*4)),(x,y,t,s)=>[x+waveX(t,s)*(i?1:.7),y]),waveA[i]))
],.56,.56);

// ---------- 7. EVENT BADGE (gesture) ----------
// A lanyard badge: the strap is fixed; clip, card and the person on it swing rigidly about the strap
// junction. Light 5°; Strong winds up 6° the other way, swings 16° and damps to rest.
// (v1 used a wide strap and a photo ring, which read as a television with antennae.)
const swing=F([[0,0],[.16,5],[.42,-2.5],[.62,0,'smooth']],[[0,0],[.08,-6],[.24,16,'hit'],[.42,-9],[.56,4],[.68,-1.5],[.78,0]]);
const badgePose=rigid((t,s)=>({angle:swing(t,s),pivot:[16,9.6]}));
const eventBadge=make('event-badge','Event badge',
 'Light: the badge swings 5° on its lanyard and settles. Strong: it winds up, swings 16° and rocks to rest with a damped decay.',[
 ...tube('strap-l','M16 9.6 L13.4 4.2'),...tube('strap-r','M16 9.6 L18.6 4.2'),
 ...posed([...bothFill('clip',rect(14.2,8.4,3.6,3,1.3)),
  ...framed('card',rect(9,11.4,14,13,1.8),holes(rect(7.7,10.1,16.6,15.6,3.1),rect(10.3,12.7,11.4,10.4,.8))),
  ...bothFill('head',circle(16,15.9,1.9)),...tube('shoulders','M12.3 21.5 C12.3 19.1 13.9 17.9 16 17.9 C18.1 17.9 19.7 19.1 19.7 21.5')],badgePose)
],.62,.78);

// ---------- 8. CHECK-IN (ready → checked-in) ----------
// A ticket feeds down into the reader slot: its visible part is the geometry above the slot, so nothing
// is occluded. Once in, a check lights inside the reader. Strong: the ticket drops fast and the reader thumps.
const READER=rect(6,15.5,20,12,2.2),READER_S=holes(rect(4.7,14.2,22.6,14.6,3.5),rect(7.3,16.8,17.4,9.4,.9));
const drop=T([[0,0],[.05,0],[.5,8.5,'smooth']],[[0,0],[.06,0,'hold'],[.22,8.5,'hit'],[.3,8.5]],[[0,8.5],[.14,8.5],[.6,0,'smooth']],[[0,8.5],[.1,8.5,'hold'],[.36,0,'hit'],[.4,0]]);
const tickD=(t,s,r)=>{const top=4.5+drop(t,s,r),h=15.5-top,rr=Math.min(1.8,h/2);return `M10.5 15.5 L10.5 ${top+rr}${corner(10.5,top+rr,10.5+rr,top,10.5,top)} L${21.5-rr} ${top}${corner(21.5-rr,top,21.5,top+rr,21.5,top)} L21.5 15.5`;};
const tickLine=(t,s,r)=>{const y=4.5+drop(t,s,r)+3.2;return `M13.3 ${y} L18.7 ${y}`;};
const checkU=T([[.42,0],[.7,1,'smooth']],[[.24,0],[.4,1,'out']],[[0,1],[.16,0,'out']],[[0,1],[.1,0,'out']]);
const readerThump=T(still(0),[[0,0],[.22,0,'hold'],[.28,.5,'hit'],[.38,-.1],[.46,0]],still(0),still(0));
const checkIn=polyDraw([[12.6,21.5],[15.2,24],[19.6,18.8]]);
const readerPose=rigid((t,s,r)=>({dy:readerThump(t,s,r)}));
const checkInIcon=toggle('check-in','Check-in',
 'Light: the ticket feeds 8.5 units down into the reader, then a check lights inside it. Strong: the ticket drops in 0.16 s, the reader thumps and the check snaps in. Reverse clears the check and lifts the ticket back out.',[
 ...posed([...framed('reader',READER,READER_S),...shown(tube('check',(t,s,r)=>checkIn(checkU(t,s,r))),(t,s,r)=>checkU(t,s,r)>.02?1:0)],readerPose),
 stroke('ticket',tickD),fill('ticket-solid',(t,s,r)=>tickD(t,s,r)+' Z','solid'),fill('ticket-interior',(t,s,r)=>tickD(t,s,r)+' Z','tint'),
 stroke('ticket-line',tickLine,{visibility:(t,s,r)=>clamp((13.7-(4.5+drop(t,s,r)+3.2))/1.2)})
],.7,.46,'ready','checked-in',.6,.4);

// ---------- 9. ADD TO CALENDAR (gesture) ----------
// A plus sits on the calendar leaf. Light: it pops to 1.2× and settles. Strong: it lifts 2.6, stamps down
// with a squash, and the leaf thumps.
const plusLift=F(still(0),[[0,0],[.12,-2.6],[.26,0,'hit'],[.34,0]]);
const plusSX=F([[0,1],[.16,1.2],[.44,1,'smooth']],[[0,1],[.12,.95],[.26,1.22,'hit'],[.36,.92],[.46,1]]);
const plusSY=F([[0,1],[.16,1.2],[.44,1,'smooth']],[[0,1],[.12,1.05],[.26,.8,'hit'],[.36,1.1],[.46,1]]);
const leafThump=F(still(0),[[0,0],[.26,0,'hold'],[.31,.45,'hit'],[.4,-.1],[.48,0]]);
const plusPose=(x,y,t,s)=>[16+(x-16)*plusSX(t,s),19.2+(y-19.2)*plusSY(t,s)+plusLift(t,s)+leafThump(t,s)];
const addToCalendar=make('add-to-calendar','Add to calendar',
 'Light: the plus pops to 1.2× and settles on the calendar leaf. Strong: it lifts 2.6 units, stamps down with a squash and the leaf thumps once.',[
 ...posed(leaf('leaf'),rigid((t,s)=>({dy:leafThump(t,s)}))),
 ...posed([...tube('plus-h','M12.4 19.2 L19.6 19.2'),...tube('plus-v','M16 15.6 L16 22.8')],plusPose)
],.44,.48);

// ---------- 10. EVENT REMINDER (gesture) ----------
// A bell inside the calendar leaf rings about its crown. Light: one 7° swing. Strong: a 20° strike
// with a damped ring; the clapper rides the bell as one rigid piece.
const ringA=F([[0,0],[.1,7],[.26,-5],[.42,2],[.56,0,'smooth']],[[0,0],[.06,-6],[.16,20,'hit'],[.3,-15],[.42,9],[.54,-4],[.64,1.5],[.74,0]]);
const bellPose=rigid((t,s)=>({angle:ringA(t,s),pivot:[16,14.4]}));
const eventReminder=make('event-reminder','Event reminder',
 'Light: the bell on the calendar leaf swings 7° and settles. Strong: it strikes 20° and rings down with a damped decay.',[
 ...leaf('leaf'),
 ...posed([...framed('bell','M11.8 21.4 L11.8 18.6 C11.8 15.6 13.6 14.4 16 14.4 C18.4 14.4 20.2 15.6 20.2 18.6 L20.2 21.4 Z'),...tube('rim','M10.6 21.4 L21.4 21.4'),...bothFill('clapper',circle(16,23.7,1))],bellPose)
],.56,.74);

// ---------- 11. TICKET SCAN (gesture) ----------
// A scan bar sweeps down over the ticket's code block and returns. Strong: two fast passes with hard
// turns while the ticket dips on the first contact.
const scanY=F([[0,12.1],[.05,12.1],[.4,19.6,'smooth'],[.75,12.1,'smooth']],[[0,12.1],[.05,12.1,'hold'],[.2,19.6,'hit'],[.26,19.6,'hold'],[.4,12.1,'hit'],[.46,12.1]]);
const scanDip=F(still(0),[[0,0],[.2,0,'hold'],[.25,.4,'hit'],[.34,-.08],[.42,0]]);
const ticketScan=make('ticket-scan','Ticket scan',
 'Light: a scan bar sweeps down over the ticket\'s code and glides back. Strong: it makes two hard passes and the ticket dips at the first turn.',
 posed([...ticketParts('ticket'),
  ...bothFill('code-0',rect(7.4,13.4,2.4,2.4,.6)),...bothFill('code-1',rect(11,13.4,2.4,2.4,.6)),...bothFill('code-2',rect(7.4,17,2.4,2.4,.6)),...bothFill('code-3',rect(11.5,17.5,1.4,1.4,.4)),
  ...tube('beam',(t,s)=>`M7 ${scanY(t,s)} L16 ${scanY(t,s)}`)],rigid((t,s)=>({dy:scanDip(t,s)}))),.75,.46);

// ---------- 12. TICKET TRANSFER (held → sent) ----------
// The ticket slides 7 units right while an arrow draws above it; the head grows at the moving tip.
// Strong: a short pull back, a fast drive past the mark and a settle. Reverse retracts both.
const TK2={x:4,y:12,w:17,h:12,r:1.8,n:2,dx:16};
const send=T([[0,0],[.05,0],[.55,1,'smooth']],[[0,0],[.1,-.06],[.32,1.07,'hit'],[.44,.98],[.54,1]],[[0,1],[.5,0,'smooth']],[[0,1],[.06,1.04],[.32,0,'hit'],[.38,0]]);
const tipX=(t,s,r)=>7+17*clamp(send(t,s,r));
const headK=(t,s,r)=>clamp((clamp(send(t,s,r))-.06)/.12); // the head paints once the stem is 1 unit long
const ticketTransfer=toggle('ticket-transfer','Ticket transfer',
 'Light: the ticket slides 7 units to the right as a forward arrow draws above it. Strong: it pulls back, drives past the mark and settles. Reverse retracts the arrow and returns the ticket.',[
 ...posed(ticketParts('ticket',TK2),rigid((t,s,r)=>({dx:7*send(t,s,r)}))),
 ...shown(tube('stem',(t,s,r)=>`M7 6.2 L${tipX(t,s,r)} 6.2`),(t,s,r)=>clamp(send(t,s,r))>.03?1:0),
 ...shown(tube('head',(t,s,r)=>{const x=tipX(t,s,r),k=headK(t,s,r);return `M${x-3*k} ${6.2-2.6*k} L${x} 6.2 L${x-3*k} ${6.2+2.6*k}`;}),(t,s,r)=>headK(t,s,r)>.3?1:0)
],.55,.54,'held','sent',.5,.38);

// ---------- 13. WAITLIST (queue advances, preview reset) ----------
// Three queue pills; you are the filled bottom one. The front pill is admitted (lifts away and fades,
// staying inside the canvas) and the queue moves up one slot. Strong: it pops out and the queue snaps up.
const pill=y=>rect(6,y-2,20,4,2);
const admitX=F([[0,0],[.06,0],[.3,-1.4,'smooth']],[[0,0],[.06,.4],[.22,-2.2,'out']]);
const admitA=F([[0,1],[.06,1],[.28,0,'smooth']],[[0,1],[.08,1,'hold'],[.2,0]]);
const queueUp=F([[.14,0],[.6,-8.2,'smooth']],[[.1,0],[.16,.6],[.36,-8.9,'hit'],[.46,-8],[.54,-8.2]]);
const pillRing=y=>holes(rect(4.7,y-3.3,22.6,6.6,3.3),rect(7.3,y-.7,17.4,1.4,.7));
const waitlist=geometricRearm(make('waitlist','Waitlist',
 'Light: the front of the queue is admitted and slides away as the list moves up one slot. Strong: it shoots out and the queue snaps up with a bounded overshoot. Retained; the preview reset restores the queue.',[
 ...shown(posed(framed('front',pill(7.8),pillRing(7.8)),rigid((t,s)=>({dy:admitX(t,s)}))),admitA),
 ...posed([...framed('next',pill(16),pillRing(16)),...bothFill('you',pill(24.2))],rigid((t,s)=>({dy:queueUp(t,s)})))
],.6,.54));

// ---------- 14. SOLD OUT (available → sold-out) ----------
// An X stamps across the ticket's main body, growing from its centre. Strong: it stamps past full size
// and the ticket dips. Reverse shrinks the X back into its centre.
const xk=T([[0,0],[.06,0],[.42,1,'smooth']],[[0,0],[.06,0,'hold'],[.2,1.12,'hit'],[.3,.96],[.38,1]],[[0,1],[.36,0,'smooth']],[[0,1],[.26,0,'out']]);
const stampDip=T(still(0),[[0,0],[.2,0,'hold'],[.26,.55,'hit'],[.36,-.1],[.44,0]],still(0),still(0));
const armD=sign=>(t,s,r)=>{const k=3*xk(t,s,r);return `M${11.4-k} ${16-sign*k} L${11.4+k} ${16+sign*k}`;};
const soldOut=toggle('sold-out','Sold out',
 'Light: an X grows across the ticket from its centre. Strong: it stamps past full size and the ticket dips under the blow. Reverse shrinks the X away.',
 posed([...ticketParts('ticket'),...shown([...tube('arm-a',armD(1)),...tube('arm-b',armD(-1))],(t,s,r)=>xk(t,s,r)>.05?1:0)],rigid((t,s,r)=>({dy:stampDip(t,s,r)}))),
 .42,.44,'available','sold-out',.36,.26);

// ---------- 15. EARLY BIRD (gesture) ----------
// A bird perches on the ticket's top edge. Light: it pecks 8° about its feet. Strong: it hops 2.6 units,
// lands and the ticket dips under it.
const TK3={x:4,y:15,w:24,h:12,r:2,n:2.2,dx:21};
const peck=F([[0,0],[.14,8],[.34,-3],[.5,0,'smooth']],[[0,0],[.08,10],[.2,-4],[.34,0,'hit'],[.44,6],[.56,0]]);
const hop=F(still(0),[[0,0],[.08,.4],[.22,-2.6],[.34,0,'hit'],[.42,-.5],[.5,0]]);
const birdPose=rigid((t,s)=>({dy:hop(t,s),angle:peck(t,s),pivot:[11.5,15]}));
const tkDip=F(still(0),[[0,0],[.34,0,'hold'],[.39,.45,'hit'],[.48,-.1],[.56,0]]);
const BIRD='M8.6 15 C8.4 12 9.6 8.6 13.4 8 C16 7.6 17.8 9.2 17.8 11.2 C17.8 13.6 16.6 15 14.8 15 Z';
const earlyBird=make('early-bird','Early bird',
 'Light: the bird on the ticket pecks 8° about its feet and settles. Strong: it hops 2.6 units, lands with a peck and the ticket dips beneath it.',[
 ...posed(ticketParts('ticket',TK3),rigid((t,s)=>({dy:tkDip(t,s)}))),
 ...posed([...framed('bird',BIRD,holes(BIRD,circle(15.4,10.2,.9))),fill('eye',circle(15.4,10.2,.75),'outline'),stroke('wing','M10.6 12.3 C11.6 10.6 13.6 10.6 14.8 12'),
  ...bothFill('beak',polygon([[17.6,10.4],[20.2,11.4],[17.6,12.4]],.3)),...tube('tail','M9 12.4 L6.2 9.8')],birdPose)
],.5,.56);

// ---------- 16. EVENT LIVE (off → on) ----------
// A stage with two spotlights. The beams fade in while the lamps swivel onto the stage. Light: an 8°
// swivel; Strong: a 25° swing with overshoot and a lamp flash. Reverse swings them out and fades.
const aim=T([[0,8],[.5,0,'smooth']],[[0,20],[.04,20,'hold'],[.22,-4,'hit'],[.34,1.5],[.44,0]],[[0,0],[.4,8,'smooth']],[[0,0],[.28,20,'out']]);
const glow=T([[0,0],[.45,1,'smooth']],[[0,0],[.06,0],[.2,1,'out']],[[0,1],[.4,0,'smooth']],[[0,1],[.22,0,'out']]);
const flash=T(still(1),[[0,1],[.18,1,'hold'],[.24,1.35,'hit'],[.36,1]],still(1),still(1));
const lamp=(n,cx)=>posed(bothFill(n,rect(cx-2.1,5.4,4.2,3,1)),(x,y,t,s,r)=>{const k=flash(t,s,r);return[cx+(x-cx)*k,6.9+(y-6.9)*k];});
// Beam feet sit 0.9 inward of the lamp so the swung-out outer corner stays 3 inside the canvas.
const beam=(n,cx,sign)=>shown(posed(framed(n,polygon([[cx,7],[cx-3+sign*.9,16.5],[cx+3+sign*.9,16.5]],[.6,.9,.9])),rigid((t,s,r)=>({angle:sign*aim(t,s,r),pivot:[cx,7]}))),glow);
const eventLive=toggle('event-live','Event live',
 'Light: the spotlights fade on and swivel 8° onto the stage. Strong: they swing in 25°, overshoot, and the lamps flash. Reverse swings them out and fades to dark.',[
 ...framed('stage',rect(5,20,22,6,1.8)),
 ...lamp('lamp-l',9.5),...lamp('lamp-r',22.5),
 ...beam('beam-l',9.5,1),...beam('beam-r',22.5,-1)
],.5,.44,'off','on',.4,.28);

// ---------- 17. EVENT REPLAY (gesture, 360° cycle) ----------
// A video card with a play triangle inside a circular arrow. The arrow turns one full revolution.
// Strong: a 25° wind-up, a fast turn past the mark and a settle while the triangle pulses.
const spin=F([[0,0],[.06,0],[.7,360,'smooth']],[[0,0],[.08,-25],[.42,380,'hit'],[.54,352],[.64,360]]);
const playK=F(still(1),[[0,1],[.42,1,'hold'],[.48,1.3,'hit'],[.6,1]]);
const arrowPose=rigid((t,s)=>({angle:spin(t,s),pivot:[16,16]}));
const eventReplay=make('event-replay','Event replay',
 'Light: the replay arrow turns one full revolution around the play mark. Strong: it winds up 25°, spins past the mark and settles while the play triangle pulses.',[
 ...framed('card',rect(4.5,5.5,23,21,2.6),holes(rect(3.2,4.2,25.6,23.6,3.9),rect(5.8,6.8,20.4,18.4,1.3))),
 ...posed([...tube('ring',arc(16,16,6,240,540)),...tube('head','M7.6 18.4 L10 16 L12.4 18.4')],arrowPose),
 ...posed(bothFill('play',polygon([[14.4,13],[14.4,19],[19.4,16]],.7)),(x,y,t,s)=>{const k=playK(t,s);return[16.4+(x-16.4)*k,16+(y-16)*k];})
],.7,.64);
eventReplay.cycleDegrees=360;

// ---------- 18. SPONSOR (gesture) ----------
// A star banner hangs from a rod and swings rigidly about the rod's centre. Light 4°; Strong a 13° gust
// with a damped return.
const sway=F([[0,0],[.16,4],[.44,-2],[.64,0,'smooth']],[[0,0],[.08,-5],[.24,13,'hit'],[.42,-8],[.56,4],[.68,-1.5],[.78,0]]);
const BANNER=polygon([[8.5,6.5],[23.5,6.5],[23.5,25.5],[16,21.8],[8.5,25.5]],[.3,.3,1,1,1]);
const sponsor=make('sponsor','Sponsor',
 'Light: the star banner sways 4° on its rod and settles. Strong: a gust swings it 13° and it rocks back to rest.',[
 ...tube('rod','M5 6.5 L27 6.5'),
 ...posed([...framed('banner',BANNER,holes(BANNER,star(16,15,4.6,2))),fill('star',star(16,15,4.4,1.9),'outline')],rigid((t,s)=>({angle:sway(t,s),pivot:[16,6.5]})))
],.64,.78);

// ---------- 19. NETWORKING (solo → connected) ----------
// Two people lean 5° toward each other as a link draws from both sides and meets at a node. Strong: a
// small lean back, a fast lean with overshoot and the node pops in. Reverse retracts the link.
const person=(n,cx)=>[...bothFill(n+'-head',circle(cx,11.6,2.4)),...framed(n+'-body',`M${cx-4.6} 22 C${cx-4.6} 18.8 ${cx-2.6} 16.6 ${cx} 16.6 C${cx+2.6} 16.6 ${cx+4.6} 18.8 ${cx+4.6} 22 Z`)];
const linkU=T([[0,0],[.06,0],[.5,1,'smooth']],[[0,0],[.06,0,'hold'],[.24,1,'hit'],[.3,1]],[[0,1],[.4,0,'smooth']],[[0,1],[.26,0,'out']]);
const lean=T([[0,0],[.06,0],[.5,5,'smooth']],[[0,0],[.08,-3],[.26,7.5,'hit'],[.38,4.4],[.48,5]],[[0,5],[.4,0,'smooth']],[[0,5],[.3,0,'out']]);
const nodeK=T([[.4,0],[.55,1,'out']],[[.22,0,'hold'],[.3,1.35,'hit'],[.42,1]],[[0,1],[.12,0,'out']],[[0,1],[.08,0,'out']]);
const networking=toggle('networking','Networking',
 'Light: two people lean 5° toward each other while a link draws in from both sides and a node appears where it meets. Strong: they lean back, snap together with overshoot and the node pops. Reverse straightens them and retracts the link.',[
 ...posed(person('left',8),rigid((t,s,r)=>({angle:lean(t,s,r),pivot:[8,22]}))),
 ...posed(person('right',24),rigid((t,s,r)=>({angle:-lean(t,s,r),pivot:[24,22]}))),
 ...shown([...tube('link-l',(t,s,r)=>`M12.6 19.6 L${12.6+3.4*clamp(linkU(t,s,r))} 19.6`),...tube('link-r',(t,s,r)=>`M19.4 19.6 L${19.4-3.4*clamp(linkU(t,s,r))} 19.6`)],(t,s,r)=>linkU(t,s,r)>.05?1:0),
 ...shown(posed(bothFill('node',circle(16,19.6,1.7)),(x,y,t,s,r)=>{const k=nodeK(t,s,r);return[16+(x-16)*k,19.6+(y-19.6)*k];}),(t,s,r)=>nodeK(t,s,r)>.15?1:0)
],.55,.48,'solo','connected',.4,.3);

// ---------- 20. EVENT MAP (gesture) ----------
// A folded venue map with a pin on its middle panel. Light: the pin lifts 1.4 and settles. Strong: it
// jumps 4.6, lands with a squash and the map jolts.
const pinLift=F([[0,0],[.06,0],[.22,-1.4,'out'],[.42,0,'in']],[[0,0],[.06,.5],[.24,-4.6],[.42,0,'hit'],[.5,-.8],[.58,0]]);
const pinSY=F(still(1),[[0,1],[.42,1,'hold'],[.47,.82,'hit'],[.56,1]]);
const mapJolt=F(still(0),[[0,0],[.42,0,'hold'],[.46,.35,'hit'],[.54,0]]);
const pinPose=(x,y,t,s)=>{const k=pinSY(t,s);return[16+(x-16)/Math.sqrt(k),18.6+(y-18.6)*k+pinLift(t,s)];};
const PIN='M16 18.6 C15 16.6 13.2 15 13.2 12.6 C13.2 11 14.4 9.8 16 9.8 C17.6 9.8 18.8 11 18.8 12.6 C18.8 15 17 16.6 16 18.6 Z';
const MAP=polygon([[3.5,9],[9.8,7],[22.2,9],[28.5,7],[28.5,24],[22.2,26],[9.8,24],[3.5,26]],.8);
const MAP_S=holes(polygon([[2.2,8.4],[9.8,5.7],[22.2,7.7],[29.8,5.7],[29.8,25.3],[22.2,27.3],[9.8,25.3],[2.2,27.3]],1.2),polygon([[4.8,9.9],[9.8,8.3],[22.2,10.3],[27.2,8.8],[27.2,22.8],[22.2,24.7],[9.8,22.7],[4.8,24.3]],.6));
const eventMap=make('event-map','Event map',
 'Light: the pin lifts 1.4 units off the folded map and settles back. Strong: it jumps 4.6 units, lands with a squash and the map jolts under it.',[
 ...posed([...framed('map',MAP,MAP_S),...tube('fold-l','M9.8 7 L9.8 24'),...tube('fold-r','M22.2 9 L22.2 26')],rigid((t,s)=>({dy:mapJolt(t,s)}))),
 ...posed([...framed('pin',PIN,holes(PIN,circle(16,12.6,1.2))),fill('pin-dot',circle(16,12.6,1.1),'outline')],pinPose)
],.42,.58);

const ORDER=[rsvp,eventInvite,venue,seatMap,agenda,eventSpeaker,eventBadge,checkInIcon,addToCalendar,eventReminder,ticketScan,ticketTransfer,waitlist,soldOut,earlyBird,eventLive,eventReplay,sponsor,networking,eventMap];
const fixed={'event-badge':['strap-l','strap-r'],'event-speaker':['column-frame','desk'],'event-live':['stage-frame'],'seat-map':['stage-frame','seat-0','seat-11'],'sponsor':['rod'],'networking':[],'event-reminder':[],'waitlist':[]};
const rigidParts={'event-badge':['card-frame','clip'],'event-reminder':['bell-frame','rim'],'sponsor':['banner-frame'],'event-replay':['ring','head'],'early-bird':['bird-frame','beak'],'event-live':['beam-l-frame','beam-r-frame'],'ticket-transfer':['ticket-frame'],'networking':['left-body-frame','right-body-frame'],'event-map':['map-frame']};
export const icons=ORDER.map((icon,index)=>({...icon,duotoneFill:true,number:926+index,constraints:[...(fixed[icon.name]||[]).map(p=>({type:'fixed',part:icon.name+'-'+p})),...(rigidParts[icon.name]||[]).map(p=>({type:'rigid',part:icon.name+'-'+p}))]}));
