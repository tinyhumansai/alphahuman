'use client';

import * as THREE from 'three';
import { useEffect, useRef } from 'react';
import { ConvexGeometry } from 'three/addons/geometries/ConvexGeometry.js';

/** Start from a regular tetrahedron and lightly truncate each corner to create small blunted edges. */
function bluntedTetrahedronPoints(scale: number, bluntness = 0.12): THREE.Vector3[] {
  const tetra = [
    new THREE.Vector3(1, 1, 1),
    new THREE.Vector3(-1, -1, 1),
    new THREE.Vector3(-1, 1, -1),
    new THREE.Vector3(1, -1, -1),
  ];

  const points: THREE.Vector3[] = [];

  for (let i = 0; i < tetra.length; i += 1) {
    for (let j = 0; j < tetra.length; j += 1) {
      if (i === j) continue;
      points.push(tetra[i].clone().lerp(tetra[j], bluntness).multiplyScalar(scale));
    }
  }

  return points;
}

export default function RotatingTetrahedronCanvas() {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const renderer = new THREE.WebGLRenderer({
      canvas,
      antialias: true,
      alpha: true,
      powerPreference: 'high-performance',
    });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.outputColorSpace = THREE.SRGBColorSpace;
    renderer.toneMapping = THREE.NoToneMapping;

    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(45, 1, 0.1, 100);
    camera.position.set(0, 0.15, 4.8);

    const geometry = new ConvexGeometry(bluntedTetrahedronPoints(0.98, 0.11));
    const fillMaterial = new THREE.MeshLambertMaterial({
      color: '#b8e986',
      transparent: true,
      opacity: 0.4,
      emissive: '#0c1208',
      emissiveIntensity: 0.08,
    });
    const fillMesh = new THREE.Mesh(geometry, fillMaterial);

    const edgeGeometry = new THREE.EdgesGeometry(geometry);
    const edgeMaterial = new THREE.LineBasicMaterial({ color: '#b8e986' });

    const edges = new THREE.LineSegments(edgeGeometry, edgeMaterial);
    fillMesh.rotation.x = 0.35;
    fillMesh.rotation.y = -0.15;
    edges.rotation.x = 0.35;
    edges.rotation.y = -0.15;
    scene.add(fillMesh);
    scene.add(edges);

    const ambientLight = new THREE.AmbientLight('#ffffff', 0.1);
    const sun = new THREE.DirectionalLight('#ffffff', 1.1);
    sun.position.set(4.5, 6, 5);
    scene.add(ambientLight);
    scene.add(sun);

    let animationFrame = 0;

    const resize = () => {
      const parent = canvas.parentElement;
      if (!parent) return;

      const { width, height } = parent.getBoundingClientRect();
      if (!width || !height) return;

      renderer.setSize(width, height, false);
      camera.aspect = width / height;
      camera.updateProjectionMatrix();
    };

    const observer = new ResizeObserver(resize);
    if (canvas.parentElement) observer.observe(canvas.parentElement);
    resize();

    const animate = () => {
      fillMesh.rotation.y += 0.0038;
      fillMesh.rotation.x += 0.0002;
      edges.rotation.y += 0.0038;
      edges.rotation.x += 0.0002;

      renderer.render(scene, camera);
      animationFrame = window.requestAnimationFrame(animate);
    };

    animate();

    return () => {
      window.cancelAnimationFrame(animationFrame);
      observer.disconnect();
      edgeGeometry.dispose();
      geometry.dispose();
      fillMaterial.dispose();
      edgeMaterial.dispose();
      renderer.dispose();
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      style={{ width: '100%', height: '100%', display: 'block' }}
      aria-label="Rotating inverted tetrahedron spacecraft"
    />
  );
}
