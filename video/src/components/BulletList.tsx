import { useCurrentFrame, useVideoConfig, spring } from 'remotion';
import { COLORS, FONTS } from '../lib/constants';

interface BulletListProps {
  items: readonly string[];
  /** Frames between each bullet appearing */
  stagger?: number;
}

/**
 * Animated bullet list — each item springs in with a stagger delay.
 */
export const BulletList: React.FC<BulletListProps> = ({
  items,
  stagger = 12,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 20,
        padding: '0 48px',
      }}
    >
      {items.map((item, i) => {
        const progress = spring({
          frame: Math.max(0, frame - i * stagger),
          fps,
          config: { damping: 25, stiffness: 100 },
        });

        return (
          <div
            key={item}
            style={{
              opacity: progress,
              transform: `translateX(${(1 - progress) * 40}px)`,
              fontFamily: FONTS.body,
              fontSize: 30,
              color: COLORS.textPrimary,
              display: 'flex',
              alignItems: 'center',
              gap: 16,
            }}
          >
            <span
              style={{
                width: 10,
                height: 10,
                borderRadius: '50%',
                backgroundColor: COLORS.accent,
                flexShrink: 0,
              }}
            />
            {item}
          </div>
        );
      })}
    </div>
  );
};
