import { useCurrentFrame, useVideoConfig, interpolate } from 'remotion';
import { CROSSFADE_FRAMES } from '../lib/constants';

interface CrossfadeProps {
  children: React.ReactNode;
  fadeInFrames?: number;
  fadeOutFrames?: number;
}

/**
 * Wraps children with a crossfade (fade in at start, fade out at end).
 */
export const Crossfade: React.FC<CrossfadeProps> = ({
  children,
  fadeInFrames = CROSSFADE_FRAMES,
  fadeOutFrames = CROSSFADE_FRAMES,
}) => {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();

  const fadeIn = interpolate(frame, [0, fadeInFrames], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  const fadeOut = interpolate(
    frame,
    [durationInFrames - fadeOutFrames, durationInFrames],
    [1, 0],
    { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' }
  );

  return (
    <div style={{ opacity: Math.min(fadeIn, fadeOut), width: '100%', height: '100%' }}>
      {children}
    </div>
  );
};
