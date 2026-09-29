// Reward objects: facet joins, a retained open chest, a quest route, a seal and a rating.
import {make,toggle,F,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,bothFill,holes,starPts,rotate} from './helpers.mjs';
const dir=(forward,reverse)=>(t,s,r)=>(r?reverse:forward)(t,s);

// GEM: a rigid cut stone rocks about its lower point; facet edges meet the rim.
// Solid has an inset triangular counter, not primary strokes painted on primary fill.
const stone=polygon([[9,8],[23,8],[27,14],[16,26],[5,14]],.65);
const facet=polygon([[12.2,15.4],[19.8,15.4],[16,22]],.4);
const gemAngle=F([[0,0],[.2,6],[.52,0]],[[0,0],[.08,-5],[.26,13],[.43,-5],[.6,0]]);
const gemPose=rigid((t,s)=>({angle:gemAngle(t,s),pivot:[16,24]}));
const gem=make('gem','Gem',
 'Light: the cut gem rocks 6 degrees around its lower point. Strong: a short back-rock releases a 13-degree tilt and one damped response; facets stay joined to the rim.',
 posed([...framed('stone',stone,holes(stone,facet)),stroke('facet-cross','M5.3 14 L26.7 14'),stroke('facet-v','M10.7 8 L16 25.5 L21.3 8')],gemPose),.52,.6);

// TREASURE CHEST: the lid turns around the left rim corner. Its lower-left anchor
// is shared with the complete fixed body rim, so the open drawing exposes no loose ends.
const body=rect(7,18,18,9,1.5),lid='M7 18 L7 15 C7 11.7 10 10 13 10 L19 10 C22 10 25 11.7 25 15 L25 18 Z';
// Solid has a complete flat rim with a small physical overlap under the lid.
// Rounded top corners would leave a notch at the closed hinge.
const bodySolid='M6.85 17.8 L25.15 17.8 L25.15 25.5 C25.15 26.4 24.4 27.1 23.5 27.1 L8.5 27.1 C7.6 27.1 6.85 26.4 6.85 25.5 Z';
const lidAngle=dir(F([[0,0],[.06,0],[.62,-23]],[[0,0],[.06,3],[.32,-27],[.48,-23]]),F([[0,-23],[.06,-23],[.58,0]],[[0,-23],[.04,-25],[.32,0,'hit'],[.42,-2],[.52,0]]));
const lidPose=rigid((t,s,r)=>({angle:lidAngle(t,s,r),pivot:[7,18]}));
const treasureChest=toggle('treasure-chest','Treasure chest',
 'Open: the lid pivots 23 degrees at its left hinge and exposes the full rim. Strong: a brief latch press leads into a 27-degree swing and settles open. Close: a separate return seats the lid; Strong has a small contact rebound.',[
 ...framed('body',body,holes(bodySolid,rect(14.5,20.1,3,4.8,.6))),
 fill('latch',rect(14.8,20.2,2.4,4.5,.5),'outline'),
 ...posed([...framed('lid',lid),stroke('lid-band','M11 11 L11 18'),stroke('lid-band-r','M21 11 L21 18')],lidPose)
 ],.62,.48,'closed','open',.58,.52);

// QUEST: a folded map keeps a clear interior corridor for the travelling marker.
// The marker travels from the lower-left start to the upper-right destination.
const map=polygon([[5,8],[12,5],[20,8],[27,5],[27,24],[20,27],[12,24],[5,27]],.7);
const route=dir(F([[0,0],[.04,0],[.78,1]],[[0,0],[.04,0,'hold'],[.42,1]]),F([[0,1],[.06,1],[.76,0]],[[0,1],[.04,1],[.46,0]]));
const routePoint=p=>{const x=9.5+12.5*p,y=20-7.5*p-2*Math.sin(Math.PI*p);return [x,y];};
const mapPose=rigid((t,s,r)=>({angle:s?F([[0,0]],[[0,0],[.08,r?2:-2],[.24,r?-3:3],[.48,0]])(t,s):0,pivot:[16,16]}));
const markerPose=(x,y,t,s,r)=>{const q=routePoint(route(t,s,r));return mapPose(x+q[0]-9.5,y+q[1]-20,t,s,r);};
const quest=toggle('quest','Quest',
 'Light: a marker crosses the folded quest map from the lower-left start to the upper-right goal. Strong: the map reacts to launch with a short 3-degree rock while the marker crosses sooner. Reverse: the marker returns along the same route.',[
 ...posed([fill('map-interior',map,'tint'),stroke('map-frame',map),{name:'map-solid',d:map,type:'stroke',style:'solid',width:2.6},
 ...tube('fold-top','M12 5.5 L12 10'),...tube('fold-bottom','M20 22 L20 26.5')],mapPose),
 ...posed(bothFill('marker',circle(9.5,20,1.8)),markerPose)
 ],.78,.48,'started','reached',.76,.48);

// ACHIEVEMENT: a rosette and its attached tails are stamped down as one assembly.
const seal=polygon(Array.from({length:24},(_,i)=>{const a=(-90+i*15)*Math.PI/180,r=i%2?7:7.8;return [16+r*Math.cos(a),13+r*Math.sin(a)];}),.6);
const check='M12.9 12.8 L15.1 15 L18.8 11.5';
const tailL='M11.1 18 L7.6 25.2 L10 24.5 L12.4 27 L14.2 19.8';
const tailR='M20.9 18 L24.4 25.2 L22 24.5 L19.6 27 L17.8 19.8';
const ribbon=(name,d)=>[fill(name+'-interior',d+' Z','tint'),stroke(name,d),fill(name+'-solid',d+' Z','solid')];
const stampY=F([[0,0],[.16,-1.3],[.36,0,'in'],[.55,0]],[[0,0],[.08,-.3],[.22,-3],[.38,0,'hit'],[.47,-.65],[.6,0]]);
const stampA=F([[0,0],[.55,0]],[[0,0],[.22,-7],[.38,0,'hit'],[.6,0]]);
const achievement=make('achievement','Achievement',
 'Light: the achievement seal lifts 1.3 units and stamps down. Strong: it lifts 3 units with a 7-degree tilt, strikes squarely and rebounds once. The check and ribbon tails stay attached to the seal.',
 posed([...ribbon('tail-left',tailL),...ribbon('tail-right',tailR),
 ...framed('seal',seal,holes(seal,circle(16,13,5.5))),...tube('check',check)],rigid((t,s)=>({dy:stampY(t,s),angle:stampA(t,s),pivot:[16,23]}))),.55,.6);

// STAR RATING: the star acknowledges a new rating; three dots retain one/three.
// A complete star stays visible even while the newly selected dots are absent.
const ratingAngle=dir(F([[0,0],[.22,5],[.6,0]],[[0,0],[.08,-5],[.24,16],[.42,-4],[.6,0]]),F([[0,0],[.18,-5],[.5,0]],[[0,0],[.14,-12],[.3,3],[.5,0]]));
const rated=i=>dir(F([[0,0],[.08+i*.13,0],[.4+i*.13,1]],[[0,0],[.06+i*.09,0],[.24+i*.09,1]]),F([[0,1],[.06+(2-i)*.12,1],[.34+(2-i)*.12,0]],[[0,1],[.04+(2-i)*.07,1],[.23+(2-i)*.07,0]]));
const ratingStar=polygon(starPts(16,13.5,9,.47),.65);
const starRating=toggle('star-rating','Star rating',
 'Select: the star rocks and the second and third rating dots fill in sequence. Light uses a 5-degree acknowledgement. Strong winds back and turns 16 degrees before settling. Reverse clears the dots in the opposite order and rocks left.',[
 ...posed(framed('star',ratingStar),rigid((t,s,r)=>({angle:ratingAngle(t,s,r),pivot:[16,13.5]}))),
 ...bothFill('dot-0',circle(9,26,1.6)),
 ...[1,2].flatMap(i=>shown(bothFill('dot-'+i,circle(9+7*i,26,1.6)),rated(i)))
 ],.66,.6,'one-star','three-stars',.58,.5);

export const icons=[gem,treasureChest,quest,achievement,starRating].map(icon=>({...icon,duotoneFill:true}));
