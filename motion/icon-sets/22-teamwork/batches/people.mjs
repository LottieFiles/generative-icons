// teamwork batch 1 — Claude revised draft v2, unverified geometry
import {make,stroke,fill,tube,bothFill,framed,rigid,curve,circle,rect,arc,hole,holes,capsule,polygon} from './helpers.mjs';

const person=(hx,hy,hr,sh)=>[fill('head-tint',circle(hx,hy,hr),'tint'),stroke('head',circle(hx,hy,hr)),fill('head-solid',circle(hx,hy,hr),'solid'),stroke('shoulders',sh),fill('shoulders-solid',sh,'solid')];

// 1 invite-teammate: sealed invitation tilts toward the teammate (gesture)
function inviteTeammate(){
 const SH='M3.5 26 L3.5 24 C3.5 20.2 6 18.2 8 18.2 C10 18.2 12.5 20.2 12.5 24 L12.5 26 C12.5 26.6 12.1 27 11.5 27 L4.5 27 C3.9 27 3.5 26.6 3.5 26 Z';
 const ENV=rect(17.5,11.5,11,11,2),SEAL='M19 13.8 L23 17.6 L27 13.8';
 const BAND=polygon([[19,13.8],[23,17.6],[27,13.8],[27,15.6],[23,19.4],[19,15.6]],.4);
 const tilt=(t,s)=>curve(t,s,[[0,0],[.32,-3,'out'],[.8,0,'out']],[[0,0],[.28,-8],[.44,-7],[.9,0]]);
 const pose=rigid((t,s)=>({angle:tilt(t,s),dy:tilt(t,s)*.2,pivot:[17.5,22.5]}));
 const parts=[
  ...person(8,11,3,SH).filter(p=>p.name!=='head-tint'),
  ...[fill('envelope-tint',ENV,'tint'),stroke('envelope',ENV),fill('envelope-solid',hole(ENV,BAND),'solid'),stroke('seal',SEAL)].map(p=>({...p,pose}))];
 return {...make('invite-teammate','Invite teammate','The sealed invitation tilts and lifts toward the teammate, then settles back',parts,.8,.9),duotoneFill:true};
}

// 2 join-team: the newcomer's seat slides into the empty place (state, rearm seat-new)
function joinTeam(){
 const seat=(a,r)=>arc(16,16,r,a-22,a+22);
 const R=(t,s)=>curve(t,s,[[0,12.5],[.7,9,'out']],[[0,12.5],[.14,12.6],[.5,8.5],[.75,9]]);
 const parts=[
  ...framed('table',circle(16,16,4)),
  ...tube('seat-top',seat(270,9)),...tube('seat-right',seat(0,9)),...tube('seat-bottom',seat(90,9)),
  ...tube('seat-new',seat(180,12.5)).map(p=>({...p,d:(t,s)=>seat(180,R(t,s))}))];
 return {...make('join-team','Join team','A fourth seat slides in to the round table; Strong drives in with one overshoot and settles',parts,.75,.85,['seat-new']),duotoneFill:true};
}

// 3 team-directory: a rear index card rises behind the front member card (gesture)
function teamDirectory(){
 const CARD=rect(7.5,11,17,16,2.2);
 const top=(t,s)=>curve(t,s,[[0,6.5],[.35,5,'out'],[.85,6.5,'out']],[[0,6.5],[.3,3.4],[.44,3.6],[.95,6.5]]);
 // sides stay pinned at y=12, inside the front card seam (named opaque join); only the exposed top moves
 const back=(t,s)=>{const y=top(t,s);return `M10.5 11 L10.5 ${y+2} C10.5 ${y+.9} 11.4 ${y} 12.5 ${y} L19.5 ${y} C20.6 ${y} 21.5 ${y+.9} 21.5 ${y+2} L21.5 11`;};
 const parts=[
  ...tube('back-card',back(0,false)).map(p=>({...p,d:(t,s)=>back(t,s)})),
  fill('card-tint',CARD,'tint'),stroke('card',CARD),
  fill('card-solid',holes(CARD,circle(16,17.5,2.6),capsule([12.5,23],[19.5,23],2.4)),'solid'),
  fill('member',circle(16,17.5,2.1),'outline'),stroke('member-line','M12.5 23 L19.5 23')];
 return {...make('team-directory','Team directory','The next index card rises behind the front member card and slides back into the file',parts,.85,.95),duotoneFill:true};
}

// 4 team-roles: a role badge swings on its lanyard clip (gesture)
function teamRoles(){
 const BADGE=rect(9.5,12,13,15,2.2);
 const swing=(t,s)=>curve(t,s,[[0,0],[.25,5,'out'],[.5,-2],[.72,0]],[[0,0],[.1,-3],[.34,14],[.6,-4],[.9,0]]);
 const pose=rigid((t,s)=>({angle:swing(t,s),pivot:[16,5.5]}));
 const parts=[
  ...bothFill('clip',circle(16,5.5,1.6)),
  ...tube('strap-left','M16 6.5 L12.5 12'),...tube('strap-right','M16 6.5 L19.5 12'), // straps buried in the clip disk and badge edge (named joins)
  fill('badge-tint',BADGE,'tint'),stroke('badge',BADGE),
  fill('badge-solid',holes(BADGE,circle(16,17.5,2.5),capsule([12.5,23],[19.5,23],2.4)),'solid'),
  fill('role',circle(16,17.5,2.2),'outline'),stroke('role-line','M12.5 23 L19.5 23')].map(p=>({...p,pose}));
 return {...make('team-roles','Team roles','The role badge swings on its clip; Strong winds back, swings wide and damps out',parts,.8,1.0),duotoneFill:true};
}

// 5 team-presence: presence arcs sweep open beside the teammate (state, rearm both pings)
function teamPresence(){
 const SH='M4.5 26 L4.5 24 C4.5 20 7.5 18.5 11 18.5 C14.5 18.5 17.5 20 17.5 24 L17.5 26 C17.5 26.6 17.1 27 16.5 27 L5.5 27 C4.9 27 4.5 26.6 4.5 26 Z';
 const ping=(name,r,l,h)=>{const a=(t,s)=>curve(t,s,l,h);return tube(name,arc(11,11,r,-1,1)).map(p=>({...p,d:(t,s)=>{const v=Math.max(a(t,s),.01);return arc(11,11,r,-v,v);},visibility:(t,s)=>a(t,s)>.5?1:0}));};
 const parts=[
  ...person(11,11,3.5,SH),
  ...ping('ping-inner',7,[[0,0],[.45,40,'out']],[[0,0],[.18,42],[.3,40]]),
  ...ping('ping-outer',10.7,[[0,0],[.3,0],[.8,40,'out']],[[0,0],[.12,0,'hold'],[.36,42],[.5,40]])];
 return {...make('team-presence','Team presence','Two presence arcs sweep open beside the teammate and stay lit; Strong over-sweeps and settles',parts,.85,.55,['ping-inner','ping-outer']),duotoneFill:true};
}

export const icons=[inviteTeammate(),joinTeam(),teamDirectory(),teamRoles(),teamPresence()];
