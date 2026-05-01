import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";
import { z } from "zod";
import { zColor } from "@remotion/zod-types";

export const ghostySchema = z.object({
  bodyColor: zColor(),
  blushColor: zColor(),
});

export type GhostyProps = z.infer<typeof ghostySchema>;

// Body: tall rounded blob with a *subtly* wider base — just a hint of
// the splayed-A stance from the reference, not an actual triangle.
const BODY_PATH =
  "M500,240 " +
  "C700,240 855,420 860,625 " + // apex → right widest
  "C863,770 810,860 700,895 " + // right widest → lower-right (slight outward swing)
  "Q500,930 300,895 " + // wide soft base
  "C190,860 137,770 140,625 " + // lower-left → left widest
  "C145,420 300,240 500,240 Z"; // left widest → back to apex

// Two soft mitten-shaped legs/feet that flare outward from the base.
const LEFT_LEG_PATH =
  "M310,880 " +
  "C220,895 150,940 175,985 " +
  "C200,1020 290,1020 360,990 " +
  "C420,965 430,915 400,890 Z";
const RIGHT_LEG_PATH =
  "M690,880 " +
  "C780,895 850,940 825,985 " +
  "C800,1020 710,1020 640,990 " +
  "C580,965 570,915 600,890 Z";

// Raised mitten arm sticking up-right from the body's right shoulder.
const ARM_PATH =
  "M800,560 " +
  "C870,485 955,500 955,575 " +
  "C955,635 880,650 800,615 Z";

export const Ghosty: React.FC<GhostyProps> = ({
  bodyColor,
  blushColor,
}) => {
  const frame = useCurrentFrame();
  const { fps, width, height } = useVideoConfig();

  // Gentle bob for the whole character.
  const bob = Math.sin((frame / fps) * Math.PI * 1.2) * 14;

  // Top dot drifts independently and squashes when it presses into the body — subtle.
  const dotPhase = (frame / fps) * Math.PI * 1.0;
  const dotDx = Math.sin(dotPhase * 0.7) * 6;
  const dotDy = Math.sin(dotPhase) * 9;
  const press = Math.max(0, Math.sin(dotPhase));
  const dotSquashY = 1 - 0.08 * press;
  const dotSquashX = 1 + 0.05 * press;

  // Wave: oscillating arm rotation.
  const wave = Math.sin((frame / fps) * Math.PI * 2.4) * 12;

  // Blink every ~2.6s for ~6 frames — offset so frame 0 is eyes open.
  const blinkPeriod = Math.round(fps * 2.6);
  const blinkOffset = Math.round(blinkPeriod / 2);
  const inBlink = (frame + blinkOffset) % blinkPeriod < 6;
  const blinkScale = inBlink ? 0.12 : 1;

  const VB = 1000;
  const size = Math.min(width, height) * 0.85;

  return (
    <AbsoluteFill style={{ justifyContent: "center", alignItems: "center" }}>
      <svg
        width={size}
        height={size}
        viewBox={`0 0 ${VB} ${VB}`}
        style={{ overflow: "visible" }}
      >
        <defs>
          {/* Body shading: many stops to keep the dark gradient smooth (avoid banding). */}
          <radialGradient id="ghosty-body" cx="0.32" cy="0.28" r="1.05">
            <stop offset="0%" stopColor="#45454a" />
            <stop offset="15%" stopColor="#393940" />
            <stop offset="30%" stopColor="#2d2d33" />
            <stop offset="45%" stopColor="#232328" />
            <stop offset="60%" stopColor="#1a1a1e" />
            <stop offset="75%" stopColor="#121215" />
            <stop offset="88%" stopColor="#0a0a0c" />
            <stop offset="100%" stopColor="#050506" />
          </radialGradient>
          <radialGradient id="ghosty-dot" cx="0.35" cy="0.3" r="1">
            <stop offset="0%" stopColor="#45454a" />
            <stop offset="20%" stopColor="#363639" />
            <stop offset="45%" stopColor="#232326" />
            <stop offset="70%" stopColor="#15151a" />
            <stop offset="100%" stopColor="#050507" />
          </radialGradient>
          {/* Film grain dither — breaks up banding in the dark gradient. */}
          <filter id="ghosty-grain" x="0%" y="0%" width="100%" height="100%">
            <feTurbulence
              type="fractalNoise"
              baseFrequency="0.9"
              numOctaves="2"
              stitchTiles="stitch"
              result="noise"
            />
            <feColorMatrix
              in="noise"
              type="matrix"
              values="0 0 0 0 1  0 0 0 0 1  0 0 0 0 1  0 0 0 0.06 0"
            />
          </filter>
          {/* Pre-blurred soft highlight — no hard edge from the ellipse. */}
          <filter id="ghosty-soft" x="-30%" y="-30%" width="160%" height="160%">
            <feGaussianBlur stdDeviation="30" />
          </filter>
          {/* Soft drop shadow under the body. */}
          <filter id="ghosty-drop" x="-20%" y="-20%" width="140%" height="160%">
            <feGaussianBlur in="SourceAlpha" stdDeviation="14" />
            <feOffset dx="0" dy="22" result="off" />
            <feComponentTransfer>
              <feFuncA type="linear" slope="0.45" />
            </feComponentTransfer>
            <feMerge>
              <feMergeNode />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
          {/* Ground shadow ellipse gradient. */}
          <radialGradient id="ghosty-ground" cx="0.5" cy="0.5" r="0.5">
            <stop offset="0%" stopColor="#000000" stopOpacity="0.35" />
            <stop offset="100%" stopColor="#000000" stopOpacity="0" />
          </radialGradient>
          {/* Body clip so the inner highlight stays inside the body silhouette. */}
          <clipPath id="ghosty-body-clip">
            <path d={BODY_PATH} />
          </clipPath>
        </defs>

        {/* Ground shadow that scales inversely with bob (further up = smaller shadow). */}
        <g
          transform={`translate(500, 970) scale(${1 - bob / 600}, 1)`}
          style={{ transformOrigin: "500px 970px" }}
        >
          <ellipse cx={0} cy={0} rx={260} ry={28} fill="url(#ghosty-ground)" />
        </g>

        {/* Legs stay planted — they don't bob with the body. */}
        <g filter="url(#ghosty-drop)">
          <path d={LEFT_LEG_PATH} fill="url(#ghosty-body)" />
          <path d={RIGHT_LEG_PATH} fill="url(#ghosty-body)" />
        </g>

        {/* Body + head-dot bob together; legs stay fixed below. */}
        <g transform={`translate(0, ${bob})`} filter="url(#ghosty-drop)">
          {/* Top head-dot — floats and squashes against the body apex. */}
          <g
            transform={
              `translate(${dotDx}, ${dotDy}) ` +
              // squash around the bottom of the dot (where it meets the body)
              `translate(520 240) scale(${dotSquashX} ${dotSquashY}) translate(-520 -240)`
            }
          >
            <ellipse cx={520} cy={155} rx={92} ry={88} fill="url(#ghosty-dot)" />
            {/* Specular highlight. */}
            <ellipse cx={490} cy={120} rx={24} ry={14} fill="#ffffff" opacity={0.18} />
          </g>
          {/* Raised right arm (waves). Pivots at the shoulder. */}
          <g transform={`rotate(${wave} 820 590)`}>
            <path d={ARM_PATH} fill="url(#ghosty-body)" />
          </g>

          {/* Body */}
          <path d={BODY_PATH} fill="url(#ghosty-body)" />

          {/* Inner shading: highlight sheen top-left + soft shadow bottom-right, clipped to body.
              Pre-blurred via filter so they blend smoothly with the gradient. */}
          <g clipPath="url(#ghosty-body-clip)">
            <g filter="url(#ghosty-soft)">
              <ellipse cx={340} cy={380} rx={220} ry={160} fill="#ffffff" opacity={0.09} />
              <ellipse cx={720} cy={800} rx={280} ry={170} fill="#000000" opacity={0.45} />
            </g>
            {/* Grain dither layer to kill banding. */}
            <rect x={0} y={0} width={1000} height={1000} filter="url(#ghosty-grain)" />
          </g>

          {/* Blush */}
          <ellipse cx={360} cy={545} rx={48} ry={22} fill={blushColor} opacity={0.85} />
          <ellipse cx={680} cy={545} rx={48} ry={22} fill={blushColor} opacity={0.85} />

          {/* Eyes */}
          <g>
            <ellipse
              cx={415}
              cy={515}
              rx={30}
              ry={40 * blinkScale}
              fill="#0a0a0a"
            />
            <ellipse
              cx={625}
              cy={515}
              rx={30}
              ry={40 * blinkScale}
              fill="#0a0a0a"
            />
            {!inBlink && (
              <>
                <circle cx={425} cy={501} r={7} fill="#ffffff" />
                <circle cx={635} cy={501} r={7} fill="#ffffff" />
              </>
            )}
          </g>

          {/* Mouth: closed smile. */}
          <path
            d="M478,570 Q520,617 562,570 Q520,597 478,570 Z"
            fill="#0a0a0a"
          />

          {/* Subtle blush glow ring to give cheeks dimension. */}
          <ellipse cx={360} cy={545} rx={56} ry={26} fill={blushColor} opacity={0.18} />
          <ellipse cx={680} cy={545} rx={56} ry={26} fill={blushColor} opacity={0.18} />
        </g>
      </svg>
    </AbsoluteFill>
  );
};
