// Claude revision accepted for native integration; render gate remains pending.
import { circle, rect, polygon, capsule, hole, holes, fill, stroke, line, rigid, framed, bothFill, tube, rotate, arc } from '../../library/drawing.mjs';
import { drive as motion, timePair, resultAction, has } from '../../library/direction.mjs';

const P=(light,strong)=>(t,s)=>motion(t,s,light,strong);
const hasKey=(...k)=>p=>has(p,...k);
const rad=d=>d*Math.PI/180;

/* ---------- image-generate ---------- */
const genPeak=P([[0,0],[.06,0],[.48,1,'out']],[[0,0],[.05,0],[.24,1.08],[.34,.97],[.42,1]]);
const genSun=P([[0,0],[.22,0],[.62,1,'out']],[[0,0],[.3,0],[.48,1]]);
const genSunVis=(t,s)=>Math.min(1,genSun(t,s)*4);
const genCanvas=rect(6.5,5.5,19,16,2);
const imageGenerate={
 name:'image-generate',label:'Image generate',heading:'Picture painted onto a blank easel canvas',
 intent:'A prompt becomes a picture: the easel stays rigid while a hill rises from the horizon line and a sun dawns beside it.',
 description:'A prompt becomes a picture: the easel stays rigid while a hill rises from the horizon line and a sun dawns beside it.',
 parts:[
  ...framed('image-generate-canvas',genCanvas,holes(genCanvas,rect(9.1,8.1,13.8,10.8,1))),
  ...line('image-generate-mast',[16,4.8],[16,5.5]),
  ...line('image-generate-leg-left',[9.8,21.5],[6.8,26.8]),
  ...line('image-generate-leg-right',[22.2,21.5],[25.2,26.8]),
  stroke('image-generate-mountain',(t,s)=>polygon([[10.2,17.6],[13.7,17.6-6.8*genPeak(t,s)],[17.2,17.6]],[1,1.2,1])),
  fill('image-generate-mountain-solid',(t,s)=>polygon([[10.2,19.6],[13.7,19.6-9.2*genPeak(t,s)],[17.2,19.6]],[1,1.2,1])),
  stroke('image-generate-sun',(t,s)=>{const p=genSun(t,s);return circle(20,14-2.8*p,.3+1.3*p);},{visibility:genSunVis}),
  fill('image-generate-sun-solid',(t,s)=>{const p=genSun(t,s);return circle(20,14-2.8*p,.3+1.6*p);},'solid',{visibility:genSunVis}),
 ],
 timings:timePair(.66,.7,[0,.06,.28,.48,.62,.66],[0,.05,.24,.34,.42,.48,.7]),
 brief:'Light: hill apex rises 17.6->10.8 (.06-.48); sun dawns from (20,14) r.3 to (20,11.2) r1.6 (.22-.62); hold to .66. Strong: the hill grows in one continuous drive, followed by a firmer sun rise and bounded spatial settling. Authored curves define timing. Fixed: canvas, mast, legs. Moving: mountain (3-vertex polygon, fixed topology), sun (cy and radius). Contacts: leg/mast caps centred on the frame centreline (buried at all three weights); Solid hill base at 19.6 lies inside the ring band 18.9..21.5. Clearances (arithmetic): Bold sun to inner frame edge 1.4 right, 1.6 top; Bold sun stroke to right slope about 1.3; Bold hill to inner left edge 1.2, to bottom 1.4; Solid sun to hole 1.0, to slope about 3.6.'
};
resultAction(imageGenerate,hasKey('mountain','sun'));

/* ---------- generative-fill ---------- */
const gfPhoto=polygon([[5.5,5.5],[25.5,5.5],[25.5,15.5],[15.5,15.5],[15.5,25.5],[5.5,25.5]],[2.5,2.5,1,1.5,1,2.5]);
const gfTile=polygon([[15.5,15.5],[25.5,15.5],[25.5,25.5],[15.5,25.5]],[1.5,1,2.5,1]);
const gfTileSolid=polygon([[16.9,16.9],[25.5,16.9],[25.5,25.5],[16.9,25.5]],[1.2,1,2.5,1]);
const gfSeat=P([[0,1.8],[.05,1.8],[.5,0,'out']],[[0,3],[.05,3],[.16,3.4],[.5,0,'hit'],[.58,.35],[.68,0],[.74,.12],[.82,0]]);
const gfShow=P([[0,0],[.05,0],[.3,1,'out']],[[0,0],[.05,0],[.22,1]]);
const generativeFill={
 name:'generative-fill',label:'Generative fill',heading:'Missing quadrant of a photo is filled by a generated tile',
 intent:'A photo with a missing corner region receives a generated tile that slides in and seats flush, completing the picture.',
 description:'A photo with a missing corner region receives a generated tile that slides in and seats flush, completing the picture.',
 parts:[
  ...framed('generative-fill-photo',gfPhoto),
  ...framed('generative-fill-tile',gfTile,gfTileSolid,{pose:rigid((t,s)=>{const d=gfSeat(t,s);return{dx:d,dy:d};}),visibility:gfShow}),
 ],
 timings:timePair(.56,.92,[0,.05,.3,.5,.56],[0,.05,.16,.5,.58,.68,.74,.82,.92]),
 brief:'Light: tile fades in (.05-.3) while sliding diagonally from +1.8 to seat (.05-.5); hold. Strong: the tile prepares, travels continuously and seats positively against the shared seam. Authored curves define timing. No negative overshoot: the tile never crosses the L outline. Fixed: L photo. Moving: tile (rigid translation). Seam: the concave corner (15.5,15.5) r1.5 and the straight seam segments are built from the same two edges with the same insets, so Outline curves coincide exactly; at (25.5,15.5) and (15.5,25.5) both shapes use r1 on different edge pairs, meeting at the inset point and leaving a 0.25-unit outer dent that reads as a rounded T. Solid tile is inset 1.4. Bounds: Strong start outer edge 27.7, Bold painted 28.95, invisible at t=0.'
};
resultAction(generativeFill,hasKey('tile'));

/* ---------- background-remove ---------- */
function arcSegs(r,a,b,n,X){let s='',start=rad(a);for(let i=1;i<=n;i++){const end=rad(a+(b-a)*i/n),k=4/3*Math.tan((end-start)/4),u=[16+r*Math.cos(start),11.2+r*Math.sin(start)],v=[16+r*Math.cos(end),11.2+r*Math.sin(end)];s+=` C${X(u[0]-k*r*Math.sin(start))} ${u[1]+k*r*Math.cos(start)} ${X(v[0]+k*r*Math.sin(end))} ${v[1]-k*r*Math.cos(end)} ${X(v[0])} ${v[1]}`;start=end;}return s;}
// One-piece bust: head arc joined to shoulders through a neck S-curve. o = outward offset.
function bust(o){const r=3.2+o,sx=9.8-o,by=22.6+o,rb=1.6+o,Lx=16+r*Math.cos(rad(130)),Ly=11.2+r*Math.sin(rad(130)),Rx=32-Lx;
 return `M${Rx} ${Ly}`+arcSegs(r,50,-230,4,x=>x)+` C${Lx-.6} ${Ly+2} ${sx} 17.2 ${sx} 19.4 L${sx} ${by-rb} C${sx} ${by-.448*rb} ${sx+.448*rb} ${by} ${sx+rb} ${by} L${32-sx-rb} ${by} C${32-sx-.448*rb} ${by} ${32-sx} ${by-.448*rb} ${32-sx} ${by-rb} L${32-sx} 19.4 C${32-sx} 17.2 ${Rx+.6} ${Ly+2} ${Rx} ${Ly} Z`;}
// Half backdrop (sg -1 left, +1 right). Outline: open bracket ending at x=16. Solid/tint: closed contour
// bounded by the outer rect, the centre seam at 16.3 (0.3 overlap while opaque) and its half of the o=1.7 cutout.
function brHalf(sg,solid){const X=x=>sg<0?x:32-x,o=1.7,r=3.2+o,sx=9.8-o,by=22.6+o,rb=1.6+o,Lx=16+r*Math.cos(rad(130)),Ly=11.2+r*Math.sin(rad(130)),e=solid?16.3:16;
 let d=`M${X(e)} 4.5 L${X(8.5)} 4.5 C${X(7.12)} 4.5 ${X(6)} 5.62 ${X(6)} 7 L${X(6)} 25 C${X(6)} 26.38 ${X(7.12)} 27.5 ${X(8.5)} 27.5 L${X(e)} 27.5`;
 if(!solid)return d;
 return d+` L${X(e)} ${by} L${X(sx+rb)} ${by} C${X(sx+.448*rb)} ${by} ${X(sx)} ${by-.448*rb} ${X(sx)} ${by-rb} L${X(sx)} 19.4 C${X(sx)} 17.2 ${X(Lx-.6)} ${Ly+2} ${X(Lx)} ${Ly}`+arcSegs(r,130,-90,3,X)+` L${X(e)} ${11.2-r} Z`;}
const brSep=P([[0,0],[.1,0],[.62,2.4,'out']],[[0,0],[.05,0],[.18,-.3],[.56,3.1],[.66,2.8],[.74,2.9]]);
const brFade=P([[0,1],[.3,1],[.62,0,'in']],[[0,1],[.4,1],[.72,0]]);
const brSide=(sg,side)=>{const pose=rigid((t,s)=>({dx:sg*brSep(t,s)}));return[
 fill(`background-remove-backdrop-${side}-interior`,brHalf(sg,true),'tint',{pose,visibility:brFade}),
 stroke(`background-remove-backdrop-${side}`,brHalf(sg,false),{pose,visibility:brFade}),
 fill(`background-remove-backdrop-${side}-solid`,brHalf(sg,true),'solid',{pose,visibility:brFade})];};
const backgroundRemove={
 name:'background-remove',label:'Background remove',heading:'Backdrop splits and slides away from a cut-out subject',
 intent:'The portrait backdrop behind a cut-out subject splits at the centre, slides apart and fades; the subject stays in place as the retained result.',
 description:'The portrait backdrop behind a cut-out subject splits at the centre, slides apart and fades; the subject stays in place as the retained result.',
 parts:[
  ...brSide(-1,'left'),
  ...brSide(1,'right'),
  stroke('background-remove-subject',bust(0)),
  fill('background-remove-subject-solid',bust(0),'solid'),
 ],
 timings:timePair(.66,1,[0,.1,.3,.5,.62,.66],[0,.05,.18,.4,.56,.66,.74,1]),
 brief:'Light: halves slide apart 2.4 each (.1-.62) and fade (.3-.62); hold. Strong: the halves squeeze briefly, then separate continuously and fade after their overlapping joins clear. Authored curves define timing. Fixed: subject (bust(0): head r3.2 at (16,11.2), shoulders 9.8..22.2, base 22.6). Moving: two halves, horizontal rigid translation only, so the Outline bracket ends stay at y4.5 and y27.5 and the Solid cutout edge only moves away from the subject. Fade begins after each half has moved more than 1.25, when Bold caps and the 0.3 Solid overlap no longer coincide, so no partial-opacity stacking. Clearances (arithmetic): Solid subject to cutout 1.7; Bold subject stroke to Duotone tint 0.45; Bold head stroke to backdrop top stroke 1.0; halves extend to x3.2 while fading. Duotone tints only the two enclosed halves.'
};
resultAction(backgroundRemove,hasKey('backdrop'));

/* ---------- object-select ---------- */
const osC=13.4;
const osPtr=polygon([[0,0],[0,7.2],[2.1,5.4],[3.7,8.6],[5.3,7.8],[3.7,4.6],[6.4,4.6]].map(([x,y])=>{const [rx,ry]=rotate(x,y,-18,0,0);return[rx+16.16,ry+16.16];}),[.9,.9,.6,.8,.8,.6,.9]);
const osArrive=P([[0,1.6],[.32,0,'out']],[[0,1.6],[.03,1.6],[.14,1.95],[.36,-.28,'hit'],[.46,.12],[.54,-.04],[.62,0]]);
const osRing=P([[0,0],[.26,0],[.58,1,'out']],[[0,0],[.38,0],[.66,1.08],[.76,.97],[.84,1]]);
const osR=(t,s)=>5.6+1.6*osRing(t,s);
const osVis=(t,s)=>Math.min(1,osRing(t,s)*3);
const objectSelect={
 name:'object-select',label:'Object select',heading:'Pointer touches an object and a selection ring settles around it',
 intent:'A pointer arrives and touches an object; a three-quarter selection ring opens around the object toward the pointer and stays as the selection.',
 description:'A pointer arrives and touches an object; a three-quarter selection ring opens around the object toward the pointer and stays as the selection.',
 parts:[
  ...bothFill('object-select-object',circle(osC,osC,4.4)),
  ...tube('object-select-halo',(t,s)=>arc(osC,osC,osR(t,s),90,360),{visibility:osVis}),
  ...framed('object-select-pointer',osPtr,osPtr,{pose:rigid((t,s)=>{const d=osArrive(t,s);return{dx:d,dy:d};})}),
 ],
 timings:timePair(.62,.95,[0,.16,.32,.45,.58,.62],[0,.14,.36,.46,.54,.66,.76,.84,.95]),
 brief:'Light: pointer slides in 1.6 diagonally to contact (.0-.32); ring expands 5.6->7.2 and fades in (.26-.58); hold. Strong: pointer arrives (.02-.34), presses 0.4 along its axis into the disc (.34-.44), releases (.44-.6); ring expands 5.6->7.2 (.42-.74) with a direct out ease, no overshoot; hold. Fixed: disc r4.4 at (13.4,13.4). Moving: pointer (rigid translation along the 45-degree axis), ring (arc 90..360 degrees, 3 cubic segments, radius only). Contact: tip vertex 0.5 inside the disc edge, rounded apex about 0.08 inside, so the Outline stroke buries into the filled disc and the Duotone tint never shows over the disc (press keeps the tint overlap under 0.75). Ring gap is the lower-right quadrant: Bold clearance from ring caps to the pointer stroke about 1.5. Bounds: ring Bold painted min 4.95; pointer Bold painted max 26.9 at the Strong start.'
};
resultAction(objectSelect,hasKey('halo','pointer'));

/* ---------- image-upscale (unchanged, accepted) ---------- */
const upGrow=P([[0,9],[.04,9],[.52,17,'out']],[[0,9],[.10,9],[.18,11,'pass'],[.36,17]]);
const upNudge=P([[0,0]],[[0,0],[.035,0],[.10,-.5,'launch'],[.36,.7],[.45,-.2],[.56,0]]);
const upFrame=s=>rect(5,27-s,s,s,2.6);
const upRing=s=>holes(upFrame(s),rect(7.6,29.6-s,s-5.2,s-5.2,1.2));
const upArrowPose={pose:rigid((t,s)=>{const o=upGrow(t,s)-9+upNudge(t,s);return{dx:o,dy:-o};})};
const imageUpscale={
 name:'image-upscale',label:'Image upscale',heading:'Frame grows from a small original, led by a corner arrow',
 intent:'A small original stays anchored bottom-left while a larger frame grows from it toward the upper-right; a drag arrow outside the corner leads the growth.',
 description:'A small original stays anchored bottom-left while a larger frame grows from it toward the upper-right; a drag arrow outside the corner leads the growth.',
 parts:[
  fill('image-upscale-frame-interior',(t,s)=>upFrame(upGrow(t,s)),'tint'),
  stroke('image-upscale-frame',(t,s)=>upFrame(upGrow(t,s))),
  fill('image-upscale-frame-solid',(t,s)=>upRing(upGrow(t,s)),'solid'),
  stroke('image-upscale-original',rect(5,18,9,9,2.6)),
  fill('image-upscale-original-solid',rect(5,18,9,9,2.6),'solid'),
  ...tube('image-upscale-arrow','M16.25 15.75 L19.5 12.5 L16.5 12.5 L19.5 12.5 L19.5 15.5',upArrowPose),
 ],
 timings:timePair(.6,1,[0,.04,.2,.36,.52,.6],[0,.04,.16,.4,.62,.72,.8,1]),
 brief:'Light: frame grows 9->17 from the bottom-left origin (.04-.52); the rigid arrow translates with the corner, offset 2.25 diagonally outside it. Strong: the arrow takes up tension and leads one continuous frame expansion to the shared final size. Authored curves define timing. Fixed: original square. Moving: frame (size only, fixed topology), arrow (one stroked path, shaft + retraced head, rigid translation). Clearance: arrow base cap to frame corner arc 4.26 (Bold 1.76), 1.05 at the deepest pull-back; arrow never approaches the original. Solid: ring frame plus filled original, original edges inside the ring buried. Bold painted extents x 3.75..28.75, y 3.25..28.25.'
};
resultAction(imageUpscale,hasKey('frame','arrow'));

export const icons=[imageGenerate,generativeFill,backgroundRemove,objectSelect,imageUpscale];
