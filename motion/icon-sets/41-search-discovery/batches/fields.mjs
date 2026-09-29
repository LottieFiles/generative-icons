// Search & discovery batch B (fields, lists and tags): search-suggest, search-clear, sort-ascending,
// sort-descending, hashtag, tag-add, tag-remove, trending-up, trending-down.
import {make,toggle,K,FWD,ring,tube,posed,shown,rigid,circle,rect,polygon,bothFill,mix,clamp,drive,F} from './helpers.mjs';

// ---------- SEARCH SUGGEST ----------
// A search field on top; typing extends the stub and three suggestion lines drop in below it in order.
// Light fades them down 2 units; Strong drops them 3 with an overshoot. Reverse lifts and hides them.
const FIELD=rect(3.5,4.5,25,7,3.5);
const typeK=K([[0,0],[.3,1,'smooth']],[[0,0],[.18,1]],[[0,1],[.3,0,'smooth']],[[0,1],[.2,0]]);
const SUG=[[16.5,24],[21,19],[25.5,22]];
const sug=i=>K([[0,0],[.14+.08*i,0],[.4+.08*i,1,'out']],[[0,0],[.1+.06*i,0,'hold'],[.26+.06*i,1.15],[.38+.06*i,.95],[.48+.06*i,1]],[[0,1],[.06+.06*i,1],[.36+.06*i,0,'smooth']],[[0,1],[.04+.05*i,1,'hold'],[.26+.05*i,0]]);
const searchSuggest=toggle('search-suggest','Search suggest',
 'Light: typing extends the field stub and three suggestions fade down into place one after another. Strong: the stub types faster and the suggestions drop 3 units with an overshoot. Reverse lifts and hides them.',[
 ...ring('field',FIELD),
 ...tube('text',(t,s,r)=>`M8 8 L${11+3*typeK(t,s,r)} 8`),
 ...SUG.flatMap(([y,x2],i)=>{const q=sug(i);return posed(shown(tube('item-'+i,`M8 ${y} L${x2} ${y}`),(t,s,r)=>clamp(q(t,s,r))),rigid((t,s,r)=>({dy:(s?-3:-2)*(1-q(t,s,r))})));})
],.58,.62,'Closed','Suggested',.5,.4);
searchSuggest.duotoneFill=true;

// ---------- SEARCH CLEAR ----------
// A search field holding a text run and a clear cross. Pressing the cross wipes the text from the right.
// Strong presses harder, the field jolts and the text is gone in 0.16 s. Reverse types the text back.
const FIELD2=rect(3.5,10,25,12,6);
const clearK=K([[0,0],[.16,0],[.5,1,'smooth']],[[0,0],[.14,0,'hold'],[.3,1]],[[0,1],[.45,0,'smooth']],[[0,1],[.04,1,'hold'],[.34,0]]);
const pressX=(t,s,r)=>r?1:drive(t,s,[[0,1],[.08,.85],[.22,1,'smooth']],[[0,1],[.06,.75,'hit'],[.16,1.08],[.26,1]]);
const jolt=FWD([[0,0]],[[0,0],[.14,0],[.2,.8,'hit'],[.34,-.3],[.46,0]]);
const crossPose=(x,y,t,s,r)=>{const k=pressX(t,s,r);return [23.5+(x-23.5)*k+jolt(t,s,r),16+(y-16)*k];};
const searchClear=toggle('search-clear','Search clear',
 'Light: the clear cross presses and the text run wipes out of the field from the right. Strong: a hard press, the field jolts and the text vanishes in 0.16 s. Reverse types the text back in.',[
 ...posed([...ring('field',FIELD2),...shown(tube('text',(t,s,r)=>`M8.5 16 L${8.5+8.5*(1-clearK(t,s,r))} 16`),(t,s,r)=>1-clearK(t,s,r)>.03?1:0)],rigid((t,s,r)=>({dx:jolt(t,s,r)}))),
 ...posed([...tube('cross-a','M21.8 14.3 L25.2 17.7'),...tube('cross-b','M25.2 14.3 L21.8 17.7')],crossPose)
],.52,.48,'Typed','Cleared',.46,.38);
searchClear.duotoneFill=true;

// ---------- SORT ASCENDING / DESCENDING ----------
// Three left-aligned bars reorganise into sorted lengths beside a direction arrow. Light morphs the
// lengths; Strong first retracts every bar to a 2-unit stub and then writes the sorted lengths in one
// stroke while the arrow hops. Reverse restores the unsorted lengths.
const ROWS=[10,16,22];
const barLen=(a,b,u,via)=>via?(u<.5?mix(a,2,u/.5):mix(2,b,(u-.5)/.5)):mix(a,b,u);
const sortK=K([[0,0],[.08,0],[.56,1,'smooth']],[[0,0],[.06,0,'hold'],[.26,.5,'pass'],[.5,1]],[[0,1],[.5,0,'smooth']],[[0,1],[.04,1,'hold'],[.28,.5,'pass'],[.52,0]]);
const arrowY=FWD([[0,0],[.1,0],[.3,-.6],[.55,0,'smooth']],[[0,0],[.22,0],[.34,-1.6],[.5,0]]);
const sortIcon=(name,label,up,init,final)=>{
 const icon=toggle(name,label,
  `Light: the three bars morph into ${up?'ascending':'descending'} lengths while the arrow nudges ${up?'up':'down'}. Strong: every bar retracts to a stub, then the sorted lengths are written in one fast stroke as the arrow hops. Reverse restores the unsorted lengths.`,[
  ...ROWS.flatMap((y,i)=>tube('bar-'+i,(t,s,r)=>`M5 ${y} L${5+barLen(init[i],final[i],sortK(t,s,r),s)} ${y}`)),
  ...posed([...tube('shaft','M25 7.5 L25 24.5'),...tube('head',up?'M21.5 11 L25 7.5 L28.5 11':'M21.5 21 L25 24.5 L28.5 21')],rigid((t,s,r)=>({dy:(up?1:-1)*arrowY(t,s,r)})))
 ],.58,.52,'Unsorted','Sorted',.52,.54);
 icon.duotoneFill=false;return icon;
};
const sortAscending=sortIcon('sort-ascending','Sort ascending',true,[13,4,8.5],[4,8.5,13]);
const sortDescending=sortIcon('sort-descending','Sort descending',false,[4,13,8.5],[13,8.5,4]);

// ---------- HASHTAG ----------
// A # of two posts and two bars. Light nudges the bars right 1 unit and back. Strong: the posts jump
// 2.4 up in turn and land with a hit while the bars kick right 2 and rebound.
const barX=[0,1].map(i=>F([[0,0],[.06,0],[.26,1],[.5,0,'smooth']],[[0,0],[.1+.06*i,0],[.26+.06*i,2],[.42+.06*i,0,'hit'],[.52+.06*i,-.35],[.62+.06*i,0]]));
const postY=[0,1].map(i=>F([[0,0],[.5,0]],[[0,0],[.05+.06*i,.4],[.2+.06*i,-2.4],[.36+.06*i,0,'hit'],[.46+.06*i,-.3],[.56+.06*i,0]]));
const hashtag=make('hashtag','Hashtag',
 'Light: the two bars of the hash nudge right 1 unit and settle back. Strong: the posts hop 2.4 units in turn and land with a hit while the bars kick right 2 units and rebound.',[
 ...[12,20].flatMap((x,i)=>posed(tube('post-'+i,`M${x} 6 L${x} 26`),rigid((t,s)=>({dy:postY[i](t,s)})))),
 ...[12,20].flatMap((y,i)=>posed(tube('bar-'+i,`M6 ${y} L26 ${y}`),rigid((t,s)=>({dx:barX[i](t,s)}))))
],.52,.7);
hashtag.duotoneFill=false;

// ---------- TAG ADD / TAG REMOVE ----------
// A label tag (point left, eyelet dot) whose body carries the action mark. Add: a plus grows from its
// centre while the tag hops. Remove: a cross grows while the tag droops 8° about its eyelet.
const TAG=polygon([[4.5,16],[11,8.5],[27.5,8.5],[27.5,23.5],[11,23.5]],[1.2,1.6,2.4,2.4,1.6]);
const tagParts=()=>[...ring('tag',TAG),...bothFill('eyelet',circle(11,16,1.2))];
const plusK=K([[0,0],[.16,0],[.42,1,'out']],[[0,0],[.2,0,'hold'],[.34,1.12],[.46,1]],[[0,1],[.36,0,'smooth']],[[0,1],[.04,1,'hold'],[.28,0]]);
const hop=FWD([[0,0],[.06,0],[.2,-.8],[.36,0,'smooth']],[[0,0],[.06,.3],[.22,-1.8],[.4,0,'hit'],[.5,-.25],[.6,0]]);
const tagAdd=toggle('tag-add','Tag add',
 'Light: the tag hops 0.8 and a plus grows in its body. Strong: it crouches, hops 1.8, lands with a hit and the plus snaps in past full size. Reverse shrinks the plus away.',[
 ...posed([...tagParts(),...shown([...tube('plus-h',(t,s,r)=>{const g=2.6*plusK(t,s,r);return `M${20-g} 16 L${20+g} 16`;}),...tube('plus-v',(t,s,r)=>{const g=2.6*plusK(t,s,r);return `M20 ${16-g} L20 ${16+g}`;})],(t,s,r)=>plusK(t,s,r)>.02?1:0)],rigid((t,s,r)=>({dy:hop(t,s,r)})))
],.44,.62,'Untagged','Tagged',.38,.3);
tagAdd.duotoneFill=true;
const droopK=K([[0,0],[.1,0],[.5,1,'smooth']],[[0,0],[.08,-.15],[.3,1.35],[.44,.85],[.56,1]],[[0,1],[.45,0,'smooth']],[[0,1],[.06,1,'hold'],[.3,-.12],[.44,0]]);
const crossK=K([[0,0],[.1,0],[.4,1,'out']],[[0,0],[.06,0,'hold'],[.2,1.1],[.3,1]],[[0,1],[.3,0,'smooth']],[[0,1],[.2,0]]);
const tagRemove=toggle('tag-remove','Tag remove',
 'Light: a cross grows in the tag while it droops 8° from its eyelet. Strong: the tag lifts, drops to 11° with a rebound and the cross snaps in. Reverse erases the cross and lifts the tag level.',[
 ...posed([...tagParts(),...shown([...tube('cross-a',(t,s,r)=>{const g=2.2*crossK(t,s,r);return `M${20-g} ${16-g} L${20+g} ${16+g}`;}),...tube('cross-b',(t,s,r)=>{const g=2.2*crossK(t,s,r);return `M${20+g} ${16-g} L${20-g} ${16+g}`;})],(t,s,r)=>crossK(t,s,r)>.02?1:0)],rigid((t,s,r)=>({angle:8*droopK(t,s,r),pivot:[11,16]})))
],.52,.58,'Kept','Removed',.46,.46);
tagRemove.duotoneFill=true;

// ---------- TRENDING UP / DOWN ----------
// A flat arrow line rises (or falls) into a trend: each point moves monotonically to its final height
// and the head follows the last segment. Strong staggers the points so the tip leads, and the head kicks.
const trendIcon=(name,label,up,flat,finals)=>{
 const tk=K([[0,0],[.08,0],[.6,1,'smooth']],[[0,0],[.08,0,'hold'],[.36,1]],[[0,1],[.5,0,'smooth']],[[0,1],[.04,1,'hold'],[.32,0]]);
 const delays=[0,.06,.03,0];
 const kAt=(i,t,s,r)=>tk(t-(s?delays[i]:0),s,r);
 const pts=(t,s,r)=>[[4.5,flat],[11.5,mix(flat,finals[0],kAt(1,t,s,r))],[16.5,mix(flat,finals[1],kAt(2,t,s,r))],[26.5,mix(flat,finals[2],kAt(3,t,s,r))]];
 const kick=FWD([[0,0]],[[0,0],[.28,0],[.38,up?-9:9],[.52,0]]);
 const headD=(t,s,r)=>{const p=pts(t,s,r),a=p[2],b=p[3],th=Math.atan2(b[1]-a[1],b[0]-a[0])+kick(t,s,r)*Math.PI/180,e=d=>[b[0]+4.5*Math.cos(th+d),b[1]+4.5*Math.sin(th+d)];return `M${e(2.356194490192345)} L${b} L${e(-2.356194490192345)}`;};
 const lineD=(t,s,r)=>'M'+pts(t,s,r).map(p=>p.join(' ')).join(' L');
 const icon=toggle(name,label,
  `Light: the flat arrow line ${up?'rises':'falls'} into a trend, its head turning to follow the last segment. Strong: the tip leads the other points in a fast 0.28 s move and the head kicks past its heading before settling. Reverse flattens the line.`,
  [...tube('line',lineD),...tube('head',headD)],.62,.54,'Flat',up?'Rising':'Falling',.52,.4);
 icon.duotoneFill=false;return icon;
};
const trendingUp=trendIcon('trending-up','Trending up',true,24,[16,20,9.5]);
const trendingDown=trendIcon('trending-down','Trending down',false,8,[16,12,22.5]);

export const icons=[searchSuggest,searchClear,sortAscending,sortDescending,hashtag,tagAdd,tagRemove,trendingUp,trendingDown];
