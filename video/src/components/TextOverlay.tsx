import { useCurrentFrame, useVideoConfig, interpolate, spring } from 'remotion';
import { COLORS, FONTS } from '../lib/constants';

interface TextOverlayProps {
  text: string;
  fontSize?: number;
  color?: string;
  position?: 'bottom-left' | 'bottom-center' | 'center' | 'top-left';
  delay?: number;
}

export const TextOverlay: React.FC<TextOverlayProps> = ({
  text,
  fontSize = 42,
  color = COLORS.textPrimary,
  position = 'bottom-left',
  delay = 0,
}) => {
  const frame = useCurrentFrame();
  const { fps, durationInFrames } = useVideoConfig();

  const adjustedFrame = frame - delay;

  // Fade in with spring
  const fadeIn = spring({
    frame: Math.max(0, adjustedFrame),
    fps,
    config: { damping: 30, stiffness: 120 },
  });

  // Fade out in last 10 frames
  const fadeOut = interpolate(
    frame,
    [durationInFrames - 10, durationInFrames],
    [1, 0],
    { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' }
  );

  const opacity = Math.min(fadeIn, fadeOut);

  // Slide up slightly on enter
  const translateY = interpolate(fadeIn, [0, 1], [20, 0]);

  const positionStyle = getPositionStyle(position);

  return (
    <div
      style={{
        ...positionStyle,
        opacity,
        transform: `translateY(${translateY}px)`,
        fontFamily: FONTS.heading,
        fontSize,
        fontWeight: 600,
        color,
        textShadow: '0 2px 12px rgba(0,0,0,0.8)',
        letterSpacing: '-0.02em',
        lineHeight: 1.2,
        padding: '24px 48px',
        maxWidth: '70%',
      }}
    >
      {text}
    </div>
  );
};

function getPositionStyle(position: TextOverlayProps['position']): React.CSSProperties {
  const base: React.CSSProperties = { position: 'absolute' };
  switch (position) {
    case 'bottom-left':
      return { ...base, bottom: 80, left: 0 };
    case 'bottom-center':
      return { ...base, bottom: 80, left: 0, right: 0, textAlign: 'center' };
    case 'center':
      return {
        ...base,
        top: '50%',
        left: '50%',
        transform: 'translate(-50%, -50%)',
        textAlign: 'center',
      };
    case 'top-left':
      return { ...base, top: 80, left: 0 };
    default:
      return { ...base, bottom: 80, left: 0 };
  }
}
