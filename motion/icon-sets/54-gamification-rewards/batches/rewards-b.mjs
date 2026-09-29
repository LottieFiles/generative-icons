// Gamification & rewards batch B: badge-unlock, leaderboard, dice, game-controller, coin.
import {make,toggle,geometricRearm,F,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,bothFill,rotate,holes,capsule,starPts,scalePose} from './helpers.mjs';
// Forward and reverse actions are directed separately; the builder passes r=true for the reverse action.
const dir=(fwd,rev)=>(t,s,r)=>(r?rev:fwd)(t,s);

// ---------- BADGE UNLOCK ----------
// Shield badge (x5.5–26.5, y4.5–28) holding a padlock. Unlock: the shackle lifts, swings open, the lock
// fades and a star pops in its place. Reverse: the star shrinks away, the lock returns and the shackle
// drops closed. Solid: the shield is a ring so the lock and star stay visible inside it.
const shieldD=i=>`M16 ${4.5+1.1*i} C${19.5-.5*i} ${5.8+.9*i} ${23-.5*i} ${7+.7*i} ${26.5-i} ${8.3+.45*i} L${26.5-i} 15.5 C${26.5-i} ${21.5-.5*i} ${22-.6*i} ${26.2-i} 16 ${28-1.2*i} C${10+.6*i} ${26.2-i} ${5.5+i} ${21.5-.5*i} ${5.5+i} 15.5 L${5.5+i} ${8.3+.45*i} C${9+.5*i} ${7+.7*i} ${12.5+.5*i} ${5.8+.9*i} 16 ${4.5+1.1*i} Z`;
const lockBody=rect(12.2,15.2,7.6,6,1.4),shackleD='M13.7 15.2 L13.7 12.6 C13.7 11.1 14.7 10 16 10 C17.3 10 18.3 11.1 18.3 12.6 L18.3 15.2';
const lift=dir(F([[0,0],[.04,0],[.22,-1.6,'smooth']],[[0,0],[.04,0],[.14,-2.1,'hit'],[.22,-1.6]]),F([[0,-1.6],[.34,-1.6],[.54,0,'in']],[[0,-1.6],[.3,-1.6,'hold'],[.42,0,'hit']]));
const openA=dir(F([[0,0],[.16,0],[.38,25,'smooth']],[[0,0],[.12,0],[.26,33,'hit'],[.36,25]]),F([[0,25],[.2,25],[.42,0,'smooth']],[[0,25],[.16,25],[.32,0,'hit']]));
const lockShow=dir(F([[0,1],[.36,1],[.52,0,'out']],[[0,1],[.32,1],[.44,0,'out']]),F([[0,0],[.14,0],[.3,1,'out']],[[0,0],[.12,0],[.24,1,'out']]));
const starK=dir(F([[0,.5],[.44,.5],[.68,1,'smooth']],[[0,.5],[.4,.5],[.52,1.18],[.62,.96],[.7,1]]),F([[0,1],[.02,1],[.22,.5,'in']],[[0,1],[.02,1],[.08,1.1],[.2,.5,'in']]));
const starShow=dir(F([[0,0],[.44,0],[.56,1,'out']],[[0,0],[.4,0],[.48,1,'out']]),F([[0,1],[.06,1],[.2,0,'out']],[[0,1],[.06,1],[.18,0,'out']]));
const badgeStar=polygon(starPts(16,16.6,4.2),.4);
const shacklePose=rigid((t,s,r)=>({dy:lift(t,s,r),angle:openA(t,s,r),pivot:[18.3,15.2]}));
const badgeUnlock=toggle('badge-unlock','Badge unlock',
 'Unlock: the shackle lifts 1.6 units and swings open 25°, the padlock fades and a star pops into the badge. Strong: a snap lift with overshoot, a hard swing and a bigger star pop. Reverse: the star shrinks away, the lock returns and the shackle drops shut.',[
 fill('shield-interior',shieldD(0),'tint'),stroke('shield',shieldD(0)),fill('shield-solid',holes(shieldD(-1.3),shieldD(1.3)),'solid'),
 ...shown(bothFill('lock',lockBody),lockShow),
 ...shown(posed(tube('shackle',shackleD),shacklePose),lockShow),
 ...shown(posed(bothFill('star',badgeStar),(x,y,t,s,r)=>scalePose(16,16.6,starK(t,s,r))(x,y)),starShow)
],.7,.72,'locked','unlocked',.56,.44);
badgeUnlock.duotoneFill=true;

// ---------- LEADERBOARD ----------
// A podium drawn as one stepped outline with two divider lines. Prepared: a flat 3.5-unit stand. Result:
// the three places rise in order (3rd, 2nd, 1st) and a star pops above first place. Result retained.
const HL=F([[0,24],[.2,24],[.55,18,'smooth']],[[0,24],[.14,24,'hold'],[.28,17.3,'hit'],[.38,18]]);
const HM=F([[0,24],[.3,24],[.75,10,'smooth']],[[0,24],[.22,24,'hold'],[.4,8.8,'hit'],[.52,10]]);
const HR=F([[0,24],[.1,24],[.4,21,'smooth']],[[0,24],[.06,24,'hold'],[.2,20.4,'hit'],[.3,21]]);
const podiumD=(t,s)=>`M4.5 27.5 L4.5 ${HL(t,s)} L12 ${HL(t,s)} L12 ${HM(t,s)} L20 ${HM(t,s)} L20 ${HR(t,s)} L27.5 ${HR(t,s)} L27.5 27.5 Z`;
const podiumS=(t,s)=>holes(podiumD(t,s),capsule([12,Math.min(HL(t,s)+1.4,26.2)],[12,26.3],1.7),capsule([20,Math.min(HR(t,s)+1.4,26.2)],[20,26.3],1.7));
const crownK=F([[0,.4],[.66,.4],[.86,1,'smooth']],[[0,.4],[.42,.4],[.54,1.2],[.64,.95],[.72,1]]);
const crownShow=F([[0,0],[.66,0],[.76,1,'out']],[[0,0],[.42,0],[.5,1,'out']]);
const topStar=polygon(starPts(16,6.2,2.6),.3);
const leaderboard=geometricRearm(make('leaderboard','Leaderboard',
 'Light: the three places rise out of a flat stand in order, third, second then first, and a star settles above first place. Strong: each place drives up with a hard stop and overshoot, and the star pops in. The podium is retained; preview reset rearms.',[
 fill('podium-interior',podiumD,'tint'),stroke('podium',podiumD),fill('podium-solid',podiumS,'solid'),
 stroke('divider-l',(t,s)=>`M12 ${Math.min(HL(t,s)+.3,26.2)} L12 27.5`),stroke('divider-r',(t,s)=>`M20 ${Math.min(HR(t,s)+.3,26.2)} L20 27.5`),
 ...shown(posed(bothFill('star',topStar),(x,y,t,s)=>scalePose(16,6.2,crownK(t,s))(x,y)),crownShow)
],.86,.72));
leaderboard.duotoneFill=true;

// ---------- DICE ----------
// A rounded die (6.5–25.5) showing three pips on the diagonal. Light: it rocks 10° on its centre. Strong:
// it jumps 2.6 units, tumbles 55 degrees and lands squarely with one small bounce.
// End at exactly zero native rotation: a 360-degree matrix changes raster edge coverage.
const PIPS=[[12,12],[16,16],[20,20]];
const dieD=rect(8,8,16,16,3.8);
const tilt=F([[0,0],[.18,10],[.4,-4],[.58,0,'smooth']],[[0,0],[.08,-12],[.27,55],[.4,0,'hit'],[.48,-8],[.58,0]]);
const jump=F([[0,0],[.58,0]],[[0,0],[.08,.6],[.24,-2.6,'out'],[.4,0,'hit'],[.48,-.6],[.58,0]]);
const diePose=rigid((t,s)=>({angle:tilt(t,s),dy:jump(t,s),pivot:[16,16]}));
const dice=make('dice','Dice',
 'Light: the die rocks 10 degrees on its centre and settles. Strong: it dips, jumps 2.6 units, tumbles 55 degrees in the air and lands squarely with one small bounce.',
 posed([...framed('die',dieD,holes(dieD,...PIPS.map(([x,y])=>circle(x,y,1.6)))),...PIPS.map(([x,y],i)=>fill('pip-'+i,circle(x,y,1.5),'outline'))],diePose),.58,.58);
dice.duotoneFill=true;

// ---------- GAME CONTROLLER ----------
// A gamepad body with two grips, a d-pad cross on the left and two buttons on the right. Light: the top
// button presses and the pad dips. Strong: the pad rumbles (a decaying shake) while both buttons press.
const padD='M8 8 L24 8 C27.2 8 29.1 11 28.2 15 L27 22.4 C26.6 24.4 24.2 25.2 22.8 23.6 L20.2 21 L11.8 21 L9.2 23.6 C7.8 25.2 5.4 24.4 5 22.4 L3.8 15 C2.9 11 4.8 8 8 8 Z';
const DP=[10.6,14.5],BTN=[[20.9,12.6],[24,15.8]];
const crossHole=polygon([[DP[0]-.9,DP[1]-3.1],[DP[0]+.9,DP[1]-3.1],[DP[0]+.9,DP[1]-.9],[DP[0]+3.1,DP[1]-.9],[DP[0]+3.1,DP[1]+.9],[DP[0]+.9,DP[1]+.9],[DP[0]+.9,DP[1]+3.1],[DP[0]-.9,DP[1]+3.1],[DP[0]-.9,DP[1]+.9],[DP[0]-3.1,DP[1]+.9],[DP[0]-3.1,DP[1]-.9],[DP[0]-.9,DP[1]-.9]],.45);
const press=i=>F(i?[[0,1],[.5,1]]:[[0,1],[.06,1],[.14,.7,'out'],[.24,.7],[.4,1,'smooth']],[[0,1],[.08+.12*i,1],[.14+.12*i,.68,'hit'],[.22+.12*i,.68],[.34+.12*i,1]]);
const dip=F([[0,0],[.06,0],[.14,.5,'out'],[.24,.5],[.4,0,'smooth']],[[0,0],[.5,0]]);
const rumble=(t,s)=>{if(!s)return 0;const e=Math.min(1,t/.06)*Math.max(0,1-(t-.06)/.44);return .8*Math.sin(t*70)*e;};
const padPose=rigid((t,s)=>({dx:rumble(t,s),dy:dip(t,s),angle:rumble(t,s)*2.5,pivot:[16,16]}));
const btnPose=i=>{const k=press(i);return (x,y,t,s)=>{const [px,py]=scalePose(...BTN[i],k(t,s))(x,y);return padPose(px,py,t,s);};};
const gameController=make('game-controller','Game controller',
 'Light: the top button presses in and the pad dips half a unit under the thumb. Strong: the pad rumbles with a decaying shake while both buttons press in turn.',[
 ...posed([...framed('pad',padD,holes(padD,crossHole,...BTN.map(([x,y])=>circle(x,y,1.5)))),
  stroke('dpad-v',`M${DP[0]} ${DP[1]-2.7} L${DP[0]} ${DP[1]+2.7}`),stroke('dpad-h',`M${DP[0]-2.7} ${DP[1]} L${DP[0]+2.7} ${DP[1]}`)],padPose),
 ...BTN.flatMap(([x,y],i)=>posed([fill('button-'+i,circle(x,y,1.35),'outline'),fill('button-'+i+'-solid',circle(x,y,1.05),'solid')],btnPose(i)))
],.4,.56);
gameController.duotoneFill=true;

// ---------- COIN ----------
// A coin (centre 16,17 r8.6) with a star face. It flips edge-on about its vertical axis: the drawing's
// width follows cos(phase). Light: one full turn with a small hop. Strong: it is tossed 4 units, turns
// twice and lands with a bounce.
const CC=[16,17],CR=8.6;
const phase=F([[0,0],[.06,0],[.7,360,'smooth']],[[0,0],[.06,0],[.5,720,'smooth']]);
const toss=F([[0,0],[.06,0],[.36,-1.5],[.7,0,'smooth']],[[0,0],[.06,.4],[.28,-4],[.5,0,'hit'],[.58,-.7],[.68,0]]);
const coinPose=(x,y,t,s)=>[CC[0]+(x-CC[0])*Math.cos(phase(t,s)*Math.PI/180),y+toss(t,s)];
const coinStar=polygon(starPts(...CC,3.6),.4);
const coin=make('coin','Coin',
 'Light: the coin turns once about its vertical axis, going edge-on and back, with a small hop. Strong: it is tossed 4 units, turns twice in the air and lands with a bounce.',
 posed([...framed('face',circle(...CC,CR),holes(circle(...CC,CR),coinStar)),fill('star',coinStar,'outline')],coinPose),.7,.68);
coin.duotoneFill=true;

export const icons=[badgeUnlock,leaderboard,dice,gameController,coin];
