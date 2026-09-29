// Claude proposal-02. Source disks and target tips remain planted.
import {route,arrowhead,annotation} from './reveal.mjs';
import {driver,phase,flex,anchor} from './drivers.mjs';
const p=driver([[.08,0],[1.02,1,'smooth']],[[.04,0],[.16,.14],[.46,1]],[[.10,1],[.28,0,'smooth']]);
const h=phase(1.02,1.24,.46,.58,0,.10);
const f=flex([[.04,0],[.30,1],[.46,.55],[.66,-.25],[.86,.06],[1.02,0]]);
const swoop=down=>{
 const y=v=>down?32-v:v;
 // Slide the final control along the approach tangent during flex.
 const d=(t,s,r)=>{const k=f(t,s,r);return `M6 ${y(23)} C9.5 ${y(27.5+1.2*k)} ${17.47-.311111*k} ${y(26.07+.7*k)} 19.5 ${y(21.5)} L25.5 ${y(8)}`;};
 return annotation('annotation-swoop-'+(down?'down':'up'),'Swoop '+(down?'down':'up'),'The planted stroke sweeps through a broad bend, then finishes at the target.',[
  ...anchor('origin',6,y(23)),...route('route',d,p),...arrowhead('head',[25.5,y(8)],[26.28,y(12.94)],[21.31,y(10.72)],h)
 ],1.30,1.06,.30);
};
const lp=driver([[.08,0],[1.12,1,'smooth']],[[.04,0],[.16,.12],[.52,1]],[[.10,1],[.28,0,'smooth']]);
const lh=phase(1.12,1.34,.52,.64,0,.10);
const lf=flex([[.04,0],[.34,1],[.52,.5],[.72,-.22],[.92,.05],[1.08,0]]);
const loopPath=(t,s,r)=>{const k=lf(t,s,r);return `M17.65 23.94 L18.48 14.48 C${18.68+.21*k} ${12.13-.77*k} ${17.37+.31*k} ${9.92-.72*k} 15.22 8.97 C${13.06+.13*k} ${8.02-.79*k} ${10.54-.35*k} ${8.55-.71*k} 8.94 10.28 C7.35 12.02 7.04 14.58 8.18 16.65 C9.31 18.71 11.64 19.83 13.96 19.42 L26.95 17.13`;};
export const swoops=[swoop(false),swoop(true),annotation('annotation-loop','Curled pointer','One deliberate curl crosses its stem and writes toward a fixed target.',[
 ...anchor('origin',17.65,23.94),...route('route',loopPath,lp),...arrowhead('head',[26.95,17.13],[23.30,20.54],[22.35,15.18],lh)
],1.4,1.12,.30)];
