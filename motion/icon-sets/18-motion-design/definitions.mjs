import{accepted}from'./batches/accepted.mjs';
import{keyframing}from'./batches/keyframing.mjs';
import{curves}from'./batches/curves.mjs';
import{layers}from'./batches/layers.mjs';
import{compositing}from'./batches/compositing.mjs';
const map={...accepted,...keyframing,...curves,...layers,...compositing};
const order=['timeline','add-keyframe','remove-keyframe','auto-keyframe','keyframe-easing','graph-editor','bezier-handles','motion-path','anchor-point','layer-stack','parent-layers','precompose','trim-layer','split-layer','time-stretch','onion-skin','shape-morph','mask-path','track-matte','render-queue'];
export const icons=order.map((name,i)=>({...map[name],number:270+i}));
if(icons.some(i=>!i.parts))throw Error('Missing motion design definition');
