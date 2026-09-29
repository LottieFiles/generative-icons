import{accepted}from'./batches/accepted.mjs';
import{transport}from'./batches/transport.mjs';
import{broadcast}from'./batches/broadcast.mjs';
import{display}from'./batches/display.mjs';
const all={...accepted,...transport,...broadcast,...display};
const order=['play-pause','stop','next-track','previous-track','replay-10','forward-10','shuffle','repeat','playlist','add-to-queue','headphones','podcast','live-broadcast','cast','screen-share','video-call','closed-captions','fullscreen','picture-in-picture','camera-off'];
export const icons=order.map((name,i)=>({...all[name],number:250+i,revision:'media-claude-reviewed-v1'}));
if(icons.length!==20||icons.some(i=>!i.name)||new Set(icons.map(i=>i.name)).size!==20)throw Error('Media scope must contain twenty distinct icons');
