import {curve,track,bothFill,circle} from './helpers.mjs';
// Explicit Claude-authored schedules. Reset is independent geometric withdrawal.
export const driver=(light,strong,reset)=>(t,s,r)=>r?track(reset,t):curve(t,s,light,strong);
export const phase=(la,lb,sa,sb,ra=.10,rb=.28)=>driver([[la,0],[lb,1,'smooth']],[[sa,0],[sb,1]],[[ra,1],[rb,0,'smooth']]);
export const flex=keys=>(t,s,r)=>s&&!r?curve(t,true,[[0,0]],keys):0;
export const anchor=(name,x,y,r=2)=>bothFill(name,circle(x,y,r));
