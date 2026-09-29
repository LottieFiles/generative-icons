// Photos & camera (43). Designs, reviews and corrections: evidence/design-review.md.
import {icons as a} from './batches/photo-a.mjs';
import {icons as b} from './batches/photo-b.mjs';
import {icons as c} from './batches/photo-c.mjs';
import {icons as d} from './batches/photo-d.mjs';
import svgpath from 'svgpath';
import {rigid} from './batches/helpers.mjs';
const all=new Map([...a,...b,...c,...d].map(i=>[i.name,i]));

// Set-local clearance corrections. Keep paint widths, timing, opacity and the
// Light/Strong endpoints; adjust only the geometry or excursion that crowds a gap.
const parts=(icon,pattern)=>all.get(icon).parts.filter(p=>pattern.test(p.name));
const mapPath=(part,change)=>{
 const source=part.d;
 part.d=typeof source==='function'?(t,s,r,w)=>change(source(t,s,r,w),t,s,r):change(source);
};
const scalePath=(d,cx,cy,sx,sy=sx)=>svgpath(d).translate(-cx,-cy).scale(sx,sy).translate(cx,cy).toString();
const trimTranslation=(part,change)=>{
 const transform=part.pose.transform;
 part.pose=rigid((t,s,r)=>change(transform(t,s,r),s,r));
};

// Flash: shorten only the upward ray's Strong overshoot above its held length.
for(const p of parts('flash-toggle',/-ray-1(?:-solid)?$/))mapPath(p,(d,t,s,r)=>{
 if(!s||r)return d;
 const [[,x,y],[,,endY]]=svgpath(d).abs().segments;
 const length=y-endY;
 return `M${x} ${y} L${x} ${y-(length>2?2+(length-2)*.45:length)}`;
});

// Switch camera: retain the half turn and reduce the filled heads by 1.5%.
for(const p of parts('switch-camera',/-head-[ab](?:-solid)?$/))mapPath(p,d=>scalePath(d,16,18.6,.985));

// Portrait: leave room for the full blur and the subject's Strong response.
for(const p of parts('portrait-mode',/-bokeh-[01](?:-solid)?$/)){
 const i=p.name.includes('bokeh-0')?0:1;
 mapPath(p,d=>scalePath(d,i?21.4:10.6,i?8.8:8.6,.8/1.4));
}
for(const p of parts('portrait-mode',/-(?:head|shoulders)(?:-solid|-solid-core)?$/)){
 const pose=p.pose;
 p.pose=(x,y,t,s,r)=>{const [px,py]=pose(x,y,t,s,r);return s?[x+(px-x)/3,y+(py-y)/3]:[px,py];};
}

// Burst and panorama: inset the landscape from the card's painted edges.
for(const p of parts('burst-mode',/-hills(?:-solid)?$/))mapPath(p,d=>scalePath(d,13.5,0,.94,1));
for(const p of parts('panorama',/-hills(?:-solid)?$/))mapPath(p,d=>scalePath(d,16,19.2,.93,.92));
for(const p of parts('panorama',/-sun(?:-solid)?$/))mapPath(p,d=>svgpath(scalePath(d,0,12.6,1/1.3)).translate(0,-.77).toString());

// Gallery: keep the row gap open during the staggered Strong hops.
for(const p of parts('photo-gallery',/-tile-/))trimTranslation(p,(tr,s)=>({...tr,dy:tr.dy*(s?.7:1)}));

// Album: inset the cover artwork before its horizontal projection narrows.
for(const p of parts('photo-album',/-hills(?:-solid)?$/))mapPath(p,d=>scalePath(d,23.4,0,.79,1));
for(const p of parts('photo-album',/-sun(?:-solid)?$/))mapPath(p,d=>svgpath(d).translate(-.8,0).toString());

// Edit: trim the leftward Strong kick; retain the pencil's complete stroke.
for(const p of parts('photo-edit',/-(?:card|hills|sun)(?:-|$)/))trimTranslation(p,(tr,s)=>({...tr,dx:s&&tr.dx<0?tr.dx*.45:tr.dx}));

// Flip: lower the peaks without moving the landscape's bottom edge.
for(const p of parts('flip-photo',/-hills(?:-solid)?$/))mapPath(p,d=>scalePath(d,0,20,1,.82));

// Brightness: move ray roots outward while preserving their moving tips.
for(const p of parts('brightness',/-ray-/))mapPath(p,d=>{
 const [[,x,y],[,ex,ey]]=svgpath(d).abs().segments;
 return `M${16+(x-16)*8.4/7.8} ${16+(y-16)*8.4/7.8} L${ex} ${ey}`;
});

// Red-eye: reserve space around the pupil and below the flashing sparkle.
for(const p of parts('red-eye-fix',/-pupil(?:-fill)?(?:-solid)?$/))mapPath(p,d=>scalePath(d,16,16,3.05/3.2));
for(const p of parts('red-eye-fix',/-sparkle(?:-solid)?$/)){
 const pose=p.pose;
 p.pose=(x,y,t,s,r)=>{const [px,py]=pose(x,y,t,s,r);return [px,py-.45];};
}

// Delete: shorten the slot bottoms and the card's travel above the closed lid.
// Its existing fade and reset remain unchanged.
for(const p of parts('photo-delete',/-slot-[lr](?:-solid)?$/))mapPath(p,d=>svgpath(d).translate(0,-19.8).scale(1,4.2/4.4).translate(0,19.8).toString());
for(const p of parts('photo-delete',/-(?:card|hills)(?:-|$)/))trimTranslation(p,tr=>({...tr,dy:tr.dy*.1}));
const ORDER=['flash-toggle','self-timer','switch-camera','tripod','aperture','portrait-mode','live-photo','burst-mode','panorama','exposure','photo-gallery','photo-album','photo-edit','rotate-photo','flip-photo','brightness','saturation','red-eye-fix','photo-delete','photo-favorite'];
if(ORDER.length!==all.size||ORDER.some(n=>!all.has(n)))throw Error('Photos & camera order and batches disagree');
const fixed={"switch-camera":["body-frame"],"aperture":["ring-frame"],"portrait-mode":["card-frame"],"live-photo":[],"panorama":[],"exposure":["frame","plus-h","plus-v","minus"],"photo-delete":[],"photo-favorite":["card-frame","sun"],"flip-photo":["axis-top","axis-bottom"],"brightness":[]};
const rigidParts={"switch-camera":["arrow-a","head-a"],"tripod":["head-frame","column","leg-l"],"live-photo":["dash-0"],"photo-edit":["pencil","ferrule"],"rotate-photo":["card-frame","arrow"],"photo-delete":["lid","card-frame"],"photo-gallery":["tile-0-frame","tile-3-frame"],"self-timer":["body-frame"]};
export const icons=ORDER.map(n=>all.get(n)).map((icon,index)=>({...icon,number:686+index,constraints:[...(fixed[icon.name]||[]).map(p=>({type:'fixed',part:icon.name+'-'+p})),...(rigidParts[icon.name]||[]).map(p=>({type:'rigid',part:icon.name+'-'+p}))]}));
