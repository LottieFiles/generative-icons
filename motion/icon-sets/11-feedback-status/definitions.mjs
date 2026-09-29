import {icons as b1} from './batches/01.mjs';
import {icons as b2} from './batches/02.mjs';
import {icons as b3} from './batches/03.mjs';
import {icons as b4} from './batches/04.mjs';
export const icons=[...b1,...b2,...b3,...b4];
const expected=["loading-spinner", "loading-dots", "progress-ring", "progress-steps", "hourglass", "queued", "paused", "cancelled", "error", "warning", "info", "help", "no-results", "empty-state", "offline", "reconnect", "cloud-sync", "saved", "notification-dot", "session-expired"];
if(JSON.stringify(icons.map(i=>i.name))!==JSON.stringify(expected))throw Error("Feedback scope is missing or reordered");
