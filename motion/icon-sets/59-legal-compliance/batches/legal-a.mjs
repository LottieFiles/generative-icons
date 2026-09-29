// Legal & compliance batch A: gavel, justice-scale, contract, rubber-stamp, official-seal.
import {make,toggle,dir,F,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,rotate,holes,curveFrom,mix} from './helpers.mjs';
import svgpath from 'svgpath';

// ---------- GAVEL ----------
// The mallet rests on its sound block; the hand end (26.5,13) is the pivot. Head 11 x 6.4 across the
// handle at (9.8,19.5). Light: a 10° tap. Strong: the gavel is raised 30°, held, and slammed down;
// the block takes the shock.
const GP=[26.5,13],GH=[9.8,19.5],GU=[.932,-.363];
const headD=svgpath(rect(-5.5,-3.2,11,6.4,2)).rotate(68.7).translate(...GH).toString();
const swing=F([[0,0],[.22,10,'out'],[.4,0,'in'],[.48,1.5,'out'],[.56,0]],[[0,0],[.2,30],[.3,30,'hold'],[.4,0,'hit'],[.48,5],[.62,0]]);
const shock=F([[0,0],[.56,0]],[[0,0],[.4,0],[.44,.5,'hit'],[.56,0]]);
const gavel=make('gavel','Gavel',
 'Light: the gavel taps, lifting 10° off its block and dropping back. Strong: it is raised 30°, held a beat and slammed down; the block takes the shock.',[
 ...posed([...framed('head',headD),...tube('handle',`M${GH[0]+2.5*GU[0]} ${GH[1]+2.5*GU[1]} L${GP}`)],rigid((t,s)=>({angle:swing(t,s),pivot:GP}))),
 ...posed(tube('block','M4.5 27.8 L16 27.8'),rigid((t,s)=>({dy:shock(t,s)})))
],.56,.62);
gavel.duotoneFill=true;

// ---------- JUSTICE SCALE ----------
// Post and base fixed; the beam pivots at the post top (16,9.5) and each pan hangs from its end on a
// V of strings, translating with it. Light tips 5° once. Strong swings 11° and rings down.
const tilt=F([[0,0],[.3,-5,'out'],[.68,0,'smooth']],[[0,0],[.08,3],[.3,-13],[.5,8],[.68,-4],[.84,1.4],[.96,0]]);
const panD=x=>`M${x-3.4} 17.5 L${x+3.4} 17.5 C${x+3.4} 19.2 ${x+1.9} 20.3 ${x} 20.3 C${x-1.9} 20.3 ${x-3.4} 19.2 ${x-3.4} 17.5 Z`;
const hang=(sign)=>rigid((t,s)=>{const a=tilt(t,s)*Math.PI/180;return {dx:sign*8.5*(Math.cos(a)-1),dy:sign*8.5*Math.sin(a)};});
const pan=(name,x,sign)=>posed([...tube(name+'-string-l',`M${x} 9.5 L${x-2.6} 17.5`),...tube(name+'-string-r',`M${x} 9.5 L${x+2.6} 17.5`),...framed(name,panD(x))],hang(sign));
const justiceScale=make('justice-scale','Justice scale',
 'Light: the beam tips 5° and the pans ride its ends back to level. Strong: it swings 13° and rings down through three shrinking swings.',[
 ...tube('post','M16 9.5 L16 27.5'),...tube('base','M10 27.5 L22 27.5'),
 ...posed(tube('beam','M7.5 9.5 L24.5 9.5'),rigid((t,s)=>({angle:tilt(t,s),pivot:[16,9.5]}))),
 ...pan('pan-l',7.5,-1),...pan('pan-r',24.5,1)
],.68,.96);
justiceScale.duotoneFill=true;

// ---------- CONTRACT ----------
// A sheet with a folded corner and two clauses. Signing draws a scrawl on the signature line from
// its start; the signed sheet is retained and the reverse erases it. Strong signs in 0.26 s and the
// sheet dips under the pen. Solid keeps the sheet as a frame ring so the strokes stay visible.
const sheetD=polygon([[7.5,4.5],[19.5,4.5],[24.5,9.5],[24.5,27.5],[7.5,27.5]],[2,.3,.3,2,2]);
const sheetS=holes(sheetD,polygon([[8.8,5.8],[18.9,5.8],[23.2,10.1],[23.2,26.2],[8.8,26.2]],[.9,.2,.2,.9,.9]));
const sig=u=>[11+10*u,22.5-1.7*Math.sin(3*Math.PI*u)*(1-.2*u)];
const ink=dir(F([[0,0],[.06,0],[.62,1,'smooth']],[[0,0],[.04,0],[.3,1]]),F([[0,1],[.4,0,'smooth']],[[0,1],[.26,0]]));
const dip=dir(F([[0,0],[.62,0]],[[0,0],[.04,0],[.3,.5,'hit'],[.4,-.2],[.5,0]]),F([[0,0],[.4,0]],[[0,0],[.26,0]]));
const contract=toggle('contract','Contract',
 'Light: a signature is drawn across the signature line of the sheet. Strong: it is dashed off in 0.26 s and the sheet dips under the pen. The signed contract is retained; the reverse erases it.',
 posed([...framed('sheet',sheetD,sheetS),stroke('fold','M19.5 4.5 L19.5 9.5 L24.5 9.5'),
  ...tube('line-0','M11 13 L21 13'),...tube('line-1','M11 16.5 L17 16.5'),
  ...shown(tube('signature',(t,s,r)=>curveFrom(u=>sig(u*Math.max(.001,ink(t,s,r))),6)),(t,s,r)=>ink(t,s,r)>.02?1:0)],
  rigid((t,s,r)=>({dy:dip(t,s,r)}))),
 .62,.5,'unsigned','signed',.4,.26,{forward:'Sign',reverse:'Erase'});
contract.duotoneFill=true;

// ---------- RUBBER STAMP ----------
// Knob, neck and base press 3.2 units onto the paper and lift; the impression appears at contact and
// is retained (preview reset rearms). Strong lifts first, slams down with a hold and the mark pops.
const press=F([[0,0],[.05,0],[.24,3.2,'in'],[.3,3.2],[.56,0,'smooth']],[[0,0],[.1,-1.5],[.22,3.2,'hit'],[.3,3.2,'hold'],[.5,-.3],[.6,0]]);
const inked=F([[0,0],[.2,0],[.24,1,'out']],[[0,0],[.21,0],[.24,1,'out']]);
const pop=F([[0,1],[.56,1]],[[0,1],[.22,1],[.25,1.12],[.38,1]]);
const markD=rect(10,23.2,12,4.2,1.5);
const rubberStamp=make('rubber-stamp','Rubber stamp',
 'Light: the stamp presses 3.2 units onto the paper and lifts, leaving its mark. Strong: it lifts, slams down with a hold and the mark pops in under it. The mark is retained; preview reset rearms.',[
 ...posed([...framed('knob',rect(12.5,5.5,7,4,2)),...tube('neck','M16 9.5 L16 13.5'),...framed('base',rect(7,13.5,18,4.5,1.6))],rigid((t,s)=>({dy:press(t,s)}))),
 ...shown(posed(framed('mark',markD),(x,y,t,s)=>{const k=pop(t,s);return [16+(x-16)*k,25.3+(y-25.3)*k];}),(t,s)=>inked(t,s))
],.56,.6,['mark']);
rubberStamp.duotoneFill=true;

// ---------- OFFICIAL SEAL ----------
// A scalloped rosette (r 8.2 ± 0.85, ten scallops) with an inner ring and two ribbon tails attached
// at its lower edge. Light rocks 5°; Strong presses (scale 0.94) and rocks 12° while the tails whip.
const SC=[16,13];
const rosetteD=curveFrom(u=>{const a=u*2*Math.PI,r=8.2+.85*Math.cos(10*a);return [SC[0]+r*Math.cos(a),SC[1]+r*Math.sin(a)];},40)+' Z';
const rock=F([[0,0],[.28,5,'out'],[.62,0,'smooth']],[[0,0],[.08,-4],[.26,12],[.44,-5],[.6,2],[.72,0]]);
const squeeze=F([[0,1],[.62,1]],[[0,1],[.08,1.05],[.2,.94,'hit'],[.34,1.03],[.5,1]]);
const whip=F([[0,0],[.1,0],[.36,8,'out'],[.7,0,'smooth']],[[0,0],[.1,0],[.3,22],[.48,-14],[.64,6],[.8,-2],[.9,0]]);
const sealPose=(x,y,t,s)=>{const k=squeeze(t,s),[rx,ry]=rotate(x,y,rock(t,s),...SC);return [SC[0]+(rx-SC[0])*k,SC[1]+(ry-SC[1])*k];};
const tail=(name,a,b,sign)=>posed(tube(name,`M${a} L${b}`),(x,y,t,s)=>sealPose(...rotate(x,y,sign*whip(t,s),...a),t,s));
const officialSeal=make('official-seal','Official seal',
 'Light: the seal rocks 5° and its ribbons sway after it. Strong: it is pressed to 94%, rocks 12° and the ribbons whip 22° before settling.',[
 ...tail('tail-l',[13.2,21],[11,28],-1),...tail('tail-r',[18.8,21],[21,28],1),
 ...posed([fill('rosette-interior',rosetteD,'tint'),stroke('rosette-frame',rosetteD),fill('rosette-solid',holes(rosetteD,circle(...SC,5.6)),'solid'),
  stroke('ring',circle(...SC,4.6)),fill('core-solid',circle(...SC,3.6),'solid')],sealPose)
],.7,.9);
officialSeal.duotoneFill=true;

export const icons=[gavel,justiceScale,contract,rubberStamp,officialSeal];
