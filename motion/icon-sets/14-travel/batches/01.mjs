import {make,P,F,posed,rigid,rect,circle,polygon,fill,stroke,ink,plainRing,surface,policy,hole,holes,capsule,arch,bothFill,track,timePair} from './helpers.mjs';
const bank=F([[0,0],[.28,-7,'smooth'],[.7,0,'smooth']],[[0,0],[.14,3],[.5,-14],[.95,2],[1.2,0]]);
const advance=F([[0,0],[.28,-.8,'smooth'],[.7,0,'smooth']],[[0,0],[.14,.25],[.5,-2],[.95,.3],[1.2,0]]);
const planeD=polygon([[16,6],[18.25,12.5],[27,19.5],[27,21.5],[18.25,18.5],[18.25,23.5],[21.5,26],[21.5,27],[16,25.7],[10.5,27],[10.5,26],[13.75,23.5],[13.75,18.5],[5,21.5],[5,19.5],[13.75,12.5]],.65);
const airplane=policy(make('airplane','Airplane','The aircraft prepares its turn, banks and advances, then returns to level flight as one rigid shape.',posed(surface('airframe',planeD),rigid((t,s)=>({angle:bank(t,s),dy:advance(t,s),pivot:s?[16,19]:[16,16]}))),.78,1.23));
airplane.constraints=[{type:'rigid',part:'airplane-airframe'}];
const bookY=F([[0,0],[.25,-.6,'out'],[.67,0,'smooth']],[[0,0],[.12,.15],[.38,-1.2],[.65,.1],[.85,0]]);
const emblem=F([[0,0],[.58,180,'smooth']],[[0,0],[.12,-12],[.42,192],[.62,176],[.8,180]]);
const cover=rect(9,5.5,14,22,2),equator='M12.75 13 L19.25 13';
const passport=policy(make('passport','Passport','The booklet is presented while its globe emblem makes one half turn; it settles back on the same reading pose.',[
 ...posed(surface('cover',cover,[circle(16,13,3.6),capsule([12.5,20.5],[19.5,20.5],2.6),capsule([12.5,24],[19.5,24],2.6)]),rigid((t,s)=>({dy:bookY(t,s)}))),
 ...posed([stroke('globe',circle(16,13,3.25)),...ink('equator',equator)],rigid((t,s)=>({angle:emblem(t,s),pivot:[16,13],dy:bookY(t,s)}))),
 ...posed([stroke('title-one','M12.5 20.5 L19.5 20.5'),stroke('title-two','M12.5 24 L19.5 24')],rigid((t,s)=>({dy:bookY(t,s)})))
],.75,.9));
passport.cycleDegrees=180;passport.constraints=[{type:'rigid',part:'passport-cover'},{type:'rigid',part:'passport-equator'}];
const passD='M8 10 L18 10 C18 10.828 18.672 11.5 19.5 11.5 C20.328 11.5 21 10.828 21 10 L24 10 C24.828 10 25.5 10.672 25.5 11.5 L25.5 20.5 C25.5 21.328 24.828 22 24 22 L21 22 C21 21.172 20.328 20.5 19.5 20.5 C18.672 20.5 18 21.172 18 22 L8 22 C7.172 22 6.5 21.328 6.5 20.5 L6.5 11.5 C6.5 10.672 7.172 10 8 10 Z';
const present=F([[0,0],[.26,-6,'out'],[.62,0,'smooth']],[[0,0],[.12,3],[.45,-14],[.9,2],[1.15,0]]);
const passY=F([[0,0],[.7,0]],[[0,0],[.12,-.3],[.45,1],[.9,-.2],[1.15,0]]);
const boarding=policy(make('boarding-pass','Boarding pass','The pass turns toward a scanner, sweeps past the scanner and lowers with all print attached.',posed([
 ...surface('ticket',passD,[capsule([10,13.5],[14.5,13.5],2.2),capsule([10,18.5],[14.5,18.5],2.2)]),stroke('line-one','M10 13.5 L14.5 13.5'),stroke('line-two','M10 18.5 L14.5 18.5')
],rigid((t,s)=>({angle:present(t,s),dy:passY(t,s)}))),.7,1.18));
boarding.constraints=[{type:'rigid',part:'boarding-pass-ticket'}];
const grip=(t,s,r)=>r?track(s?[[0,5.7],[.12,5.7],[.5,8,'smooth']]:[[0,5.7],[.4,8,'smooth']],t):F([[0,8],[.05,8],[.55,5.5,'out'],[.7,5.7,'smooth']],[[0,8],[.08,8.2],[.4,5.3],[.6,5.8],[.75,5.7]])(t,s);
const tip=F([[0,0],[.8,0]],[[0,0],[.25,-3],[.62,7],[1.02,-1.2],[1.28,0]]);
const suitcase=policy(make('suitcase','Suitcase','The grip extends from fixed rails before the case tips onto its rear wheel. The raised handle remains.',posed([
 ...ink('handle',(t,s,r)=>arch(13,19,12,grip(t,s,r),1.5)),
 ...surface('case',rect(8,12,16,13,2),[capsule([11,18.5],[21,18.5],2.6)]),stroke('strap','M8 18.5 L24 18.5'),
 ...bothFill('wheel-left',circle(11,26,1.5)),...bothFill('wheel-right',circle(21,26,1.5))
],rigid((t,s,r)=>({angle:r?0:tip(t,s),pivot:[21,26]}))),.78,1.28));
Object.assign(suitcase,{stateful:true,initialState:'lowered',finalState:'raised',reverseSuffix:'-retract',forwardLabel:'Raise handle',reverseLabel:'Retract handle',reverseTimings:timePair(.45,.55)});
suitcase.constraints=[{type:'rigid',part:'suitcase-case'},{type:'rigid',part:'suitcase-wheel-left'}];
const packD='M8 15 C8 11.667 10.667 10 16 10 C21.333 10 24 11.667 24 15 L24 25 C24 26.381 22.881 27.5 21.5 27.5 L10.5 27.5 C9.119 27.5 8 26.381 8 25 Z';
const hitch=F([[0,0],[.28,-1,'out'],[.65,0,'smooth']],[[0,0],[.15,.15],[.48,-1.7],[.85,.2],[1.1,0]]);
const loopX=F([[0,0],[.08,0],[.36,.5,'out'],[.7,0,'smooth']],[[0,0],[.2,-.3],[.55,1.4],[.9,-.3],[1.12,0]]);
const pocket=F([[0,0],[.7,0]],[[0,0],[.3,-.2],[.62,.7],[.95,-.15],[1.15,0]]);
const pocketD=(t,s)=>rect(11.5,18+pocket(t,s),9,5.5,1.5);
const backpack=policy(make('backpack','Backpack','The pack is hitched upward; its attached grab loop flexes and its front pocket follows the load.',posed([
 ...ink('loop',(t,s)=>`M12.5 10.3125 C${14.5+loopX(t,s)*.667} 4.5625 ${17.5+loopX(t,s)*.667} 4.5625 19.5 10.3125`),
 ...surface('body',packD,[pocketD,capsule([10.2,14.5],[21.8,14.5],2.3)]),
 stroke('flap','M8.3 14.5 L23.7 14.5'),stroke('pocket',pocketD)
],rigid((t,s)=>({dy:hitch(t,s)}))),.78,1.18));
backpack.constraints=[{type:'rigid',part:'backpack-body'}];
export const icons=[airplane,passport,boarding,suitcase,backpack];
