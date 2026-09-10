/* Beam in GLSL: instanced cylinders (core + halo), alpha falls off from the head-on view to the
   silhouette (dot(normal, view)), dust animated by 1D noise along the length, additive, no depthWrite.
   set(segs) takes [[a,b,[r,g,b]], ...] in world coordinates. ponytail: no volume raymarch. */
window.BEAM = function (THREE, scene, max, core, halo) {
  "use strict";
  var VS = "varying vec3 vN; varying vec3 vV; varying float vY; varying vec3 vC;\n" +
    "void main(){ vec4 wp = modelMatrix * instanceMatrix * vec4(position,1.); vN = normalize(mat3(modelMatrix*instanceMatrix) * normal); vV = normalize(cameraPosition - wp.xyz); vY = uv.y; vC = instanceColor; gl_Position = projectionMatrix * viewMatrix * wp; }";
  // uD: the beam dust became a row of the VIDEO menu. At 0 the noise drops out of the maths (smooth
  // beam) without recompiling the shader or swapping the material — a `mix` costs less than either.
  var FS = "uniform float uT; uniform float uA; uniform float uK; uniform float uD; varying vec3 vN; varying vec3 vV; varying float vY; varying vec3 vC;\n" +
    "float hash(float n){ return fract(sin(n)*43758.5453); } float noise(float x){ float i=floor(x), f=fract(x); f=f*f*(3.-2.*f); return mix(hash(i),hash(i+1.),f); }\n" +
    "void main(){ float d = abs(dot(normalize(vN), normalize(vV))); float core = pow(d, uK); float dust = mix(1., .7 + .3*noise(vY*60. + uT*3.)*noise(vY*9. - uT*.8), uD); gl_FragColor = vec4(vC * core * dust * uA, 1.); }";
  function mat(a, k) { return new THREE.ShaderMaterial({ vertexShader: VS, fragmentShader: FS, uniforms: { uT: { value: 0 }, uA: { value: a }, uK: { value: k }, uD: { value: 1 } }, transparent: true, blending: THREE.AdditiveBlending, depthWrite: false, side: THREE.DoubleSide }); }
  var geo = new THREE.CylinderGeometry(1, 1, 1, 12, 1, true), Y = new THREE.Vector3(0, 1, 0), dummy = new THREE.Object3D(), col = new THREE.Color(), dir = new THREE.Vector3();
  function im(a, k) { var o = new THREE.InstancedMesh(geo, mat(a, k), max); o.setColorAt(0, new THREE.Color(1, 1, 1)); o.count = 0; o.frustumCulled = false; o.castShadow = o.receiveShadow = false; scene.add(o); return o; }
  var A = im(1, 2.2), B = im(.35, 4), rA = core, rB = halo;
  function set(segs, gain) { var n = Math.min(max, segs.length); for (var i = 0; i < n; i++) { var s = segs[i], a = s[0], b = s[1], len = dir.set(b[0] - a[0], b[1] - a[1], b[2] - a[2]).length(); dummy.position.set((a[0] + b[0]) / 2, (a[1] + b[1]) / 2, (a[2] + b[2]) / 2); dummy.quaternion.setFromUnitVectors(Y, dir.normalize()); col.setRGB(s[2][0], s[2][1], s[2][2]);
      dummy.scale.set(rA, len, rA); dummy.updateMatrix(); A.setMatrixAt(i, dummy.matrix); A.setColorAt(i, col); dummy.scale.set(rB, len, rB); dummy.updateMatrix(); B.setMatrixAt(i, dummy.matrix); B.setColorAt(i, col); }
    A.count = B.count = n; A.instanceMatrix.needsUpdate = B.instanceMatrix.needsUpdate = true; if (A.instanceColor) A.instanceColor.needsUpdate = true; if (B.instanceColor) B.instanceColor.needsUpdate = true; A.material.uniforms.uA.value = gain; B.material.uniforms.uA.value = .35 * gain; }
  return { set: set, tick: function (t) { A.material.uniforms.uT.value = t; B.material.uniforms.uT.value = t; },
    dust: function (on) { A.material.uniforms.uD.value = B.material.uniforms.uD.value = on ? 1 : 0; }, meshes: [A, B] };
};
