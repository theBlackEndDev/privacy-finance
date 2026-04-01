import { AbsoluteFill, OffthreadVideo, staticFile, useCurrentFrame, getStaticFiles } from 'remotion';
import { TextOverlay } from './TextOverlay';
import { Crossfade } from './Crossfade';
import { COLORS } from '../lib/constants';

interface FeatureSceneProps {
  /** Feature identifier — matches filename in public/recordings/{id}.mp4 */
  featureId: string;
  /** Text overlay shown during this scene */
  overlayText: string;
}

/**
 * A single feature demonstration scene.
 * Shows a screen recording (or placeholder) with a text overlay.
 * Wraps content in a crossfade for smooth transitions.
 */
export const FeatureScene: React.FC<FeatureSceneProps> = ({
  featureId,
  overlayText,
}) => {
  const frame = useCurrentFrame();

  // Try to use video recording if it exists, fall back to placeholder
  const videoSrc = `recordings/${featureId}.mp4`;
  const hasVideo = videoExists(featureId);

  return (
    <Crossfade>
      <AbsoluteFill
        style={{ backgroundColor: COLORS.bgPrimary }}
      >
        {hasVideo ? (
          <OffthreadVideo
            src={staticFile(videoSrc)}
            style={{
              width: '100%',
              height: '100%',
              objectFit: 'contain',
            }}
          />
        ) : (
          <Placeholder featureId={featureId} frame={frame} />
        )}

        <TextOverlay
          text={overlayText}
          position="bottom-left"
          delay={5}
        />
      </AbsoluteFill>
    </Crossfade>
  );
};

/** Placeholder shown when video recording doesn't exist yet */
const Placeholder: React.FC<{ featureId: string; frame: number }> = ({
  featureId,
}) => {
  // Use placeholder image if available, otherwise solid color
  return (
    <AbsoluteFill
      style={{
        backgroundColor: COLORS.bgSecondary,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        flexDirection: 'column',
        gap: 16,
      }}
    >
      <div
        style={{
          width: 1600,
          height: 900,
          backgroundColor: COLORS.bgPrimary,
          borderRadius: 12,
          border: `1px solid ${COLORS.border}`,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }}
      >
        <span
          style={{
            color: COLORS.textMuted,
            fontSize: 28,
            fontFamily: 'Inter, sans-serif',
          }}
        >
          Recording: {featureId}.mp4
        </span>
      </div>
    </AbsoluteFill>
  );
};

/**
 * Check if a video recording exists in public/recordings/.
 * Uses Remotion's getStaticFiles() to detect available assets at render time.
 */
function videoExists(featureId: string): boolean {
  try {
    const files = getStaticFiles();
    return files.some(f => f.name === `recordings/${featureId}.mp4`);
  } catch {
    // getStaticFiles() may not be available in all contexts
    return false;
  }
}
