import{icons as a}from'./batches/01.mjs';
import{icons as b}from'./batches/02.mjs';
import{icons as c}from'./batches/03.mjs';
import{icons as d}from'./batches/04.mjs';
export const icons=[...a,...b,...c,...d].map((icon,i)=>({...icon,number:160+i,...(['pinwheel','bead-relay'].includes(icon.name)?{timings:Object.fromEntries(Object.entries(icon.timings).map(([profile,timing])=>[profile,{...timing,keyPoses:[0,.13,.37,.71,1.19,1.83]}]))}:{})}));
