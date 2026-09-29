import test from 'node:test';
import assert from 'node:assert/strict';
import {sceneSubpath} from './geometry-subpath.mjs';
test('a retraced hole seam is excluded while its actual contour and animation are retained',()=>{
 const n={commands:[0,1,1,1,1,2,2,3],points:[[0,[0,0,30,0,0,0,10,10,10,10,11,10,12,11,12,12,12,13,11,14,10,10]],[1,[0,0,30,0,0,0,10,11,10,11,11,11,12,12,12,13,12,14,11,15,10,11]]]};
 assert.deepEqual(sceneSubpath(n,[4,7]),{commands:[0,2,2],points:[[0,[10,10,11,10,12,11,12,12,12,13,11,14,10,10]],[1,[10,11,11,11,12,12,12,13,12,14,11,15,10,11]]]});
 assert.equal(sceneSubpath(n,undefined),n);assert.throws(()=>sceneSubpath(n,[5,7]),/start with a point/);
});
