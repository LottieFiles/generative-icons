// An ellipsis uses filled dots in Outline. Check their actual exported diameters
// instead of requiring invisible stroke paints. All other icons keep stroke checks.
import assert from 'node:assert/strict';
export function verifyFilledEllipsis(data, label) {
  for (const [preset, radius] of Object.entries({light:1.5, regular:2, bold:2.5})) {
    const layers=data.layers.filter(l=>l.shapes.some(s=>s.ty==='fl'&&s.o?.sid==='style.outline.'+preset));
    assert.equal(layers.length,3,label+' must contain three '+preset+' dots');
    for(const layer of layers) {
      assert.ok(!layer.shapes.some(s=>s.ty==='st'),label+' must not use dummy strokes');
      const paths=layer.shapes.filter(s=>s.ty==='sh');
      assert.equal(paths.length,1,label+' each dot is one contour');
      const ks=paths[0].ks;
      const shapes=ks.a?ks.k.flatMap(k=>[...(k.s||[]),...(k.e||[])]):[ks.k];
      assert.ok(shapes.length,label+' missing dot geometry');
      for(const shape of shapes) {
        assert.ok(shape.c,label+' dot must be closed');
        for(const axis of [0,1]) {
          const values=shape.v.map(v=>v[axis]);
          assert.ok(Math.abs(Math.max(...values)-Math.min(...values)-2*radius)<.00005,label+' incorrect '+preset+' dot diameter');
        }
      }
    }
  }
}
