// Claude proposal-04 + revision-connectors: strict parent/child draw order.
import {route,annotation} from './reveal.mjs';
import {driver,phase,flex,anchor} from './drivers.mjs';
const st=phase(0,.22,0,.12,.20,.30),sb=phase(.22,.78,.12,.46,.08,.20),sh=phase(.78,.96,.46,.58,0,.08);
const sf=flex([[.12,0],[.46,1.6,'pass'],[.72,-.5],[.92,0]]);
const branch=down=>(t,s,r)=>{const x=15+sf(t,s,r),y=down?22.5:9.5;return `M12 16 C${x} 16 ${x} ${y} 18.5 ${y} L26 ${y}`;};
const split=annotation('annotation-split','Split connector','One source reaches its junction before two branches write to separate targets.',[
 ...anchor('origin',6,16,1.6),...route('trunk','M6 16 L12 16',st),...route('branch-up',branch(false),sb),...route('branch-down',branch(true),sb),
 ...route('head-up','M26 9.5 L23 6.5 L26 9.5 L23 12.5',sh),...route('head-down','M26 22.5 L23 25.5 L26 22.5 L23 19.5',sh)
],1,.96,.30);
const mu=phase(0,.62,0,.34,.18,.30),md=phase(.06,.68,.05,.39,.18,.30),mt=phase(.68,.84,.39,.50,.08,.18),mh=phase(.84,1,.50,.62,0,.08);
const mf=flex([[.30,0],[.50,-1.4,'pass'],[.74,.4],[.92,0]]);
const inbound=down=>(t,s,r)=>{const f=mf(t,s,r),y=down?22.5:9.5;return `M6 ${y} L13.5 ${y} C17 ${y} 17 ${16+(down?-f:f)} 20 16`;};
const merge=annotation('annotation-merge','Merge connector','Two incoming strokes meet before the shared route reaches its target.',[
 ...anchor('origin-up',6,9.5,1.6),...anchor('origin-down',6,22.5,1.6),...route('branch-up',inbound(false),mu),...route('branch-down',inbound(true),md),
 ...route('trunk','M20 16 L26 16',mt),...route('head','M26 16 L23 13 L26 16 L23 19',mh)
],1.04,.96,.30);
export const connectors=[split,merge];
