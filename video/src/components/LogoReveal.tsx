import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from 'remotion';
import { COLORS, FONTS } from '../lib/constants';

interface LogoRevealProps {
  tagline?: string;
  showCTA?: boolean;
}

/**
 * Animated logo reveal with optional tagline and CTA.
 */
export const LogoReveal: React.FC<LogoRevealProps> = ({
  tagline = 'Your finances. Your data. Your machine.',
  showCTA = false,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Logo scales in with spring
  const logoScale = spring({
    frame,
    fps,
    config: { damping: 20, stiffness: 100 },
  });

  // Tagline fades in after logo
  const taglineOpacity = spring({
    frame: Math.max(0, frame - 15),
    fps,
    config: { damping: 30, stiffness: 80 },
  });

  // CTA slides up after tagline
  const ctaProgress = spring({
    frame: Math.max(0, frame - 30),
    fps,
    config: { damping: 25, stiffness: 90 },
  });
  const ctaTranslateY = interpolate(ctaProgress, [0, 1], [30, 0]);

  return (
    <AbsoluteFill
      style={{
        backgroundColor: COLORS.bgPrimary,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 24,
      }}
    >
      {/* Logo */}
      <div
        style={{
          transform: `scale(${logoScale})`,
          fontFamily: FONTS.heading,
          fontSize: 72,
          fontWeight: 700,
          color: COLORS.textPrimary,
          letterSpacing: '-0.03em',
        }}
      >
        <span style={{ color: COLORS.accent }}>Privacy</span>Finance
      </div>

      {/* Tagline */}
      <div
        style={{
          opacity: taglineOpacity,
          fontFamily: FONTS.body,
          fontSize: 28,
          color: COLORS.textSecondary,
          letterSpacing: '0.02em',
        }}
      >
        {tagline}
      </div>

      {/* CTA */}
      {showCTA && (
        <div
          style={{
            opacity: ctaProgress,
            transform: `translateY(${ctaTranslateY}px)`,
            marginTop: 24,
            padding: '16px 48px',
            backgroundColor: COLORS.accent,
            borderRadius: 12,
            fontFamily: FONTS.heading,
            fontSize: 22,
            fontWeight: 600,
            color: '#ffffff',
          }}
        >
          Download Free
        </div>
      )}
    </AbsoluteFill>
  );
};
