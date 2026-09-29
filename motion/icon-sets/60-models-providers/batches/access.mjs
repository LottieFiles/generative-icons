import {finish,well,F,setting,framed,tube,annulus,circle,rect,rigid,posed,bothFill} from './helpers.mjs';

// Credential bow + code brackets distinguish API authorization from a house key.
const keyTurn=F([[0,0],[.18,-9],[.5,0,'smooth']],[[0,0],[.07,4],[.23,-21],[.38,3],[.54,0]]);
const apiKey=finish('api-key','API key','Authorize an app: the credential turns in its bow. Light turns 9 degrees; Strong prepares 4 degrees, turns 21 and catches once. Code brackets remain fixed; the shaft and teeth stay rigid.',[
 ...posed([...framed('bow',circle(9,11,4.4),annulus(9,11,4.4,2.6)),...tube('shaft','M13.1 11 L26 11 L26 15'),...tube('tooth','M21 11 L21 14')],rigid((t,s)=>({angle:keyTurn(t,s),pivot:[9,11]}))),
 ...tube('code-left','M11 21 L8 24 L11 27'),...tube('code-right','M21 21 L24 24 L21 27')
],.5,.54);

// Docking has a complete socket, prongs and plug; contact is at x21.
const dock=setting([[0,0],[.06,0],[.65,5,'smooth']],[[0,0],[.08,-.8],[.34,5,'hit'],[.5,5]],[[0,5],[.06,5],[.65,0,'smooth']],[[0,5],[.12,5,'hold'],[.38,0],[.5,0]]);
const endpoint=finish('endpoint','Endpoint','Connect a configured endpoint: the plug travels 5 units into the fixed socket and holds. Strong draws back 0.8 before a decisive dock. Reverse withdraws the prongs; the complete socket remains visible.',[
 ...well('socket',21,7,6,18,2),
 ...posed([...well('plug',6,12,6,8,1.7),...tube('prong-top','M12 14 L16 14'),...tube('prong-bottom','M12 18 L16 18'),...tube('lead','M4 16 L6 16')],rigid((t,s,r)=>({dx:dock(t,s,r),pivot:[6,16]})))
],.65,.5,['disconnected','connected']);

const approach=F([[0,0],[.22,3],[.4,3],[.7,0,'smooth']],[[0,0],[.07,-.5],[.25,7,'hit'],[.36,6],[.48,7],[.74,0]]);
const rateLimit=finish('rate-limit','Rate limit','Throttle arriving requests: two packets approach a closed gate and wait, then withdraw. Light advances 3 units; Strong travels 7, stops hard and recoils one unit. Gate and foot stay fixed.',[
 ...tube('gate','M20 6 L20 22'),...framed('foot',rect(16.5,24,7,3,1.5)),
 ...posed([...framed('packet-a',rect(5,8,5,5,1.5)),...framed('packet-b',rect(5,17,5,5,1.5))],rigid((t,s)=>({dx:approach(t,s),pivot:[7.5,15]})))
],.7,.74);

const beadA=F([[0,0],[.3,3.5],[.65,0,'smooth']],[[0,0],[.08,0,'hold'],[.28,8],[.42,7.2],[.52,8],[.82,0]]);
const beadB=F([[0,0],[.16,0],[.46,-3.5],[.8,0,'smooth']],[[0,0],[.2,0,'hold'],[.4,-8],[.54,-7.2],[.64,-8],[.94,0]]);
const tokenCounter=finish('token-counter','Token counter','Count token groups on two abacus rails. Light slides each bead 3.5 units; Strong drives each 8 units with staggered contact. The full beads slide along the rails and return to the starting tally.',[
 ...tube('rail-top','M5 11 L27 11'),...tube('rail-bottom','M5 21 L27 21'),...tube('left-post','M5 6 L5 26'),...tube('right-post','M27 6 L27 26'),
 ...posed(framed('upper-bead',circle(10,11,2.8),annulus(10,11,2.8,2.6)),rigid((t,s)=>({dx:beadA(t,s),pivot:[10,11]}))),
 ...posed(framed('lower-bead',circle(22,21,2.8),annulus(22,21,2.8,2.6)),rigid((t,s)=>({dx:beadB(t,s),pivot:[22,21]})))
],.8,.94);

const deadline=F([[0,0],[.38,65],[.76,0,'smooth']],[[0,0],[.08,-12],[.34,150,'hit'],[.47,144],[.57,150],[.92,0]]);
const requestTimeout=finish('request-timeout','Request timeout','A request reaches its time boundary: the timer hand sweeps toward the fixed stop on the outgoing path. Light sweeps 65 degrees; Strong winds back then sweeps 150 degrees and catches before returning for the next request.',[
 ...framed('timer',circle(13,13,8),annulus(13,13,8,2.6)),...tube('hour','M13 13 L16 13'),
 ...posed(tube('hand','M13 13 L13 8.5'),rigid((t,s)=>({angle:deadline(t,s),pivot:[13,13]}))),
 ...tube('request-path','M13 21 L13 26 L23 26'),...tube('stop','M27 23 L27 28')
],.76,.92);
export const icons=[apiKey,endpoint,rateLimit,tokenCounter,requestTimeout];
