import { AbsoluteFill, Sequence, useCurrentFrame, useVideoConfig, spring, interpolate } from 'remotion';
import { BulletList } from '../components/BulletList';
import { LogoReveal } from '../components/LogoReveal';
import { Crossfade } from '../components/Crossfade';
import { COLORS, FONTS, PRIVACY_BULLETS, sec } from '../lib/constants';

/**
 * Act 3: The Close (1:15 - 1:30)
 *
 * 0:00-0:05 — Split screen: app + local DB file
 * 0:05-0:10 — Privacy bullet list animation
 * 0:10-0:15 — Logo + tagline + CTA
 */
export const Act3Close: React.FC = () => {
  return (
    <AbsoluteFill style={{ backgroundColor: COLORS.bgPrimary }}>
      {/* Scene 1: Split screen — app + local DB */}
      <Sequence durationInFrames={sec(5)} name="Split Screen">
        <SplitScreen />
      </Sequence>

      {/* Scene 2: Privacy bullet list */}
      <Sequence from={sec(5)} durationInFrames={sec(5)} name="Privacy Bullets">
        <Crossfade>
          <AbsoluteFill
            style={{
              backgroundColor: COLORS.bgPrimary,
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'center',
              paddingLeft: 200,
            }}
          >
            <div
              style={{
                fontFamily: FONTS.heading,
                fontSize: 36,
                fontWeight: 600,
                color: COLORS.accent,
                marginBottom: 32,
                paddingLeft: 48,
              }}
            >
              100% local. Zero cloud. Zero tracking.
            </div>
            <BulletList items={PRIVACY_BULLETS} stagger={10} />
          </AbsoluteFill>
        </Crossfade>
      </Sequence>

      {/* Scene 3: Final logo + CTA */}
      <Sequence from={sec(10)} durationInFrames={sec(5)} name="Final CTA">
        <LogoReveal showCTA tagline="Your finances. Your data. Your machine." />
      </Sequence>
    </AbsoluteFill>
  );
};

/** Split screen showing app on left, terminal/DB on right */
const SplitScreen: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const slideIn = spring({
    frame,
    fps,
    config: { damping: 20, stiffness: 80 },
  });

  return (
    <Crossfade>
      <AbsoluteFill
        style={{
          backgroundColor: COLORS.bgPrimary,
          display: 'flex',
          flexDirection: 'row',
          alignItems: 'center',
          justifyContent: 'center',
          gap: 48,
          padding: '0 80px',
        }}
      >
        {/* Left: App screenshot placeholder */}
        <div
          style={{
            width: 800,
            height: 560,
            backgroundColor: COLORS.bgSecondary,
            borderRadius: 12,
            border: `1px solid ${COLORS.border}`,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            opacity: slideIn,
            transform: `translateX(${(1 - slideIn) * -60}px)`,
          }}
        >
          <span style={{ color: COLORS.textMuted, fontFamily: FONTS.body, fontSize: 20 }}>
            App Screenshot
          </span>
        </div>

        {/* Right: Terminal showing local DB */}
        <div
          style={{
            width: 800,
            height: 560,
            backgroundColor: '#1e1e2e',
            borderRadius: 12,
            border: `1px solid ${COLORS.border}`,
            padding: 32,
            opacity: slideIn,
            transform: `translateX(${(1 - slideIn) * 60}px)`,
            fontFamily: FONTS.mono,
            fontSize: 16,
            color: COLORS.income,
            lineHeight: 2,
            overflow: 'hidden',
          }}
        >
          <div style={{ color: COLORS.textMuted }}>$ ls ~/PrivacyFinance/</div>
          <div>privacyfinance.db</div>
          <div style={{ color: COLORS.textMuted, marginTop: 16 }}>$ file privacyfinance.db</div>
          <div>SQLite 3.x database</div>
          <div style={{ color: COLORS.textMuted, marginTop: 16 }}>$ du -h privacyfinance.db</div>
          <div>2.4M    privacyfinance.db</div>
          <div style={{ color: COLORS.textMuted, marginTop: 24 }}>
            <span style={{ color: COLORS.accent }}>Your data.</span> On{' '}
            <span style={{ color: COLORS.accent }}>your machine.</span>
          </div>
        </div>
      </AbsoluteFill>
    </Crossfade>
  );
};
