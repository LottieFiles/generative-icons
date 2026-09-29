import {icons as a} from './batches/access.mjs';
import {icons as b} from './batches/reading.mjs';
const fixed={"accessibility": ["ring-frame", "head", "torso", "leg"], "hearing-assistance": ["helix", "fold"], "high-contrast": ["ring-frame"], "reduced-motion": ["ball-frame"], "text-size": ["rule"], "focus-indicator": ["item-a", "item-b"], "reading-mode": ["page", "title"]};
const rigid={"accessibility": ["arm-left", "arm-right"], "wheelchair-access": ["head", "body", "wheel-frame"], "hearing-assistance": ["arc-small", "arc-large"], "high-contrast": ["half"], "keyboard": ["body", "key1", "key2", "key3", "space"], "keyboard-shortcuts": ["mod-frame", "mod-bar", "key-frame", "key-mark"], "focus-indicator": ["bracket-l", "bracket-r"]};
export const icons=[...a,...b].map((icon,index)=>({...icon,number:476+index,constraints:[...(fixed[icon.name]||[]).map(p=>({type:'fixed',part:icon.name+'-'+p})),...(rigid[icon.name]||[]).map(p=>({type:'rigid',part:icon.name+'-'+p}))]}));
