// 10-infrastructure-devices / usb-drive revision 03. Draft, not live.
import {make,F,rigid,posed,stroke,fill,framed,circle,polygon,capsule,holes} from './helpers.mjs';

// Shell polygon r1.5. Fillet lengths from polygon(): min(r, edge*.35), so the 2.5-long step
// edges get .875. Straight step segments on x=12: y 8.375..9.125 (top), y 22.875..23.625 (bottom).
const usbOut=polygon([[4,22],[4,10],[12,10],[12,7.5],[25,7.5],[25,24.5],[12,24.5],[12,22]],1.5);

// Solid: exterior offset 1.3 (unchanged). Seam cut is now x12, y11.5..20.5, width 2.6:
// paint x10.7..13.3, y10.2..21.8. Walls: 1.5 to top y8.7 and bottom y23.3; 1.79 from each cap
// centre to the sharp step corners (10.7,8.7)/(10.7,23.3), more once the concave fillet adds
// material; 2.0 from the dot holes (right edge 8.7) to the cut (10.7). Exterior walls retained.
const usbSolid=holes(
 polygon([[2.7,23.3],[2.7,8.7],[10.7,8.7],[10.7,6.2],[26.3,6.2],[26.3,25.8],[10.7,25.8],[10.7,23.3]],2.5),
 capsule([12,11.5],[12,20.5],2.6),circle(7.6,13.6,1.1),circle(7.6,18.4,1.1));

// Rigid eject drive, unchanged. Sweep: Solid left 2.1, right 29.3; Bold outline left 2.15.
const usbDx=F([[0,0],[.3,1.2,'smooth'],[.7,0,'smooth']],
              [[0,0],[.12,-.6],[.28,3],[.52,-.3],[.68,0]]);

const usbDrive=make('usb-drive','USB drive','The flash drive is tugged out of its port and slides back; Strong seats inward first then pulls a full 3 units.',
 posed([
  ...framed('drive',usbOut,usbSolid),
  // Partition on the shell step centerline. Endpoints sit mid-segment on the straight step
  // walls, so each round cap lies wholly inside the shell stroke at 1.5/2/2.5.
  stroke('junction','M12 8.75 L12 23.25'),
  fill('contact-a',circle(7.6,13.6,1.1),'outline'),fill('contact-b',circle(7.6,18.4,1.1),'outline'),
 ],rigid((t,s)=>({dx:usbDx(t,s)}))),.7,.68);
usbDrive.duotoneFill=true;

export const icons=[usbDrive];
