import { AbsoluteFill, Sequence, useCurrentFrame, interpolate, spring, useVideoConfig } from 'remotion';
import { LogoReveal } from '../components/LogoReveal';
import { COLORS, FONTS, sec } from '../lib/constants';

/**
 * Act 1: The Problem (0:00 - 0:15)
 *
 * 0:00-0:05 — "Your bank app knows everything about you."
 * 0:05-0:10 — Competitor montage placeholder + "Cloud-synced. Tracked. Monetized."
 * 0:10-0:15 — Logo reveal with tagline
 */
export const Act1Problem: React.FC = () => {
  return (
    <AbsoluteFill style={{ backgroundColor: COLORS.bgPrimary }}>
      {/* Scene 1: Pain point text */}
      <Sequence durationInFrames={sec(5)} name="Pain Point">
        <PainPointText text="Your bank app knows everything about you." />
      </Sequence>

      {/* Scene 2: Competitor montage */}
      <Sequence from={sec(5)} durationInFrames={sec(5)} name="Competitor Montage">
        <CompetitorMontage />
      </Sequence>

      {/* Scene 3: Logo reveal */}
      <Sequence from={sec(10)} durationInFrames={sec(5)} name="Logo Reveal">
        <LogoReveal />
      </Sequence>
    </AbsoluteFill>
  );
};

/** Fade-in text on black background */
const PainPointText: React.FC<{ text: string }> = ({ text }) => {
  const frame = useCurrentFrame();
  const { fps, durationInFrames } = useVideoConfig();

  const fadeIn = spring({
    frame,
    fps,
    config: { damping: 30, stiffness: 60 },
  });

  const fadeOut = interpolate(
    frame,
    [durationInFrames - 15, durationInFrames],
    [1, 0],
    { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' }
  );

  return (
    <AbsoluteFill
      style={{
        backgroundColor: COLORS.bgPrimary,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '0 200px',
      }}
    >
      <div
        style={{
          opacity: Math.min(fadeIn, fadeOut),
          fontFamily: FONTS.heading,
          fontSize: 56,
          fontWeight: 600,
          color: COLORS.textPrimary,
          textAlign: 'center',
          lineHeight: 1.3,
          letterSpacing: '-0.02em',
        }}
      >
        {text}
      </div>
    </AbsoluteFill>
  );
};

/** Placeholder for competitor blur shots — replace with actual footage */
const CompetitorMontage: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps, durationInFrames } = useVideoConfig();

  const fadeIn = spring({ frame, fps, config: { damping: 30, stiffness: 80 } });
  const fadeOut = interpolate(
    frame,
    [durationInFrames - 10, durationInFrames],
    [1, 0],
    { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' }
  );

  return (
    <AbsoluteFill
      style={{
        backgroundColor: COLORS.bgPrimary,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 40,
      }}
    >
      {/* Blurred placeholder boxes representing competitor apps */}
      <div
        style={{
          opacity: Math.min(fadeIn, fadeOut) * 0.3,
          display: 'flex',
          gap: 24,
        }}
      >
        {['Mint', 'YNAB', 'Quicken'].map((name) => (
          <div
            key={name}
            style={{
              width: 200,
              height: 120,
              backgroundColor: COLORS.bgSecondary,
              borderRadius: 12,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              filter: 'blur(4px)',
              color: COLORS.textMuted,
              fontFamily: FONTS.body,
              fontSize: 20,
            }}
          >
            {name}
          </div>
        ))}
      </div>

      {/* Overlay text */}
      <div
        style={{
          opacity: Math.min(fadeIn, fadeOut),
          fontFamily: FONTS.heading,
          fontSize: 48,
          fontWeight: 600,
          color: COLORS.expense,
          textAlign: 'center',
          letterSpacing: '-0.01em',
        }}
      >
        Cloud-synced. Tracked. Monetized.
      </div>
    </AbsoluteFill>
  );
};
