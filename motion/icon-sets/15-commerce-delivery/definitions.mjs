import {icons as a} from './batches/01.mjs';
import {icons as b} from './batches/02.mjs';
import {icons as c} from './batches/03.mjs';
import {icons as d} from './batches/04.mjs';
export const icons=[...a,...b,...c,...d];
const names=['storefront','shopping-bag','basket','product-tag','product-variants','discount','coupon','loyalty-points','gift-card','subscription-box','checkout','order-confirmed','open-package','packing-tape','return-package','delivery-truck','order-tracking','pickup-locker','warehouse','inventory'];
if(JSON.stringify(icons.map(i=>i.name))!==JSON.stringify(names))throw Error('Commerce scope differs from approved set');

// User-directed correction: Strong changes the primary action, with current visual review.
const strengthRevisions=new Set(["delivery-truck", "storefront", "shopping-bag", "basket", "product-tag", "product-variants", "discount", "coupon", "loyalty-points", "gift-card", "subscription-box", "checkout", "packing-tape", "order-tracking", "pickup-locker", "warehouse", "inventory"]);
for(const icon of icons)if(strengthRevisions.has(icon.name))icon.revision="strong-motion-audit-v2";
