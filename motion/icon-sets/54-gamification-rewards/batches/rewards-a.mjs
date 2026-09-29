// Gamification & rewards batch A: trophy, medal, level-up, xp-gain, streak-flame.
import {make,geometricRearm,F,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,bothFill,rotate,holes,starPts} from './helpers.mjs';

// ---------- TROPHY ----------
// Cup x8.5–23.5 with two ear handles (to x4.8/27.2), a stem and a base bar at y26.8. The whole trophy
// is hoisted as one rigid body about the base centre. Strong: a crouch, a 2.6-unit hoist with a 3° rock,
// a landing and one small rebound.
const cupD='M8.5 6 L23.5 6 L23.5 12.5 C23.5 17 20.2 20 16 20 C11.8 20 8.5 17 8.5 12.5 Z';
const hoist=F([[0,0],[.06,0],[.26,-1.2],[.56,0,'smooth']],[[0,0],[.08,.5],[.28,-2.6],[.46,0,'hit'],[.56,-.5],[.66,0]]);
const rock=F([[0,0],[.56,0]],[[0,0],[.08,0],[.24,-3],[.4,2.4],[.54,-.8],[.66,0]]);
const trophyPose=rigid((t,s)=>({dy:hoist(t,s),angle:rock(t,s),pivot:[16,26.8]}));
const trophy=make('trophy','Trophy',
 'Light: the trophy is raised 1.2 units and set back down. Strong: it crouches, is hoisted 2.6 units with a 3° rock, lands and rebounds once.',
 posed([...framed('cup',cupD),
  ...tube('handle-l','M8.5 8.5 C5.7 8.5 4.6 10 4.8 12.3 C5 14.4 6.6 15.7 8.9 15.6'),
  ...tube('handle-r','M23.5 8.5 C26.3 8.5 27.4 10 27.2 12.3 C27 14.4 25.4 15.7 23.1 15.6'),
  ...tube('stem','M16 20 L16 25.4'),...tube('base','M10 26.8 L22 26.8')],trophyPose),.56,.66);
trophy.duotoneFill=true;

// ---------- MEDAL ----------
// A V ribbon from (11.6,4.8)/(20.4,4.8) meets at (16,13.6) on the disc rim (centre 16,20.2 r6.4). The star is a
// filled mark in Outline and a real counter in Solid. Everything swings as a pendulum about the ribbon's top.
const DISC=[16,20.2],DR=6.4,PIV=[16,4.8];
const swing=F([[0,0],[.2,7],[.42,-4],[.62,0,'smooth']],[[0,0],[.08,-6],[.26,18],[.46,-12],[.62,6],[.76,-2],[.88,0]]);
const medalPose=rigid((t,s)=>({angle:swing(t,s),pivot:PIV}));
const starD=polygon(starPts(...DISC,2.9),.35);
const medal=make('medal','Medal',
 'Light: the medal swings 7° on its ribbon and settles. Strong: it is knocked back, swings 18° the other way and rings down through a damped pendulum.',
 posed([...tube('ribbon-l','M11.6 4.8 L16 13.6'),...tube('ribbon-r','M20.4 4.8 L16 13.6'),
  ...framed('disc',circle(...DISC,DR),holes(circle(...DISC,DR),starD)),fill('star',starD,'outline')],medalPose),.62,.88);
medal.duotoneFill=true;

// ---------- LEVEL UP ----------
// Three ascending steps; a ball rests on the second step and hops onto the top step. Result retained.
// Strong: a faster, higher hop with a hard landing and one small bounce.
const stairD='M4.5 27.5 L4.5 22.5 L12 22.5 L12 17 L19.5 17 L19.5 11.5 L27.5 11.5 L27.5 27.5 Z';
const B0=[15.5,13.25],B1=[23.5,7.75];
const hopX=F([[0,0],[.1,0],[.5,B1[0]-B0[0],'smooth']],[[0,0],[.1,0,'hold'],[.36,B1[0]-B0[0]]]);
const hopY=F([[0,0],[.1,0],[.3,-7.6,'out'],[.5,B1[1]-B0[1],'in']],[[0,0],[.06,.4],[.22,-8.6,'out'],[.36,B1[1]-B0[1],'hit'],[.44,B1[1]-B0[1]-.9],[.54,B1[1]-B0[1]]]);
const levelUp=geometricRearm(make('level-up','Level up',
 'Light: the ball hops from the second step up onto the top step. Strong: it crouches, springs higher and faster, lands hard and bounces once. The new level is retained; preview reset rearms.',[
 ...framed('stairs',stairD),
 ...posed(bothFill('ball',circle(...B0,2.2)),rigid((t,s)=>({dx:hopX(t,s),dy:hopY(t,s),pivot:B0})))
],.5,.54));
levelUp.duotoneFill=true;

// ---------- XP GAIN ----------
// A progress track (x4.5–27.5, y18–26) holds a filled bar at 40%. A star pellet drops onto the track; on
// contact it is absorbed and the bar extends to 85%. Solid: the track is a ring so the bar stays visible.
const TRACK=rect(4.5,18,23,8,4),TRACK_S=holes(rect(3.2,16.7,25.6,10.6,5.3),rect(5.8,19.3,20.4,5.4,2.7));
const barX0=6.6,barSpan=25.4-6.6;
const fillP=F([[0,.4],[.36,.4],[.86,.85,'smooth']],[[0,.4],[.2,.4,'hold'],[.5,.85,'out']]);
const barD=(t,s)=>rect(barX0,20.5,barSpan*fillP(t,s),3,1.5);
const dropY=F([[0,0],[.06,0],[.36,5.4,'in'],[.5,5.4]],[[0,0],[.04,0],[.2,5.4,'hit'],[.28,3.6],[.36,5.4]]);
const starShow=F([[0,1],[.36,1],[.5,0,'out']],[[0,1],[.3,1],[.42,0,'out']]);
const pellet=polygon(starPts(16,9,2.6),.3);
const xpGain=geometricRearm(make('xp-gain','XP gain',
 'Light: a star pellet drops onto the progress track, is absorbed and the bar extends from 40% to 85%. Strong: it drops fast, bounces once on the track and the bar drives out sooner. The gained progress is retained; preview reset rearms.',[
 fill('track-interior',TRACK,'tint'),stroke('track',TRACK),fill('track-solid',TRACK_S,'solid'),
 ...bothFill('bar',barD),
 ...shown(posed(bothFill('pellet',pellet),rigid((t,s)=>({dy:dropY(t,s),pivot:[16,9]}))),starShow)
],.86,.5));
xpGain.duotoneFill=true;

// ---------- STREAK FLAME ----------
// A flame (base y27.5, tip y6.5) with an inner tongue counter. Light: it sways 3 units at the tip and the
// tongue breathes. Strong: it crouches, flares 10% taller with a lean and settles through two swings.
const FB=27.5,FH=21;
const flameD='M16 6.5 C20 10.8 23 14.4 23 19 C23 23.7 19.9 27.5 16 27.5 C12.1 27.5 9 23.7 9 19 C9 15.2 11 12.4 12.4 10.6 C12.6 13.2 13.8 14.4 15 14.2 C14.2 11.6 14.8 8.8 16 6.5 Z';
const tongueD='M16.2 16.4 C17.6 18.2 18.6 19.8 18.6 21.4 C18.6 23.1 17.5 24.2 16 24.2 C14.5 24.2 13.4 23.1 13.4 21.4 C13.4 19.8 14.6 18.2 16.2 16.4 Z';
const lean=F([[0,0],[.24,3],[.5,-2],[.74,0,'smooth']],[[0,0],[.1,-1.5],[.28,4],[.46,-2.6],[.62,1],[.74,0]]);
const flare=F([[0,1],[.74,1]],[[0,1],[.1,.94],[.28,1.1],[.46,.97],[.62,1.02],[.74,1]]);
const breathe=F([[0,1],[.18,1.08],[.46,.96],[.7,1,'smooth']],[[0,1],[.1,.9],[.3,1.14],[.48,.95],[.64,1.02],[.74,1]]);
const flamePose=(x,y,t,s)=>{const h=(FB-y)/FH;return [x+lean(t,s)*h*h,FB-(FB-y)*flare(t,s)];};
const tonguePose=(x,y,t,s)=>{const [px,py]=flamePose(x,y,t,s),k=breathe(t,s);return [px,FB-(FB-py)*k-2.4*(1-k)];};
const streakFlame=make('streak-flame','Streak flame',
 'Light: the flame sways 3 units at its tip while the inner tongue breathes. Strong: it crouches, flares 10% taller with a 4-unit lean and settles through two smaller swings.',[
 ...posed(framed('flame',flameD,holes(flameD,tongueD)),flamePose),
 ...posed([stroke('tongue',tongueD)],tonguePose)
],.74,.74);
streakFlame.duotoneFill=true;

export const icons=[trophy,medal,levelUp,xpGain,streakFlame];
