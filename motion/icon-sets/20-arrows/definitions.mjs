import{directions}from'./batches/directions.mjs';
import{chevrons}from'./batches/chevrons.mjs';
import{axisIcons}from'./batches/axes.mjs';
import{turnIcons}from'./batches/turns.mjs';
import{cornerIcons}from'./batches/corners.mjs';
const order=['arrow-right','arrow-left','arrow-up','arrow-down','arrow-up-right','arrow-up-left','arrow-down-right','arrow-down-left','chevron-right','chevron-left','chevron-up','chevron-down','arrows-horizontal','arrows-vertical','arrows-swap','arrow-turn-right','arrow-turn-left','arrow-u-turn','arrows-expand','arrows-collapse'];
const all=[...directions,...chevrons,...axisIcons,...turnIcons,...cornerIcons];
export const icons=order.map((name,i)=>({...all.find(v=>v.name===name),number:310+i}));
if(icons.some(i=>!i.parts))throw Error('Missing arrow definition');
