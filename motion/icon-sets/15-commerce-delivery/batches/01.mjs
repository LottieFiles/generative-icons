import {make,F as P,posed,rigid,rect,circle,polygon,fill,stroke,ink,surface,policy,capsule,arch,held,bidir,mix,track,holes} from './helpers.mjs';
import {flow} from '../../library/direction.mjs';
const hem=(t,s,r,i)=>r?(s?flow([[0,3],[.08+i*.025,3],[.4+i*.025,-1.8],[.72+i*.025,0]],t):track([[0,3],[.65,0,'smooth']],t)):s?flow([[0,0],[.05+i*.03,0],[.19+i*.03,-2,'launch'],[.44+i*.03,3.7],[.7+i*.03,2.55],[1.02+i*.03,3]],t):track([[0,0],[.03,0],[.7,3,'out']],t);
function scallops(t,s,r){let d='M6 12';for(let i=0;i<4;i++){let x=6+i*5,y=12+hem(t,s,r,i);d+=` C${x+1.5} ${y} ${x+3.5} ${y} ${x+5} 12`;}return d;}
const cloth=(t,s,r)=>`M6 8.5 L6 12 ${scallops(t,s,r).replace(/^M6 12/,'')} L26 8.5 Z`;
const facade='M6 18.5 L26 18.5 L26 26 C26 26.828 25.328 27.5 24.5 27.5 L7.5 27.5 C6.672 27.5 6 26.828 6 26 Z';
const storefront=policy(held(make('storefront','Storefront','Light unrolls the awning quietly. Strong pulls the cloth up first, releases a travelling wave, then settles the hem. The header and shop remain fixed.',[
 ...surface('header',rect(6,5.5,20,3,1.5)),fill('cloth-tint',cloth,'tint'),fill('cloth-solid',cloth,'solid'),...ink('hem',scallops),...ink('left-wall','M6 8.5 L6 18.5'),...ink('right-wall','M26 8.5 L26 18.5'),
 ...surface('facade',facade,[rect(13.5,22,5,4.2,1)]),stroke('door',arch(13.5,18.5,27.5,22,1))
],.78,1.33),'rolled','extended',.73,.88));storefront.constraints=[{type:'fixed',part:'storefront-header'},{type:'fixed',part:'storefront-facade'}];
const lift=bidir([[0,0],[.15,0],[.7,-1.2,'out']],[[0,0],[.08,0],[.34,-2.7],[.62,-.95],[1.05,-1.2]],[[0,-1.2],[.65,0,'smooth']],[[0,-1.2],[.32,.2,'in'],[.65,0,'smooth']]);
const taut=bidir([[0,0],[.03,0],[.3,1,'out']],[[0,0],[.03,0],[.3,1,'out']],[[0,1],[.65,0,'smooth']],[[0,1],[.75,0,'smooth']]);
const pinch=P([[0,0]],[[0,0],[.03,0],[.2,2.5],[.66,0]]);
const sway=P([[0,0]],[[0,0],[.1,0],[.36,-7],[.7,3.5],[1.15,0]]);
const bagD='M9 13 L23 13 C23.7 13 24.2 13.5 24.3 14.2 L25 25 C25.05 25.55 24.6 26 24 26 L8 26 C7.4 26 6.95 25.55 7 25 L7.7 14.2 C7.8 13.5 8.3 13 9 13 Z';
const shopping=policy(held(make('shopping-bag','Shopping bag','Light lifts the bag quietly. Strong gathers the handle, lifts higher and swings the rigid bag from the handle before settling at the same height.',posed([
 ...surface('body',bagD),...ink('handle',(t,s,r)=>{let a=taut(t,s,r),p=r?0:pinch(t,s);return `M11 13 C${11+p} ${8-.6*a} 13 ${6.8-.6*a} 16 ${6.8-.6*a} C19 ${6.8-.6*a} ${21-p} ${8-.6*a} 21 13`;})
],rigid((t,s,r)=>({dy:1+lift(t,s,r),angle:r?(s?flow([[0,0],[.26,3.5],[.65,0]],t):0):sway(t,s),pivot:[16,6.8]}))),.78,1.33),'resting','lifted',.73,.83));shopping.constraints=[{type:'rigid',part:'shopping-bag-body'}];
const bowH=bidir([[0,5.5],[.03,5.5],[.7,9,'smooth']],[[0,5.5],[.03,5.5],[.2,6.4,'pass'],[.46,9.5],[.76,8.5],[1.1,9]],[[0,9],[.65,5.5,'smooth']],[[0,9],[.08,9],[.42,5.1,'out'],[.72,5.5,'smooth']]);
const strapX=bidir([[0,0]],[[0,0],[.06,0],[.36,5.5],[.68,-2],[1.1,0]],[[0,0]],[[0,0],[.24,-3.5],[.72,0]]);
const basketD='M6.5 14 L8.5 25.5 C8.6 26.1 9 26.5 9.6 26.5 L22.4 26.5 C23 26.5 23.4 26.1 23.5 25.5 L25.5 14';
const basket=policy(held(make('basket','Basket','Light lowers the basket strap. Strong sweeps it sideways between fixed attachments, rebounds once and settles with its opening clear.',[
 ...ink('rim','M5.5 14 L26.5 14'),...ink('body',basketD),...ink('weave-left','M12 14 L12.8 26.5'),...ink('weave-right','M20 14 L19.2 26.5'),...ink('handle',(t,s,r)=>`M7.5 14 C${7.5+strapX(t,s,r)} ${bowH(t,s,r)} ${24.5+strapX(t,s,r)} ${bowH(t,s,r)} 24.5 14`)
],.78,1.33),'upright','lowered',.73,.83),false);basket.constraints=[{type:'fixed',part:'basket-body'},{type:'fixed',part:'basket-rim'}];
const tagD='M7 12 L22 12 C22.5 12 23 12.2 23.4 12.6 L26.2 16.4 C26.9 17.1 26.9 18.9 26.2 19.6 L23.4 23.4 C23 23.8 22.5 24 22 24 L7 24 C6.2 24 5.5 23.3 5.5 22.5 L5.5 13.5 C5.5 12.7 6.2 12 7 12 Z';
const tug=P([[0,0],[.03,0],[.35,-3,'out'],[.7,0,'smooth']],[[0,0],[.03,0],[.3,-15],[.62,3.5],[1.1,0]]);
const tx=P([[0,0],[.03,0],[.35,.5,'out'],[.7,0,'smooth']],[[0,0],[.03,0],[.3,1],[1.1,0]]);
const productTag=policy(make('product-tag','Product tag','Light nudges the price tag. Strong pulls through the eyelet with a 14-degree swing and a small return swing. The tag stays rigid.',posed([
 ...surface('tag',tagD,[circle(21,18,2.2)]),stroke('eyelet',circle(21,18,2.2))
],rigid((t,s)=>({angle:tug(t,s),dx:tx(t,s),pivot:[21,18]}))),.78,1.33));productTag.constraints=[{type:'rigid',part:'product-tag-tag'}];
const lead=bidir([[0,9.5],[.03,9.5],[.7,18.5,'smooth']],[[0,9.5],[.06,9.5],[.38,20.8],[.74,18.5]],[[0,18.5],[.65,9.5,'smooth']],[[0,18.5],[.08,18.5],[.38,8.2,'out'],[.7,9.5,'smooth']]);
const trail=bidir([[0,4.5],[.03,4.5],[.7,13.5,'smooth']],[[0,4.5],[.16,4.5],[.54,13.5]],[[0,13.5],[.65,4.5,'smooth']],[[0,13.5],[.12,13.5],[.38,4.5,'out'],[.7,4.5]]);
const press=P([[0,0]],[[0,0],[.34,0],[.46,1.1,'hit'],[.7,-.4],[1.05,0]]);
const variants=policy(held(make('product-variants','Product variants','Light slides the selection underline. Strong stretches its leading edge past the next swatch, catches up with its trailing edge and presses the chosen swatch.',[
 ...surface('swatch-one',rect(4.5,10,5,5,1.25)),...posed(surface('swatch-two',rect(13.5,10,5,5,1.25)),rigid((t,s,r)=>({dy:r?0:press(t,s)}))),...surface('swatch-three',rect(22.5,10,5,5,1.25)),...ink('indicator',(t,s,r)=>`M${trail(t,s,r)} 19.5 L${lead(t,s,r)} 19.5`)
],.78,1.33),'first','second',.73,.83));variants.constraints=[{type:'fixed',part:'product-variants-swatch-one'},{type:'rigid',part:'product-variants-swatch-two'}];
export const icons=[storefront,shopping,basket,productTag,variants];
