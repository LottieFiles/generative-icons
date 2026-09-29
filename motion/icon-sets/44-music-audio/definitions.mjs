// Music & audio (44). Designs, reviews and corrections: evidence/design-review.md.
import {icons as a} from './batches/music-a.mjs';
import {icons as b} from './batches/music-b.mjs';
import {icons as c} from './batches/music-c.mjs';
const all=new Map([...a,...b,...c].map(i=>[i.name,i]));
// Full-turn native transforms leave tiny raster differences at 360/720 degrees.
// Sample only these spinning parts into vector paths so their rounded start and
// end geometry is identical. Keep the authored pivots, travel and timing.
for(const [name,parts] of [
 ['vinyl-record',['platter-solid','groove-0','groove-1']],
 ['cassette-tape',['reel-0','reel-1']],
]){
 const icon=all.get(name);
 icon.parts=icon.parts.map(part=>{
  if(!parts.some(suffix=>part.name===name+'-'+suffix))return part;
  const pose=part.pose;
  return {...part,pose:(x,y,t,s,r)=>pose(x,y,t,s,r)};
 });
}
const ORDER=['music-note','vinyl-record','cassette-tape','audio-waveform','loudspeaker','radio','metronome','tuner','piano-keys','guitar','drum','lyrics','music-album','earbuds','mute-audio','crossfade','equalizer-preset','audio-jack','spatial-audio','tuning-fork'];
if(ORDER.length!==all.size||ORDER.some(n=>!all.has(n)))throw Error('Music & audio order and batches disagree');
const fixed={"vinyl-record":["platter","label","arm-pivot"],"cassette-tape":["shell-frame","window"],"radio":["body","scale"],"metronome":["body-frame"],"tuner":["dial","tick-0","tick-1","tick-2","hub"],"piano-keys":["keyboard-frame","divider-0","divider-1","divider-2"],"lyrics":["line-0","line-1","line-2"],"crossfade":["curve-a","curve-b","track"],"equalizer-preset":["track-0","track-1","track-2"],"audio-jack":["socket"],"spatial-audio":["head","shoulders"]};
const rigidParts={"music-note":["head","stem","flag"],"vinyl-record":["arm","groove-0","groove-1"],"cassette-tape":["reel-0","reel-1"],"metronome":["rod","weight"],"tuner":["needle"],"piano-keys":["black-0","black-1","black-2"],"guitar":["body","neck","head"],"drum":["shell","rim","stick-a","stick-b"],"music-album":["sleeve"],"earbuds":["bud-a","bud-a-stem","bud-b","bud-b-stem"],"mute-audio":["speaker"],"crossfade":["knob"],"equalizer-preset":["knob-0","knob-1","knob-2"],"audio-jack":["pin","body-frame","cable"],"tuning-fork":["stem","knob"]};
export const icons=ORDER.map(n=>all.get(n)).map((icon,index)=>({...icon,number:706+index,constraints:[...(fixed[icon.name]||[]).map(p=>({type:'fixed',part:icon.name+'-'+p})),...(rigidParts[icon.name]||[]).map(p=>({type:'rigid',part:icon.name+'-'+p}))]}));
