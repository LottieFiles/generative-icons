import {icons as compute} from './compute-integrated-03.mjs';
import {icons as hard} from './hard-drive-03.mjs';
import {icons as network} from './network-devices-01.mjs';
import {icons as revised} from './network-devices-02.mjs';
import {icons as load} from './load-balancer-03.mjs';
import {icons as usb} from './usb-drive-03.mjs';
const replacements=new Map([...hard,...revised,...load,...usb].map(i=>[i.name,i]));
export const icons=[...compute,...network].map(i=>replacements.get(i.name)||i);
