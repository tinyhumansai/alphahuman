import { useEffect, useRef } from 'react';

import { Gradient } from '../lib/meshGradient';

/**
 * Animated WebGL mesh gradient background (Stripe-style).
 * Renders behind the dotted-canvas overlay so dots remain visible on top.
 */
export default function MeshGradient() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const gradient = new Gradient();
    gradient.initGradient('#mesh-gradient');

    return () => {
      gradient.pause();
      // Release WebGL context on unmount
      if (canvasRef.current) {
        const gl = canvasRef.current.getContext('webgl') || canvasRef.current.getContext('webgl2');
        if (gl) {
          gl.getExtension('WEBGL_lose_context')?.loseContext();
        }
      }
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      id="mesh-gradient"
      data-transition-in
      className="absolute inset-0 w-full h-full opacity-20"
      style={
        {
          '--gradient-color-1': '#ffffff',
          '--gradient-color-2': '#EFF6FF', // primary-50
          '--gradient-color-3': '#cccccc', // primary-100
          '--gradient-color-4': '#BFDBFE', // primary-200
        } as React.CSSProperties
      }
    />
  );
}
